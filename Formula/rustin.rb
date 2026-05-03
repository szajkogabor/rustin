class Rustin < Formula
  desc "A fast, modern, cross-platform to-do list manager written in Rust"
  homepage "https://github.com/szajkogabor/rustin"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/szajkogabor/rustin/releases/download/#{version}/rustin-#{version}-aarch64-apple-darwin.tar.gz"
      # sha256 "PLACEHOLDER" # Updated automatically by CI
    end
  end

  on_linux do
    if Hardware::CPU.intel?
      url "https://github.com/szajkogabor/rustin/releases/download/#{version}/rustin-#{version}-x86_64-unknown-linux-gnu.tar.gz"
      # sha256 "PLACEHOLDER" # Updated automatically by CI
    end
  end

  def install
    bin.install "rustin"
  end

  test do
    system bin / "rustin", "--version"
  end
end

