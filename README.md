# ConvertAll

Conversor de archivos **local** para Linux y Windows. Sin subidas, sin cuentas, 100% local.

Usa motores nativos:

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
- Paquete portable de Windows (`.7z`) con FFmpeg, Pandoc e ImageMagick incluidos.
- Motores de Windows probados bajo Wine: 13/13 conversiones correctas.
- La app arranca y detecta FFmpeg, ImageMagick, libvips, Pandoc, LibreOffice y Microsoft Office (Windows).
- El CI de Linux instala LibreOffice y ejecuta `scripts/test-office.sh`: 23/23 conversiones correctas (DOCX/DOC/ODT/RTF/TXT/HTML, CSV/XLSX/XLS/ODS, PPTX/PPT/ODP y PDF).

### Seguridad

- CSP estricta en producción: scripts, estilos, imágenes y conexiones limitados a la propia app y al IPC de Tauri.
- Los motores descargados para el build de Windows se verifican con SHA-256 contra el checksum publicado por su origen.
- Sin telemetría, sin subidas y sin red en tiempo de ejecución.

### Ajustes por archivo

Cada archivo muestra solo las opciones válidas para su formato de destino:

- **Vídeo:** 2160p, 1440p, 1080p, 720p, 480p y 360p. Limita el lado corto y no amplía el vídeo.
- **Imagen:** lado mayor entre 3840 y 480 px. No amplía la imagen; ICO y CUR mantienen sus tamaños de icono.
- **Audio:** bitrate según el códec (MP3, AAC/M4A, Opus, Vorbis, WMA, AC-3 y DTS). «Automático» usa la calidad global.

### Office y PDF

- **Entradas:** DOCX, DOC, ODT, RTF, TXT, HTML, XLSX, XLS, ODS, CSV, PPTX, PPT y ODP.
- **Salidas:** las mismas, más PDF.
- **PDF es solo salida.** La importación de PDF de LibreOffice produce un dibujo, no un documento editable fiable.
- Cada archivo muestra únicamente los destinos válidos: Word, hoja de cálculo o presentación.
- **Linux:** se usa el LibreOffice del sistema (`soffice` o `libreoffice`). No se empaqueta.
- **Windows:** se detecta Microsoft Office en el registro y en las rutas habituales. Si Word, Excel o PowerPoint están instalados, se usan para su familia; si no, se recurre a LibreOffice. Tampoco se empaqueta Office.
- Si no hay ninguno de los dos, la app sigue funcionando: solo se ocultan los destinos de Office/PDF y se muestra un aviso en «Motores».

Pendiente:

- Presets y perfiles reutilizables; más ajustes por formato (códec, FPS, canales).

### Limitaciones con PDF

**PDF solo se puede usar como destino (salida), no como archivo de entrada.**

ConvertAll puede exportar documentos de Office, texto e imágenes a PDF, pero **no puede abrir un PDF para convertirlo a Word (.doc/.docx), PowerPoint (.ppt/.pptx) ni Excel (.xls/.xlsx)**.

**¿Por qué?**
La importación de PDF en LibreOffice (el motor que usa ConvertAll para Office) trata el PDF como un dibujo o objeto gráfico, no como un documento con texto, tablas, celdas o diapositivas editables. El resultado no sería fiable: perderías formato, tablas, posiciones y no podrías editar el contenido como esperas en un archivo de Office.

**Alternativa**
Si necesitas convertir un PDF a un documento Office editable con fidelidad, tendrías que usar herramientas externas (por ejemplo, JOPDF o PDFCraft) antes de traer el archivo a ConvertAll. Esta limitación se mantiene intencionada para que la app siga siendo **100 % local, ligera y sin dependencias pesadas**.

Exportar **a** PDF desde otros formatos sí funciona normalmente.

## Descargas

A partir de **v0.1.2** se mantiene una sola release con la última versión: [Releases](https://github.com/Acd212/ConvertAll/releases/latest).

- Windows: `ConvertAll_x.y.z_x64-portable.7z`, sin instalador. Descomprime, abre la carpeta `ConvertAll` y ejecuta `ConvertAll.exe`. Windows 11 abre `.7z` de forma nativa; en Windows 10 necesitas 7-Zip u otro descompresor compatible.
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
