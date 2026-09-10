#!/usr/bin/env bash
# Build forkstify inside its container and put the binary on the PATH.
# Run by the bar widget's "Install" button, from the plugin directory
# (the clone `omarchy plugin add` made), or by hand.
set -euo pipefail
cd "$(dirname "$0")/.."

if ! command -v docker >/dev/null 2>&1; then
  echo "docker missing — \"omarchy install docker\", then try again" >&2
  exit 1
fi
echo "building in the forkstify-build container (a few minutes the first time)…"
bin/build
mkdir -p "$HOME/.local/bin"
ln -sf "$PWD/target/release/forkstify" "$HOME/.local/bin/forkstify"
echo "forkstify installed: $HOME/.local/bin/forkstify"
