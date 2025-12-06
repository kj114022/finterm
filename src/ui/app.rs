//! Application module
//! 
//! Main application state and event handling with provider-based architecture

use crate::cache::CacheManager;
use crate::config::Config;
use crate::models::FeedItem;
use crate::providers::{FinnhubProvider, HackerNewsProvider, CratesIoProvider, ProviderRegistry};
use crate::utils::Action;
use crate::ui::views;
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use ratatui::backend::Backend;
use ratatui::Terminal;
use std::io;
use std::time::{Duration, Instant};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("Provider error: {0}")]
    Provider(String),
}

pub type Result<T> = std::result::Result<T, AppError>;

/// Application view state
#[derive(Debug, Clone, PartialEq)]
pub enum AppState {
    /// Landing page for source selection
    Landing,
    /// Dashboard with all feeds
    Dashboard,
    /// Single feed view
    Feed(String),  // provider_id
    /// Article/item detail view
    Article,
    /// Help screen
    Help,
}

/// Main application struct with provider-based architecture
pub struct App {
    pub config: Config,
    pub state: AppState,
    pub should_quit: bool,
    
    // Provider system
    pub registry: ProviderRegistry,
    pub cache: CacheManager,
    
    // Data
    pub items: Vec<FeedItem>,
    pub selected_idx: usize,
    pub current_item: Option<FeedItem>,
    
    // Landing page state
    pub landing_selected: usize,
    
    // UI state
    pub scroll_offset: usize,
    pub status_message: Option<String>,
    pub last_update: Instant,
    pub loading: bool,
}

impl App {
    /// Create a new application instance
    pub fn new(config: Config) -> Result<Self> {
        // Create provider registry
        let mut registry = ProviderRegistry::new();
        
        // Register HackerNews provider first (most used)
        if let Ok(hn) = HackerNewsProvider::new(None) {
            registry.register(hn);
        }
        
        // Register Finnhub provider
        if let Ok(finnhub) = FinnhubProvider::new(
            config.finnhub.api_key.clone(),
            Some(config.finnhub.category.clone()),
        ) {
            registry.register(finnhub);
        }
        
        // Register Crates.io provider
        if let Ok(cratesio) = CratesIoProvider::new(None) {
            registry.register(cratesio);
        }
        
        // Create cache
        let cache_dir = config.cache_dir();
        let cache = CacheManager::new(cache_dir, config.cache.max_size_mb)
            .map_err(|e| AppError::Config(e.to_string()))?;
        
        Ok(Self {
            config,
            state: AppState::Landing,  // Start at landing page
            should_quit: false,
            registry,
            cache,
            items: Vec::new(),
            selected_idx: 0,
            current_item: None,
            landing_selected: 0,
            scroll_offset: 0,
            status_message: None,
            last_update: Instant::now(),
            loading: false,
        })
    }
    
    /// Run the application main loop
    pub async fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        loop {
            // Render
            terminal.draw(|f| self.render(f))?;
            
            // Handle input with timeout
            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    self.handle_key_event(key).await?;
                }
            }
            
            if self.should_quit {
                break;
            }
        }
        
        Ok(())
    }
    
    /// Handle keyboard events
    async fn handle_key_event(&mut self, key: KeyEvent) -> Result<()> {
        use crate::utils::map_key_event;
        
        let action = map_key_event(key, self.config.ui.vim_mode);
        
        match &self.state {
            AppState::Landing => self.handle_landing_input(key, action).await?,
            AppState::Dashboard | AppState::Feed(_) => self.handle_feed_input(key, action).await?,
            AppState::Article => self.handle_article_input(action),
            AppState::Help => self.handle_help_input(action),
        }
        
        Ok(())
    }
    
    /// Handle landing page input
    async fn handle_landing_input(&mut self, key: KeyEvent, action: Action) -> Result<()> {
        let provider_count = self.registry.len();
        
        match action {
            Action::Quit => self.should_quit = true,
            Action::Help => self.state = AppState::Help,
            Action::NavigateUp => {
                if self.landing_selected > 0 {
                    self.landing_selected -= 1;
                }
            }
            Action::NavigateDown => {
                if self.landing_selected < provider_count {  // +1 for "All" option
                    self.landing_selected += 1;
                }
            }
            Action::Select => {
                self.select_from_landing().await?;
            }
            _ => {
                // Handle number keys 1-9 for quick selection
                if let KeyCode::Char(c) = key.code {
                    if let Some(digit) = c.to_digit(10) {
                        if digit >= 1 && (digit as usize) <= provider_count {
                            self.landing_selected = (digit - 1) as usize;
                            self.select_from_landing().await?;
                        }
                    }
                    // 'A' or 'a' for All
                    if c == 'a' || c == 'A' {
                        self.landing_selected = provider_count;  // Select "All"
                        self.select_from_landing().await?;
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Handle feed list input
    async fn handle_feed_input(&mut self, _key: KeyEvent, action: Action) -> Result<()> {
        match action {
            Action::Quit => self.should_quit = true,
            Action::Help => self.state = AppState::Help,
            Action::Back => {
                self.state = AppState::Landing;
                self.items.clear();
                self.selected_idx = 0;
            }
            Action::NavigateUp => {
                if self.selected_idx > 0 {
                    self.selected_idx -= 1;
                }
            }
            Action::NavigateDown => {
                if self.selected_idx < self.items.len().saturating_sub(1) {
                    self.selected_idx += 1;
                }
            }
            Action::GoToTop => {
                self.selected_idx = 0;
            }
            Action::GoToBottom => {
                self.selected_idx = self.items.len().saturating_sub(1);
            }
            Action::Select => {
                if let Some(item) = self.items.get(self.selected_idx) {
                    self.current_item = Some(item.clone());
                    self.state = AppState::Article;
                    self.scroll_offset = 0;
                }
            }
            Action::Refresh => {
                self.refresh_current_feed().await?;
            }
            Action::OpenInBrowser => {
                if let Some(item) = self.items.get(self.selected_idx) {
                    if let Some(url) = &item.url {
                        #[cfg(target_os = "macos")]
                        let _ = std::process::Command::new("open").arg(url).spawn();
                        self.status_message = Some("Opened in browser".to_string());
                    }
                }
            }
            _ => {}
        }
        
        Ok(())
    }
    
    /// Handle article view input
    fn handle_article_input(&mut self, action: Action) {
        match action {
            Action::Quit => self.should_quit = true,
            Action::Back => {
                // Go back to feed list
                if self.state == AppState::Article {
                    // Determine what state to return to
                    let provider_id = self.current_item
                        .as_ref()
                        .map(|i| i.provider_id.clone());
                    
                    if let Some(id) = provider_id {
                        self.state = AppState::Feed(id);
                    } else {
                        self.state = AppState::Dashboard;
                    }
                }
                self.current_item = None;
                self.scroll_offset = 0;
            }
            Action::PageDown | Action::NavigateDown => {
                self.scroll_offset = self.scroll_offset.saturating_add(5);
            }
            Action::PageUp | Action::NavigateUp => {
                self.scroll_offset = self.scroll_offset.saturating_sub(5);
            }
            Action::GoToTop => {
                self.scroll_offset = 0;
            }
            Action::OpenInBrowser => {
                if let Some(item) = &self.current_item {
                    if let Some(url) = &item.url {
                        #[cfg(target_os = "macos")]
                        let _ = std::process::Command::new("open").arg(url).spawn();
                        self.status_message = Some("Opened in browser".to_string());
                    }
                }
            }
            Action::NextArticle => {
                if self.selected_idx < self.items.len().saturating_sub(1) {
                    self.selected_idx += 1;
                    self.current_item = self.items.get(self.selected_idx).cloned();
                    self.scroll_offset = 0;
                }
            }
            Action::PrevArticle => {
                if self.selected_idx > 0 {
                    self.selected_idx -= 1;
                    self.current_item = self.items.get(self.selected_idx).cloned();
                    self.scroll_offset = 0;
                }
            }
            _ => {}
        }
    }
    
    /// Handle help view input
    fn handle_help_input(&mut self, action: Action) {
        match action {
            Action::Quit => self.should_quit = true,
            Action::Back | Action::Help => {
                self.state = AppState::Landing;
            }
            _ => {}
        }
    }
    
    /// Select a provider from landing page
    async fn select_from_landing(&mut self) -> Result<()> {
        let provider_count = self.registry.len();
        
        if self.landing_selected >= provider_count {
            // "All" selected - go to dashboard
            self.state = AppState::Dashboard;
            self.fetch_all_items().await?;
        } else {
            // Specific provider selected
            let ids = self.registry.ids();
            if let Some(id) = ids.get(self.landing_selected) {
                let id = id.to_string();
                self.state = AppState::Feed(id.clone());
                self.fetch_provider_items(&id).await?;
            }
        }
        
        Ok(())
    }
    
    /// Fetch items from all providers
    async fn fetch_all_items(&mut self) -> Result<()> {
        self.loading = true;
        self.status_message = Some("Loading...".to_string());
        
        let limit = self.config.finnhub.max_articles.max(100);
        self.items = self.registry.fetch_all(limit).await;
        self.selected_idx = 0;
        
        self.loading = false;
        self.status_message = Some(format!("Loaded {} items", self.items.len()));
        self.last_update = Instant::now();
        
        Ok(())
    }
    
    /// Fetch items from a specific provider
    async fn fetch_provider_items(&mut self, provider_id: &str) -> Result<()> {
        self.loading = true;
        self.status_message = Some("Loading...".to_string());
        
        let limit = self.config.finnhub.max_articles.max(100);
        
        match self.registry.fetch_from(provider_id, limit).await {
            Ok(items) => {
                self.items = items;
                self.selected_idx = 0;
                self.status_message = Some(format!("Loaded {} items", self.items.len()));
            }
            Err(e) => {
                self.status_message = Some(format!("Error: {}", e));
            }
        }
        
        self.loading = false;
        self.last_update = Instant::now();
        
        Ok(())
    }
    
    /// Refresh current feed
    async fn refresh_current_feed(&mut self) -> Result<()> {
        match &self.state {
            AppState::Dashboard => {
                self.fetch_all_items().await?;
            }
            AppState::Feed(id) => {
                let id = id.clone();
                self.fetch_provider_items(&id).await?;
            }
            _ => {}
        }
        Ok(())
    }
    
    /// Render the UI based on current state
    fn render(&mut self, f: &mut ratatui::Frame) {
        match &self.state {
            AppState::Landing => {
                views::landing::render(f, &self.registry, self.landing_selected);
            }
            AppState::Dashboard => {
                views::feed::render(
                    f,
                    "All Sources",
                    "🌐",
                    &self.items,
                    self.selected_idx,
                    self.status_message.as_deref(),
                    self.loading,
                );
            }
            AppState::Feed(provider_id) => {
                let (name, icon) = self.registry.get(provider_id)
                    .map(|p| (p.name().to_string(), p.icon().to_string()))
                    .unwrap_or(("Unknown".to_string(), "❓".to_string()));
                
                views::feed::render(
                    f,
                    &name,
                    &icon,
                    &self.items,
                    self.selected_idx,
                    self.status_message.as_deref(),
                    self.loading,
                );
            }
            AppState::Article => {
                if let Some(item) = &self.current_item {
                    views::article::render_feed_item(f, item, self.scroll_offset);
                }
            }
            AppState::Help => {
                views::help::render(f, &crate::utils::get_help_text(self.config.ui.vim_mode));
            }
        }
    }
}
