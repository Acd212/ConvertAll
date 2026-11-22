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

### Instalar LibreOffice

En Linux basta con Writer, Calc e Impress:

```bash
# Debian/Ubuntu
sudo apt install libreoffice-writer libreoffice-calc libreoffice-impress

# Arch/CachyOS
sudo pacman -S libreoffice-fresh
```

En Windows, descárgalo desde [libreoffice.org](https://es.libreoffice.org/descarga/libreoffice/). Si ya tienes Microsoft Office instalado, no necesitas LibreOffice para Word, Excel o PowerPoint.

## Licencia

MIT. ConvertAll es un proyecto original inspirado en la experiencia de VERT; no reutiliza su código (AGPL-3.0).

Los motores externos conservan sus propias licencias: FFmpeg (GPL/LGPL según build), ImageMagick, libvips (LGPL), Pandoc (GPL-2.0+), LibreOffice (MPL-2.0/LGPL) y Microsoft Office (licencia del usuario; no se distribuye). Ver `THIRD_PARTY_LICENSES.md`.
