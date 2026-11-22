use serde::Serialize;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use tauri::{AppHandle, Manager};

pub const FFMPEG: &str = "ffmpeg";
pub const MAGICK: &str = "magick";
pub const VIPS: &str = "vips";
pub const PANDOC: &str = "pandoc";
pub const LIBREOFFICE: &str = "libreoffice";
pub const MSOFFICE: &str = "msoffice";
/// Motor lógico para los formatos de Office: lo resuelve LibreOffice o,
/// en Windows, Microsoft Office.
pub const OFFICE: &str = "office";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OfficeApp {
    Word,
    Excel,
    PowerPoint,
}

impl OfficeApp {
    #[allow(dead_code)]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Word => "word",
            Self::Excel => "excel",
            Self::PowerPoint => "powerpoint",
        }
    }

    #[cfg(windows)]
    fn exe_name(self) -> &'static str {
        match self {
            Self::Word => "WINWORD.EXE",
            Self::Excel => "EXCEL.EXE",
            Self::PowerPoint => "POWERPNT.EXE",
        }
    }

    #[cfg(windows)]
    fn registry_name(self) -> &'static str {
        match self {
            Self::Word => "WINWORD.EXE",
            Self::Excel => "EXCEL.EXE",
            Self::PowerPoint => "POWERPNT.EXE",
        }
    }
}

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
        LIBREOFFICE => "LibreOffice",
        MSOFFICE => "Microsoft Office",
        OFFICE => "Office",
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

#[cfg(windows)]
fn program_files_roots() -> Vec<PathBuf> {
    ["ProgramFiles", "ProgramFiles(x86)"]
        .iter()
        .filter_map(std::env::var_os)
        .map(PathBuf::from)
        .collect()
}

#[cfg(windows)]
fn reg_query_default(key: &str) -> Option<PathBuf> {
    let output = hidden_command(Path::new("reg.exe"))
        .args(["query", key, "/ve"])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let text = String::from_utf8_lossy(&output.stdout);
    let line = text.lines().find(|line| line.contains("REG_SZ"))?;
    let value = line.split("REG_SZ").nth(1)?.trim().trim_matches('"');
    let path = PathBuf::from(value);
    path.is_file().then_some(path)
}

#[cfg(windows)]
pub fn resolve_msoffice_exe(_app: &AppHandle, app_kind: OfficeApp) -> Option<PathBuf> {
    let exe = app_kind.exe_name();

    // 1. Registro: App Paths (funciona con Click-to-Run y MSI).
    for hive in ["HKLM", "HKCU"] {
        for wow in ["", "WOW6432Node\\"] {
            let key = format!(
                "{hive}\\SOFTWARE\\{wow}Microsoft\\Windows\\CurrentVersion\\App Paths\\{}",
                app_kind.registry_name()
            );
            if let Some(path) = reg_query_default(&key) {
                return Some(path);
            }
        }
    }

    // 2. Rutas habituales de Office 2016/2013/2010.
    for base in program_files_roots() {
        for version in ["Office16", "Office15", "Office14"] {
            let candidates = [
                base.join("Microsoft Office")
                    .join("root")
                    .join(version)
                    .join(exe),
                base.join("Microsoft Office").join(version).join(exe),
            ];
            for candidate in candidates {
                if candidate.is_file() {
                    return Some(candidate);
                }
            }
        }
    }

    // 3. PATH (instalaciones personalizadas).
    find_in_path(exe)
}

#[cfg(not(windows))]
#[allow(dead_code)]
pub fn resolve_msoffice_exe(_app: &AppHandle, _app_kind: OfficeApp) -> Option<PathBuf> {
    None
}

#[allow(dead_code)]
pub fn msoffice_available_for(app: &AppHandle, app_kind: OfficeApp) -> bool {
    resolve_msoffice_exe(app, app_kind).is_some()
}

#[cfg(windows)]
pub fn resolve_msoffice(app: &AppHandle) -> Option<PathBuf> {
    [
        OfficeApp::Word,
        OfficeApp::Excel,
        OfficeApp::PowerPoint,
    ]
    .into_iter()
    .find_map(|kind| resolve_msoffice_exe(app, kind))
}

#[cfg(not(windows))]
pub fn resolve_msoffice(_app: &AppHandle) -> Option<PathBuf> {
    None
}

#[cfg(windows)]
pub fn resolve_powershell(_app: &AppHandle) -> Option<PathBuf> {
    find_in_path("powershell.exe")
        .or_else(|| find_in_path("pwsh.exe"))
        .or_else(|| {
            let root = std::env::var_os("SystemRoot")?;
            let candidate = PathBuf::from(root)
                .join("System32")
                .join("WindowsPowerShell")
                .join("v1.0")
                .join("powershell.exe");
            candidate.is_file().then_some(candidate)
        })
}

#[cfg(not(windows))]
pub fn resolve_powershell(_app: &AppHandle) -> Option<PathBuf> {
    None
}

fn libreoffice_candidates() -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    #[cfg(windows)]
    {
        for base in program_files_roots() {
            candidates.push(base.join("LibreOffice").join("program").join("soffice.exe"));
        }
        if let Some(local) = std::env::var_os("LOCALAPPDATA") {
            candidates.push(
                PathBuf::from(local)
                    .join("Programs")
                    .join("LibreOffice")
                    .join("program")
                    .join("soffice.exe"),
            );
        }
    }

    #[cfg(target_os = "linux")]
    {
        candidates.push(PathBuf::from("/usr/bin/soffice"));
        candidates.push(PathBuf::from("/usr/local/bin/soffice"));
        candidates.push(PathBuf::from("/opt/libreoffice/program/soffice"));
    }

    candidates
}

pub fn resolve_libreoffice(app: &AppHandle) -> Option<PathBuf> {
    // Permite una instalación propia o un `soffice` de prueba en tools/.
    for id in ["soffice", "libreoffice"] {
        if let Some(path) = bundled_tool(app, id) {
            return Some(path);
        }
    }

    for id in ["soffice", "libreoffice"] {
        if let Some(path) = find_in_path(&exe_name(id)) {
            return Some(path);
        }
    }

    libreoffice_candidates()
        .into_iter()
        .find(|candidate| candidate.is_file())
}

pub fn resolve(app: &AppHandle, id: &str) -> Option<PathBuf> {
    match id {
        LIBREOFFICE => resolve_libreoffice(app),
        MSOFFICE => resolve_msoffice(app),
        _ => bundled_tool(app, id).or_else(|| find_in_path(&exe_name(id))),
    }
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

pub fn office_available(app: &AppHandle) -> bool {
    available(app, LIBREOFFICE) || available(app, MSOFFICE)
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
    // La versión de Microsoft Office no se obtiene de forma fiable por línea
    // de comandos; la interfaz muestra "Disponible".
    if id == MSOFFICE {
        return None;
    }

    let argument = match id {
        FFMPEG => "-version",
        MAGICK => "-version",
        VIPS => "--version",
        PANDOC => "--version",
        LIBREOFFICE => "--version",
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
        LIBREOFFICE => line.strip_prefix("LibreOffice ").map(token),
        _ => Some(line.trim().to_string()),
    }
}

pub fn detect(app: &AppHandle) -> Vec<EngineInfo> {
    #[allow(unused_mut)]
    let mut ids = vec![FFMPEG, MAGICK, VIPS, PANDOC, LIBREOFFICE];
    #[cfg(windows)]
    ids.push(MSOFFICE);

    ids.into_iter()
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

    if engine == LIBREOFFICE {
        // Fuerza el backend de dibujo sin servidor X en Linux.
        command.env("SAL_USE_VCLPLUGIN", "svp");
    }

    #[cfg(windows)]
    command.creation_flags(0x08000000);
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(
            short_version(
                "libreoffice",
                "LibreOffice 25.2.5.2 420(Build:2) - https://www.libreoffice.org/"
            )
            .as_deref(),
            Some("25.2.5.2")
        );
    }

    #[test]
    fn ignores_loader_errors() {
        let line = "/usr/bin/vips: /tmp/AppImage/usr/lib/libtiff.so.6: version `LIBTIFF_4.6.1' not found";
        assert_eq!(short_version("vips", line), None);
        assert_eq!(short_version("ffmpeg", "error while loading shared libraries"), None);
    }

    #[test]
    fn office_apps_have_stable_ids() {
        assert_eq!(OfficeApp::Word.as_str(), "word");
        assert_eq!(OfficeApp::Excel.as_str(), "excel");
        assert_eq!(OfficeApp::PowerPoint.as_str(), "powerpoint");
    }
}
