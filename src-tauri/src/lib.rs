mod capabilities;
mod commands;
mod convert;
mod engines;
mod formats;

use std::collections::HashMap;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

#[derive(Default)]
pub struct AppState {
    pub jobs: Mutex<HashMap<String, CancellationToken>>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            commands::detect_engines,
            commands::get_catalog,
            commands::get_targets,
            commands::inspect_file,
            commands::convert_file,
            commands::cancel_job,
            commands::default_output_dir,
            commands::scan_folder,
            commands::open_in_file_manager,
        ])
        .run(tauri::generate_context!())
        .expect("error mientras se iniciaba ConvertAll");
}
