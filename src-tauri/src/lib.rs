mod compress;
mod error;
mod ghostscript;

use std::path::PathBuf;
use tauri::AppHandle;

use compress::{CompressionReport, Preset};
use error::UserError;

#[tauri::command]
fn app_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[tauri::command]
async fn compress_pdf(app: AppHandle, path: String, preset: Preset) -> Result<CompressionReport, UserError> {
    let input = PathBuf::from(path);
    compress::run(&app, &input, preset).await.map_err(|e| e.to_user())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![compress_pdf, app_version])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
