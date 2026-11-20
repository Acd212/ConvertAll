# CONTEXTO DEL PROYECTO — ConvertAll

> **Léeme primero.** Este archivo es la fuente de contexto para cualquier agente de IA que retome el proyecto. No necesitas escanear todo el repositorio: aquí está la arquitectura, las decisiones, el estado y los siguientes pasos.

Última actualización: 2026-11-20
Repositorio: `https://github.com/Acd212/ConvertAll` (privado)
Licencia: MIT
Idioma de la interfaz: español
Plataformas objetivo: Linux (principal) y Windows

---

## 1. Qué es

ConvertAll es un conversor de archivos **local** para escritorio. Sustituye la experiencia web de VERT por una app nativa:

- Sin subir archivos.
- Sin cuentas, anuncios ni telemetría.
- Sin las limitaciones de memoria de WebAssembly.
- Conversión por lotes, progreso y cancelación.

La app **no reutiliza código de VERT**. VERT es AGPL-3.0 y está hecho en Svelte/TypeScript con WASM. ConvertAll es una implementación original en Rust + Tauri, inspirada en su experiencia de uso. No copiar código, assets, logo ni textos de VERT.

---

## 2. Decisiones cerradas

| Tema | Decisión |
|---|---|
| Nombre | ConvertAll |
| Arquitectura | Tauri 2 + Svelte 5 + Vite + Tailwind CSS 4 |
| Backend | Rust |
| Motores | FFmpeg (audio/vídeo), ImageMagick + libvips (imagen), Pandoc (documentos) |
| Alcance v1 | Imagen + audio + vídeo + documentos. Office/PDF de alta fidelidad en fase 2 |
| Windows | Motores incluidos en el instalador (FFmpeg, Pandoc, ImageMagick portable) |
| Linux | Usa motores del sistema; Pandoc opcional vía `cache/tools` |
| Tema | Oscuro por defecto + tema claro |
| Idioma | Solo español en v1 (código e identificadores en inglés) |
| Licencia | MIT |
| Repo | Privado por ahora |
| Caché | Todo lo compilado/descargado en `./cache/` |
| Distribución | Sin firma de código por ahora (SmartScreen avisará en Windows) |
| Artefactos de release | Windows: `setup.exe` (NSIS). Linux: `.AppImage`. No se publican MSI ni DEB (desde 0.1.1) |

---

## 3. Arquitectura

```text
┌──────────────────────────────────────────────────────────────┐
│ Svelte 5 + Tailwind 4 (WebView de Tauri)                     │
│  - drag & drop, cola, selector de formato, progreso          │
│  - español, tema oscuro/claro                                │
└───────────────────────────┬──────────────────────────────────┘
                            │ invoke / eventos
┌───────────────────────────▼──────────────────────────────────┐
│ Rust (Tauri 2)                                               │
│  commands.rs  → detect_engines, get_catalog, get_targets,    │
│                 inspect_file, convert_file, cancel_job,      │
│                 default_output_dir, startup_files,           │
│                 scan_folder, open_in_file_manager            │
│  formats.rs   → catálogo de formatos y reglas de conversión  │
│  engines.rs   → localiza FFmpeg/ImageMagick/libvips/Pandoc   │
│  capabilities.rs → codecs/formatos reales de cada motor      │
│  convert.rs   → construye comandos, progreso, cancelación    │
└───────────────┬───────────────────────────┬──────────────────┘
                │                           │
        ┌───────▼────────┐          ┌───────▼────────┐
        │ Motores del    │          │ Sidecars       │
        │ sistema (Linux)│          │ (Windows)      │
        └────────────────┘          └────────────────┘
```

### Por qué no se usa el WASM de VERT

- `ffmpeg.wasm` limita memoria y velocidad; los archivos grandes fallan.
- `magick-wasm` y `vips-wasm` son más lentos que sus versiones nativas.
- El vídeo "pesado" en VERT depende de `vertd` (Rust + FFmpeg), lo que confirma que el camino nativo es el correcto.
- Tauri entrega rutas de archivo al backend, no contenidos: se evita cargar archivos en JS.

---

## 4. Stack y versiones

Verificado en el equipo de desarrollo (CachyOS, 2026-11-20):

- Rust 1.99.1, Cargo 1.99.1
- Node 24.22.1, Bun 1.4.4
- Tauri 2.12.7, Svelte 5.59.1, Vite 8.3.8, TypeScript 6.0.3
- FFmpeg 9.0.2
- ImageMagick 7.1.2-35
- libvips 8.18.7
- Pandoc 3.14 (portable en `cache/tools`, instalado con el script)
- Python 3.14.9 (solo para skills de UI/UX)
- GTK3 3.24.52, WebKit2GTK 4.1, libsoup 3.8.1, librsvg, patchelf

Dependencias JS principales (se resuelven con `bun install`):

- `@tauri-apps/api`, `@tauri-apps/plugin-dialog`, `@tauri-apps/cli`
- `svelte`, `@sveltejs/vite-plugin-svelte`, `vite`
- `tailwindcss`, `@tailwindcss/vite`
- `@lucide/svelte`, `@fontsource-variable/inter`
- `svelte-check`, `typescript` (fijado a `~6`; `svelte-check` aún no soporta TypeScript 7)

Dependencias Rust principales:

- `tauri`, `tauri-plugin-dialog`
- `serde`, `serde_json`
- `tokio`, `tokio-util` (cancelación)
- `dirs`

---

## 5. Mapa de archivos

```text
.
├── CONTEXTO.md                  ← este archivo
├── README.md
├── LICENSE
├── THIRD_PARTY_LICENSES.md
├── assets/icon.svg               fuente de los iconos
├── docs/captura.png             captura de la interfaz
├── package.json
├── bun.lock
├── vite.config.ts               salida del frontend → cache/dist
├── tsconfig.json
├── svelte.config.js
├── index.html
├── src/
│   ├── main.ts
│   ├── app.css                  tokens de Tailwind 4 y temas
│   ├── vite-env.d.ts
│   ├── App.svelte               shell, cola y estado global
│   └── lib/
│       ├── api.ts               envoltorio de invoke/listen
│       ├── types.ts             contratos TypeScript
│       ├── queue.ts             tipo QueueItem y estados
│       ├── format.ts            utilidades de tamaño/etiquetas
│       └── components/
│           ├── Logo.svelte
│           ├── EngineChips.svelte
│           ├── DropZone.svelte
│           ├── QueueItem.svelte
│           └── SettingsPanel.svelte
├── src-tauri/
│   ├── Cargo.toml
│   ├── build.rs
│   ├── tauri.conf.json
│   ├── tauri.windows.conf.json  recursos de motores para Windows
│   ├── capabilities/default.json
│   ├── icons/                   generados con `tauri icon`
│   ├── resources/windows/       binarios descargados (gitignored)
│   └── src/
│       ├── main.rs
│       ├── lib.rs               estado global y builder de Tauri
│       ├── commands.rs
│       ├── formats.rs
│       ├── engines.rs
│       ├── capabilities.rs      sondeo real de codecs/formatos por motor
│       └── convert.rs
├── scripts/
│   ├── env.sh                   redirección de caché
│   ├── dev.sh
│   ├── build.sh
│   ├── check.sh
│   ├── test-conversions.sh      pruebas funcionales de los motores
│   ├── fetch-pandoc-linux.sh
│   └── fetch-tools-windows.ps1
├── .github/workflows/build.yml  CI Linux + Windows
└── cache/                       TODO lo compilado/descargado (gitignored)
```

---

## 6. Comandos

```bash
# Dependencias JS
bun install

# Desarrollo (Vite + Tauri)
./scripts/dev.sh

# Comprobaciones (svelte-check + build + cargo check)
./scripts/check.sh

# Build local (Linux)
./scripts/build.sh

# Pandoc portable en Linux
./scripts/fetch-pandoc-linux.sh

# Pruebas funcionales de todos los motores (imagen, audio, vídeo, documentos)
./scripts/test-conversions.sh

# Tests Rust
source scripts/env.sh
cargo test --manifest-path src-tauri/Cargo.toml
```

### Caché

`scripts/env.sh` define:

```bash
CARGO_HOME=cache/cargo-home
CARGO_TARGET_DIR=cache/cargo-target
BUN_INSTALL_CACHE_DIR=cache/bun
npm_config_cache=cache/npm
TMPDIR=cache/tmp
XDG_CACHE_HOME=cache/xdg
```

`node_modules/` se queda en la raíz del proyecto porque es una dependencia necesaria, no caché. Está en `.gitignore`.

Los instaladores quedan en:

```text
cache/cargo-target/release/bundle/
```

---

## 7. Contratos backend ↔ frontend

Los comandos de Tauri viven en `src-tauri/src/commands.rs`.

| Comando | Entrada | Salida |
|---|---|---|
| `detect_engines` | — | `EngineInfo[]` |
| `get_catalog` | — | `FormatDef[]` |
| `get_targets` | `{ category }` | `FormatDef[]` disponibles |
| `inspect_file` | `{ path }` | `FileInfo` |
| `convert_file` | `{ request: ConvertRequest }` | `ConvertResult` |
| `cancel_job` | `{ jobId }` | `void` |
| `default_output_dir` | — | `string` |
| `startup_files` | — | `string[]` (archivos pasados por CLI) |
| `scan_folder` | `{ path }` | `string[]` (máx. 1000, profundidad 16) |
| `open_in_file_manager` | `{ path }` | `void` |

Tipos (camelCase en JSON):

```ts
interface EngineInfo { id; name; available; version; path }
interface FormatDef { id; label; extension; category; canRead; canWrite; engine }
interface FileInfo {
  path; name; extension; size; category; formatId;
  engine; engineAvailable; suggestedTarget
}
interface ConvertRequest {
  jobId; inputPath; outputDir; targetFormat;
  quality; preserveMetadata; overwrite
}
interface ConvertResult { outputPath; outputSize; durationMs }
```

Evento de progreso:

```text
job-progress → { jobId, progress: number (0..100), stage: string }
```

`progress` puede ser 0..100. La UI usa `jobId` para actualizar la fila.

---

## 8. Reglas de conversión

`formats.rs` es la fuente de verdad.

- Categorías: `image`, `audio`, `video`, `document`, `unknown`.
- `targets_for(category)`:
  - Imagen → formatos de imagen escribibles.
  - Audio → formatos de audio escribibles.
  - Vídeo → formatos de vídeo + audio + GIF.
  - Documento → formatos de documento escribibles.
- No se permite PDF todavía (fase 2).
- No se permite Office de entrada (`xlsx`, `pptx`): Pandoc no los lee.
- `pptx` solo es salida (Pandoc puede generar presentaciones desde Markdown).
- Alias normalizados: `jpeg→jpg`, `tif→tiff`, `mpeg→mpg`, `markdown→md`, `htm→html`, etc.

Motores por categoría:

- Imagen → ImageMagick; si no está, libvips.
- Audio/vídeo → FFmpeg.
- Documento → Pandoc.

Si el motor no está disponible, `get_targets` filtra esos formatos y `inspect_file` marca `engineAvailable=false`.

### Capacidades reales (`capabilities.rs`)

Además de comprobar que el binario existe, `get_targets` consulta sus capacidades reales:

- FFmpeg: `ffmpeg -encoders` y exige los codecs necesarios por destino (`libx264`, `libvpx-vp9`, `libtheora`, `libopencore_amrnb`, etc.). Si un codec no está en el build, el formato no se ofrece.
- ImageMagick: `magick -list format` y exige modo de escritura (`rw-`/`rw+`). `jpg` se mapea a `JPEG`. Linux imprime `JPEG* JPEG rw- ...`; Windows imprime `JPEG* rw- ...` (sin columna de módulo), así que el parser busca el token de modo en los primeros cuatro en vez de usar un índice fijo.
- Pandoc: `pandoc --list-output-formats`. `md→gfm`, `txt→plain`, `tex→latex`.

Si el sondeo falla, se permite el formato (`Capabilities::All`) y el error aparecerá al convertir; es preferible a ocultar opciones válidas. Esto es importante porque la build de Windows puede no incluir todos los codecs de la de Linux.

---

## 9. Interfaz y diseño

- Español, tono directo.
- Oscuro por defecto (`data-theme="dark"`), claro disponible.
- Paleta: fondo `#0c0a09`, panel `#171412`, acento naranja `#f97316`, texto `#f5f5f4`.
- Tipografía: Inter Variable (`@fontsource-variable/inter`).
- Iconos: Lucide (`@lucide/svelte`; el paquete `lucide-svelte` está deprecado).
- Radio: 12–16 px; bordes sutiles; densidad media.
- Componentes: DropZone, QueueItem, SettingsPanel, EngineChips.
- Accesibilidad: foco visible, `aria-label`, `role="progressbar"`, `aria-live` para avisos, respeto a `prefers-reduced-motion`.
- La skill `ui-ux-pro-max` está instalada globalmente en OpenCode (`~/.opencode/skills`). Se puede consultar con:

```bash
python3 ~/.opencode/skills/ui-ux-pro-max/scripts/search.py "desktop file converter" --design-system -p "ConvertAll"
```

---

## 10. Windows

`tauri.windows.conf.json` empaqueta:

```text
resources/windows/ffmpeg.exe   → tools/ffmpeg.exe
resources/windows/ffprobe.exe  → tools/ffprobe.exe
resources/windows/pandoc.exe   → tools/pandoc.exe
resources/windows/magick/      → tools/magick/        (carpeta completa)
resources/windows/licenses/    → licenses/           (FFmpeg, Pandoc)
```

La carpeta de ImageMagick se mapea como **directorio**, no como glob. Tauri aplana los globs (`**/*`) en el destino; al usar la carpeta preserva subcarpetas y DLLs.

`scripts/fetch-tools-windows.ps1` descarga:

- FFmpeg essentials de gyan.dev.
- Pandoc latest de GitHub Releases.
- ImageMagick portable Q16 x64 (`.7z`) latest de GitHub Releases; solo se empaqueta `magick.exe` y sus XML de configuración.

El workflow `.github/workflows/build.yml`:

- Linux: `ubuntu-24.04`, `bunx tauri build --bundles appimage`; sube solo `.AppImage`.
- Windows: `windows-latest`, descarga motores, `bunx tauri build --ci --bundles nsis`; sube solo `.exe` (NSIS).
- MSI y DEB ya no se generan en CI ni se publican en releases nuevas.

`engines.rs` busca primero `CONVERTALL_TOOLS_DIR`, luego `resource_dir()/tools`, luego `PATH`. Para ImageMagick portable define `MAGICK_HOME` y añade su carpeta al `PATH`.

Pendiente de verificar en CI:

- [x] La carpeta de ImageMagick se copia completa.
- [x] `MAGICK_HOME` funciona con la build portable.
- [x] NSIS/MSI incluyen los binarios.
- [x] Codecs del FFmpeg essentials: incluye todos los usados (`libx264`, `libvpx-vp9`, `libtheora`, `libopencore_amrnb`, `dca`, etc.).

Resultados reales (run 44804003751):

```text
MSI:  132,06 MiB
NSIS:  92,13 MiB
tools/ffmpeg.exe, tools/ffprobe.exe, tools/pandoc.exe, tools/magick/magick.exe
```

Los binarios de Windows se probaron bajo Wine:

- `ffmpeg 9.1-essentials_build` y `ffprobe 9.1`.
- `pandoc 3.14`.
- `ImageMagick 7.1.2-36 Q16 x64`.
- 13/13 conversiones correctas (JPG, ICO, WebP, AVIF, HEIC, JXL, MP3, FLAC, MKV, WebM, GIF, DOCX, HTML).

### Linux: deb + AppImage

El CI compila solo `--bundles appimage`:

- `.AppImage` (~80 MB en CI): incluye WebKit2GTK 4.1, GTK3, libsoup y los procesos WebKit, por lo que no hace falta instalar WebKit en el sistema. Los motores siguen siendo del sistema.
- El `.deb` ya no se genera ni se publica (política desde 0.1.1).

El AppImage se probó bajo Xvfb y abre la cola correctamente con archivos pasados por CLI.
---

## 11. Estado actual

Implementado y verificado:

- [x] Estructura Tauri 2 + Svelte 5 + Tailwind 4.
- [x] Caché centralizada en `./cache`.
- [x] Catálogo de ~100 formatos.
- [x] Detección de motores y versiones (normalizadas: `9.0.2`, `7.1.2-35`, `8.18.7`, `3.14`).
- [x] Sondeo real de codecs/formatos por motor (`capabilities.rs`).
- [x] Inspección de archivos y carpeta.
- [x] Cola con progreso, cancelación y reintento.
- [x] Conversión con FFmpeg/ImageMagick/libvips/Pandoc.
- [x] Tema oscuro/claro y UI en español.
- [x] Workflow de CI para Linux y Windows.
- [x] `bun install` + `svelte-check` (0 errores) + `vite build` + `cargo check`.
- [x] `cargo test`: 12 pruebas correctas.
- [x] Iconos generados con `tauri icon` desde `assets/icon.svg`.
- [x] La app arranca en Xvfb y muestra los cuatro motores en verde.
- [x] Conversiones reales: 77/77 correctas con `scripts/test-conversions.sh`.
- [x] Repo privado creado y primer push: `Acd212/ConvertAll`.
- [x] CI Linux + Windows verde (run 44822379426).
- [x] Instaladores de Windows con motores y licencias incluidas; 13/13 conversiones bajo Wine.
- [x] AppImage de Linux con WebKit2GTK 4.1 incluido; probado bajo Xvfb.
- [x] v0.1.1 corrige el sondeo de ImageMagick en Windows (sin columna de módulo).
- [x] Cola probada de extremo a extremo desde la GUI: 4/4 archivos convertidos.
- [x] Release privada `v0.1.1` con MSI, NSIS, DEB, AppImage y `SHA256SUMS.txt`.
- [x] Política desde 0.1.1: las releases nuevas incluyen solo `AppImage` y `setup.exe` (NSIS).

Resultados de la última ejecución (2026-11-20):

```text
Imagen:      24/24 destinos
Audio:       16/16 destinos
Vídeo:       18/18 destinos (incluye MP3 y GIF)
Documentos:  19/19 destinos
Total:       77/77
GUI:         PNG→JPG, WAV→MP3, MP4→MKV, MD→DOCX
Windows:     13/13 bajo Wine
```

> Actualizar esta sección al terminar cada fase.

---

## 12. Siguientes pasos recomendados

1. **Firma de código** en Windows para reducir avisos de SmartScreen (opcional).
2. **Reducir tamaño**: valorar un FFmpeg más ligero o un Pandoc sin datos innecesarios; hoy el NSIS pesa 92 MB.
3. **Ajustes de UX**: nombres largos, carpetas con muchos archivos, textos de error de los motores.
4. **Fase 2**: LibreOffice para Office/PDF, presets, perfiles de calidad, conversión recursiva opcional, notificaciones.

---

## 13. Convenciones y trampas

- No usar `sh -c` ni interpolar rutas en strings de shell. Siempre `Command::arg(...)`.
- En Windows, ocultar consolas con `creation_flags(0x08000000)`.
- En Windows, usar `PathBuf`/`OsStr`; no asumir UTF-8.
- No bloquear el hilo de UI: las conversiones son comandos async.
- `kill_on_drop(true)` evita procesos huérfanos.
- El progreso de FFmpeg se lee de `-progress pipe:1` (`out_time_us`).
- La duración se obtiene con `ffprobe`; si falla, el progreso queda indeterminado.
- No prometer conversiones que el motor no soporte; `capabilities.rs` filtra según el binario real.
- Los formatos de imagen son un caso especial: `engine` es el pseudo-motor `"image"`, que se resuelve a ImageMagick o libvips.
- `gif` es categoría imagen, pero también es destino válido desde vídeo (FFmpeg).
- ICO y CUR exigen `-define icon:auto-resize=256,128,64,48,32,16`; sin ello ImageMagick falla si la imagen supera 256 px.
- ImageMagick en Windows omite la columna de módulo en `-list format`. No asumir índices fijos: buscar el token con forma de modo (`rw-`, `rw+`, etc.). Hay una prueba de regresión.
- libvips es opcional en Windows porque ImageMagick es el motor de imagen. La UI lo muestra como «No necesario» (gris) si ImageMagick está disponible.
- El aviso de Pandoc debe servir para AppImage/deb: no referenciar scripts del repositorio.
- EPS/PostScript depende de Ghostscript; por eso EPS es solo lectura en el catálogo y puede fallar en Windows.
- Las conversiones escriben a un archivo temporal y renombran al terminar; así no quedan salidas corruptas al cancelar o fallar.
- DTS usa el encoder experimental `dca`: requiere `-strict -2`.
- APE es solo lectura: FFmpeg lo decodifica pero no lo codifica.
- Pandoc se descarga como `bin/pandoc`; el script lo mueve a `cache/tools/pandoc`.
- TypeScript está fijado a `~6` porque `svelte-check` 4.x no soporta TypeScript 7.
- Los archivos pasados por línea de comandos se recogen con `startup_files` y se añaden a la cola al iniciar.
- PDF y Office están fuera de v1 a propósito.
- No copiar código de VERT (AGPL). Inspiración de UX, no de implementación.
- El instalador incluye `licenses/` con los textos de FFmpeg y Pandoc; ImageMagick trae los suyos en `tools/magick/`. Ver `THIRD_PARTY_LICENSES.md`.
- `cache/` y `node_modules/` nunca se commitean.
- El token de GitHub no se guarda en el repo.

---

## 14. Verificación rápida

```bash
# 1. Dependencias
bun install

# 2. Tipos + frontend + Rust
./scripts/check.sh

# 3. Tests Rust
source scripts/env.sh
cargo test --manifest-path src-tauri/Cargo.toml

# 4. Conversiones reales (genera muestras en cache/tmp)
./scripts/test-conversions.sh

# 5. Desarrollo
./scripts/dev.sh
```

Si algo falla, revisar primero:

- `src-tauri/tauri.conf.json` (`frontendDist` debe ser `../cache/dist`).
- `vite.config.ts` (`outDir: cache/dist`).
- `capabilities/default.json` (`dialog:default`).
- Rutas de motores en `engines.rs`.

---

## 15. Contexto de Git/GitHub

- Cuenta: `Acd212`.
- `gh` autenticado con scopes `repo`, `workflow`, `read:org`, `gist`.
- Repo creado: `Acd212/ConvertAll` (privado), rama `main`.
- Identidad configurada solo en el repo:

```bash
git config user.name "Acd212"
git config user.email "349978655+Acd212@users.noreply.github.com"
```

- CI: `.github/workflows/build.yml`; última ejecución verde `44822379426`.
- Artefactos del run: `convertall-linux` (.AppImage) y `convertall-windows` (.exe NSIS).
- El instalador incluye `licenses/` (FFmpeg y Pandoc) además de `tools/`.
- Release: `https://github.com/Acd212/ConvertAll/releases/tag/v0.1.1` (MSI, NSIS, DEB, AppImage y checksums).
- **Política de release:** a partir de 0.1.1 solo se publican `AppImage` y `setup.exe`. No volver a subir MSI ni DEB en versiones nuevas.
- El AppImage incluye WebKit2GTK 4.1/GTK3; no requiere instalar WebKit. Los motores siguen siendo del sistema.

No hay secretos dentro del proyecto. La autenticación de GitHub vive fuera del repo.
