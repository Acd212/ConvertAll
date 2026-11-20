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
        other => engines::available(app, other),
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
pub fn get_targets(app: AppHandle, category: String) -> Vec<FormatDef> {
    let category = Category::parse(&category);
    let capabilities = capabilities::load(&app, category);

    formats::targets_for(category)
        .into_iter()
        .filter(|format| {
            let engine_ok = if category == Category::Video && format.id == "gif" {
                engines::available(&app, engines::FFMPEG)
            } else {
                engine_is_available(&app, &format.engine)
            };

            engine_ok && capabilities.supports(&format.id)
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
    let (category, format_id, engine) = match &format {
        Some(format) => (
            format.category,
            format.id.clone(),
            format.engine.clone(),
        ),
        None => (Category::Unknown, extension.clone(), String::new()),
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
        engine: engine.clone(),
        engine_available: !engine.is_empty() && engine_is_available(&app, &engine),
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
