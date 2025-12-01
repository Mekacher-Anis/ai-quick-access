use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager, PhysicalPosition, WebviewUrl, WebviewWindowBuilder,
};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut};
use mouse_position::mouse_position::Mouse;

use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub api_key: String,
    pub selected_model: String,
    pub dark_mode: bool,
    pub auto_start: bool,
    #[serde(default)]
    pub system_prompt: String,
    #[serde(default)]
    pub model_shortcuts: HashMap<String, String>,
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
        let mut default_shortcuts = HashMap::new();
        default_shortcuts.insert("h".to_string(), "google/gemini-3-pro-preview".to_string());
        default_shortcuts.insert("f".to_string(), "google/gemini-2.5-flash-preview-09-2025".to_string());
        default_shortcuts.insert("o".to_string(), "openai/gpt-oss-120b".to_string());
        return Ok(Settings {
            api_key: String::new(),
            selected_model: "openai/gpt-oss-120b".to_string(),
            dark_mode: true,
            auto_start: false,
            system_prompt: "Keep your responses as concise, precise, to the point.\nAnswer the question in as few words as possible.\nNo Yapping.".to_string(),
            model_shortcuts: default_shortcuts,
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

#[tauri::command]
async fn quit_app(app: tauri::AppHandle) {
    app.exit(0);
}

fn get_mouse_position() -> Option<(i32, i32)> {
    match Mouse::get_mouse_position() {
        Mouse::Position { x, y } => Some((x, y)),
        Mouse::Error => None,
    }
}

fn center_window_on_monitor_with_mouse(app: &tauri::AppHandle, window: &tauri::WebviewWindow) {
    // Get mouse position
    let mouse_pos = match get_mouse_position() {
        Some(pos) => pos,
        None => return, // Fall back to default behavior if we can't get mouse position
    };

    // Find the monitor that contains the mouse cursor
    let monitors = match app.available_monitors() {
        Ok(m) => m,
        Err(_) => return,
    };

    for monitor in monitors {
        let pos = monitor.position();
        let size = monitor.size();
        
        let monitor_x = pos.x;
        let monitor_y = pos.y;
        let monitor_width = size.width as i32;
        let monitor_height = size.height as i32;

        // Check if mouse is within this monitor
        if mouse_pos.0 >= monitor_x
            && mouse_pos.0 < monitor_x + monitor_width
            && mouse_pos.1 >= monitor_y
            && mouse_pos.1 < monitor_y + monitor_height
        {
            // Get window size
            if let Ok(window_size) = window.outer_size() {
                let window_width = window_size.width as i32;
                let window_height = window_size.height as i32;

                // Center the window on this monitor
                let x = monitor_x + (monitor_width - window_width) / 2;
                let y = monitor_y + (monitor_height - window_height) / 2;

                let _ = window.set_position(PhysicalPosition::new(x, y));
            }
            break;
        }
    }
}

fn create_or_focus_main_window(app: &tauri::AppHandle, new_chat: bool) {
    if let Some(window) = app.get_webview_window("main") {
        // Move window to the monitor where the mouse is
        center_window_on_monitor_with_mouse(app, &window);
        // Show and bring to front
        let _ = window.show();
        let _ = window.set_focus();
        // Set always on top temporarily to ensure it's above all other windows
        let _ = window.set_always_on_top(true);
        // Then disable always on top so it behaves normally after
        let _ = window.set_always_on_top(false);
        if new_chat {
            let _ = app.emit("new-chat", ());
        }
    } else {
        // Create a new main window if it doesn't exist
        #[allow(unused_variables)]
        if let Ok(window) = WebviewWindowBuilder::new(
            app,
            "main",
            WebviewUrl::App("/".into()),
        )
        .title("ai-quick-access")
        .inner_size(500.0, 150.0)
        .decorations(false)
        .build()
        {
            // Position window on the monitor where the mouse is
            center_window_on_monitor_with_mouse(app, &window);
            
            // Bring to front
            let _ = window.set_always_on_top(true);
            let _ = window.set_always_on_top(false);

            #[cfg(target_os = "macos")]
            {
                let _ = window_vibrancy::apply_vibrancy(
                    &window,
                    window_vibrancy::NSVisualEffectMaterial::UnderWindowBackground,
                    None,
                    Some(10.0),
                );
            }

            #[cfg(target_os = "windows")]
            {
                let _ = window_vibrancy::apply_blur(&window, Some((18, 18, 18, 125)));
            }

            if new_chat {
                let _ = app.emit("new-chat", ());
            }
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, shortcut, _event| {
                    #[cfg(target_os = "macos")]
                    let mod_key = Modifiers::SUPER;
                    #[cfg(not(target_os = "macos"))]
                    let mod_key = Modifiers::CONTROL;

                    let focus_shortcut = Shortcut::new(Some(mod_key), Code::KeyE);
                    let new_chat_shortcut = Shortcut::new(Some(mod_key | Modifiers::SHIFT), Code::KeyE);

                    if shortcut == &focus_shortcut {
                        // Focus window (or create if not exists)
                        create_or_focus_main_window(app, false);
                    } else if shortcut == &new_chat_shortcut {
                        // Focus and start new chat
                        create_or_focus_main_window(app, true);
                    }
                })
                .build(),
        )
        .invoke_handler(tauri::generate_handler![greet, open_settings, load_settings, save_settings, quit_app])
        .setup(|app| {
            // Register global shortcuts based on OS
            #[cfg(target_os = "macos")]
            let mod_key = Modifiers::SUPER;
            #[cfg(not(target_os = "macos"))]
            let mod_key = Modifiers::CONTROL;

            let focus_shortcut = Shortcut::new(Some(mod_key), Code::KeyE);
            let new_chat_shortcut = Shortcut::new(Some(mod_key | Modifiers::SHIFT), Code::KeyE);

            // Try to register shortcuts, log errors but don't fail
            if let Err(e) = app.global_shortcut().register(focus_shortcut) {
                eprintln!("Failed to register focus shortcut (Ctrl/Cmd+E): {}", e);
            }
            if let Err(e) = app.global_shortcut().register(new_chat_shortcut) {
                eprintln!("Failed to register new chat shortcut (Ctrl/Cmd+Shift+E): {}", e);
            }

            // Create system tray
            let show_item = MenuItem::with_id(app, "show", "Show", true, None::<&str>)?;
            let new_chat_item = MenuItem::with_id(app, "new_chat", "New Chat", true, None::<&str>)?;
            let settings_item = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

            let menu = Menu::with_items(app, &[&show_item, &new_chat_item, &settings_item, &quit_item])?;

            let icon = app.default_window_icon().cloned().expect("no icon found");

            let _tray = TrayIconBuilder::new()
                .icon(icon)
                .menu(&menu)
                .tooltip("AI Quick Access")
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        create_or_focus_main_window(app, false);
                    }
                    "new_chat" => {
                        create_or_focus_main_window(app, true);
                    }
                    "settings" => {
                        if let Some(window) = app.get_webview_window("settings") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        create_or_focus_main_window(app, false);
                    }
                })
                .build(app)?;

            // Apply vibrancy to main window
            #[allow(unused_variables)]
            if let Some(main_window) = app.get_webview_window("main") {
                #[cfg(target_os = "macos")]
                window_vibrancy::apply_vibrancy(
                    &main_window,
                    window_vibrancy::NSVisualEffectMaterial::UnderWindowBackground,
                    None,
                    Some(10.0),
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
