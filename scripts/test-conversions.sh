#!/usr/bin/env bash
# Pruebas funcionales de los motores con los mismos argumentos que usa el backend.
# Genera muestras y conversiones dentro de cache/tmp/samples.
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
# shellcheck source=scripts/env.sh
source "$ROOT/scripts/env.sh"
cd "$ROOT"

# Evita volcados de núcleo si un motor falla (algunos entornos matan la sesión).
ulimit -c 0

SAMPLES="$CACHE_DIR/tmp/samples"
IN="$SAMPLES/in"
OUT="$SAMPLES/out"
rm -rf "$SAMPLES"
mkdir -p "$IN" "$OUT"

FFMPEG="$(command -v ffmpeg)"
MAGICK="$(command -v magick)"
PANDOC="$(command -v pandoc || true)"

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

check_value() {
  local label="$1" actual="$2" expected="$3"
  if [ "$actual" = "$expected" ]; then
    pass=$((pass + 1))
    printf '  ok   %s (%s)\n' "$label" "$actual"
  else
    fail=$((fail + 1))
    failed_list+=("$label")
    printf '  FAIL %s (esperado %s, obtenido %s)\n' "$label" "$expected" "$actual"
  fi
}

# ---------- Muestras ----------
"$MAGICK" -size 640x360 gradient:'#f97316-#171412' -quality 92 "$IN/imagen.png" 2>/dev/null
"$FFMPEG" -hide_banner -loglevel error -y -f lavfi -i "sine=frequency=440:duration=2" "$IN/audio.wav"
"$FFMPEG" -hide_banner -loglevel error -y \
  -f lavfi -i "testsrc2=size=640x360:rate=25:duration=3" \
  -f lavfi -i "sine=frequency=440:duration=3" \
  -c:v libx264 -pix_fmt yuv420p -c:a aac -shortest "$IN/video.mp4"
"$FFMPEG" -hide_banner -loglevel error -y \
  -f lavfi -i "testsrc2=size=1920x1080:rate=25:duration=1" \
  -c:v libx264 -pix_fmt yuv420p "$IN/video-hd.mp4"
"$FFMPEG" -hide_banner -loglevel error -y \
  -f lavfi -i "testsrc2=size=1080x1920:rate=25:duration=1" \
  -c:v libx264 -pix_fmt yuv420p "$IN/video-vertical.mp4"
cat > "$IN/documento.md" <<'MD'
# ConvertAll

Conversor **local** de archivos.

- Imágenes
- Audio
- Vídeo
- Documentos

> Nada sale de tu equipo.
MD

if [ -z "$PANDOC" ]; then
  echo "Pandoc no disponible: se omiten las pruebas de documentos."
fi

echo "== Imagen (ImageMagick) =="
for target in jpg png webp avif heic jxl gif bmp tiff ico tga dds exr hdr ppm pgm pbm pnm xbm xpm wbmp pcx cur eps; do
  out="$OUT/imagen.$target"
  extra=()
  if [ "$target" = "ico" ] || [ "$target" = "cur" ]; then
    extra=(-define "icon:auto-resize=256,128,64,48,32,16")
  fi
  "$MAGICK" "$IN/imagen.png" -auto-orient -strip -quality 80 "${extra[@]}" "$out" >/dev/null 2>&1
  report $? "imagen.png -> $target" "$out"
done

echo
echo "== Audio (FFmpeg) =="
for target in mp3 wav flac ogg opus aac m4a wma aiff ac3 dts amr mka caf wv m4b; do
  out="$OUT/audio.$target"
  case "$target" in
    mp3) codec=(-c:a libmp3lame -q:a 2) ;;
    wav) codec=(-c:a pcm_s16le) ;;
    flac) codec=(-c:a flac) ;;
    ogg) codec=(-c:a libvorbis -q:a 4) ;;
    opus) codec=(-c:a libopus -b:a 128k) ;;
    aac|m4a|m4b) codec=(-c:a aac -b:a 128k) ;;
    wma) codec=(-c:a wmav2 -b:a 192k) ;;
    aiff) codec=(-c:a pcm_s16be) ;;
    ac3) codec=(-c:a ac3 -b:a 192k) ;;
    dts) codec=(-c:a dca -strict -2 -b:a 768k) ;;
    amr) codec=(-c:a libopencore_amrnb -ar 8000 -ac 1 -b:a 12.2k) ;;
    mka) codec=(-c:a copy) ;;
    caf) codec=(-c:a pcm_s16le) ;;
    wv) codec=(-c:a wavpack) ;;
  esac
  "$FFMPEG" -hide_banner -loglevel error -y -i "$IN/audio.wav" \
    -progress pipe:1 -nostats -map_metadata -1 -vn "${codec[@]}" "$out" >/dev/null 2>&1
  report $? "audio.wav -> $target" "$out"
done

echo
echo "== Vídeo (FFmpeg) =="
for target in mp4 mkv webm mov avi wmv flv m4v mpg ts m2ts 3gp ogv vob asf f4v; do
  out="$OUT/video.$target"
  case "$target" in
    mp4|m4v|mov|f4v) codec=(-c:v libx264 -preset medium -crf 24 -pix_fmt yuv420p -c:a aac -b:a 192k -movflags +faststart) ;;
    mkv) codec=(-c:v libx264 -preset medium -crf 24 -pix_fmt yuv420p -c:a aac -b:a 192k) ;;
    webm) codec=(-c:v libvpx-vp9 -crf 24 -b:v 0 -c:a libopus -b:a 128k) ;;
    avi) codec=(-c:v libx264 -crf 24 -c:a libmp3lame -q:a 4) ;;
    wmv|asf) codec=(-c:v wmv2 -b:v 2M -c:a wmav2 -b:a 192k) ;;
    flv) codec=(-c:v libx264 -crf 24 -c:a aac -b:a 128k) ;;
    mpg|vob) codec=(-c:v mpeg2video -q:v 3 -c:a mp2 -b:a 192k) ;;
    ts|m2ts) codec=(-c:v libx264 -crf 24 -c:a aac -b:a 192k) ;;
    3gp) codec=(-c:v libx264 -crf 24 -c:a aac -ar 44100 -ac 1) ;;
    ogv) codec=(-c:v libtheora -q:v 6 -c:a libvorbis -q:a 4) ;;
  esac
  "$FFMPEG" -hide_banner -loglevel error -y -i "$IN/video.mp4" \
    -progress pipe:1 -nostats -map_metadata -1 "${codec[@]}" "$out" >/dev/null 2>&1
  report $? "video.mp4 -> $target" "$out"
done

# Vídeo -> audio y GIF
out="$OUT/video.mp3"
"$FFMPEG" -hide_banner -loglevel error -y -i "$IN/video.mp4" -progress pipe:1 -nostats \
  -map_metadata -1 -vn -c:a libmp3lame -q:a 2 "$out" >/dev/null 2>&1
report $? "video.mp4 -> mp3" "$out"

out="$OUT/video.gif"
"$FFMPEG" -hide_banner -loglevel error -y -i "$IN/video.mp4" -progress pipe:1 -nostats \
  -map_metadata -1 -an -vf "fps=15,scale=1280:-2:flags=lanczos" -loop 0 "$out" >/dev/null 2>&1
report $? "video.mp4 -> gif" "$out"

echo
echo "== Opciones por formato =="

video_hd_out="$OUT/video-hd-720.mp4"
"$FFMPEG" -hide_banner -loglevel error -y -i "$IN/video-hd.mp4" -progress pipe:1 -nostats \
  -map_metadata -1 \
  -vf "scale=w='if(gt(iw,ih),-2,min(iw,720))':h='if(gt(iw,ih),min(ih,720),-2)':flags=lanczos" \
  -c:v libx264 -preset medium -crf 24 -pix_fmt yuv420p -movflags +faststart "$video_hd_out" >/dev/null 2>&1
check_value "video 1920x1080 -> 720p" \
  "$(ffprobe -v error -select_streams v:0 -show_entries stream=width,height -of csv=p=0:s=x "$video_hd_out")" \
  "1280x720"

video_vertical_out="$OUT/video-vertical-720.mp4"
"$FFMPEG" -hide_banner -loglevel error -y -i "$IN/video-vertical.mp4" -progress pipe:1 -nostats \
  -map_metadata -1 \
  -vf "scale=w='if(gt(iw,ih),-2,min(iw,720))':h='if(gt(iw,ih),min(ih,720),-2)':flags=lanczos" \
  -c:v libx264 -preset medium -crf 24 -pix_fmt yuv420p -movflags +faststart "$video_vertical_out" >/dev/null 2>&1
check_value "vídeo vertical 1080x1920 -> 720p" \
  "$(ffprobe -v error -select_streams v:0 -show_entries stream=width,height -of csv=p=0:s=x "$video_vertical_out")" \
  "720x1280"

image_opt="$OUT/imagen-480.jpg"
"$MAGICK" "$IN/imagen.png" -auto-orient -resize "480x480>" -strip -quality 80 "$image_opt" >/dev/null 2>&1
check_value "imagen 640x360 -> 480 px" \
  "$("$MAGICK" identify -format '%wx%h' "$image_opt")" "480x270"

audio_opt="$OUT/audio-128k.mp3"
"$FFMPEG" -hide_banner -loglevel error -y -i "$IN/audio.wav" -progress pipe:1 -nostats \
  -map_metadata -1 -vn -c:a libmp3lame -b:a 128k "$audio_opt" >/dev/null 2>&1
check_value "audio.wav -> 128 kbps" \
  "$(ffprobe -v error -select_streams a:0 -show_entries stream=bit_rate -of default=nw=1:nk=1 "$audio_opt")" \
  "128000"

gif_opt="$OUT/video-640.gif"
"$FFMPEG" -hide_banner -loglevel error -y -i "$IN/video.mp4" -progress pipe:1 -nostats \
  -map_metadata -1 -an \
  -vf "fps=15,scale=w='min(iw,640)':h='min(ih,640)':force_original_aspect_ratio=decrease:force_divisible_by=2:flags=lanczos" \
  -loop 0 "$gif_opt" >/dev/null 2>&1
check_value "video -> gif 640 px" \
  "$(ffprobe -v error -select_streams v:0 -show_entries stream=width,height -of csv=p=0:s=x "$gif_opt")" \
  "640x360"

echo
echo "== Documentos (Pandoc) =="
if [ -n "$PANDOC" ]; then
  for target in docx odt rtf html txt rst org tex epub ipynb json docbook mediawiki textile asciidoc man opml native pptx; do
    out="$OUT/documento.$target"
    extra=()
    case "$target" in
      html|epub) extra=(--standalone) ;;
      txt) extra=(-t plain) ;;
      md) extra=(-t gfm) ;;
    esac
    "$PANDOC" "$IN/documento.md" -o "$out" "${extra[@]}" >/dev/null 2>&1
    report $? "documento.md -> $target" "$out"
  done
else
  echo "  (omitido)"
fi

echo
if command -v soffice >/dev/null 2>&1 || command -v libreoffice >/dev/null 2>&1; then
  "$ROOT/scripts/test-office.sh" || fail=$((fail + 1))
else
  echo "== Office/PDF (LibreOffice) =="
  echo "  (omitido: LibreOffice no está instalado)"
fi

echo
echo "== Resultado =="
echo "  correctos: $pass"
echo "  fallidos:  $fail"
if [ "$fail" -gt 0 ]; then
  printf '  fallos: %s\n' "${failed_list[*]}"
  exit 1
fi
