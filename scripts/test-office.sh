#!/usr/bin/env bash
# Pruebas de Office y PDF con LibreOffice, usando los mismos filtros que el
# backend. Genera las muestras con Pandoc y una hoja CSV.
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
# shellcheck source=scripts/env.sh
source "$ROOT/scripts/env.sh"
cd "$ROOT"

ulimit -c 0

SOFFICE="$(command -v soffice || command -v libreoffice || true)"
PANDOC="$(command -v pandoc || true)"

echo "== Office/PDF (LibreOffice) =="

if [ -z "$SOFFICE" ]; then
  echo "  (omitido: LibreOffice no está instalado)"
  exit 0
fi

if [ -z "$PANDOC" ]; then
  echo "  FAIL no se puede generar DOCX/PPTX sin Pandoc"
  exit 1
fi

SAMPLES="$CACHE_DIR/tmp/office"
IN="$SAMPLES/in"
OUT="$SAMPLES/out"
PROFILE="$SAMPLES/profile"
rm -rf "$SAMPLES"
mkdir -p "$IN" "$OUT"

pass=0
fail=0
failed_list=()

report() {
  local status="$1" label="$2" file="$3"
  if [ "$status" -eq 0 ] && [ -s "$file" ]; then
    pass=$((pass + 1))
    printf '  ok   %s (%s)\n' "$label" "$(du -h "$file" | cut -f1)"
  else
    fail=$((fail + 1))
    failed_list+=("$label")
    printf '  FAIL %s\n' "$label"
  fi
}

cat > "$IN/documento.md" <<'MD'
# ConvertAll

Conversor **local** de archivos.

- Imágenes
- Audio
- Vídeo
- Documentos

> Nada sale de tu equipo.
MD

cat > "$IN/hoja.csv" <<'CSV'
Nombre,Cantidad,Precio
Uno,1,10
Dos,2,20
Tres,3,30
CSV

"$PANDOC" "$IN/documento.md" -o "$IN/documento.docx" >/dev/null 2>&1
"$PANDOC" "$IN/documento.md" -o "$IN/presentacion.pptx" >/dev/null 2>&1

if [ ! -s "$IN/documento.docx" ] || [ ! -s "$IN/presentacion.pptx" ]; then
  echo "  FAIL Pandoc no generó las muestras DOCX/PPTX"
  exit 1
fi

convert() {
  local input="$1" filter="$2" extension="$3" label="$4"
  local outdir="${5:-$OUT}"
  local stem
  stem="$(basename "$input")"
  stem="${stem%.*}"
  local output="$outdir/$stem.$extension"

  "$SOFFICE" \
    --headless \
    --norestore \
    --nolockcheck \
    --nodefault \
    --nofirststartwizard \
    -env:UserInstallation="file://$PROFILE" \
    --convert-to "$filter" \
    --outdir "$outdir" \
    "$input" >/dev/null 2>&1

  report $? "$label" "$output"
}

# Word: DOCX como entrada y todas sus salidas.
convert "$IN/documento.docx" "pdf:writer_pdf_Export" "pdf" "docx -> pdf"
convert "$IN/documento.docx" "odt" "odt" "docx -> odt"
convert "$IN/documento.docx" "doc" "doc" "docx -> doc"
convert "$IN/documento.docx" "rtf" "rtf" "docx -> rtf"
convert "$IN/documento.docx" "txt:Text (encoded):UTF8" "txt" "docx -> txt"
convert "$IN/documento.docx" "html" "html" "docx -> html"

# Hoja de cálculo: CSV como entrada, luego XLSX generado.
convert "$IN/hoja.csv" "xlsx" "xlsx" "csv -> xlsx"
if [ -s "$OUT/hoja.xlsx" ]; then
  cp "$OUT/hoja.xlsx" "$IN/hoja.xlsx"
fi
convert "$IN/hoja.xlsx" "pdf:calc_pdf_Export" "pdf" "xlsx -> pdf"
convert "$IN/hoja.xlsx" "ods" "ods" "xlsx -> ods"
convert "$IN/hoja.xlsx" "xls" "xls" "xlsx -> xls"
convert "$IN/hoja.xlsx" "csv:Text - txt - csv (StarCalc):44,34,76,1" "csv" "xlsx -> csv"

# Presentaciones: PPTX como entrada y todas sus salidas.
convert "$IN/presentacion.pptx" "pdf:impress_pdf_Export" "pdf" "pptx -> pdf"
convert "$IN/presentacion.pptx" "ppt" "ppt" "pptx -> ppt"
convert "$IN/presentacion.pptx" "odp" "odp" "pptx -> odp"

# Entradas ODT/RTF/TXT/HTML generadas por LibreOffice.
for extension in odt rtf txt html; do
  if [ -s "$OUT/documento.$extension" ]; then
    cp "$OUT/documento.$extension" "$IN/entrada-$extension.$extension"
  fi
done
convert "$IN/entrada-odt.odt" "pdf:writer_pdf_Export" "pdf" "odt -> pdf"
convert "$IN/entrada-rtf.rtf" "pdf:writer_pdf_Export" "pdf" "rtf -> pdf"
convert "$IN/entrada-txt.txt" "pdf:writer_pdf_Export" "pdf" "txt -> pdf"
convert "$IN/entrada-html.html" "pdf:writer_pdf_Export" "pdf" "html -> pdf"

# Entradas XLS/ODS generadas por LibreOffice.
for extension in xls ods; do
  if [ -s "$OUT/hoja.$extension" ]; then
    cp "$OUT/hoja.$extension" "$IN/entrada-$extension.$extension"
  fi
done
convert "$IN/entrada-xls.xls" "pdf:calc_pdf_Export" "pdf" "xls -> pdf"
convert "$IN/entrada-ods.ods" "pdf:calc_pdf_Export" "pdf" "ods -> pdf"

# Entradas PPT/ODP generadas por LibreOffice.
for extension in ppt odp; do
  if [ -s "$OUT/presentacion.$extension" ]; then
    cp "$OUT/presentacion.$extension" "$IN/entrada-$extension.$extension"
  fi
done
convert "$IN/entrada-ppt.ppt" "pdf:impress_pdf_Export" "pdf" "ppt -> pdf"
convert "$IN/entrada-odp.odp" "pdf:impress_pdf_Export" "pdf" "odp -> pdf"

if [ -s "$OUT/documento.pdf" ] && head -c 4 "$OUT/documento.pdf" | grep -q '%PDF'; then
  pass=$((pass + 1))
  echo "  ok   docx -> pdf tiene cabecera %PDF"
else
  fail=$((fail + 1))
  failed_list+=("cabecera PDF")
  echo "  FAIL docx -> pdf no tiene cabecera %PDF"
fi

echo "  correctos: $pass"
echo "  fallidos:  $fail"

if [ "$fail" -gt 0 ]; then
  printf '  fallos: %s\n' "${failed_list[*]}"
  exit 1
fi
