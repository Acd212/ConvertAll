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

/// Familia documental. Decide qué motor y qué destinos tienen sentido para
/// una entrada concreta.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocFamily {
    Word,
    Sheet,
    Slide,
    Markup,
}

/// Motor que debe ejecutar una conversión.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EngineKind {
    Ffmpeg,
    Image,
    Pandoc,
    Office,
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

fn document_pandoc(id: &str, label: &str) -> FormatDef {
    format(id, label, id, Category::Document, true, true, "pandoc")
}

fn document_pandoc_read_only(id: &str, label: &str) -> FormatDef {
    format(id, label, id, Category::Document, true, false, "pandoc")
}

fn document_office(id: &str, label: &str) -> FormatDef {
    format(id, label, id, Category::Document, true, true, "office")
}

fn document_pdf() -> FormatDef {
    // Se puede escribir, pero no leer: la importación de PDF de LibreOffice
    // produce un dibujo, no un documento editable fiable.
    format("pdf", "PDF", "pdf", Category::Document, false, true, "office")
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
        // Documentos de Office (LibreOffice o Microsoft Office)
        document_office("docx", "Word"),
        document_office("doc", "Word 97-2003"),
        document_office("odt", "OpenDocument"),
        document_office("rtf", "RTF"),
        document_office("txt", "Texto plano"),
        document_office("html", "HTML"),
        document_office("xlsx", "Excel"),
        document_office("xls", "Excel 97-2003"),
        document_office("ods", "Hoja OpenDocument"),
        document_office("csv", "CSV"),
        document_office("pptx", "PowerPoint"),
        document_office("ppt", "PowerPoint 97-2003"),
        document_office("odp", "Presentación OpenDocument"),
        document_pdf(),
        // Documentos de marcado (Pandoc)
        document_pandoc("md", "Markdown"),
        document_pandoc("rst", "reStructuredText"),
        document_pandoc("org", "Org"),
        document_pandoc("tex", "LaTeX"),
        document_pandoc("epub", "EPUB"),
        document_pandoc("ipynb", "Jupyter"),
        document_pandoc("json", "Pandoc JSON"),
        document_pandoc("docbook", "DocBook"),
        document_pandoc("mediawiki", "MediaWiki"),
        document_pandoc("textile", "Textile"),
        document_pandoc("asciidoc", "AsciiDoc"),
        document_pandoc("man", "Man"),
        document_pandoc("opml", "OPML"),
        document_pandoc("native", "Pandoc Native"),
        document_pandoc_read_only("bibtex", "BibTeX"),
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

/// Familia de un documento. `None` si el formato no es documental.
pub fn doc_family(id: &str) -> Option<DocFamily> {
    match id {
        "docx" | "doc" | "odt" | "rtf" | "txt" | "html" => Some(DocFamily::Word),
        "xlsx" | "xls" | "ods" | "csv" => Some(DocFamily::Sheet),
        "pptx" | "ppt" | "odp" => Some(DocFamily::Slide),
        "md" | "rst" | "org" | "tex" | "epub" | "ipynb" | "json" | "docbook"
        | "mediawiki" | "textile" | "asciidoc" | "man" | "opml" | "native" | "bibtex" => {
            Some(DocFamily::Markup)
        }
        _ => None,
    }
}

/// Formatos que Pandoc puede leer.
pub fn pandoc_reads(id: &str) -> bool {
    matches!(
        id,
        "md" | "rst"
            | "org"
            | "tex"
            | "epub"
            | "ipynb"
            | "json"
            | "docbook"
            | "mediawiki"
            | "textile"
            | "asciidoc"
            | "man"
            | "opml"
            | "native"
            | "bibtex"
            | "docx"
            | "odt"
            | "rtf"
            | "txt"
            | "html"
    )
}

/// Formatos que Pandoc puede escribir.
pub fn pandoc_writes(id: &str) -> bool {
    matches!(
        id,
        "md" | "rst"
            | "org"
            | "tex"
            | "epub"
            | "ipynb"
            | "json"
            | "docbook"
            | "mediawiki"
            | "textile"
            | "asciidoc"
            | "man"
            | "opml"
            | "native"
            | "docx"
            | "odt"
            | "rtf"
            | "txt"
            | "html"
            | "pptx"
    )
}

/// Microsoft Office guarda HTML con una carpeta de recursos asociada, algo
/// que no encaja con la salida de un solo archivo de ConvertAll. Para HTML se
/// usa LibreOffice o Pandoc.
pub fn msoffice_native_target(id: &str) -> bool {
    matches!(
        id,
        "docx"
            | "doc"
            | "odt"
            | "rtf"
            | "txt"
            | "xlsx"
            | "xls"
            | "ods"
            | "csv"
            | "pptx"
            | "ppt"
            | "odp"
            | "pdf"
    )
}

fn is_office_target(id: &str) -> bool {
    matches!(
        id,
        "docx"
            | "doc"
            | "odt"
            | "rtf"
            | "txt"
            | "html"
            | "xlsx"
            | "xls"
            | "ods"
            | "csv"
            | "pptx"
            | "ppt"
            | "odp"
            | "pdf"
    )
}

const WORD_TARGETS: &[&str] = &[
    "pdf", "docx", "doc", "odt", "rtf", "txt", "html", "md", "rst", "org", "tex", "epub",
    "docbook", "mediawiki", "textile", "asciidoc", "man", "opml",
];

const SHEET_TARGETS: &[&str] = &["pdf", "xlsx", "xls", "ods", "csv"];

const SLIDE_TARGETS: &[&str] = &["pdf", "pptx", "ppt", "odp"];

const MARKUP_TARGETS: &[&str] = &[
    "docx", "odt", "rtf", "txt", "html", "md", "rst", "org", "tex", "epub", "ipynb", "json",
    "docbook", "mediawiki", "textile", "asciidoc", "man", "opml", "native", "pptx",
];

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

/// Destinos válidos para una entrada concreta.
pub fn targets_for_input(input_id: &str) -> Vec<FormatDef> {
    let Some(input) = find_by_id(input_id) else {
        return Vec::new();
    };

    if input.category != Category::Document {
        return targets_for(input.category);
    }

    let (ids, exclude_self): (&[&str], bool) = match doc_family(&input.id) {
        Some(DocFamily::Word) => (WORD_TARGETS, true),
        Some(DocFamily::Sheet) => (SHEET_TARGETS, true),
        Some(DocFamily::Slide) => (SLIDE_TARGETS, true),
        Some(DocFamily::Markup) => (MARKUP_TARGETS, false),
        None => return Vec::new(),
    };

    ids.iter()
        .filter(|id| !(exclude_self && **id == input.id))
        .filter_map(|id| find_by_id(id))
        .collect()
}

pub fn is_target_allowed(input_id: &str, target_id: &str) -> bool {
    targets_for_input(input_id)
        .iter()
        .any(|format| format.id == target_id)
}

/// Motor que debe usarse para convertir una entrada en un destino.
pub fn route(input_id: &str, target_id: &str) -> EngineKind {
    let category = find_by_id(input_id)
        .map(|format| format.category)
        .unwrap_or(Category::Unknown);

    match category {
        Category::Image => EngineKind::Image,
        Category::Audio | Category::Video => EngineKind::Ffmpeg,
        Category::Document => {
            if target_id == "pdf" {
                return EngineKind::Office;
            }

            match doc_family(input_id) {
                Some(DocFamily::Word) | Some(DocFamily::Sheet) | Some(DocFamily::Slide) => {
                    if is_office_target(target_id) {
                        EngineKind::Office
                    } else {
                        EngineKind::Pandoc
                    }
                }
                _ => EngineKind::Pandoc,
            }
        }
        Category::Unknown => EngineKind::Pandoc,
    }
}

pub fn suggested_target(category: Category, input_id: &str) -> String {
    let (candidates, skip_self): (&[&str], bool) = match category {
        Category::Image => (&["jpg", "png", "webp"], true),
        Category::Audio => (&["mp3", "wav", "flac"], true),
        Category::Video => (&["mp4", "mkv", "webm"], true),
        Category::Document => match doc_family(input_id) {
            Some(DocFamily::Word) => (&["pdf", "docx", "odt", "html", "txt"], true),
            Some(DocFamily::Sheet) => (&["xlsx", "ods", "csv", "pdf"], true),
            Some(DocFamily::Slide) => (&["pptx", "odp", "pdf"], true),
            _ => (&["docx", "html", "md", "txt"], true),
        },
        Category::Unknown => (&[], true),
    };

    candidates
        .iter()
        .find(|candidate| !skip_self || **candidate != input_id)
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
    fn pdf_is_a_write_only_target() {
        let pdf = find_by_id("pdf").expect("pdf debe existir");
        assert!(!pdf.can_read);
        assert!(pdf.can_write);
        assert_eq!(route("docx", "pdf"), EngineKind::Office);
        assert_eq!(route("xlsx", "pdf"), EngineKind::Office);
        assert_eq!(route("pptx", "pdf"), EngineKind::Office);
        assert!(!is_target_allowed("md", "pdf"));
    }

    #[test]
    fn new_office_formats_are_available() {
        for id in ["doc", "xlsx", "xls", "ods", "ppt", "odp"] {
            let format = find_by_id(id).unwrap_or_else(|| panic!("{id} debe existir"));
            assert!(format.can_read, "{id} debe poder leerse");
            assert!(format.can_write, "{id} debe poder escribirse");
            assert_eq!(format.category, Category::Document);
        }

        let pptx = find_by_id("pptx").expect("pptx");
        assert!(pptx.can_read);
        assert!(pptx.can_write);

        let csv = find_by_id("csv").expect("csv");
        assert!(csv.can_read);
        assert!(csv.can_write);
    }

    #[test]
    fn word_targets_are_family_specific() {
        let targets = targets_for_input("docx");
        let ids: Vec<&str> = targets.iter().map(|format| format.id.as_str()).collect();
        assert!(ids.contains(&"pdf"));
        assert!(ids.contains(&"odt"));
        assert!(ids.contains(&"doc"));
        assert!(ids.contains(&"html"));
        assert!(ids.contains(&"md"));
        assert!(!ids.contains(&"docx"));
        assert!(!ids.contains(&"xlsx"));
    }

    #[test]
    fn sheet_and_slide_targets_stay_in_their_family() {
        let sheet: Vec<String> = targets_for_input("xlsx")
            .into_iter()
            .map(|format| format.id)
            .collect();
        assert!(sheet.contains(&"pdf".to_string()));
        assert!(sheet.contains(&"ods".to_string()));
        assert!(sheet.contains(&"csv".to_string()));
        assert!(!sheet.contains(&"docx".to_string()));

        let slide: Vec<String> = targets_for_input("pptx")
            .into_iter()
            .map(|format| format.id)
            .collect();
        assert!(slide.contains(&"pdf".to_string()));
        assert!(slide.contains(&"odp".to_string()));
        assert!(!slide.contains(&"docx".to_string()));
    }

    #[test]
    fn markup_inputs_keep_pandoc_targets() {
        let targets = targets_for_input("md");
        let ids: Vec<&str> = targets.iter().map(|format| format.id.as_str()).collect();
        assert!(ids.contains(&"docx"));
        assert!(ids.contains(&"epub"));
        assert!(ids.contains(&"pptx"));
        assert!(!ids.contains(&"pdf"));
        assert!(ids.contains(&"md"));
    }

    #[test]
    fn routing_prefers_office_for_office_targets() {
        assert_eq!(route("docx", "odt"), EngineKind::Office);
        assert_eq!(route("xlsx", "csv"), EngineKind::Office);
        assert_eq!(route("pptx", "odp"), EngineKind::Office);
        assert_eq!(route("docx", "md"), EngineKind::Pandoc);
        assert_eq!(route("md", "docx"), EngineKind::Pandoc);
    }

    #[test]
    fn suggested_target_changes_when_possible() {
        assert_eq!(suggested_target(Category::Image, "jpg"), "png");
        assert_eq!(suggested_target(Category::Audio, "mp3"), "wav");
        assert_eq!(suggested_target(Category::Video, "mp4"), "mkv");
        assert_eq!(suggested_target(Category::Document, "docx"), "pdf");
        assert_eq!(suggested_target(Category::Document, "xlsx"), "ods");
        assert_eq!(suggested_target(Category::Document, "pptx"), "odp");
        assert_eq!(suggested_target(Category::Document, "md"), "docx");
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

    #[test]
    fn pandoc_capabilities_match_the_router() {
        assert!(pandoc_reads("docx"));
        assert!(pandoc_writes("docx"));
        assert!(!pandoc_reads("xlsx"));
        assert!(!pandoc_writes("doc"));
        assert!(!pandoc_writes("pdf"));
        assert!(pandoc_writes("pptx"));
        assert!(msoffice_native_target("pdf"));
        assert!(!msoffice_native_target("html"));
    }
}
