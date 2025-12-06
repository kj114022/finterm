class Finterm < Formula
  desc "Fast terminal news aggregator for Hacker News and financial markets"
  homepage "https://github.com/kj114022/finterm"
  url "https://github.com/kj114022/finterm/archive/refs/tags/v0.1.0.tar.gz"
  sha256 "PLACEHOLDER_SHA256"
  license "AGPL-3.0"
  head "https://github.com/kj114022/finterm.git", branch: "main"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    assert_match "finterm", shell_output("#{bin}/finterm --version")
  end
end
