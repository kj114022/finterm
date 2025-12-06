# FinTerm

A terminal-based news aggregator for Hacker News and financial markets.

![Rust](https://img.shields.io/badge/rust-1.75+-orange.svg)
![License](https://img.shields.io/badge/license-MIT-blue.svg)
[![CI](https://github.com/kj114022/finterm/actions/workflows/ci.yml/badge.svg)](https://github.com/kj114022/finterm/actions)

## Features

- **Hacker News** - Browse Top, New, Best, Ask HN, Show HN, and Jobs
- **Financial News** - Real-time market news via Finnhub API
- **Performance** - Parallel requests for fast loading
- **Terminal UI** - Clean, keyboard-driven interface
- **Vim Keybindings** - Navigate with j/k

## Installation

### MacPorts

```bash
sudo port install finterm
```

### Homebrew

```bash
brew tap kj114022/finterm
brew install finterm
```

### Cargo

```bash
cargo install finterm
```

### Binary Download

Download from [Releases](https://github.com/kj114022/finterm/releases):

| Platform | File |
|----------|------|
| macOS (ARM) | `finterm-macos-aarch64.tar.gz` |
| macOS (Intel) | `finterm-macos-x86_64.tar.gz` |
| Linux | `finterm-linux-x86_64.tar.gz` |
| Windows | `finterm-windows-x86_64.exe` |

### From Source

```bash
git clone https://github.com/kj114022/finterm.git
cd finterm
cargo build --release
sudo cp target/release/finterm /usr/local/bin/
```

## Usage

```bash
finterm
```

### Keybindings

| Key | Action |
|-----|--------|
| `j/k` | Navigate down/up |
| `Enter` | Open item |
| `o` | Open in browser |
| `Esc` | Go back |
| `r` | Refresh |
| `q` | Quit |

## Configuration

Config file: `~/.config/finterm/config.toml`

```toml
[finnhub]
api_key = ""
category = "general"

[ui]
vim_mode = true

[cache]
enabled = true
max_size_mb = 50
```

Get a Finnhub API key at [finnhub.io](https://finnhub.io/register).

## License

AGPL-3.0 with Commons Clause

- Free to use, modify, and distribute for non-commercial purposes
- Must share source code of derivative works
- Commercial use requires written permission
- See [LICENSE](LICENSE) for details
