<p align="center">
  <img src="https://img.shields.io/badge/rust-1.75+-orange.svg" alt="Rust">
  <img src="https://img.shields.io/badge/license-AGPL--3.0-blue.svg" alt="License">
  <img src="https://img.shields.io/github/v/release/kj114022/finterm" alt="Release">
  <img src="https://img.shields.io/badge/platform-macOS%20%7C%20Linux%20%7C%20Windows-lightgrey" alt="Platform">
</p>

# FinTerm

> A fast, beautiful terminal news aggregator for developers and tech enthusiasts.

## Installation

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

## Quick Start

```bash
finterm
```

Use number keys `1-3` to select a feed, or arrow keys to navigate.

## Keybindings

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

## Configuration

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

> Get a free Finnhub API key at [finnhub.io/register](https://finnhub.io/register)


## License

AGPL-3.0 — Free for personal and open-source use. See [LICENSE](LICENSE) for details.

---

<p align="center">
  Made with love for the terminal.
</p>
