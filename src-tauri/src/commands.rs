use crate::capabilities;
use crate::convert::{self, ConvertRequest, ConvertResult};
use crate::engines;
use crate::formats::{self, Category, FileInfo, FormatDef};
use crate::AppState;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, State};
use tokio_util::sync::CancellationToken;

fn engine_is_available(app: &AppHandle, engine: &str) -> bool {
    match engine {
        "image" => {
            engines::available(app, engines::MAGICK) || engines::available(app, engines::VIPS)
        }
        // Un formato de Office también puede leerse con Pandoc si este lo
        // soporta; la comprobación exacta se hace por formato.
        engines::OFFICE => {
            engines::office_available(app) || engines::available(app, engines::PANDOC)
        }
        other => engines::available(app, other),
    }
}

fn format_engine_available(app: &AppHandle, format: &FormatDef) -> bool {
    match format.engine.as_str() {
        "office" => {
            engines::office_available(app)
                || (engines::available(app, engines::PANDOC)
                    && formats::pandoc_reads(&format.id))
        }
        other => engine_is_available(app, other),
    }
}

fn msoffice_available_for_input(app: &AppHandle, input_id: &str) -> bool {
    #[cfg(windows)]
    {
        let app_kind = match formats::doc_family(input_id) {
            Some(formats::DocFamily::Word) => Some(engines::OfficeApp::Word),
            Some(formats::DocFamily::Sheet) => Some(engines::OfficeApp::Excel),
            Some(formats::DocFamily::Slide) => Some(engines::OfficeApp::PowerPoint),
            _ => None,
        };
        if let Some(app_kind) = app_kind {
            return engines::msoffice_available_for(app, app_kind);
        }
    }
    let _ = (app, input_id);
    false
}

/// Comprueba si hay un motor real para una ruta entrada → destino.
fn target_is_available(app: &AppHandle, input_id: &str, target_id: &str) -> bool {
    match formats::route(input_id, target_id) {
        formats::EngineKind::Image => {
            engines::available(app, engines::MAGICK) || engines::available(app, engines::VIPS)
        }
        formats::EngineKind::Ffmpeg => engines::available(app, engines::FFMPEG),
        formats::EngineKind::Pandoc => engines::available(app, engines::PANDOC),
        formats::EngineKind::Office => {
            // Microsoft Office no produce HTML de un solo archivo.
            if target_id == "html" {
                return engines::available(app, engines::LIBREOFFICE)
                    || (engines::available(app, engines::PANDOC)
                        && formats::pandoc_reads(input_id));
            }

            if target_id == "pdf" {
                return engines::available(app, engines::LIBREOFFICE)
                    || msoffice_available_for_input(app, input_id);
            }

            if engines::available(app, engines::LIBREOFFICE) {
                return true;
            }

            if formats::msoffice_native_target(target_id)
                && msoffice_available_for_input(app, input_id)
            {
                return true;
            }

            engines::available(app, engines::PANDOC)
                && formats::pandoc_reads(input_id)
                && formats::pandoc_writes(target_id)
        }
    }
}

#[tauri::command]
pub fn detect_engines(app: AppHandle) -> Vec<engines::EngineInfo> {
    engines::detect(&app)
}

#[tauri::command]
pub fn get_catalog() -> Vec<FormatDef> {
    formats::catalog()
}

#[tauri::command]
pub fn get_targets(
    app: AppHandle,
    category: String,
    input_format: Option<String>,
) -> Vec<FormatDef> {
    let category = Category::parse(&category);
    let capabilities = capabilities::load(&app, category);

    let input_format = input_format.filter(|value| !value.trim().is_empty());
    let candidates = match input_format.as_deref() {
        Some(input) => formats::targets_for_input(input),
        None => formats::targets_for(category),
    };

    candidates
        .into_iter()
        .filter(|format| {
            let engine_ok = match input_format.as_deref() {
                Some(input) => target_is_available(&app, input, &format.id),
                None if category == Category::Video && format.id == "gif" => {
                    engines::available(&app, engines::FFMPEG)
                }
                None => format_engine_available(&app, format),
            };

            // Las capacidades de Pandoc solo limitan los destinos que Pandoc
            // va a escribir. Los de Office los resuelve LibreOffice u Office.
            let capabilities_ok = if format.engine == "office" {
                true
            } else {
                capabilities.supports(&format.id)
            };

            engine_ok && capabilities_ok
        })
        .collect()
}

#[tauri::command]
pub fn inspect_file(app: AppHandle, path: String) -> Result<FileInfo, String> {
    let path = PathBuf::from(&path);
    let metadata = std::fs::metadata(&path)
        .map_err(|error| format!("No se pudo leer el archivo: {error}"))?;

    if !metadata.is_file() {
        return Err("La ruta seleccionada no es un archivo.".to_string());
    }

    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    let format = formats::find_by_extension(&extension);

    if let Some(format) = &format {
        if !format.can_read {
            return Err(format!(
                "«{}» todavía no se puede usar como archivo de entrada.",
                format.label
            ));
        }
    }

    let (category, format_id, engine, engine_available) = match &format {
        Some(format) => (
            format.category,
            format.id.clone(),
            format.engine.clone(),
            format_engine_available(&app, format),
        ),
        None => (
            Category::Unknown,
            extension.clone(),
            String::new(),
            false,
        ),
    };

    let name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("archivo")
        .to_string();

    Ok(FileInfo {
        path: path.to_string_lossy().to_string(),
        name,
        extension,
        size: metadata.len(),
        category,
        format_id: format_id.clone(),
        engine,
        engine_available,
        suggested_target: formats::suggested_target(category, &format_id),
    })
}

#[tauri::command]
pub async fn convert_file(
    app: AppHandle,
    state: State<'_, AppState>,
    request: ConvertRequest,
) -> Result<ConvertResult, String> {
    let token = CancellationToken::new();

    {
        let mut jobs = state.jobs.lock().await;
        if jobs.contains_key(&request.job_id) {
            return Err("Ya hay una conversión con ese identificador.".to_string());
        }
        jobs.insert(request.job_id.clone(), token.clone());
    }

    let result = convert::convert(&app, token, request.clone()).await;
    state.jobs.lock().await.remove(&request.job_id);
    result
}

#[tauri::command]
pub async fn cancel_job(state: State<'_, AppState>, job_id: String) -> Result<(), String> {
    let token = state.jobs.lock().await.get(&job_id).cloned();
    if let Some(token) = token {
        token.cancel();
    }
    Ok(())
}

#[tauri::command]
pub fn default_output_dir() -> String {
    dirs::download_dir()
        .or_else(dirs::home_dir)
        .map(|path| path.to_string_lossy().to_string())
        .unwrap_or_else(|| ".".to_string())
}

#[tauri::command]
pub fn startup_files() -> Vec<String> {
    std::env::args_os()
        .skip(1)
        .filter(|argument| !argument.to_string_lossy().starts_with('-'))
        .filter(|argument| Path::new(argument).is_file())
        .map(|argument| argument.to_string_lossy().to_string())
        .collect()
}

#[tauri::command]
pub fn scan_folder(path: String) -> Result<Vec<String>, String> {
    let root = PathBuf::from(&path);
    if !root.is_dir() {
        return Err("La ruta seleccionada no es una carpeta.".to_string());
    }

    const MAX_FILES: usize = 1000;
    const MAX_DEPTH: usize = 16;

    fn visit(directory: &Path, files: &mut Vec<String>, depth: usize) {
        if depth > MAX_DEPTH || files.len() >= MAX_FILES {
            return;
        }

        let Ok(entries) = std::fs::read_dir(directory) else {
            return;
        };

        for entry in entries.flatten() {
            if files.len() >= MAX_FILES {
                return;
            }

            let path = entry.path();
            let Ok(metadata) = entry.metadata() else {
                continue;
            };

            if metadata.file_type().is_symlink() {
                continue;
            }

            if metadata.is_dir() {
                let hidden = entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with('.');
                if !hidden {
                    visit(&path, files, depth + 1);
                }
            } else if metadata.is_file() {
                files.push(path.to_string_lossy().to_string());
            }
        }
    }

    let mut files = Vec::new();
    visit(&root, &mut files, 0);

    if files.len() >= MAX_FILES {
        files.truncate(MAX_FILES);
    }

    Ok(files)
}

#[cfg(target_os = "windows")]
fn hidden_command(program: &str) -> std::process::Command {
    use std::os::windows::process::CommandExt;
    let mut command = std::process::Command::new(program);
    command.creation_flags(0x08000000);
    command
}

#[tauri::command]
pub fn open_in_file_manager(path: String) -> Result<(), String> {
    let path = PathBuf::from(&path);
    let target = if path.is_dir() {
        path.clone()
    } else {
        path.parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| path.clone())
    };

    if !target.exists() {
        return Err("La carpeta no existe.".to_string());
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&target)
            .spawn()
            .map_err(|error| format!("No se pudo abrir la carpeta: {error}"))?;
    }

    #[cfg(target_os = "windows")]
    {
        hidden_command("explorer")
            .arg(&target)
            .spawn()
            .map_err(|error| format!("No se pudo abrir la carpeta: {error}"))?;
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&target)
            .spawn()
            .map_err(|error| format!("No se pudo abrir la carpeta: {error}"))?;
    }

    Ok(())
}
