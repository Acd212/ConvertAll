#!/usr/bin/env bash
# Descarga Pandoc portable a cache/tools para desarrollo en Linux.
# No instala nada en el sistema.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
# shellcheck source=scripts/env.sh
source "$ROOT/scripts/env.sh"

if command -v pandoc >/dev/null 2>&1; then
  echo "Pandoc ya está disponible: $(command -v pandoc)"
  exit 0
fi

mkdir -p "$CACHE_DIR/downloads" "$CACHE_DIR/tools"
cd "$CACHE_DIR/downloads"

echo "==> Consultando la última versión de Pandoc"
url="$(curl -fsSL https://api.github.com/repos/jgm/pandoc/releases/latest \
  | grep -oE 'https://[^"]*linux-amd64\.tar\.gz' \
  | head -n1)"

if [ -z "$url" ]; then
  echo "No se pudo encontrar el paquete Linux de Pandoc." >&2
  exit 1
fi

file="$(basename "$url")"
echo "==> Descargando $file"
curl -fL -o "$file" "$url"
tar -xzf "$file" -C "$CACHE_DIR/tools" --strip-components=1

# El tarball oficial coloca el binario en bin/pandoc; lo dejamos en tools/pandoc.
if [ -x "$CACHE_DIR/tools/bin/pandoc" ]; then
  mv "$CACHE_DIR/tools/bin/pandoc" "$CACHE_DIR/tools/pandoc"
  rmdir "$CACHE_DIR/tools/bin" 2>/dev/null || true
fi

if [ ! -x "$CACHE_DIR/tools/pandoc" ]; then
  echo "No se encontró el binario de Pandoc tras extraer $file." >&2
  exit 1
fi

echo "==> Pandoc instalado en $CACHE_DIR/tools/pandoc"
"$CACHE_DIR/tools/pandoc" --version | head -n1
