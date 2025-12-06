# Homebrew Tap Setup for FinTerm

This directory contains the Homebrew formula for FinTerm.

## For Users

```bash
brew tap kj114022/finterm https://github.com/kj114022/finterm
brew install finterm
```

## For Maintainers

### Creating a Release

1. **Create a git tag and push:**
   ```bash
   git tag v0.1.0
   git push origin v0.1.0
   ```

2. **Download the tarball and calculate checksum:**
   ```bash
   curl -sL https://github.com/kj114022/finterm/archive/refs/tags/v0.1.0.tar.gz -o finterm-0.1.0.tar.gz
   shasum -a 256 finterm-0.1.0.tar.gz
   ```

3. **Update `finterm.rb` with the sha256 checksum**

4. **Or use `brew tap` with HEAD:**
   ```bash
   brew install --HEAD kj114022/finterm/finterm
   ```

## Alternative: Separate Tap Repository

For a cleaner setup, create a separate repo `homebrew-finterm`:

```
homebrew-finterm/
└── Formula/
    └── finterm.rb
```

Then users can: `brew tap kj114022/finterm && brew install finterm`
