# ConvertAll

Conversor de archivos **local** para Linux y Windows. Sin subidas, sin cuentas y sin telemetría.

Usa motores nativos en lugar de WebAssembly:

- **FFmpeg** para audio y vídeo.
- **ImageMagick** y **libvips** para imágenes.
- **Pandoc** para documentos.

La interfaz está construida con **Tauri 2 + Svelte 5 + Tailwind CSS 4** y el backend con **Rust**.

## Estado

Proyecto en fase **M1**: shell funcional, detección de motores, cola de archivos, selección de formato, progreso, cancelación y conversión local.

Verificado en Linux:

- `svelte-check` sin errores, build de Vite y `cargo check` correctos.
- 11 pruebas Rust correctas.
- 77 conversiones reales correctas (imagen, audio, vídeo y documentos).
- La app arranca y detecta FFmpeg, ImageMagick, libvips y Pandoc.

Pendiente:

- Verificar el instalador de Windows en GitHub Actions.
- Probar la cola desde la GUI de forma manual.
- Office/PDF de alta fidelidad con LibreOffice (fase 2).

## Requisitos de desarrollo

- Rust estable (`rustc`, `cargo`).
- Bun (o Node 20+ y npm/pnpm).
- Linux: GTK3, WebKit2GTK 4.1, libsoup3, librsvg, patchelf.
- Motores: `ffmpeg`, `magick`, `vips` y, opcionalmente, `pandoc`.

En CachyOS/Arch:

```bash
sudo pacman -S --needed ffmpeg imagemagick libvips
# Pandoc no es obligatorio para compilar:
./scripts/fetch-pandoc-linux.sh
```

## Desarrollo

```bash
bun install
./scripts/dev.sh
```

`scripts/env.sh` redirige `CARGO_HOME`, `CARGO_TARGET_DIR`, la caché de Bun/npm y `TMPDIR` a `./cache/`.

## Comprobar

```bash
./scripts/check.sh
```

## Probar conversiones

Genera muestras con FFmpeg/ImageMagick y prueba todos los destinos del catálogo:

```bash
./scripts/test-conversions.sh
```

Los archivos de prueba se crean en `cache/tmp/samples/`.

## Compilar

```bash
./scripts/build.sh
```

Los instaladores se generan dentro de `cache/cargo-target/release/bundle/`.

## Windows

El repositorio incluye `.github/workflows/build.yml`. En cada push a `main`:

1. Se descargan FFmpeg, Pandoc e ImageMagick portables.
2. Se compila la app en `windows-latest`.
3. Se publican `.msi`/`.exe` como artefactos.

Los binarios descargados no se versionan: `src-tauri/resources/windows/` está en `.gitignore`.

## Estructura

```text
src/                 Interfaz Svelte
src/lib/             Tipos, API y componentes
src-tauri/src/       Backend Rust
  formats.rs         Catálogo y reglas de conversión
  engines.rs         Detección de FFmpeg/ImageMagick/libvips/Pandoc
  capabilities.rs    Formatos/codecs reales de cada motor
  convert.rs         Ejecución, progreso y cancelación
  commands.rs        Comandos de Tauri
scripts/             Entorno, builds, pruebas y descarga de herramientas
assets/icon.svg      Fuente de los iconos
cache/               Caché y artefactos (ignorado por git)
CONTEXTO.md          Contexto para retomar el proyecto con otro agente
```

## Licencia

MIT. ConvertAll es un proyecto original inspirado en la experiencia de VERT; no reutiliza su código (AGPL-3.0).

Los motores externos conservan sus propias licencias: FFmpeg (GPL/LGPL según build), ImageMagick, libvips (LGPL) y Pandoc (GPL-2.0+).
