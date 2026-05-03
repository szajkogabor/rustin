#!/usr/bin/env bash
#
# Updates the Homebrew formula with the correct version, URLs, and SHA256
# checksums from a GitHub release.
#
# Usage: ./scripts/update-homebrew.sh <version>
# Example: ./scripts/update-homebrew.sh 1.0.0
#
set -euo pipefail

VERSION="${1:?Usage: $0 <version>}"
REPO="szajkogabor/rustin"
FORMULA="Formula/rustin.rb"

MACOS_ASSET="rustin-${VERSION}-aarch64-apple-darwin.tar.gz"
LINUX_ASSET="rustin-${VERSION}-x86_64-unknown-linux-gnu.tar.gz"

BASE_URL="https://github.com/${REPO}/releases/download/${VERSION}"

echo "Downloading checksums from release ${VERSION}..."
CHECKSUMS=$(curl -sL "${BASE_URL}/SHA256SUMS")

MACOS_SHA=$(echo "$CHECKSUMS" | grep "$MACOS_ASSET" | awk '{print $1}')
LINUX_SHA=$(echo "$CHECKSUMS" | grep "$LINUX_ASSET" | awk '{print $1}')

if [[ -z "$MACOS_SHA" || -z "$LINUX_SHA" ]]; then
  echo "ERROR: Could not find checksums for version ${VERSION}" >&2
  echo "Available checksums:" >&2
  echo "$CHECKSUMS" >&2
  exit 1
fi

echo "macOS (arm64): ${MACOS_SHA}"
echo "Linux (x86_64): ${LINUX_SHA}"

cat > "$FORMULA" <<EOF
class Rustin < Formula
  desc "A fast, modern, cross-platform to-do list manager written in Rust"
  homepage "https://github.com/${REPO}"
  version "${VERSION}"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "${BASE_URL}/${MACOS_ASSET}"
      sha256 "${MACOS_SHA}"
    end
  end

  on_linux do
    if Hardware::CPU.intel?
      url "${BASE_URL}/${LINUX_ASSET}"
      sha256 "${LINUX_SHA}"
    end
  end

  def install
    bin.install "rustin"
  end

  test do
    system bin / "rustin", "--version"
  end
end
EOF

echo "Formula updated: ${FORMULA}"

