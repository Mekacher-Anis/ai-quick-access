use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::Manager;

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub api_key: String,
    pub selected_model: String,
    pub dark_mode: bool,
    pub auto_start: bool,
}

fn get_config_path() -> Result<PathBuf, String> {
    let config_dir = dirs::config_dir().ok_or("Could not find config directory")?;
    let app_config_dir = config_dir.join("ai-quick-access");
    Ok(app_config_dir.join("configs.json"))
}

fn ensure_config_dir() -> Result<(), String> {
    let config_path = get_config_path()?;
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create config directory: {}", e))?;
    }
    Ok(())
}

#[tauri::command]
fn load_settings() -> Result<Settings, String> {
    let config_path = get_config_path()?;
    
    if !config_path.exists() {
        // Return default settings if file doesn't exist
        return Ok(Settings {
            api_key: String::new(),
            selected_model: "openai/gpt-oss-120b".to_string(),
            dark_mode: true,
            auto_start: false,
        });
    }
    
    let contents = fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read config file: {}", e))?;
    
    serde_json::from_str(&contents)
        .map_err(|e| format!("Failed to parse config file: {}", e))
}

#[tauri::command]
fn save_settings(settings: Settings) -> Result<(), String> {
    ensure_config_dir()?;
    let config_path = get_config_path()?;
    
    let contents = serde_json::to_string_pretty(&settings)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;
    
    fs::write(&config_path, contents)
        .map_err(|e| format!("Failed to write config file: {}", e))?;
    
    Ok(())
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn open_settings(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("settings") {
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, open_settings, load_settings, save_settings])
        .setup(|app| {
            // Apply vibrancy to main window
            if let Some(main_window) = app.get_webview_window("main") {
                #[cfg(target_os = "macos")]
                window_vibrancy::apply_vibrancy(
                    &main_window,
                    window_vibrancy::NSVisualEffectMaterial::HudWindow,
                    None,
                    None,
                )
                .expect("Failed to apply vibrancy to main window");

                #[cfg(target_os = "windows")]
                window_vibrancy::apply_blur(&main_window, Some((18, 18, 18, 125)))
                    .expect("Failed to apply blur to main window");
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
