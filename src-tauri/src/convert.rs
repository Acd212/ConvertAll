use crate::engines;
use crate::formats::{self, Category};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, BufReader};
use tokio::process::Command;
use tokio_util::sync::CancellationToken;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConvertRequest {
    pub job_id: String,
    pub input_path: String,
    pub output_dir: String,
    pub target_format: String,
    #[serde(default = "default_quality")]
    pub quality: u8,
    #[serde(default = "default_true")]
    pub preserve_metadata: bool,
    #[serde(default)]
    pub overwrite: bool,
    #[serde(default)]
    pub video_height: Option<u32>,
    #[serde(default)]
    pub image_max_side: Option<u32>,
    #[serde(default)]
    pub audio_bitrate: Option<u32>,
}

fn default_quality() -> u8 {
    80
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConvertResult {
    pub output_path: String,
    pub output_size: u64,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProgressPayload {
    pub job_id: String,
    pub progress: f64,
    pub stage: String,
}

fn emit_progress(app: &AppHandle, job_id: &str, progress: f64, stage: &str) {
    let _ = app.emit(
        "job-progress",
        ProgressPayload {
            job_id: job_id.to_string(),
            progress,
            stage: stage.to_string(),
        },
    );
}

fn choose_engine(
    app: &AppHandle,
    category: Category,
    target_id: &str,
) -> Result<&'static str, String> {
    match category {
        Category::Image => {
            if engines::available(app, engines::MAGICK) {
                Ok(engines::MAGICK)
            } else if engines::available(app, engines::VIPS) {
                Ok(engines::VIPS)
            } else {
                Err("No hay un motor de imágenes disponible (ImageMagick o libvips).".to_string())
            }
        }
        Category::Audio | Category::Video => {
            if engines::available(app, engines::FFMPEG) {
                Ok(engines::FFMPEG)
            } else {
                Err("FFmpeg no está disponible.".to_string())
            }
        }
        Category::Document => {
            if engines::available(app, engines::PANDOC) {
                Ok(engines::PANDOC)
            } else {
                Err("Pandoc no está disponible. Instálalo o descárgalo en cache/tools.".to_string())
            }
        }
        Category::Unknown => Err(format!(
            "El formato de destino «{target_id}» no se puede usar con este archivo."
        )),
    }
}

fn quality_to_crf(quality: u8) -> u8 {
    let quality = quality.clamp(1, 100) as i32;
    (35 - ((quality - 1) * 20 / 99)).clamp(15, 35) as u8
}

fn audio_quality(quality: u8) -> u8 {
    let quality = quality.clamp(1, 100) as i32;
    (9 - (quality * 9 / 100)).clamp(0, 9) as u8
}

fn bitrate_kbps(quality: u8) -> u32 {
    64 + (quality.clamp(1, 100) as u32 * 192 / 100)
}

/// `1080p` limita el lado corto, así que el vídeo vertical conserva su
/// orientación y no se amplía si ya es más pequeño.
fn video_scale_filter(height: u32) -> String {
    format!(
        "scale=w='if(gt(iw,ih),-2,min(iw,{height}))':h='if(gt(iw,ih),min(ih,{height}),-2)':flags=lanczos"
    )
}

/// Caja cuadrada: limita el lado mayor y nunca amplía la imagen.
fn image_box_filter(side: u32) -> String {
    format!(
        "scale=w='min(iw,{side})':h='min(ih,{side})':force_original_aspect_ratio=decrease:force_divisible_by=2:flags=lanczos"
    )
}

fn validate_options(target: &formats::FormatDef, request: &ConvertRequest) -> Result<(), String> {
    if let Some(height) = request.video_height {
        if !target.video_heights.contains(&height) {
            return Err(
                "La resolución de vídeo seleccionada no es válida para este formato.".to_string(),
            );
        }
    }

    if let Some(side) = request.image_max_side {
        if !target.image_sides.contains(&side) {
            return Err(
                "El tamaño de imagen seleccionado no es válido para este formato.".to_string(),
            );
        }
    }

    if let Some(rate) = request.audio_bitrate {
        if !target.audio_bitrates.contains(&rate) {
            return Err("El bitrate seleccionado no es válido para este formato.".to_string());
        }
    }

    Ok(())
}

fn build_ffmpeg(
    command: &mut Command,
    input: &Path,
    output: &Path,
    target_id: &str,
    request: &ConvertRequest,
) {
    command
        .arg("-hide_banner")
        .arg("-loglevel")
        .arg("error")
        .arg("-y")
        .arg("-i")
        .arg(input)
        .arg("-progress")
        .arg("pipe:1")
        .arg("-nostats");

    if !request.preserve_metadata {
        command.arg("-map_metadata").arg("-1");
    }

    let audio_targets = [
        "mp3", "wav", "flac", "ogg", "opus", "aac", "m4a", "wma", "aiff", "ac3", "dts", "amr",
        "ape", "mka", "caf", "wv", "m4b",
    ];

    if audio_targets.contains(&target_id) {
        command.arg("-vn");
        let bitrate = request.audio_bitrate.map(|value| format!("{value}k"));

        match target_id {
            "mp3" => {
                command.args(["-c:a", "libmp3lame"]);
                if let Some(value) = bitrate.as_deref() {
                    command.args(["-b:a", value]);
                } else {
                    command
                        .args(["-q:a"])
                        .arg(audio_quality(request.quality).to_string());
                }
            }
            "wav" => {
                command.args(["-c:a", "pcm_s16le"]);
            }
            "flac" => {
                command.args(["-c:a", "flac"]);
            }
            "ogg" => {
                command.args(["-c:a", "libvorbis"]);
                if let Some(value) = bitrate.as_deref() {
                    command.args(["-b:a", value]);
                } else {
                    command
                        .args(["-q:a"])
                        .arg(audio_quality(request.quality).to_string());
                }
            }
            "opus" => {
                command.args(["-c:a", "libopus", "-b:a"]).arg(
                    bitrate.unwrap_or_else(|| format!("{}k", bitrate_kbps(request.quality))),
                );
            }
            "aac" | "m4a" | "m4b" => {
                command.args(["-c:a", "aac", "-b:a"]).arg(
                    bitrate.unwrap_or_else(|| format!("{}k", bitrate_kbps(request.quality))),
                );
            }
            "wma" => {
                command
                    .args(["-c:a", "wmav2", "-b:a"])
                    .arg(bitrate.unwrap_or_else(|| "192k".to_string()));
            }
            "aiff" => {
                command.args(["-c:a", "pcm_s16be"]);
            }
            "ac3" => {
                command
                    .args(["-c:a", "ac3", "-b:a"])
                    .arg(bitrate.unwrap_or_else(|| "192k".to_string()));
            }
            "dts" => {
                command
                    .args(["-c:a", "dca", "-strict", "-2", "-b:a"])
                    .arg(bitrate.unwrap_or_else(|| "768k".to_string()));
            }
            "amr" => {
                command
                    .args(["-c:a", "libopencore_amrnb", "-ar", "8000", "-ac", "1", "-b:a", "12.2k"]);
            }
            "ape" => {
                command.args(["-c:a", "ape"]);
            }
            "mka" => {
                command.args(["-c:a", "copy"]);
            }
            "caf" => {
                command.args(["-c:a", "pcm_s16le"]);
            }
            "wv" => {
                command.args(["-c:a", "wavpack"]);
            }
            _ => {}
        }
        command.arg(output);
        return;
    }

    if target_id == "gif" {
        let filter = format!(
            "fps=15,{}",
            image_box_filter(request.image_max_side.unwrap_or(1280))
        );
        command
            .args(["-an", "-vf"])
            .arg(filter)
            .args(["-loop", "0"])
            .arg(output);
        return;
    }

    let crf = quality_to_crf(request.quality).to_string();

    if let Some(height) = request.video_height {
        command.args(["-vf"]).arg(video_scale_filter(height));
    }

    match target_id {
        "mp4" | "m4v" | "mov" | "f4v" => {
            command
                .args(["-c:v", "libx264", "-preset", "medium", "-crf"])
                .arg(&crf)
                .args(["-pix_fmt", "yuv420p", "-c:a", "aac", "-b:a", "192k", "-movflags", "+faststart"]);
        }
        "mkv" => {
            command
                .args(["-c:v", "libx264", "-preset", "medium", "-crf"])
                .arg(&crf)
                .args(["-pix_fmt", "yuv420p", "-c:a", "aac", "-b:a", "192k"]);
        }
        "webm" => {
            command
                .args(["-c:v", "libvpx-vp9", "-crf"])
                .arg(&crf)
                .args(["-b:v", "0", "-c:a", "libopus", "-b:a", "128k"]);
        }
        "avi" => {
            command
                .args(["-c:v", "libx264", "-crf"])
                .arg(&crf)
                .args(["-c:a", "libmp3lame", "-q:a", "4"]);
        }
        "wmv" | "asf" => {
            command.args([
                "-c:v", "wmv2", "-b:v", "2M", "-c:a", "wmav2", "-b:a", "192k",
            ]);
        }
        "flv" => {
            command
                .args(["-c:v", "libx264", "-crf"])
                .arg(&crf)
                .args(["-c:a", "aac", "-b:a", "128k"]);
        }
        "mpg" | "vob" => {
            command.args([
                "-c:v", "mpeg2video", "-q:v", "3", "-c:a", "mp2", "-b:a", "192k",
            ]);
        }
        "ts" | "m2ts" => {
            command
                .args(["-c:v", "libx264", "-crf"])
                .arg(&crf)
                .args(["-c:a", "aac", "-b:a", "192k"]);
        }
        "3gp" => {
            command
                .args(["-c:v", "libx264", "-crf"])
                .arg(&crf)
                .args(["-c:a", "aac", "-ar", "44100", "-ac", "1"]);
        }
        "ogv" => {
            command.args([
                "-c:v", "libtheora", "-q:v", "6", "-c:a", "libvorbis", "-q:a", "4",
            ]);
        }
        _ => {}
    }

    command.arg(output);
}

fn build_magick(
    command: &mut Command,
    input: &Path,
    output: &Path,
    target_id: &str,
    request: &ConvertRequest,
) {
    command.arg(input).arg("-auto-orient");

    if let Some(side) = request.image_max_side {
        command
            .arg("-resize")
            .arg(format!("{side}x{side}>"));
    }

    if !request.preserve_metadata {
        command.arg("-strip");
    }

    if matches!(target_id, "jpg" | "webp" | "avif" | "jxl" | "heic") {
        command
            .arg("-quality")
            .arg(request.quality.clamp(1, 100).to_string());
    }

    if matches!(target_id, "ico" | "cur") {
        command.args(["-define", "icon:auto-resize=256,128,64,48,32,16"]);
    }

    command.arg(output);
}

fn build_vips(
    command: &mut Command,
    input: &Path,
    output: &Path,
    request: &ConvertRequest,
) {
    match request.image_max_side {
        Some(side) => {
            let side_text = side.to_string();
            command
                .arg("thumbnail")
                .arg(input)
                .arg(output)
                .arg(&side_text)
                .args(["--height", side_text.as_str(), "--size", "down"]);
        }
        None => {
            command.arg("copy").arg(input).arg(output);
        }
    }
}

fn build_pandoc(command: &mut Command, input: &Path, output: &Path, target_id: &str) {
    command.arg(input).arg("-o").arg(output);

    match target_id {
        "html" | "epub" => {
            command.arg("--standalone");
        }
        "txt" => {
            command.args(["-t", "plain"]);
        }
        "md" => {
            command.args(["-t", "gfm"]);
        }
        _ => {}
    }
}

pub fn unique_output_path(path: &Path) -> PathBuf {
    if !path.exists() {
        return path.to_path_buf();
    }

    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let stem = path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("salida");
    let extension = path.extension().and_then(|value| value.to_str());

    for index in 1..1000 {
        let name = match extension {
            Some(extension) => format!("{stem} ({index}).{extension}"),
            None => format!("{stem} ({index})"),
        };
        let candidate = parent.join(name);
        if !candidate.exists() {
            return candidate;
        }
    }

    path.to_path_buf()
}

async fn probe_duration(app: &AppHandle, input: &Path) -> Option<f64> {
    let ffprobe = engines::resolve_ffprobe(app)?;
    let mut command = Command::new(ffprobe);
    engines::sanitize_tokio_command(&mut command);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
    }
    command
        .args([
            "-v",
            "error",
            "-show_entries",
            "format=duration",
            "-of",
            "default=nw=1:nk=1",
        ])
        .arg(input)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());

    let output = command.output().await.ok()?;
    String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse::<f64>()
        .ok()
        .filter(|value| value.is_finite() && *value > 0.0)
}

pub async fn convert(
    app: &AppHandle,
    token: CancellationToken,
    request: ConvertRequest,
) -> Result<ConvertResult, String> {
    let started = Instant::now();
    let input = PathBuf::from(&request.input_path);

    if !input.is_file() {
        return Err("El archivo de entrada no existe o no es un archivo.".to_string());
    }

    let input_format = formats::format_for_path(&input)
        .ok_or_else(|| "El formato del archivo de entrada no está soportado.".to_string())?;
    let target = formats::find_by_id(&request.target_format)
        .ok_or_else(|| "El formato de salida no está soportado.".to_string())?;

    if !formats::is_target_allowed(input_format.category, &target.id) {
        return Err(format!(
            "No se puede convertir «{}» a «{}».",
            input_format.label, target.label
        ));
    }

    validate_options(&target, &request)?;

    let engine_id = choose_engine(app, input_format.category, &target.id)?;
    let tool = engines::resolve(app, engine_id)
        .ok_or_else(|| format!("No se encontró el motor «{engine_id}»."))?;

    let output_dir = if request.output_dir.trim().is_empty() {
        input
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."))
    } else {
        PathBuf::from(request.output_dir.trim())
    };
    std::fs::create_dir_all(&output_dir)
        .map_err(|error| format!("No se pudo crear la carpeta de salida: {error}"))?;

    let stem = input
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("salida");
    let mut output = output_dir.join(format!("{stem}.{}", target.extension));

    if request.overwrite {
        if output.exists() && !output.is_file() {
            return Err("La ruta de salida existe y no es un archivo.".to_string());
        }
    } else {
        output = unique_output_path(&output);
    }

    // Se escribe a un archivo temporal y se renombra solo al terminar, para no
    // dejar salidas corruptas si el motor falla o se cancela.
    let safe_job: String = request
        .job_id
        .chars()
        .filter(|character| character.is_ascii_alphanumeric() || *character == '-')
        .collect();
    let suffix = if safe_job.is_empty() {
        std::process::id().to_string()
    } else {
        safe_job
    };
    let temp_output = output_dir.join(format!(
        ".{stem}.convertall-{suffix}.{}",
        target.extension
    ));
    let _ = std::fs::remove_file(&temp_output);

    let mut command = Command::new(&tool);
    command.kill_on_drop(true);
    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    engines::configure_tokio_command(&mut command, engine_id, &tool);

    let is_ffmpeg = engine_id == engines::FFMPEG;
    let mut duration = None;

    match engine_id {
        engines::FFMPEG => {
            build_ffmpeg(&mut command, &input, &temp_output, &target.id, &request);
            duration = probe_duration(app, &input).await;
        }
        engines::MAGICK => build_magick(&mut command, &input, &temp_output, &target.id, &request),
        engines::VIPS => build_vips(&mut command, &input, &temp_output, &request),
        engines::PANDOC => build_pandoc(&mut command, &input, &temp_output, &target.id),
        _ => return Err("Motor de conversión desconocido.".to_string()),
    }

    emit_progress(app, &request.job_id, 0.0, "preparando");

    let mut child = command
        .spawn()
        .map_err(|error| format!("No se pudo iniciar {engine_id}: {error}"))?;

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "No se pudo leer la salida del motor.".to_string())?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| "No se pudo leer el error del motor.".to_string())?;

    let mut progress_task = if is_ffmpeg {
        let app = app.clone();
        let job_id = request.job_id.clone();
        Some(tokio::spawn(async move {
            let mut lines = BufReader::new(stdout).lines();
            let mut last_emit: Option<Instant> = None;

            while let Ok(Some(line)) = lines.next_line().await {
                let mut progress = None;

                if let Some(raw) = line.strip_prefix("out_time_us=") {
                    if let (Ok(microseconds), Some(total)) = (raw.trim().parse::<f64>(), duration) {
                        if total > 0.0 {
                            progress =
                                Some((microseconds / 1_000_000.0 / total * 100.0).clamp(0.0, 99.0));
                        }
                    }
                }

                if line.trim() == "progress=end" {
                    progress = Some(100.0);
                }

                if let Some(value) = progress {
                    let should_emit = value >= 100.0
                        || last_emit
                            .map(|instant| instant.elapsed() >= Duration::from_millis(200))
                            .unwrap_or(true);
                    if should_emit {
                        emit_progress(&app, &job_id, value, "convirtiendo");
                        last_emit = Some(Instant::now());
                    }
                }
            }
        }))
    } else {
        Some(tokio::spawn(async move {
            let mut sink = Vec::new();
            let _ = BufReader::new(stdout).read_to_end(&mut sink).await;
        }))
    };

    let stderr_task = tokio::spawn(async move {
        let mut text = String::new();
        let mut stderr = stderr;
        let _ = stderr.read_to_string(&mut text).await;
        text
    });

    let mut stderr_task = Some(stderr_task);

    let status = tokio::select! {
        result = child.wait() => Some(result),
        _ = token.cancelled() => None,
    };

    if status.is_none() {
        let _ = child.kill().await;
        if let Some(task) = progress_task.take() {
            let _ = task.await;
        }
        if let Some(task) = stderr_task.take() {
            let _ = task.await;
        }
        let _ = std::fs::remove_file(&temp_output);
        return Err("Conversión cancelada.".to_string());
    }

    if let Some(task) = progress_task.take() {
        let _ = task.await;
    }

    let stderr_text = if let Some(task) = stderr_task.take() {
        task.await.unwrap_or_default()
    } else {
        String::new()
    };

    let status = match status.unwrap() {
        Ok(status) => status,
        Err(error) => {
            let _ = std::fs::remove_file(&temp_output);
            return Err(format!("Error esperando al motor: {error}"));
        }
    };

    if !status.success() {
        let detail = stderr_text.trim();
        let _ = std::fs::remove_file(&temp_output);
        return Err(if detail.is_empty() {
            "El motor terminó con un error desconocido.".to_string()
        } else {
            format!("El motor terminó con un error:\n{detail}")
        });
    }

    let metadata = match std::fs::metadata(&temp_output) {
        Ok(metadata) => metadata,
        Err(error) => {
            let _ = std::fs::remove_file(&temp_output);
            return Err(format!("No se generó el archivo de salida: {error}"));
        }
    };

    if metadata.len() == 0 {
        let _ = std::fs::remove_file(&temp_output);
        return Err("El archivo de salida quedó vacío.".to_string());
    }

    if request.overwrite && output.exists() {
        if let Err(error) = std::fs::remove_file(&output) {
            let _ = std::fs::remove_file(&temp_output);
            return Err(format!("No se pudo reemplazar el archivo existente: {error}"));
        }
    }

    if let Err(error) = std::fs::rename(&temp_output, &output) {
        let _ = std::fs::remove_file(&temp_output);
        return Err(format!("No se pudo finalizar el archivo de salida: {error}"));
    }

    emit_progress(app, &request.job_id, 100.0, "completado");

    Ok(ConvertResult {
        output_path: output.to_string_lossy().to_string(),
        output_size: metadata.len(),
        duration_ms: started.elapsed().as_millis() as u64,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unique_output_path_adds_suffix() {
        let directory =
            std::env::temp_dir().join(format!("convertall-test-{}", std::process::id()));
        std::fs::create_dir_all(&directory).expect("crear directorio de prueba");
        let first = directory.join("archivo.png");
        std::fs::write(&first, b"x").expect("crear archivo de prueba");

        let second = unique_output_path(&first);
        assert_eq!(
            second.file_name().and_then(|value| value.to_str()),
            Some("archivo (1).png")
        );

        std::fs::remove_dir_all(&directory).ok();
    }

    #[test]
    fn quality_maps_to_sensible_crf() {
        assert_eq!(quality_to_crf(1), 35);
        assert_eq!(quality_to_crf(100), 15);
        assert!(quality_to_crf(80) < quality_to_crf(30));
    }

    fn request(
        video_height: Option<u32>,
        image_max_side: Option<u32>,
        audio_bitrate: Option<u32>,
    ) -> ConvertRequest {
        ConvertRequest {
            job_id: "prueba".to_string(),
            input_path: "entrada.mp4".to_string(),
            output_dir: String::new(),
            target_format: "mp4".to_string(),
            quality: 80,
            preserve_metadata: true,
            overwrite: false,
            video_height,
            image_max_side,
            audio_bitrate,
        }
    }

    #[test]
    fn video_filter_limits_the_short_side_without_upscaling() {
        assert_eq!(
            video_scale_filter(720),
            "scale=w='if(gt(iw,ih),-2,min(iw,720))':h='if(gt(iw,ih),min(ih,720),-2)':flags=lanczos"
        );
        assert_eq!(
            image_box_filter(1280),
            "scale=w='min(iw,1280)':h='min(ih,1280)':force_original_aspect_ratio=decrease:force_divisible_by=2:flags=lanczos"
        );
    }

    #[test]
    fn validates_options_against_the_target_format() {
        let mp4 = formats::find_by_id("mp4").expect("mp4");
        assert!(validate_options(&mp4, &request(Some(720), None, None)).is_ok());
        assert!(validate_options(&mp4, &request(Some(999), None, None)).is_err());
        assert!(validate_options(&mp4, &request(None, Some(1920), None)).is_err());
        assert!(validate_options(&mp4, &request(None, None, Some(128))).is_err());

        let mp3 = formats::find_by_id("mp3").expect("mp3");
        assert!(validate_options(&mp3, &request(None, None, Some(128))).is_ok());
        assert!(validate_options(&mp3, &request(None, None, Some(123))).is_err());
        assert!(validate_options(&mp3, &request(Some(720), None, None)).is_err());

        let png = formats::find_by_id("png").expect("png");
        assert!(validate_options(&png, &request(None, Some(1920), None)).is_ok());
        assert!(validate_options(&png, &request(None, Some(1000), None)).is_err());
    }

    fn args_of(command: &Command) -> Vec<String> {
        command
            .as_std()
            .get_args()
            .map(|value| value.to_string_lossy().to_string())
            .collect()
    }

    #[test]
    fn builds_video_and_audio_options_into_ffmpeg() {
        let mut video = Command::new("ffmpeg");
        build_ffmpeg(
            &mut video,
            Path::new("entrada.mp4"),
            Path::new("salida.mp4"),
            "mp4",
            &request(Some(720), None, None),
        );
        let video_args = args_of(&video);
        assert!(video_args
            .windows(2)
            .any(|pair| pair[0] == "-vf" && pair[1] == video_scale_filter(720)));

        let mut audio = Command::new("ffmpeg");
        build_ffmpeg(
            &mut audio,
            Path::new("entrada.wav"),
            Path::new("salida.mp3"),
            "mp3",
            &request(None, None, Some(128)),
        );
        let audio_args = args_of(&audio);
        assert!(audio_args
            .windows(2)
            .any(|pair| pair[0] == "-b:a" && pair[1] == "128k"));
        assert!(!audio_args.iter().any(|arg| arg == "-q:a"));
    }

    #[test]
    fn builds_image_size_options_into_magick_and_vips() {
        let mut magick = Command::new("magick");
        build_magick(
            &mut magick,
            Path::new("entrada.png"),
            Path::new("salida.jpg"),
            "jpg",
            &request(None, Some(1920), None),
        );
        let magick_args = args_of(&magick);
        assert!(magick_args
            .windows(2)
            .any(|pair| pair[0] == "-resize" && pair[1] == "1920x1920>"));

        let mut vips = Command::new("vips");
        build_vips(
            &mut vips,
            Path::new("entrada.png"),
            Path::new("salida.jpg"),
            &request(None, Some(1920), None),
        );
        let vips_args = args_of(&vips);
        assert_eq!(vips_args.first().map(String::as_str), Some("thumbnail"));
        assert!(vips_args
            .windows(2)
            .any(|pair| pair[0] == "--height" && pair[1] == "1920"));
        assert!(vips_args
            .windows(2)
            .any(|pair| pair[0] == "--size" && pair[1] == "down"));
    }
}
