# Licencias de terceros

ConvertAll se distribuye bajo la licencia MIT (ver `LICENSE`). El código de la
aplicación es original. Los motores externos conservan sus propias licencias.

## Linux

En Linux, ConvertAll usa los motores instalados en el sistema:

- **FFmpeg**: GPL/LGPL según la build de la distribución.
- **ImageMagick**: ImageMagick License.
- **libvips**: LGPL-2.1 o posterior.
- **Pandoc** (opcional, portable en `cache/tools`): GPL-2.0 o posterior.
- **LibreOffice** (opcional, del sistema): MPL-2.0/LGPL. ConvertAll no lo distribuye.

## Windows

El paquete portable incluye copias de:

| Motor | Build | Licencia |
|---|---|---|
| FFmpeg / ffprobe | gyan.dev essentials | GPLv3 |
| Pandoc | release oficial | GPL-2.0+ |
| ImageMagick | portable Q16 x64 | ImageMagick License |

Los textos de licencia se incluyen en la carpeta `licenses/` del paquete y,
para ImageMagick, también dentro de `tools/magick/`.

## LibreOffice y Microsoft Office

ConvertAll no distribuye LibreOffice ni Microsoft Office. LibreOffice se detecta
en el sistema y conserva su licencia (MPL-2.0/LGPL). Microsoft Office se usa solo
si el usuario lo tiene instalado, bajo su propia licencia.

La build de FFmpeg de gyan.dev es GPLv3. Si distribuyes ConvertAll públicamente,
debes cumplir la GPL de FFmpeg (incluir el texto de la licencia y ofrecer el
código fuente correspondiente o un enlace al mismo). Lo mismo aplica a Pandoc.
El código MIT de ConvertAll no cambia esas obligaciones.
