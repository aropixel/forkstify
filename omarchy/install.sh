#!/usr/bin/env bash
# Put forkstify on the PATH. Run by the bar widget's "Install" button, from
# the plugin directory (the clone `omarchy plugin add` made), or by hand.
#
# A **released binary** by default: no Docker, no Rust toolchain (Joel,
# 21/09/2026). The container build stays for developers, for architectures
# without an asset, and whenever the download cannot be trusted.
#
#   install.sh               the released binary, else the container build
#   install.sh --from-source the container build, always
set -euo pipefail
cd "$(dirname "$0")/.."

REPO="aropixel/forkstify"
BIN="$HOME/.local/bin/forkstify"
TMP=""

say() { printf '%s\n' "$*"; }
warn() { printf '%s\n' "$*" >&2; }
cleanup() { [ -n "$TMP" ] && rm -rf "$TMP"; }
trap cleanup EXIT

# Installed by a package manager (the AUR package, say)? Leave it alone.
already_packaged() {
  local found
  found=$(command -v forkstify 2>/dev/null) || return 1
  case "$found" in
    "$BIN" | "$HOME"/.local/bin/*) return 1 ;;
    *) say "forkstify is already installed by a package: $found"; return 0 ;;
  esac
}

version_wanted() {
  sed -n 's/.*"version"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' manifest.json | head -1
}

from_release() {
  [ "$(uname -m)" = "x86_64" ] || { warn "no released binary for $(uname -m)"; return 1; }
  command -v curl >/dev/null 2>&1 || { warn "curl missing"; return 1; }
  command -v sha256sum >/dev/null 2>&1 || { warn "sha256sum missing"; return 1; }

  local version name base
  version=$(version_wanted)
  [ -n "$version" ] || { warn "no version in manifest.json"; return 1; }
  name="forkstify-$version-x86_64-linux.tar.gz"
  base="https://github.com/$REPO/releases/download/v$version"

  TMP=$(mktemp -d)
  say "fetching forkstify $version…"
  curl -fsSL "$base/$name" -o "$TMP/$name" || { warn "download failed"; return 1; }
  curl -fsSL "$base/SHA256SUMS" -o "$TMP/SHA256SUMS" || { warn "no checksums published"; return 1; }

  # the binary is only as trustworthy as its checksum: never install past a
  # mismatch, build from source instead
  ( cd "$TMP" && grep -F " $name" SHA256SUMS | sha256sum -c --status - ) || {
    warn "checksum mismatch for $name — refusing it"
    return 1
  }

  tar -xzf "$TMP/$name" -C "$TMP" forkstify
  mkdir -p "$(dirname "$BIN")"
  install -m755 "$TMP/forkstify" "$BIN"
  say "forkstify $version installed: $BIN"
}

from_source() {
  command -v docker >/dev/null 2>&1 || {
    warn "docker missing — \"omarchy install docker\", then try again"
    return 1
  }
  say "building in the forkstify-build container (a few minutes the first time)…"
  bin/build
  mkdir -p "$(dirname "$BIN")"
  ln -sf "$PWD/target/release/forkstify" "$BIN"
  say "forkstify installed from source: $BIN"
}

case "${1:-}" in
  --from-source) from_source ;;
  "") already_packaged || from_release || { say "falling back to the container build"; from_source; } ;;
  *) warn "usage: install.sh [--from-source]"; exit 2 ;;
esac

case ":$PATH:" in
  *":$HOME/.local/bin:"*) ;;
  *) say "note: $HOME/.local/bin is not on your PATH" ;;
esac
