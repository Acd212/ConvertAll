# ConvertAll

Conversor de archivos **local** para Linux y Windows. Sin subidas, sin cuentas y sin telemetría.

Usa motores nativos en lugar de WebAssembly:

- **FFmpeg** para audio y vídeo.
- **ImageMagick** y **libvips** para imágenes.
- **Pandoc** para documentos de marcado.
- **LibreOffice** del sistema para Office y PDF. En Windows, si **Microsoft Office** está instalado, se usa automáticamente para Word, Excel y PowerPoint.

La interfaz está construida con **Tauri 2 + Svelte 5 + Tailwind CSS 4** y el backend con **Rust**.

![Captura de ConvertAll](docs/captura.png)

## Estado

Proyecto en **fase 2**: shell funcional, detección de motores, cola de archivos, selección de formato y ajustes por archivo, progreso, cancelación y conversión local, con Office/PDF mediante LibreOffice o Microsoft Office.

Verificado en Linux y Windows:

- `svelte-check` sin errores, build de Vite y `cargo check` correctos.
- 32 pruebas Rust correctas.
- 82 conversiones reales correctas en Linux (imagen, audio, vídeo y documentos).
- 4 archivos convertidos desde la GUI y desde el AppImage con resolución, tamaño y bitrate (PNG→JPG, WAV→MP3, MP4→MKV, MD→DOCX).
- El AppImage aísla los motores de sus propias bibliotecas para que libvips e ImageMagick usen las del sistema.
- Paquete portable de Windows (`.zip`) con FFmpeg, Pandoc e ImageMagick incluidos.
- Motores de Windows probados bajo Wine: 13/13 conversiones correctas.
- La app arranca y detecta FFmpeg, ImageMagick, libvips, Pandoc, LibreOffice y Microsoft Office (Windows).
- El CI de Linux instala LibreOffice y ejecuta `scripts/test-office.sh`: 23/23 conversiones correctas (DOCX/DOC/ODT/RTF/TXT/HTML, CSV/XLSX/XLS/ODS, PPTX/PPT/ODP y PDF).

### Ajustes por archivo

Cada archivo muestra solo las opciones válidas para su formato de destino:

- **Vídeo:** 2160p, 1440p, 1080p, 720p, 480p y 360p. Limita el lado corto y no amplía el vídeo.
- **Imagen:** lado mayor entre 3840 y 480 px. No amplía la imagen; ICO y CUR mantienen sus tamaños de icono.
- **Audio:** bitrate según el códec (MP3, AAC/M4A, Opus, Vorbis, WMA, AC-3 y DTS). «Automático» usa la calidad global.

### Office y PDF

- **Entradas:** DOCX, DOC, ODT, RTF, TXT, HTML, XLSX, XLS, ODS, CSV, PPTX, PPT y ODP.
- **Salidas:** las mismas familias, más PDF.
- **PDF es solo salida.** La importación de PDF de LibreOffice produce un dibujo, no un documento editable fiable.
- Cada archivo muestra únicamente los destinos válidos para su familia: Word, hoja de cálculo o presentación.
- **Linux:** se usa el LibreOffice del sistema (`soffice` o `libreoffice`). No se empaqueta.
- **Windows:** se detecta Microsoft Office en el registro y en las rutas habituales. Si Word, Excel o PowerPoint están instalados, se usan para su familia; si no, se recurre a LibreOffice. Tampoco se empaqueta Office.
- Si no hay ninguno de los dos, la app sigue funcionando: solo se ocultan los destinos de Office/PDF y se muestra un aviso en «Motores».

Pendiente:

- Presets y perfiles reutilizables; más ajustes por formato (códec, FPS, canales).

## Descargas

A partir de **v0.1.2** se mantiene una sola release con la última versión: [Releases](https://github.com/Acd212/ConvertAll/releases/latest).

- Windows: `ConvertAll_x.y.z_x64-portable.zip`, sin instalador. Descomprime y ejecuta `ConvertAll.exe`.
- Linux: `ConvertAll_x.y.z_amd64.AppImage`.

Las versiones nuevas reemplazan los archivos de esa misma release; no se crea una entrada de release por versión. Ya no se publican MSI, NSIS ni DEB.

En Windows, el zip necesita WebView2. Windows 11 lo incluye y la mayoría de Windows 10 también; si falta, instálalo desde [Microsoft Edge WebView2](https://developer.microsoft.com/microsoft-edge/webview2/).

El AppImage incluye WebKit2GTK 4.1, GTK3 y libsoup, así que no requiere instalar WebKit en el sistema. Los motores (FFmpeg, ImageMagick, libvips, Pandoc y LibreOffice) sí siguen siendo del sistema. En Windows, FFmpeg, Pandoc e ImageMagick van incluidos; LibreOffice y Microsoft Office son opcionales.

### Instalar LibreOffice

En Linux basta con Writer, Calc e Impress:

```bash
# Debian/Ubuntu
sudo apt install libreoffice-writer libreoffice-calc libreoffice-impress

# Arch/CachyOS
sudo pacman -S libreoffice-fresh
```

En Windows, descárgalo desde [libreoffice.org](https://es.libreoffice.org/descarga/libreoffice/). Si ya tienes Microsoft Office instalado, no necesitas LibreOffice para Word, Excel o PowerPoint.

## Requisitos de desarrollo

- Rust estable (`rustc`, `cargo`).
- Bun (o Node 20+ y npm/pnpm).
- Linux: GTK3, WebKit2GTK 4.1, libsoup3, librsvg, patchelf.
- Motores: `ffmpeg`, `magick`, `vips`, `pandoc` y, para Office/PDF, LibreOffice (`soffice`). Todos son opcionales salvo que se use su categoría.

En CachyOS/Arch:

```bash
sudo pacman -S --needed ffmpeg imagemagick libvips
# Pandoc no es obligatorio para compilar:
./scripts/fetch-pandoc-linux.sh
# Office/PDF (opcional):
sudo pacman -S libreoffice-fresh
```

## Desarrollo

```bash
bun install
./scripts/dev.sh
```

`scripts/env.sh` redirige `CARGO_HOME`, `CARGO_TARGET_DIR`, la caché de Bun/npm y `TMPDIR` a `./cache/`.

También puedes abrir archivos directamente desde la terminal (o con «Abrir con…»):

```bash
convertall imagen.png audio.wav video.mp4 documento.md
```

## Comprobar

```bash
./scripts/check.sh
```

## Probar conversiones

Genera muestras con FFmpeg/ImageMagick y prueba todos los destinos del catálogo:

```bash
./scripts/test-conversions.sh
```

Los archivos de prueba se crean en `cache/tmp/samples/`. Si LibreOffice está instalado, también se ejecutan las pruebas de Office/PDF. Para ejecutarlas por separado:

```bash
./scripts/test-office.sh
```

## Compilar

```bash
./scripts/build.sh
```

Los paquetes locales se generan dentro de `cache/cargo-target/release/bundle/`. El CI de Windows crea el `.zip` portable en `cache/portable/`.

## Windows

El repositorio incluye `.github/workflows/build.yml`. En cada push a `main`:

1. Se descargan FFmpeg, Pandoc e ImageMagick portables.
2. Se compila la app en `windows-latest`.
3. Se empaqueta `ConvertAll.exe`, `tools/` y `licenses/` en un `.zip` portable.

El zip no se instala: se descomprime en cualquier carpeta y se ejecuta `ConvertAll.exe`. La carpeta `tools/` debe quedar junto al ejecutable.

Los binarios descargados no se versionan: `src-tauri/resources/windows/` está en `.gitignore`.

## Estructura

```text
src/                 Interfaz Svelte
src/lib/             Tipos, API y componentes
src-tauri/src/       Backend Rust
  formats.rs         Catálogo y reglas de conversión
  engines.rs         Detección de FFmpeg/ImageMagick/libvips/Pandoc/LibreOffice/Office
  capabilities.rs    Formatos/codecs reales de cada motor
  convert.rs         Ejecución, progreso y cancelación
  commands.rs        Comandos de Tauri
scripts/             Entorno, builds, pruebas y descarga de herramientas
assets/icon.svg      Fuente de los iconos
docs/captura.png     Captura de la interfaz
cache/               Caché y artefactos (ignorado por git)
```

## Licencia

MIT. ConvertAll es un proyecto original inspirado en la experiencia de VERT; no reutiliza su código (AGPL-3.0).

Los motores externos conservan sus propias licencias: FFmpeg (GPL/LGPL según build), ImageMagick, libvips (LGPL), Pandoc (GPL-2.0+), LibreOffice (MPL-2.0/LGPL) y Microsoft Office (licencia del usuario; no se distribuye). Ver `THIRD_PARTY_LICENSES.md`.
