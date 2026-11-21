use serde::Serialize;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use tauri::{AppHandle, Manager};

pub const FFMPEG: &str = "ffmpeg";
pub const MAGICK: &str = "magick";
pub const VIPS: &str = "vips";
pub const PANDOC: &str = "pandoc";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineInfo {
    pub id: String,
    pub name: String,
    pub available: bool,
    pub version: Option<String>,
    pub path: Option<String>,
}

fn engine_name(id: &str) -> &'static str {
    match id {
        FFMPEG => "FFmpeg",
        MAGICK => "ImageMagick",
        VIPS => "libvips",
        PANDOC => "Pandoc",
        _ => "Motor",
    }
}

fn exe_name(id: &str) -> String {
    format!("{id}{}", std::env::consts::EXE_SUFFIX)
}

fn find_in_path(exe: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    for directory in std::env::split_paths(&path) {
        let candidate = directory.join(exe);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

fn tool_roots(app: &AppHandle) -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Some(directory) = std::env::var_os("CONVERTALL_TOOLS_DIR") {
        roots.push(PathBuf::from(directory));
    }
    if let Ok(resource_dir) = app.path().resource_dir() {
        roots.push(resource_dir.join("tools"));
    }
    roots
}

fn bundled_tool(app: &AppHandle, id: &str) -> Option<PathBuf> {
    let exe = exe_name(id);
    for root in tool_roots(app) {
        let candidates = [root.join(&exe), root.join(id).join(&exe)];
        for candidate in candidates {
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

pub fn resolve(app: &AppHandle, id: &str) -> Option<PathBuf> {
    bundled_tool(app, id).or_else(|| find_in_path(&exe_name(id)))
}

pub fn resolve_ffprobe(app: &AppHandle) -> Option<PathBuf> {
    if let Some(path) = bundled_tool(app, "ffprobe") {
        return Some(path);
    }
    if let Some(ffmpeg) = resolve(app, FFMPEG) {
        if let Some(parent) = ffmpeg.parent() {
            let candidate = parent.join(exe_name("ffprobe"));
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    find_in_path(&exe_name("ffprobe"))
}

pub fn available(app: &AppHandle, id: &str) -> bool {
    resolve(app, id).is_some()
}

pub(crate) fn hidden_command(path: &Path) -> Command {
    #[allow(unused_mut)]
    let mut command = Command::new(path);
    sanitize_std_command(&mut command);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
    }
    command
}

/// Dentro de un AppImage se hereda `LD_LIBRARY_PATH` con las bibliotecas
/// empaquetadas (por ejemplo, un libtiff más antiguo). Eso rompe los motores
/// del sistema, como libvips. Los procesos hijos deben usar las bibliotecas
/// del sistema, así que se limpia ese entorno para ellos.
fn inside_appimage() -> bool {
    std::env::var_os("APPIMAGE").is_some() || std::env::var_os("APPDIR").is_some()
}

pub(crate) fn sanitize_std_command(command: &mut Command) {
    if inside_appimage() {
        command.env_remove("LD_LIBRARY_PATH");
        command.env_remove("LD_PRELOAD");
    }
}

pub(crate) fn sanitize_tokio_command(command: &mut tokio::process::Command) {
    if inside_appimage() {
        command.env_remove("LD_LIBRARY_PATH");
        command.env_remove("LD_PRELOAD");
    }
}

fn version_of(path: &Path, id: &str) -> Option<String> {
    let argument = match id {
        FFMPEG => "-version",
        MAGICK => "-version",
        VIPS => "--version",
        PANDOC => "--version",
        _ => "--version",
    };

    let output = hidden_command(path)
        .arg(argument)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .ok()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let text = if stdout.trim().is_empty() {
        stderr
    } else {
        stdout
    };

    text.lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .and_then(|line| short_version(id, line))
}

fn short_version(id: &str, line: &str) -> Option<String> {
    let token = |text: &str| {
        text.split_whitespace()
            .next()
            .unwrap_or(text)
            .trim()
            .to_string()
    };

    match id {
        FFMPEG => line
            .strip_prefix("ffmpeg version ")
            .map(token),
        MAGICK => line
            .strip_prefix("Version: ImageMagick ")
            .map(token),
        VIPS => line
            .trim()
            .strip_prefix("vips-")
            .map(str::to_string)
            .or_else(|| {
                let trimmed = line.trim();
                trimmed
                    .chars()
                    .next()
                    .filter(char::is_ascii_digit)
                    .map(|_| trimmed.to_string())
            }),
        PANDOC => line.strip_prefix("pandoc ").map(token),
        _ => Some(line.trim().to_string()),
    }
}

pub fn detect(app: &AppHandle) -> Vec<EngineInfo> {
    [FFMPEG, MAGICK, VIPS, PANDOC]
        .into_iter()
        .map(|id| {
            let path = resolve(app, id);
            let version = path.as_deref().and_then(|value| version_of(value, id));
            EngineInfo {
                id: id.to_string(),
                name: engine_name(id).to_string(),
                available: path.is_some(),
                version,
                path: path.map(|value| value.to_string_lossy().to_string()),
            }
        })
        .collect()
}

pub fn configure_tokio_command(
    command: &mut tokio::process::Command,
    engine: &str,
    path: &Path,
) {
    sanitize_tokio_command(command);

    if let Some(parent) = path.parent() {
        let mut paths = vec![parent.to_path_buf()];
        if let Some(existing) = std::env::var_os("PATH") {
            paths.extend(std::env::split_paths(&existing));
        }
        if let Ok(joined) = std::env::join_paths(paths) {
            command.env("PATH", joined);
        }

        let bundled = std::env::var_os("CONVERTALL_TOOLS_DIR").is_some()
            || path.to_string_lossy().contains("tools");
        if engine == MAGICK && bundled {
            command.env("MAGICK_HOME", parent);
        }
    }

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
    }
}

#[cfg(test)]
mod tests {
    use super::short_version;

    #[test]
    fn parses_known_version_lines() {
        assert_eq!(
            short_version("ffmpeg", "ffmpeg version n9.1-essentials_build Copyright").as_deref(),
            Some("n9.1-essentials_build")
        );
        assert_eq!(
            short_version("magick", "Version: ImageMagick 7.1.2-36 Q16 x64").as_deref(),
            Some("7.1.2-36")
        );
        assert_eq!(short_version("vips", "vips-8.18.7").as_deref(), Some("8.18.7"));
        assert_eq!(short_version("pandoc", "pandoc 3.14").as_deref(), Some("3.14"));
    }

    #[test]
    fn ignores_loader_errors() {
        let line = "/usr/bin/vips: /tmp/AppImage/usr/lib/libtiff.so.6: version `LIBTIFF_4.6.1' not found";
        assert_eq!(short_version("vips", line), None);
        assert_eq!(short_version("ffmpeg", "error while loading shared libraries"), None);
    }
}
