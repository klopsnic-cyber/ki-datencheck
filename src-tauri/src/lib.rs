mod detect;
mod extract;

use serde_json::{Value, json};
use std::path::Path;

#[tauri::command]
async fn analyze_text(text: String) -> detect::Report {
    detect::analyze(&text)
}

#[tauri::command]
async fn analyze_file(path: String) -> Result<Value, String> {
    let p = Path::new(&path);
    let text = extract::text_of(p)?;
    let name = p.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    let mut v = serde_json::to_value(detect::analyze(&text)).map_err(|e| e.to_string())?;
    v["name"] = json!(name);
    v["path"] = json!(path);
    Ok(v)
}

#[tauri::command]
fn save_text(path: String, text: String) -> Result<(), String> {
    std::fs::write(path, text).map_err(|e| e.to_string())
}

/// Nur für Tests: `KD_DEMO=1` lädt beim Start den Beispieltext (nicht in Release-Builds).
#[tauri::command]
fn dev_demo() -> bool {
    cfg!(debug_assertions) && std::env::var("KD_DEMO").is_ok()
}

#[tauri::command]
fn frontend_log(msg: String) {
    eprintln!("[frontend] {msg}");
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .invoke_handler(tauri::generate_handler![analyze_text, analyze_file, save_text, frontend_log, dev_demo])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
