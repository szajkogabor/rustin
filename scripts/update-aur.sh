#!/usr/bin/env bash
#
# Updates the AUR PKGBUILD with the correct version and SHA256 checksum.
#
# Usage: ./scripts/update-aur.sh <version>
# Example: ./scripts/update-aur.sh 1.0.0
#
set -euo pipefail

VERSION="${1:?Usage: $0 <version>}"
REPO="szajkogabor/rustin"
PKGBUILD="aur/PKGBUILD"

ASSET="rustin-${VERSION}-x86_64-unknown-linux-gnu.tar.gz"
BASE_URL="https://github.com/${REPO}/releases/download/${VERSION}"

echo "Downloading checksums from release ${VERSION}..."
CHECKSUMS=$(curl -sL "${BASE_URL}/SHA256SUMS")

SHA=$(echo "$CHECKSUMS" | grep "$ASSET" | awk '{print $1}')

if [[ -z "$SHA" ]]; then
  echo "ERROR: Could not find checksum for ${ASSET}" >&2
  echo "Available checksums:" >&2
  echo "$CHECKSUMS" >&2
  exit 1
fi

echo "Linux (x86_64): ${SHA}"

sed -i "s/^pkgver=.*/pkgver=${VERSION}/" "$PKGBUILD"
sed -i "s/^sha256sums=.*/sha256sums=('${SHA}')/" "$PKGBUILD"

echo "PKGBUILD updated: ${PKGBUILD}"

