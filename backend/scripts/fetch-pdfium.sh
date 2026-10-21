#!/bin/sh
# Downloads a pinned pdfium prebuilt into backend/.pdfium/
# Usage: fetch-pdfium.sh [os] [arch]
#   os:   linux (default) | mac
#   arch: x64 (default) | arm64
set -eu

VER="chromium/8044"                    # pin; bump deliberately
OS="${1:-linux}"
case "$(uname -m)" in
  x86_64|amd64)      HOST_ARCH=x64 ;;
  aarch64|arm64)     HOST_ARCH=arm64 ;;
  *)                 HOST_ARCH=x64 ;;
esac
ARCH="${2:-$HOST_ARCH}"
DEST="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)/.pdfium"

mkdir -p "$DEST"
URL="https://github.com/bblanchon/pdfium-binaries/releases/download/${VER}/pdfium-${OS}-${ARCH}.tgz"
echo "fetching $URL"
archive="$(mktemp)"
trap 'rm -f "$archive"' EXIT HUP INT TERM
# A stalled release-asset connection must not leave CI waiting indefinitely.
# Finish the download before extraction so a failed curl cannot be hidden by a pipe.
curl -fsSL --connect-timeout 15 --max-time 180 --retry 3 --retry-all-errors "$URL" -o "$archive"
tar -xzf "$archive" -C "$DEST"
echo "libpdfium at $DEST/lib/"
