// DevpWareSoft v2.0 - Dependency Analyzer
// Backend Rust with Tauri 2.x

pub mod commands;
pub mod core;
pub mod models;
pub mod parsers;

use commands::*;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            detect_project_type,
            list_parsers,
            scan_directory,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// Temporary command for testing
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! DevpWareSoft v2.0 is ready!", name)
}
