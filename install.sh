#!/bin/sh
# OpenCase installer — downloads the latest pre-built binary from GitHub
# Releases, verified against its SHA-256 checksum. No Rust toolchain required.
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/realmorrisliu/opencase/main/install.sh | sh
#   # or: VERSION=v0.1.0 sh install.sh [install-dir]
set -e

REPO="realmorrisliu/opencase"
VERSION="${VERSION:-latest}"
DEST="${1:-/usr/local/bin}"

os="$(uname -s | tr '[:upper:]' '[:lower:]')"
arch="$(uname -m)"
case "$arch" in
  x86_64 | amd64) arch="x86_64" ;;
  aarch64 | arm64) arch="arm64" ;;
  *) echo "unsupported architecture: $arch" >&2; exit 1 ;;
esac
case "$os" in
  darwin | linux) ;;
  *) echo "unsupported OS: $os (Windows users: download the binary from the releases page)" >&2; exit 1 ;;
esac

if [ "$VERSION" = "latest" ]; then
  url="https://github.com/$REPO/releases/latest/download/opencase-${os}-${arch}"
else
  url="https://github.com/$REPO/releases/${VERSION}/download/opencase-${os}-${arch}"
fi

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

echo "Downloading $url"
if command -v curl >/dev/null 2>&1; then
  curl -fsSL "$url" -o "$tmp/opencase"
else
  wget -q "$url" -O "$tmp/opencase"
fi

echo "Verifying SHA-256 checksum"
if command -v curl >/dev/null 2>&1; then
  curl -fsSL "$url.sha256" -o "$tmp/opencase.sha256"
else
  wget -q "$url.sha256" -O "$tmp/opencase.sha256"
fi
(cd "$tmp" && shasum -a 256 -c opencase.sha256)

chmod +x "$tmp/opencase"
if install -m 755 "$tmp/opencase" "$DEST/opencase" 2>/dev/null; then
  echo "Installed opencase to $DEST/opencase"
else
  echo "Need permissions to write to $DEST — retrying with sudo"
  sudo install -m 755 "$tmp/opencase" "$DEST/opencase"
  echo "Installed opencase to $DEST/opencase"
fi

echo "Next: run 'opencase init' in your repo, then 'opencase skill install'"
