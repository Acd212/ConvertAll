# Descarga y prepara los motores que se empaquetan en Windows.
# Uso: pwsh ./scripts/fetch-tools-windows.ps1
$ErrorActionPreference = "Stop"

$root = Split-Path -Parent $PSScriptRoot
$tools = Join-Path $root "src-tauri/resources/windows"
$licenses = Join-Path $tools "licenses"
$tempRoot = if ($env:RUNNER_TEMP) { $env:RUNNER_TEMP } else { $env:TEMP }
$temp = Join-Path $tempRoot "convertall-tools"
New-Item -ItemType Directory -Force -Path $tools | Out-Null
New-Item -ItemType Directory -Force -Path $licenses | Out-Null
New-Item -ItemType Directory -Force -Path $temp | Out-Null

Write-Host "==> Descargando FFmpeg"
$ffmpegZip = Join-Path $temp "ffmpeg.zip"
Invoke-WebRequest -Uri "https://www.gyan.dev/ffmpeg/builds/ffmpeg-release-essentials.zip" -OutFile $ffmpegZip
Expand-Archive -Path $ffmpegZip -DestinationPath (Join-Path $temp "ffmpeg") -Force
$ffmpegExe = Get-ChildItem -Path (Join-Path $temp "ffmpeg") -Filter "ffmpeg.exe" -Recurse | Select-Object -First 1
$ffprobeExe = Get-ChildItem -Path (Join-Path $temp "ffmpeg") -Filter "ffprobe.exe" -Recurse | Select-Object -First 1
if (-not $ffmpegExe -or -not $ffprobeExe) { throw "No se encontraron ffmpeg.exe/ffprobe.exe" }
Copy-Item $ffmpegExe.FullName (Join-Path $tools "ffmpeg.exe") -Force
Copy-Item $ffprobeExe.FullName (Join-Path $tools "ffprobe.exe") -Force

$ffmpegRoot = Split-Path -Parent (Split-Path -Parent $ffmpegExe.FullName)
Get-ChildItem $ffmpegRoot -File |
  Where-Object { $_.Name -match "^(LICENSE|README)" } |
  ForEach-Object {
    Copy-Item $_.FullName (Join-Path $licenses "ffmpeg-$($_.Name)") -Force
  }

Write-Host "==> Descargando Pandoc"
$release = Invoke-RestMethod -Uri "https://api.github.com/repos/jgm/pandoc/releases/latest" -Headers @{ "User-Agent" = "ConvertAll-Build" }
$asset = $release.assets | Where-Object { $_.name -match "windows-x86_64\.zip$" } | Select-Object -First 1
if (-not $asset) { throw "No se encontró el paquete Windows de Pandoc" }
$pandocZip = Join-Path $temp "pandoc.zip"
Invoke-WebRequest -Uri $asset.browser_download_url -OutFile $pandocZip
Expand-Archive -Path $pandocZip -DestinationPath (Join-Path $temp "pandoc") -Force
$pandocExe = Get-ChildItem -Path (Join-Path $temp "pandoc") -Filter "pandoc.exe" -Recurse | Select-Object -First 1
if (-not $pandocExe) { throw "No se encontró pandoc.exe" }
Copy-Item $pandocExe.FullName (Join-Path $tools "pandoc.exe") -Force

$pandocRoot = Split-Path -Parent $pandocExe.FullName
Get-ChildItem $pandocRoot -File |
  Where-Object { $_.Name -match "^(COPYING|COPYRIGHT|LICENSE)" } |
  ForEach-Object {
    Copy-Item $_.FullName (Join-Path $licenses "pandoc-$($_.Name)") -Force
  }

Write-Host "==> Descargando ImageMagick"
# ImageMagick ya no publica .zip portable; el asset oficial es .7z.
$magickRelease = Invoke-RestMethod -Uri "https://api.github.com/repos/ImageMagick/ImageMagick/releases/latest" -Headers @{ "User-Agent" = "ConvertAll-Build" }
$magickAsset = $magickRelease.assets | Where-Object { $_.name -match "-portable-Q16-x64\.7z$" } | Select-Object -First 1
if (-not $magickAsset) { throw "No se encontró el paquete portable de ImageMagick" }
$magickArchive = Join-Path $temp "magick.7z"
Invoke-WebRequest -Uri $magickAsset.browser_download_url -OutFile $magickArchive

$sevenZip = Join-Path $env:ProgramFiles "7-Zip/7z.exe"
if (-not (Test-Path $sevenZip)) { $sevenZip = "7z" }
$magickDir = Join-Path $temp "magick"
& $sevenZip x $magickArchive "-o$magickDir" -y | Out-Null
if ($LASTEXITCODE -ne 0) { throw "No se pudo extraer ImageMagick con 7-Zip" }

$magickExe = Get-ChildItem -Path $magickDir -Filter "magick.exe" -Recurse | Select-Object -First 1
if (-not $magickExe) { throw "No se encontró magick.exe" }
$magickRoot = Split-Path -Parent $magickExe.FullName
$destMagick = Join-Path $tools "magick"
New-Item -ItemType Directory -Force -Path $destMagick | Out-Null

# El paquete portable es estático: cada .exe pesa ~32 MB. Solo necesitamos
# magick.exe y los XML/ICC de configuración, no compare/mogrify/montage/etc.
Copy-Item $magickExe.FullName (Join-Path $destMagick "magick.exe") -Force
Get-ChildItem $magickRoot -File |
  Where-Object { $_.Extension -ne ".exe" -and $_.Name -ne "ChangeLog.md" } |
  ForEach-Object { Copy-Item $_.FullName (Join-Path $destMagick $_.Name) -Force }

Write-Host "==> Herramientas preparadas en $tools"
Get-ChildItem $tools | Select-Object Name, Length
