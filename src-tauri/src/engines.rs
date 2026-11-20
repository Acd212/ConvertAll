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
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
    }
    command
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
        .map(|line| short_version(id, line))
}

fn short_version(id: &str, line: &str) -> String {
    let token = |text: &str| {
        text.split_whitespace()
            .next()
            .unwrap_or(text)
            .trim()
            .to_string()
    };

    match id {
        FFMPEG => token(line.strip_prefix("ffmpeg version ").unwrap_or(line)),
        MAGICK => token(line.strip_prefix("Version: ImageMagick ").unwrap_or(line)),
        VIPS => line
            .trim()
            .strip_prefix("vips-")
            .map(str::to_string)
            .unwrap_or_else(|| line.trim().to_string()),
        PANDOC => token(line.strip_prefix("pandoc ").unwrap_or(line)),
        _ => line.trim().to_string(),
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
