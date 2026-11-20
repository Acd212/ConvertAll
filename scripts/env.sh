#!/usr/bin/env bash
# Redirige toda la caché de desarrollo al directorio ./cache del proyecto.
# Uso: source scripts/env.sh
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
export PROJECT_ROOT="$ROOT"
export CACHE_DIR="$ROOT/cache"

export CARGO_HOME="$CACHE_DIR/cargo-home"
export CARGO_TARGET_DIR="$CACHE_DIR/cargo-target"
export BUN_INSTALL_CACHE_DIR="$CACHE_DIR/bun"
export npm_config_cache="$CACHE_DIR/npm"
export TMPDIR="$CACHE_DIR/tmp"
export XDG_CACHE_HOME="$CACHE_DIR/xdg"

mkdir -p \
  "$CARGO_HOME" \
  "$CARGO_TARGET_DIR" \
  "$BUN_INSTALL_CACHE_DIR" \
  "$npm_config_cache" \
  "$TMPDIR" \
  "$XDG_CACHE_HOME" \
  "$CACHE_DIR/tools" \
  "$CACHE_DIR/downloads"

# Herramientas portables opcionales (pandoc, etc.)
case ":$PATH:" in
  *":$CACHE_DIR/tools:"*) ;;
  *) export PATH="$CACHE_DIR/tools:$PATH" ;;
esac
