use crate::engines;
use crate::formats::Category;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::process::Stdio;
use tauri::AppHandle;

/// Capacidades reales del motor que se usará para una categoría.
///
/// `All` significa "no se pudo comprobar": se permite todo y el error, si lo
/// hay, aparecerá al convertir. Es mejor que ocultar formatos válidos.
pub enum Capabilities {
    All,
    Encoders(HashSet<String>),
    Magick(HashMap<String, String>),
    Pandoc(HashSet<String>),
}

impl Capabilities {
    pub fn supports(&self, id: &str) -> bool {
        match self {
            Self::All => true,
            Self::Encoders(available) => required_encoders(id)
                .map(|required| required.iter().all(|name| available.contains(*name)))
                .unwrap_or(true),
            Self::Magick(formats) => {
                let name = magick_name(id);
                formats
                    .get(&name)
                    .map(|mode| mode.as_bytes().get(1) == Some(&b'w'))
                    .unwrap_or(false)
            }
            Self::Pandoc(outputs) => outputs.contains(pandoc_output_name(id)),
        }
    }
}

fn run(engine: &str, path: &Path, args: &[&str]) -> Option<String> {
    let mut command = engines::hidden_command(path);
    command
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());

    if engine == engines::MAGICK {
        if let Some(parent) = path.parent() {
            command.env("MAGICK_HOME", parent);
            let mut paths = vec![parent.to_path_buf()];
            if let Some(existing) = std::env::var_os("PATH") {
                paths.extend(std::env::split_paths(&existing));
            }
            if let Ok(joined) = std::env::join_paths(paths) {
                command.env("PATH", joined);
            }
        }
    }

    let output = command.output().ok()?;
    if !output.status.success() {
        return None;
    }

    Some(String::from_utf8_lossy(&output.stdout).to_string())
}

pub fn load(app: &AppHandle, category: Category) -> Capabilities {
    match category {
        Category::Image => {
            if !engines::available(app, engines::MAGICK) {
                return Capabilities::All;
            }
            engines::resolve(app, engines::MAGICK)
                .and_then(|path| run(engines::MAGICK, &path, &["-list", "format"]))
                .map(|text| Capabilities::Magick(parse_magick_formats(&text)))
                .unwrap_or(Capabilities::All)
        }
        Category::Audio | Category::Video => {
            if !engines::available(app, engines::FFMPEG) {
                return Capabilities::All;
            }
            engines::resolve(app, engines::FFMPEG)
                .and_then(|path| run(engines::FFMPEG, &path, &["-hide_banner", "-encoders"]))
                .map(|text| Capabilities::Encoders(parse_encoders(&text)))
                .unwrap_or(Capabilities::All)
        }
        Category::Document => {
            if !engines::available(app, engines::PANDOC) {
                return Capabilities::All;
            }
            engines::resolve(app, engines::PANDOC)
                .and_then(|path| run(engines::PANDOC, &path, &["--list-output-formats"]))
                .map(|text| Capabilities::Pandoc(parse_lines(&text)))
                .unwrap_or(Capabilities::All)
        }
        Category::Unknown => Capabilities::All,
    }
}

fn parse_lines(text: &str) -> HashSet<String> {
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect()
}

fn parse_encoders(text: &str) -> HashSet<String> {
    text.lines()
        .filter_map(|line| {
            let mut tokens = line.split_whitespace();
            let flags = tokens.next()?;
            if flags.len() != 6 || !flags.chars().all(|c| "VASFXBD.".contains(c)) {
                return None;
            }
            let name = tokens.next()?;
            if name == "=" {
                return None;
            }
            Some(name.to_string())
        })
        .collect()
}

fn parse_magick_formats(text: &str) -> HashMap<String, String> {
    let mut formats = HashMap::new();

    for line in text.lines() {
        let tokens: Vec<&str> = line.split_whitespace().collect();
        if tokens.len() < 3 {
            continue;
        }
        let mode = tokens[2];
        if mode.len() != 3 || !mode.chars().all(|c| "rw+-".contains(c)) {
            continue;
        }
        let name = tokens[0].trim_end_matches('*').to_ascii_uppercase();
        if !name.is_empty() {
            formats.insert(name, mode.to_string());
        }
    }

    formats
}

fn magick_name(id: &str) -> String {
    match id {
        "jpg" => "JPEG".to_string(),
        other => other.to_ascii_uppercase(),
    }
}

fn pandoc_output_name(id: &str) -> &str {
    match id {
        "md" => "gfm",
        "txt" => "plain",
        "tex" => "latex",
        other => other,
    }
}

fn required_encoders(target: &str) -> Option<&'static [&'static str]> {
    let required: &'static [&'static str] = match target {
        "mp3" => &["libmp3lame"],
        "wav" | "caf" => &["pcm_s16le"],
        "flac" => &["flac"],
        "ogg" => &["libvorbis"],
        "opus" => &["libopus"],
        "aac" | "m4a" | "m4b" => &["aac"],
        "wma" => &["wmav2"],
        "aiff" => &["pcm_s16be"],
        "ac3" => &["ac3"],
        "dts" => &["dca"],
        "amr" => &["libopencore_amrnb"],
        "wv" => &["wavpack"],
        "mka" => &[],
        "mp4" | "mkv" | "mov" | "m4v" | "f4v" | "flv" | "ts" | "m2ts" | "3gp" => {
            &["libx264", "aac"]
        }
        "avi" => &["libx264", "libmp3lame"],
        "webm" => &["libvpx-vp9", "libopus"],
        "wmv" | "asf" => &["wmv2", "wmav2"],
        "mpg" | "vob" => &["mpeg2video", "mp2"],
        "ogv" => &["libtheora", "libvorbis"],
        "gif" => &["gif"],
        _ => return None,
    };

    Some(required)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_ffmpeg_encoders() {
        let text = "Encoders:\n V..... = Video\n ------\n V....D libx264   H.264\n A....D aac       AAC\n";
        let encoders = parse_encoders(text);
        assert!(encoders.contains("libx264"));
        assert!(encoders.contains("aac"));
        assert!(!encoders.contains("="));
        assert!(!encoders.contains("Encoders:"));
    }

    #[test]
    fn parses_magick_modes() {
        let text = "   JPEG* JPEG       rw-   JFIF\n   AVIF  HEIC       rw+   AV1\n   ARW  DNG        r--   Sony\n   * native blob support\n";
        let formats = parse_magick_formats(text);
        assert_eq!(formats.get("JPEG").map(String::as_str), Some("rw-"));
        assert_eq!(formats.get("AVIF").map(String::as_str), Some("rw+"));
        assert_eq!(formats.get("ARW").map(String::as_str), Some("r--"));
    }

    #[test]
    fn magick_jpg_maps_to_jpeg() {
        assert_eq!(magick_name("jpg"), "JPEG");
        assert_eq!(magick_name("webp"), "WEBP");
    }

    #[test]
    fn required_encoders_cover_video_targets() {
        let webm = required_encoders("webm").expect("webm");
        assert!(webm.contains(&"libvpx-vp9"));
        assert!(webm.contains(&"libopus"));
        assert_eq!(required_encoders("mka").map(<[&str]>::len), Some(0));
    }
}
