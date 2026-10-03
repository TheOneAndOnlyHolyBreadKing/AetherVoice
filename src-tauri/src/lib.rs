pub mod audio;
pub mod downloader;
pub mod injector;
pub mod refinement;
pub mod whisper;

use audio::AudioRecorder;
use downloader::{download_model, get_models_dir, is_model_installed, DEFAULT_MODEL_FILENAME, DEFAULT_MODEL_URL};
use injector::inject_text;
use refinement::{refine_text, refine_text_with_config, AppConfig};
use std::sync::{Arc, Mutex, RwLock};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager, State};
use whisper::WhisperEngine;

pub struct AppState {
    pub recorder: Arc<Mutex<Option<AudioRecorder>>>,
    pub whisper: Arc<tokio::sync::Mutex<Option<WhisperEngine>>>,
    pub config: Arc<RwLock<AppConfig>>,
}

fn get_config_path(app: &AppHandle) -> Result<std::path::PathBuf, String> {
    let app_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app_data_dir: {}", e))?;
    if !app_dir.exists() {
        let _ = std::fs::create_dir_all(&app_dir);
    }
    Ok(app_dir.join("aethervoice_config.json"))
}

#[tauri::command]
fn get_user_config(state: State<'_, AppState>, app: AppHandle) -> Result<AppConfig, String> {
    let path = get_config_path(&app)?;
    if path.exists() {
        if let Ok(data) = std::fs::read_to_string(&path) {
            if let Ok(loaded) = serde_json::from_str::<AppConfig>(&data) {
                let mut lock = state.config.write().map_err(|e| e.to_string())?;
                *lock = loaded.clone();
                return Ok(loaded);
            }
        }
    }
    let lock = state.config.read().map_err(|e| e.to_string())?;
    Ok(lock.clone())
}

#[tauri::command]
fn save_user_config(
    state: State<'_, AppState>,
    app: AppHandle,
    config: AppConfig,
) -> Result<(), String> {
    let path = get_config_path(&app)?;
    let serialized = serde_json::to_string_pretty(&config).map_err(|e| e.to_string())?;
    std::fs::write(&path, serialized).map_err(|e| e.to_string())?;
    let mut lock = state.config.write().map_err(|e| e.to_string())?;
    *lock = config;
    Ok(())
}

#[tauri::command]
fn open_settings_window(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("settings") {
        window.show().map_err(|e| e.to_string())?;
        window.unminimize().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
        Ok(())
    } else {
        tauri::WebviewWindowBuilder::new(
            &app,
            "settings",
            tauri::WebviewUrl::App("/settings.html".into()),
        )
        .title("AetherVoice Settings")
        .inner_size(960.0, 650.0)
        .min_inner_size(800.0, 550.0)
        .center()
        .build()
        .map_err(|e| e.to_string())?;
        Ok(())
    }
}

#[tauri::command]
fn start_dragging(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window.start_dragging().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn check_model_status(app: AppHandle) -> Result<bool, String> {
    let models_dir = get_models_dir(&app)?;
    Ok(is_model_installed(&models_dir))
}

#[tauri::command]
async fn ensure_model(app: AppHandle) -> Result<String, String> {
    let models_dir = get_models_dir(&app)?;
    if is_model_installed(&models_dir) {
        return Ok(models_dir
            .join(DEFAULT_MODEL_FILENAME)
            .to_string_lossy()
            .to_string());
    }

    let _ = app.emit("dictation-state", "transcribing");
    let downloaded_path = download_model(app.clone(), &models_dir, DEFAULT_MODEL_URL).await?;
    let _ = app.emit("dictation-state", "idle");
    Ok(downloaded_path.to_string_lossy().to_string())
}

#[tauri::command]
fn start_dictation(state: State<'_, AppState>, app: AppHandle) -> Result<(), String> {
    let recorder_lock = state.recorder.lock().map_err(|e| e.to_string())?;
    if let Some(recorder) = recorder_lock.as_ref() {
        recorder.start_recording();
        let _ = app.emit("dictation-state", "listening");
    }
    Ok(())
}

#[tauri::command]
async fn stop_dictation(state: State<'_, AppState>, app: AppHandle) -> Result<String, String> {
    let samples = {
        let recorder_lock = state.recorder.lock().map_err(|e| e.to_string())?;
        if let Some(recorder) = recorder_lock.as_ref() {
            recorder.stop_recording()
        } else {
            return Err("Audio recorder not initialized".to_string());
        }
    };

    let _ = app.emit("dictation-state", "transcribing");

    let whisper_arc = Arc::clone(&state.whisper);
    let raw_text = {
        let whisper_guard = whisper_arc.lock().await;
        if let Some(whisper) = whisper_guard.as_ref() {
            whisper.transcribe(samples).await?
        } else {
            String::new()
        }
    };

    let config = {
        let lock = state.config.read().map_err(|e| e.to_string())?;
        lock.clone()
    };

    let cleaned_text = refine_text_with_config(&raw_text, &config);

    if !cleaned_text.is_empty() {
        let _ = inject_text(&cleaned_text);
    }

    let _ = app.emit("dictation-state", "idle");
    Ok(cleaned_text)
}

#[tauri::command]
fn test_inject(text: &str) -> Result<String, String> {
    let refined = refine_text(text);
    inject_text(&refined)?;
    Ok(refined)
}

#[tauri::command]
fn toggle_capsule_visibility(app: AppHandle) -> Result<bool, String> {
    if let Some(window) = app.get_webview_window("main") {
        let is_visible = window.is_visible().map_err(|e| e.to_string())?;
        if is_visible {
            window.hide().map_err(|e| e.to_string())?;
            Ok(false)
        } else {
            window.show().map_err(|e| e.to_string())?;
            window.set_focus().map_err(|e| e.to_string())?;
            Ok(true)
        }
    } else {
        Err("Main window not found".to_string())
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let recorder_instance = AudioRecorder::new().ok();

    let state = AppState {
        recorder: Arc::new(Mutex::new(recorder_instance)),
        whisper: Arc::new(tokio::sync::Mutex::new(None)),
        config: Arc::new(RwLock::new(AppConfig::default())),
    };

    tauri::Builder::default()
        .manage(state)
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            check_model_status,
            ensure_model,
            start_dictation,
            stop_dictation,
            test_inject,
            toggle_capsule_visibility,
            open_settings_window,
            get_user_config,
            save_user_config,
            start_dragging
        ])
        .setup(|app| {
            let settings_i = MenuItem::with_id(app, "settings", "Settings...", true, None::<&str>)?;
            let toggle_i = MenuItem::with_id(app, "toggle", "Toggle Capsule", true, None::<&str>)?;
            let quit_i = MenuItem::with_id(app, "quit", "Quit AetherVoice", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&settings_i, &toggle_i, &quit_i])?;

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("AetherVoice - Local Dictation")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "settings" => {
                        let _ = open_settings_window(app.clone());
                    }
                    "toggle" => {
                        let _ = toggle_capsule_visibility(app.clone());
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click { button, .. } = event {
                        if button == tauri::tray::MouseButton::Left {
                            let app = tray.app_handle();
                            let _ = toggle_capsule_visibility(app.clone());
                        }
                    }
                })
                .build(app)?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

