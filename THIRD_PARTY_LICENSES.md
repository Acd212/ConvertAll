# Licencias de terceros

ConvertAll se distribuye bajo la licencia MIT (ver `LICENSE`). El código de la
aplicación es original. Los motores externos conservan sus propias licencias.

## Linux

En Linux, ConvertAll usa los motores instalados en el sistema:

- **FFmpeg**: GPL/LGPL según la build de la distribución.
- **ImageMagick**: ImageMagick License.
- **libvips**: LGPL-2.1 o posterior.
- **Pandoc** (opcional, portable en `cache/tools`): GPL-2.0 o posterior.

## Windows

El instalador incluye copias portables de:

| Motor | Build | Licencia |
|---|---|---|
| FFmpeg / ffprobe | gyan.dev essentials | GPLv3 |
| Pandoc | release oficial | GPL-2.0+ |
| ImageMagick | portable Q16 x64 | ImageMagick License |

Los textos de licencia se incluyen en la carpeta `licenses/` del instalador y,
para ImageMagick, también dentro de `tools/magick/`.

La build de FFmpeg de gyan.dev es GPLv3. Si distribuyes ConvertAll públicamente,
debes cumplir la GPL de FFmpeg (incluir el texto de la licencia y ofrecer el
código fuente correspondiente o un enlace al mismo). Lo mismo aplica a Pandoc.
El código MIT de ConvertAll no cambia esas obligaciones.
