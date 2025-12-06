# FinTerm

**A fast, futuristic terminal news aggregator for Hacker News and financial markets.**

![Rust](https://img.shields.io/badge/rust-1.75+-orange.svg)
![License](https://img.shields.io/badge/license-MIT-blue.svg)
[![CI](https://github.com/kj114022/finterm/actions/workflows/ci.yml/badge.svg)](https://github.com/kj114022/finterm/actions)

## ✨ Features

- 🟠 **Hacker News** - Top, New, Best, Ask HN, Show HN, Jobs
- 📈 **Financial News** - Real-time market news from Finnhub
- ⚡ **Fast** - Parallel loading (10 concurrent requests)
- ♾️ **Infinite Scroll** - Load more as you scroll
- 🎨 **Futuristic UI** - Beautiful terminal interface
- ⌨️ **Vim-style** - j/k navigation, / to search

## 🚀 Installation

### MacPorts (macOS)

```bash
# Install from MacPorts (after official submission)
sudo port install finterm

# Or use local portfile
cd /path/to/finterm/ports
sudo portindex
sudo port install finterm
```

### Homebrew (macOS/Linux)

```bash
brew tap kj114022/finterm
brew install finterm
```

### Cargo (All platforms)

```bash
cargo install finterm
```

### Binary Download

Download pre-built binaries from [GitHub Releases](https://github.com/kj114022/finterm/releases):

| Platform | Download |
|----------|----------|
| macOS (Apple Silicon) | `finterm-macos-aarch64.tar.gz` |
| macOS (Intel) | `finterm-macos-x86_64.tar.gz` |
| Linux (x86_64) | `finterm-linux-x86_64.tar.gz` |
| Windows | `finterm-windows-x86_64.exe` |

```bash
# Example: Install on macOS
curl -LO https://github.com/kj114022/finterm/releases/latest/download/finterm-macos-aarch64.tar.gz
tar -xzf finterm-macos-aarch64.tar.gz
sudo mv finterm /usr/local/bin/
```

### From Source

```bash
git clone https://github.com/kj114022/finterm.git
cd finterm
cargo build --release
sudo cp target/release/finterm /usr/local/bin/
```

## 📖 Usage

```bash
# Run the app
finterm

# View only Hacker News
finterm hn

# View only financial news
finterm news
```

### Keybindings

| Key | Action |
|-----|--------|
| `1-9` | Select feed source |
| `j/k` or `↑/↓` | Navigate |
| `Enter` | Open item |
| `o` | Open in browser |
| `Esc` | Go back |
| `r` | Refresh |
| `[` / `]` | Prev/Next article |
| `?` | Help |
| `q` | Quit |

## ⚙️ Configuration

Configuration is stored at `~/.config/finterm/config.toml`:

```toml
[finnhub]
api_key = "your_finnhub_api_key"     # Get free key at finnhub.io
category = "general"                  # general, forex, crypto, merger

[hackernews]
default_category = "top"              # top, new, best, ask, show, job

[ui]
vim_mode = true
theme = "dark"

[cache]
enabled = true
max_size_mb = 50
ttl_minutes = 5
```

Get a free Finnhub API key at [finnhub.io](https://finnhub.io/register).

## 🏗️ Architecture

```
src/
├── providers/          # Feed providers (HN, Finnhub, RSS...)
│   ├── mod.rs         # FeedProvider trait
│   ├── hackernews.rs  # Hacker News provider
│   ├── finnhub.rs     # Finnhub financial news
│   └── registry.rs    # Provider registry
├── models/
│   └── feed_item.rs   # Unified FeedItem model
├── ui/
│   ├── app.rs         # Main application
│   └── views/         # UI views
└── main.rs            # Entry point
```

## 📄 License

MIT License - see [LICENSE](LICENSE) for details.

## 🤝 Contributing

Contributions welcome! Please read our [Contributing Guide](CONTRIBUTING.md).
