#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
# shellcheck source=scripts/env.sh
source "$ROOT/scripts/env.sh"
cd "$ROOT"

echo "==> Comprobando TypeScript/Svelte"
bun run check

echo "==> Compilando frontend"
bun run build

echo "==> Comprobando Rust"
cargo check --manifest-path src-tauri/Cargo.toml

echo "==> Todo correcto"
