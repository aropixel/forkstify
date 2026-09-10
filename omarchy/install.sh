#!/usr/bin/env bash
# Build forkstify inside its container and put the binary on the PATH.
# Run by the bar widget's « Installer » button, from the plugin directory
# (the clone `omarchy plugin add` made), or by hand.
set -euo pipefail
cd "$(dirname "$0")/.."

if ! command -v docker >/dev/null 2>&1; then
  echo "docker manquant — « omarchy install docker », puis réessayer" >&2
  exit 1
fi
echo "construction dans le conteneur forkstify-build (quelques minutes la première fois)…"
bin/build
mkdir -p "$HOME/.local/bin"
ln -sf "$PWD/target/release/forkstify" "$HOME/.local/bin/forkstify"
echo "forkstify installé : $HOME/.local/bin/forkstify"
