use clap::{Parser, Subcommand};
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use finterm::{App, Config};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::path::PathBuf;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser)]
#[command(name = "finterm")]
#[command(about = "A terminal-based financial news aggregator", long_about = None)]
#[command(version)]
struct Cli {
    /// Path to config file
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,
    
    /// Disable caching
    #[arg(long)]
    no_cache: bool,
    
    /// Log level (trace, debug, info, warn, error)
    #[arg(long, default_value = "info")]
    log_level: String,
    
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize configuration file
    ConfigInit,
    
    /// Set configuration value
    ConfigSet {
        /// API key for Finnhub
        #[arg(long)]
        api_key: Option<String>,
    },
    
    /// Clear cache
    CacheClear,
    
    /// View financial news only
    News,
    
    /// View Hacker News only
    Hn,
    
    /// Search across all sources
    Search {
        /// Search query
        query: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    
    // Initialize logging
    let log_filter = format!("finterm={}", cli.log_level);
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| log_filter.into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    
    // Handle subcommands
    if let Some(command) = cli.command {
        return handle_command(command, cli.config).await;
    }
    
    // Load configuration
    let config_path = cli.config.unwrap_or_else(Config::default_path);
    let mut config = if config_path.exists() {
        Config::load(&config_path)?
    } else {
        eprintln!("Config file not found at: {}", config_path.display());
        eprintln!("Creating default configuration file...");
        let default_config = Config::default();
        default_config.save(&config_path)?;
        eprintln!("Please edit the config file and add your API keys:");
        eprintln!("  {}", config_path.display());
        eprintln!("\nYou can also use: finterm config-set --api-key YOUR_FINNHUB_KEY");
        return Ok(());
    };
    
    // Override cache setting if --no-cache is used
    if cli.no_cache {
        config.cache.enabled = false;
    }
    
    // Note about demo mode (no longer blocking)
    if config.finnhub.api_key.is_empty() {
        // Finnhub not configured, but app will still work with HN
    }
    
    // Create app
    let mut app = App::new(config)?;
    
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    
    // Run app
    let res = app.run(&mut terminal).await;
    
    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    
    if let Err(err) = res {
        eprintln!("Error: {}", err);
    }
    
    Ok(())
}

async fn handle_command(
    command: Commands,
    config_path: Option<PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    let config_path = config_path.unwrap_or_else(Config::default_path);
    
    match command {
        Commands::ConfigInit => {
            if config_path.exists() {
                eprintln!("Config file already exists at: {}", config_path.display());
            } else {
                let config = Config::default();
                config.save(&config_path)?;
                println!("Created config file at: {}", config_path.display());
                println!("\nExample configuration:");
                println!("{}", Config::create_example());
            }
        }
        
        Commands::ConfigSet { api_key } => {
            let mut config = if config_path.exists() {
                Config::load(&config_path)?
            } else {
                Config::default()
            };
            
            if let Some(key) = api_key {
                config.finnhub.api_key = key;
                config.save(&config_path)?;
                println!("API key updated successfully!");
            }
        }
        
        Commands::CacheClear => {
            let config = Config::load(&config_path)?;
            let cache_dir = config.cache_dir();
            
            if cache_dir.exists() {
                std::fs::remove_dir_all(&cache_dir)?;
                println!("Cache cleared successfully!");
            } else {
                println!("Cache directory not found.");
            }
        }
        
        Commands::News => {
            println!("Launching finance news view...");
            // Would launch TUI in finance-only mode
        }
        
        Commands::Hn => {
            println!("Launching Hacker News view...");
            // Would launch TUI in HN-only mode
        }
        
        Commands::Search { query } => {
            println!("Searching for: {}", query);
            // Would perform search and display results
        }
    }
    
    Ok(())
}
