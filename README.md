<p align="center">
  <img src="https://img.shields.io/badge/rust-1.75+-orange.svg" alt="Rust">
  <img src="https://img.shields.io/badge/license-AGPL--3.0-blue.svg" alt="License">
  <img src="https://img.shields.io/github/v/release/kj114022/finterm" alt="Release">
  <img src="https://img.shields.io/badge/platform-macOS%20%7C%20Linux%20%7C%20Windows-lightgrey" alt="Platform">
</p>

# FinTerm

> A fast, beautiful terminal news aggregator for developers and tech enthusiasts.

Browse **Hacker News**, **Crates.io**, and **Financial Markets** — all from your terminal with vim-style navigation.

## ✨ Features

| Feature | Description |
|---------|-------------|
| 🟠 **Hacker News** | Top, New, Best, Ask HN, Show HN, Jobs |
| 📦 **Crates.io** | New, Updated, Popular Rust packages |
| 📈 **Financial News** | Real-time market news via Finnhub |
| ⚡ **Fast** | Parallel requests (10 concurrent) |
| 🎨 **Beautiful TUI** | Clean, modern terminal interface |
| ⌨️ **Vim Navigation** | `j/k` to navigate, `Enter` to open |

## 📦 Installation

### Homebrew (macOS/Linux)

```bash
brew tap kj114022/finterm
brew install finterm
```

### Cargo

```bash
cargo install finterm
```

### From Source

```bash
git clone https://github.com/kj114022/finterm.git
cd finterm
cargo build --release
./target/release/finterm
```

### Binary Download

Download pre-built binaries from [Releases](https://github.com/kj114022/finterm/releases):

| Platform | Architecture |
|----------|--------------|
| macOS | ARM64, x86_64 |
| Linux | x86_64 |
| Windows | x86_64 |

## 🚀 Quick Start

```bash
finterm
```

Use number keys `1-3` to select a feed, or arrow keys to navigate.

## ⌨️ Keybindings

| Key | Action |
|-----|--------|
| `j` / `↓` | Navigate down |
| `k` / `↑` | Navigate up |
| `Enter` | Open item |
| `o` | Open in browser |
| `r` | Refresh |
| `Esc` | Go back |
| `q` | Quit |
| `?` | Help |

## ⚙️ Configuration

Config file: `~/.config/finterm/config.toml`

```toml
[finnhub]
api_key = ""           # Get free key at finnhub.io
category = "general"

[ui]
vim_mode = true

[cache]
enabled = true
max_size_mb = 50
```

> 💡 Get a free Finnhub API key at [finnhub.io/register](https://finnhub.io/register)

## 🏗️ Built With

- [Ratatui](https://github.com/ratatui-org/ratatui) - Terminal UI framework
- [Tokio](https://tokio.rs) - Async runtime
- [Reqwest](https://github.com/seanmonstar/reqwest) - HTTP client

## 📄 License

AGPL-3.0 — Free for personal and open-source use. See [LICENSE](LICENSE) for details.

---

<p align="center">
  Made with ❤️ for the terminal
</p>
