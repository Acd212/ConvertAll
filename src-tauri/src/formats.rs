use serde::Serialize;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Category {
    Image,
    Audio,
    Video,
    Document,
    Unknown,
}

impl Category {
    pub fn parse(value: &str) -> Self {
        match value.to_ascii_lowercase().as_str() {
            "image" | "imagen" => Self::Image,
            "audio" => Self::Audio,
            "video" => Self::Video,
            "document" | "documento" => Self::Document,
            _ => Self::Unknown,
        }
    }
}

/// Alturas admitidas para vídeo. `1080p` limita el lado corto a 1080 px,
/// así que también funciona con vídeo vertical sin ampliarlo.
pub const VIDEO_HEIGHTS: &[u32] = &[2160, 1440, 1080, 720, 480, 360];

/// Lados mayores admitidos para imágenes. Se aplica como caja cuadrada y
/// nunca se amplía la imagen original.
pub const IMAGE_SIDES: &[u32] = &[3840, 2560, 1920, 1280, 1024, 800, 640, 480];

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FormatDef {
    pub id: String,
    pub label: String,
    pub extension: String,
    pub category: Category,
    pub can_read: bool,
    pub can_write: bool,
    pub engine: String,
    /// Alturas que acepta el formato de salida (vacío si no es vídeo).
    pub video_heights: Vec<u32>,
    /// Lados mayores que acepta el formato de salida (vacío si no es imagen).
    pub image_sides: Vec<u32>,
    /// Bitrates en kbps que acepta el formato de salida (vacío si no aplica).
    pub audio_bitrates: Vec<u32>,
}

fn video_heights_for(category: Category, can_write: bool) -> Vec<u32> {
    if category == Category::Video && can_write {
        VIDEO_HEIGHTS.to_vec()
    } else {
        Vec::new()
    }
}

fn image_sides_for(id: &str, category: Category, can_write: bool) -> Vec<u32> {
    // ICO y CUR generan sus propios tamaños de icono; no se les aplica un
    // tamaño arbitrario para no romper el formato.
    if category == Category::Image && can_write && !matches!(id, "ico" | "cur") {
        IMAGE_SIDES.to_vec()
    } else {
        Vec::new()
    }
}

fn audio_bitrates_for(id: &str) -> Vec<u32> {
    match id {
        "mp3" => vec![64, 96, 128, 160, 192, 256, 320],
        // libvorbis rechaza 256 y 320 kbps en esta compilación.
        "ogg" => vec![64, 96, 128, 160, 192],
        "opus" => vec![32, 48, 64, 96, 128, 160, 192, 256],
        "aac" | "m4a" | "m4b" => vec![64, 96, 128, 160, 192, 256, 320],
        "wma" => vec![64, 96, 128, 160, 192],
        "ac3" => vec![128, 192, 256, 384, 448, 640],
        "dts" => vec![768, 1536],
        _ => Vec::new(),
    }
}

fn format(
    id: &str,
    label: &str,
    extension: &str,
    category: Category,
    can_read: bool,
    can_write: bool,
    engine: &str,
) -> FormatDef {
    FormatDef {
        id: id.to_string(),
        label: label.to_string(),
        extension: extension.to_string(),
        category,
        can_read,
        can_write,
        engine: engine.to_string(),
        video_heights: video_heights_for(category, can_write),
        image_sides: image_sides_for(id, category, can_write),
        audio_bitrates: if category == Category::Audio && can_write {
            audio_bitrates_for(id)
        } else {
            Vec::new()
        },
    }
}

fn image(id: &str, label: &str) -> FormatDef {
    format(id, label, id, Category::Image, true, true, "image")
}

fn image_read_only(id: &str, label: &str) -> FormatDef {
    format(id, label, id, Category::Image, true, false, "image")
}

fn audio(id: &str, label: &str) -> FormatDef {
    format(id, label, id, Category::Audio, true, true, "ffmpeg")
}

fn audio_read_only(id: &str, label: &str) -> FormatDef {
    format(id, label, id, Category::Audio, true, false, "ffmpeg")
}

fn video(id: &str, label: &str) -> FormatDef {
    format(id, label, id, Category::Video, true, true, "ffmpeg")
}

fn document(id: &str, label: &str) -> FormatDef {
    format(id, label, id, Category::Document, true, true, "pandoc")
}

fn document_read_only(id: &str, label: &str) -> FormatDef {
    format(id, label, id, Category::Document, true, false, "pandoc")
}

fn document_write_only(id: &str, label: &str) -> FormatDef {
    format(id, label, id, Category::Document, false, true, "pandoc")
}

pub fn catalog() -> Vec<FormatDef> {
    vec![
        // Imagen
        image("jpg", "JPEG"),
        image("png", "PNG"),
        image("webp", "WebP"),
        image("avif", "AVIF"),
        image("gif", "GIF"),
        image("bmp", "BMP"),
        image("tiff", "TIFF"),
        image("ico", "ICO"),
        image("heic", "HEIC"),
        image("jxl", "JPEG XL"),
        image("psd", "Photoshop"),
        image("tga", "TGA"),
        image("dds", "DDS"),
        image("exr", "OpenEXR"),
        image("hdr", "HDR"),
        image("ppm", "PPM"),
        image("pgm", "PGM"),
        image("pbm", "PBM"),
        image("pnm", "PNM"),
        image("xbm", "XBM"),
        image("xpm", "XPM"),
        image("wbmp", "WBMP"),
        image("pcx", "PCX"),
        image("cur", "CUR"),
        // PostScript/EPS requiere Ghostscript; en Windows puede no estar.
        image_read_only("eps", "EPS"),
        image_read_only("svg", "SVG"),
        image_read_only("ai", "Illustrator"),
        image_read_only("dng", "DNG"),
        image_read_only("cr2", "Canon RAW"),
        image_read_only("nef", "Nikon RAW"),
        image_read_only("arw", "Sony RAW"),
        image_read_only("raf", "Fuji RAW"),
        image_read_only("orf", "Olympus RAW"),
        image_read_only("rw2", "Panasonic RAW"),
        // Audio
        audio("mp3", "MP3"),
        audio("wav", "WAV"),
        audio("flac", "FLAC"),
        audio("ogg", "OGG Vorbis"),
        audio("opus", "Opus"),
        audio("aac", "AAC"),
        audio("m4a", "M4A"),
        audio("wma", "WMA"),
        audio("aiff", "AIFF"),
        audio("ac3", "AC-3"),
        audio("dts", "DTS"),
        audio("amr", "AMR"),
        audio_read_only("ape", "Monkey's Audio"),
        audio("mka", "MKA"),
        audio("caf", "CAF"),
        audio("wv", "WavPack"),
        audio("m4b", "M4B"),
        // Vídeo
        video("mp4", "MP4"),
        video("mkv", "Matroska"),
        video("webm", "WebM"),
        video("mov", "QuickTime"),
        video("avi", "AVI"),
        video("wmv", "WMV"),
        video("flv", "FLV"),
        video("m4v", "M4V"),
        video("mpg", "MPEG"),
        video("ts", "MPEG-TS"),
        video("m2ts", "M2TS"),
        video("3gp", "3GP"),
        video("ogv", "OGG Video"),
        video("vob", "VOB"),
        video("asf", "ASF"),
        video("f4v", "F4V"),
        // Documentos
        document("md", "Markdown"),
        document("html", "HTML"),
        document("docx", "Word"),
        document("odt", "OpenDocument"),
        document("rtf", "RTF"),
        document("txt", "Texto plano"),
        document("rst", "reStructuredText"),
        document("org", "Org"),
        document("tex", "LaTeX"),
        document("epub", "EPUB"),
        document("ipynb", "Jupyter"),
        document("json", "Pandoc JSON"),
        document("docbook", "DocBook"),
        document("mediawiki", "MediaWiki"),
        document("textile", "Textile"),
        document("asciidoc", "AsciiDoc"),
        document("man", "Man"),
        document("opml", "OPML"),
        document("native", "Pandoc Native"),
        document_read_only("bibtex", "BibTeX"),
        document_read_only("csv", "CSV"),
        document_write_only("pptx", "PowerPoint"),
    ]
}

fn canonical_id(ext: &str) -> &str {
    match ext {
        "jpeg" | "jpe" | "jfif" => "jpg",
        "tif" => "tiff",
        "aif" | "aifc" => "aiff",
        "oga" => "ogg",
        "mpeg" => "mpg",
        "markdown" | "mdown" | "mkd" => "md",
        "htm" | "xhtml" => "html",
        "latex" => "tex",
        "text" => "txt",
        "dbk" => "docbook",
        "adoc" => "asciidoc",
        _ => ext,
    }
}

pub fn find_by_extension(ext: &str) -> Option<FormatDef> {
    let normalized = ext
        .trim()
        .trim_start_matches('.')
        .to_ascii_lowercase();
    if normalized.is_empty() {
        return None;
    }
    let id = canonical_id(&normalized);
    find_by_id(id)
}

pub fn find_by_id(id: &str) -> Option<FormatDef> {
    let normalized = id.trim().to_ascii_lowercase();
    catalog().into_iter().find(|format| format.id == normalized)
}

pub fn format_for_path(path: &Path) -> Option<FormatDef> {
    path.extension()
        .and_then(|extension| extension.to_str())
        .and_then(find_by_extension)
}

pub fn targets_for(category: Category) -> Vec<FormatDef> {
    catalog()
        .into_iter()
        .filter(|format| {
            if !format.can_write {
                return false;
            }
            match category {
                Category::Image => format.category == Category::Image,
                Category::Audio => format.category == Category::Audio,
                Category::Video => {
                    format.category == Category::Video
                        || format.category == Category::Audio
                        || format.id == "gif"
                }
                Category::Document => format.category == Category::Document,
                Category::Unknown => false,
            }
        })
        .collect()
}

pub fn is_target_allowed(category: Category, target_id: &str) -> bool {
    targets_for(category)
        .iter()
        .any(|format| format.id == target_id)
}

pub fn suggested_target(category: Category, input_id: &str) -> String {
    let candidates: &[&str] = match category {
        Category::Image => &["jpg", "png", "webp"],
        Category::Audio => &["mp3", "wav", "flac"],
        Category::Video => &["mp4", "mkv", "webm"],
        Category::Document => &["docx", "html", "md", "txt"],
        Category::Unknown => &[],
    };

    candidates
        .iter()
        .find(|candidate| **candidate != input_id)
        .or_else(|| candidates.first())
        .copied()
        .unwrap_or("")
        .to_string()
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileInfo {
    pub path: String,
    pub name: String,
    pub extension: String,
    pub size: u64,
    pub category: Category,
    pub format_id: String,
    pub engine: String,
    pub engine_available: bool,
    pub suggested_target: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jpeg_alias_maps_to_jpg() {
        let format = find_by_extension(".JPEG").expect("jpeg debe existir");
        assert_eq!(format.id, "jpg");
        assert_eq!(format.category, Category::Image);
    }

    #[test]
    fn video_targets_include_audio_and_gif() {
        let targets = targets_for(Category::Video);
        assert!(targets.iter().any(|format| format.id == "mp3"));
        assert!(targets.iter().any(|format| format.id == "gif"));
        assert!(targets.iter().any(|format| format.id == "mp4"));
        assert!(!targets.iter().any(|format| format.id == "png"));
    }

    #[test]
    fn document_targets_do_not_include_pdf_yet() {
        let targets = targets_for(Category::Document);
        assert!(!targets.iter().any(|format| format.id == "pdf"));
        assert!(targets.iter().any(|format| format.id == "docx"));
    }

    #[test]
    fn suggested_target_changes_when_possible() {
        assert_eq!(suggested_target(Category::Image, "jpg"), "png");
        assert_eq!(suggested_target(Category::Audio, "mp3"), "wav");
        assert_eq!(suggested_target(Category::Video, "mp4"), "mkv");
    }

    #[test]
    fn ape_is_read_only_because_ffmpeg_cannot_encode_it() {
        let targets = targets_for(Category::Audio);
        assert!(!targets.iter().any(|format| format.id == "ape"));
        assert!(find_by_id("ape").expect("ape debe existir").can_read);
    }

    #[test]
    fn options_are_exposed_per_format() {
        let mp4 = find_by_id("mp4").expect("mp4");
        assert_eq!(mp4.video_heights, VIDEO_HEIGHTS);
        assert!(mp4.image_sides.is_empty());
        assert!(mp4.audio_bitrates.is_empty());

        let mp3 = find_by_id("mp3").expect("mp3");
        assert!(mp3.audio_bitrates.contains(&128));
        assert!(mp3.video_heights.is_empty());

        let flac = find_by_id("flac").expect("flac");
        assert!(flac.audio_bitrates.is_empty());

        let png = find_by_id("png").expect("png");
        assert_eq!(png.image_sides, IMAGE_SIDES);
        assert!(png.audio_bitrates.is_empty());

        let ico = find_by_id("ico").expect("ico");
        assert!(ico.image_sides.is_empty());
    }

    #[test]
    fn gif_accepts_image_size_and_video_accepts_audio_targets() {
        let gif = find_by_id("gif").expect("gif");
        assert!(!gif.image_sides.is_empty());

        let video_targets = targets_for(Category::Video);
        assert!(video_targets.iter().any(|format| format.id == "mp3"));
        assert!(video_targets.iter().any(|format| format.id == "gif"));
    }
}
