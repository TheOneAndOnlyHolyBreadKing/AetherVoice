pub mod audio;
pub mod downloader;
pub mod injector;
pub mod refinement;
pub mod whisper;

use audio::AudioRecorder;
use downloader::{download_model, get_models_dir, is_model_installed, DEFAULT_MODEL_FILENAME, DEFAULT_MODEL_URL};
use injector::inject_text;
use refinement::{refine_text, refine_with_llm, AppConfig, HistoryItem};
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

fn get_persistent_backup_dir() -> Result<std::path::PathBuf, String> {
    if let Ok(appdata) = std::env::var("APPDATA") {
        let dir = std::path::PathBuf::from(appdata).join("AetherVoice");
        if !dir.exists() {
            let _ = std::fs::create_dir_all(&dir);
        }
        return Ok(dir);
    }
    if let Ok(userprofile) = std::env::var("USERPROFILE") {
        let dir = std::path::PathBuf::from(userprofile).join(".aethervoice");
        if !dir.exists() {
            let _ = std::fs::create_dir_all(&dir);
        }
        return Ok(dir);
    }
    Err("Failed to resolve persistent AppData/UserProfile directory".to_string())
}

fn mirror_to_persistent_backup(filename: &str, content: &str) {
    if let Ok(backup_dir) = get_persistent_backup_dir() {
        let backup_file = backup_dir.join(filename);
        let _ = std::fs::write(backup_file, content);
    }
}

pub fn init_persistent_storage(app: &AppHandle) {
    if let Ok(primary_dir) = app.path().app_data_dir() {
        let _ = std::fs::create_dir_all(&primary_dir);
        if let Ok(backup_dir) = get_persistent_backup_dir() {
            let files = [
                "aethervoice_config.json",
                "aethervoice_history.json",
                "aethervoice_stats.json",
                "aethervoice_companion.json",
            ];
            for file_name in &files {
                let primary_file = primary_dir.join(file_name);
                let backup_file = backup_dir.join(file_name);

                if !primary_file.exists() && backup_file.exists() {
                    println!(
                        "[AetherVoice] Restoring {} from persistent storage: {:?}",
                        file_name, backup_file
                    );
                    let _ = std::fs::copy(&backup_file, &primary_file);
                } else if primary_file.exists() && !backup_file.exists() {
                    let _ = std::fs::copy(&primary_file, &backup_file);
                }
            }
        }
    }
}

fn get_config_path(app: &AppHandle) -> Result<std::path::PathBuf, String> {
    let app_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app_data_dir: {}", e))?;
    if !app_dir.exists() {
        let _ = std::fs::create_dir_all(&app_dir);
    }
    let primary = app_dir.join("aethervoice_config.json");
    if !primary.exists() {
        if let Ok(backup_dir) = get_persistent_backup_dir() {
            let backup = backup_dir.join("aethervoice_config.json");
            if backup.exists() {
                let _ = std::fs::copy(&backup, &primary);
            }
        }
    }
    Ok(primary)
}

fn save_user_config_internal(
    app: &AppHandle,
    state: &AppState,
    config: AppConfig,
) -> Result<(), String> {
    let path = get_config_path(app)?;
    let serialized = serde_json::to_string_pretty(&config).map_err(|e| e.to_string())?;
    std::fs::write(&path, &serialized).map_err(|e| e.to_string())?;
    mirror_to_persistent_backup("aethervoice_config.json", &serialized);

    let device_changed = {
        let lock = state.config.read().map_err(|e| e.to_string())?;
        lock.audio_device != config.audio_device
    };

    if device_changed {
        if let Ok(mut rec_lock) = state.recorder.lock() {
            *rec_lock = None;
        }
    }

    let mut lock = state.config.write().map_err(|e| e.to_string())?;
    *lock = config;
    Ok(())
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
    save_user_config_internal(&app, &state, config)
}

fn get_history_path(app: &AppHandle) -> Result<std::path::PathBuf, String> {
    let app_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app_data_dir: {}", e))?;
    if !app_dir.exists() {
        let _ = std::fs::create_dir_all(&app_dir);
    }
    let primary = app_dir.join("aethervoice_history.json");
    if !primary.exists() {
        if let Ok(backup_dir) = get_persistent_backup_dir() {
            let backup = backup_dir.join("aethervoice_history.json");
            if backup.exists() {
                let _ = std::fs::copy(&backup, &primary);
            }
        }
    }
    Ok(primary)
}

#[tauri::command]
fn get_dictation_history(app: AppHandle) -> Result<Vec<HistoryItem>, String> {
    let path = get_history_path(&app)?;
    if path.exists() {
        if let Ok(data) = std::fs::read_to_string(&path) {
            if let Ok(items) = serde_json::from_str::<Vec<HistoryItem>>(&data) {
                return Ok(items);
            }
        }
    }
    Ok(Vec::new())
}

#[tauri::command]
fn save_dictation_history(app: AppHandle, mut history: Vec<HistoryItem>) -> Result<(), String> {
    // Automatically enforce that only the 5 most recent entries are ever kept
    if history.len() > 5 {
        history.truncate(5);
    }
    let path = get_history_path(&app)?;
    let serialized = serde_json::to_string_pretty(&history).map_err(|e| e.to_string())?;
    std::fs::write(&path, &serialized).map_err(|e| e.to_string())?;
    mirror_to_persistent_backup("aethervoice_history.json", &serialized);
    let _ = app.emit("dictation-history-updated", ());
    Ok(())
}

fn get_stats_path(app: &AppHandle) -> Result<std::path::PathBuf, String> {
    let app_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app_data_dir: {}", e))?;
    if !app_dir.exists() {
        let _ = std::fs::create_dir_all(&app_dir);
    }
    let primary = app_dir.join("aethervoice_stats.json");
    if !primary.exists() {
        if let Ok(backup_dir) = get_persistent_backup_dir() {
            let backup = backup_dir.join("aethervoice_stats.json");
            if backup.exists() {
                let _ = std::fs::copy(&backup, &primary);
            }
        }
    }
    Ok(primary)
}

#[tauri::command]
fn get_user_stats(app: AppHandle) -> Result<serde_json::Value, String> {
    let path = get_stats_path(&app)?;
    if path.exists() {
        if let Ok(data) = std::fs::read_to_string(&path) {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&data) {
                return Ok(val);
            }
        }
    }
    Ok(serde_json::json!({
        "words": 0,
        "secondsSaved": 0,
        "sessions": 0,
        "totalDuration": 0,
        "appUsage": {}
    }))
}

#[tauri::command]
fn save_user_stats(app: AppHandle, stats: serde_json::Value) -> Result<(), String> {
    let path = get_stats_path(&app)?;
    let serialized = serde_json::to_string_pretty(&stats).map_err(|e| e.to_string())?;
    std::fs::write(&path, &serialized).map_err(|e| e.to_string())?;
    mirror_to_persistent_backup("aethervoice_stats.json", &serialized);
    Ok(())
}

fn get_companion_path(app: &AppHandle) -> Result<std::path::PathBuf, String> {
    let app_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app_data_dir: {}", e))?;
    if !app_dir.exists() {
        let _ = std::fs::create_dir_all(&app_dir);
    }
    let primary = app_dir.join("aethervoice_companion.json");
    if !primary.exists() {
        if let Ok(backup_dir) = get_persistent_backup_dir() {
            let backup = backup_dir.join("aethervoice_companion.json");
            if backup.exists() {
                let _ = std::fs::copy(&backup, &primary);
            }
        }
    }
    Ok(primary)
}

#[tauri::command]
fn get_companion_state(app: AppHandle) -> Result<serde_json::Value, String> {
    let path = get_companion_path(&app)?;
    if path.exists() {
        if let Ok(data) = std::fs::read_to_string(&path) {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&data) {
                return Ok(val);
            }
        }
    }
    Ok(serde_json::json!({
        "enabled": true,
        "scale": 1.0,
        "words_today": 0,
        "words_total": 0,
        "daily_streak": 1,
        "last_dictation_date": "2026-10-17",
        "equipped_cosmetics": ["default_halo"],
        "unlocked_cosmetics": ["default_halo"]
    }))
}

#[tauri::command]
fn save_companion_state(app: AppHandle, state: serde_json::Value) -> Result<(), String> {
    let path = get_companion_path(&app)?;
    let serialized = serde_json::to_string_pretty(&state).map_err(|e| e.to_string())?;
    std::fs::write(&path, &serialized).map_err(|e| e.to_string())?;
    mirror_to_persistent_backup("aethervoice_companion.json", &serialized);
    let _ = app.emit("companion-settings-updated", state);
    Ok(())
}

#[tauri::command]
fn toggle_companion_window(app: AppHandle, show: bool) -> Result<(), String> {
    if let Some(win) = app.get_webview_window("companion") {
        if show {
            let _ = win.show();
        } else {
            let _ = win.hide();
        }
    }
    Ok(())
}

#[tauri::command]
fn open_settings(app: AppHandle, tab: Option<String>) -> Result<(), String> {
    if let Some(win) = app.get_webview_window("settings") {
        let _ = win.show();
        let _ = win.set_focus();
        if let Some(t) = tab {
            let _ = win.emit("open-settings-tab", serde_json::json!({ "tab": t }));
        }
    }
    Ok(())
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ModelVerificationSummary {
    pub confirmed_models: Vec<String>,
    pub verified_models: Vec<String>,
    pub active_model: String,
    pub ollama_available: bool,
}

pub const CATALOG_MODEL_IDS: &[&str] = &[
    "gemma2:2b",
    "llama3.2:1b",
    "llama3.2:3b",
    "qwen2.5:0.5b",
    "qwen2.5:1.5b",
    "qwen2.5:3b",
    "deepseek-r1:1.5b",
    "mistral:7b",
];

pub async fn verify_and_sync_models_internal(
    app: &AppHandle,
    state: &AppState,
) -> Result<ModelVerificationSummary, String> {
    let mut config = {
        let lock = state.config.read().map_err(|e| e.to_string())?;
        lock.clone()
    };

    let client = reqwest::Client::builder()
        .no_proxy()
        .timeout(std::time::Duration::from_millis(3000))
        .build()
        .map_err(|e| e.to_string())?;

    let ollama_res = client.get("http://127.0.0.1:11434/api/tags").send().await;

    let mut verified_models = Vec::new();
    let mut changed = false;

    let is_model_match = |tag: &str, cat_id: &str| -> bool {
        let t = tag.to_lowercase();
        let c = cat_id.to_lowercase();
        t == c
            || t == format!("{}:latest", c)
            || t.trim_end_matches(":latest") == c
            || t.starts_with(&format!("{}:", c))
            || t.ends_with(&format!("/{}", c))
            || t.ends_with(&format!("/{}:latest", c))
    };

    let ollama_available = match ollama_res {
        Ok(resp) if resp.status().is_success() => {
            if let Ok(val) = resp.json::<serde_json::Value>().await {
                let mut installed_tags: Vec<String> = Vec::new();
                if let Some(models) = val["models"].as_array() {
                    for m in models {
                        if let Some(name) = m["name"].as_str() {
                            installed_tags.push(name.to_lowercase());
                        }
                    }
                }
                println!("[AetherVoice] Ollama tags reported: {:?}", installed_tags);

                // Auto-detect and verify all models on disk matching our official catalog
                for &cat_id in CATALOG_MODEL_IDS {
                    let is_present = installed_tags.iter().any(|tag| is_model_match(tag, cat_id));
                    if is_present {
                        if !verified_models.contains(&cat_id.to_string()) {
                            verified_models.push(cat_id.to_string());
                        }
                        if !config.confirmed_models.contains(&cat_id.to_string()) {
                            println!("[AetherVoice] Discovered matching catalog model '{}' in Ollama. Recording as confirmed.", cat_id);
                            config.confirmed_models.push(cat_id.to_string());
                            changed = true;
                        }
                    } else if config.confirmed_models.contains(&cat_id.to_string()) {
                        // User uninstalled model outside AetherVoice: un-confirm it
                        println!("[AetherVoice] Model '{}' no longer detected in Ollama. Un-confirming.", cat_id);
                        config.confirmed_models.retain(|m| m != cat_id);
                        changed = true;
                    }
                }
                true
            } else {
                false
            }
        }
        _ => false,
    };

    // If active model is "none" or empty, and we have verified catalog models:
    if (config.llm_model == "none" || config.llm_model.is_empty()) && !verified_models.is_empty() {
        if verified_models.contains(&"gemma2:2b".to_string()) {
            config.llm_model = "gemma2:2b".to_string();
        } else {
            config.llm_model = verified_models[0].clone();
        }
        changed = true;
    } else if config.llm_model != "none" && !config.llm_model.is_empty() {
        // If active model is set but not verified on disk:
        if ollama_available && !verified_models.contains(&config.llm_model) {
            println!(
                "[AetherVoice] Active model '{}' is not present in local Ollama. Falling back.",
                config.llm_model
            );
            if let Some(first) = verified_models.first() {
                config.llm_model = first.clone();
            } else {
                config.llm_model = "none".to_string();
            }
            changed = true;
        }
    }

    if changed {
        let _ = save_user_config_internal(app, state, config.clone());
    }

    Ok(ModelVerificationSummary {
        confirmed_models: config.confirmed_models.clone(),
        verified_models,
        active_model: config.llm_model.clone(),
        ollama_available,
    })
}

#[tauri::command]
async fn verify_and_sync_models(
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<ModelVerificationSummary, String> {
    verify_and_sync_models_internal(&app, &state).await
}

#[tauri::command]
async fn confirm_model_installed(
    state: State<'_, AppState>,
    app: AppHandle,
    model: String,
) -> Result<ModelVerificationSummary, String> {
    let mut config = {
        let lock = state.config.read().map_err(|e| e.to_string())?;
        lock.clone()
    };

    if !config.confirmed_models.contains(&model) {
        config.confirmed_models.push(model.clone());
    }
    if config.llm_model == "none" || config.llm_model.is_empty() {
        config.llm_model = model;
    }
    save_user_config_internal(&app, &state, config)?;
    verify_and_sync_models_internal(&app, &state).await
}

#[tauri::command]
async fn unconfirm_model(
    state: State<'_, AppState>,
    app: AppHandle,
    model: String,
) -> Result<ModelVerificationSummary, String> {
    let mut config = {
        let lock = state.config.read().map_err(|e| e.to_string())?;
        lock.clone()
    };

    config.confirmed_models.retain(|m| m != &model);
    if config.llm_model == model {
        config.llm_model = config
            .confirmed_models
            .first()
            .cloned()
            .unwrap_or_else(|| "none".to_string());
    }
    save_user_config_internal(&app, &state, config)?;
    verify_and_sync_models_internal(&app, &state).await
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
async fn get_installed_llm_models() -> Result<Vec<String>, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_millis(3000))
        .build()
        .map_err(|e| e.to_string())?;

    let resp = client
        .get("http://127.0.0.1:11434/api/tags")
        .send()
        .await
        .map_err(|e| format!("Failed to connect to local Ollama: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("Ollama returned HTTP {}", resp.status()));
    }

    let val = resp.json::<serde_json::Value>().await.map_err(|e| e.to_string())?;
    let mut names = Vec::new();
    if let Some(models) = val["models"].as_array() {
        for m in models {
            if let Some(name) = m["name"].as_str() {
                names.push(name.to_string());
            }
        }
    }
    Ok(names)
}

#[tauri::command]
async fn pull_llm_model(app: AppHandle, model: String) -> Result<(), String> {
    use futures_util::StreamExt;
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(1800))
        .build()
        .map_err(|e| e.to_string())?;

    let body = serde_json::json!({
        "name": model,
        "stream": true
    });

    let resp = client
        .post("http://127.0.0.1:11434/api/pull")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Failed to initiate model pull: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("Ollama pull error: HTTP {}", resp.status()));
    }

    let mut stream = resp.bytes_stream();
    while let Some(chunk_res) = stream.next().await {
        if let Ok(bytes) = chunk_res {
            if let Ok(text) = std::str::from_utf8(&bytes) {
                for line in text.lines() {
                    if let Ok(json_obj) = serde_json::from_str::<serde_json::Value>(line) {
                        let _ = app.emit("model-pull-progress", &json_obj);
                    }
                }
            }
        }
    }
    let _ = app.emit("model-pull-completed", serde_json::json!({ "model": model }));
    Ok(())
}

#[tauri::command]
async fn delete_llm_model(model: String) -> Result<(), String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| e.to_string())?;

    let body = serde_json::json!({
        "name": model
    });

    let resp = client
        .delete("http://127.0.0.1:11434/api/delete")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Failed to delete model: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("Failed to delete model: HTTP {}", resp.status()));
    }
    Ok(())
}

#[tauri::command]
fn get_audio_devices() -> Vec<String> {
    AudioRecorder::list_input_devices()
}

#[cfg(windows)]
static ATOMIC_SYSTEM_MUTED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

#[cfg(windows)]
fn toggle_system_mute() {
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
        SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, VK_VOLUME_MUTE,
    };
    unsafe {
        let mut inputs = [
            INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: VK_VOLUME_MUTE,
                        wScan: 0,
                        dwFlags: 0,
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            },
            INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: VK_VOLUME_MUTE,
                        wScan: 0,
                        dwFlags: KEYEVENTF_KEYUP,
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            },
        ];
        SendInput(2, inputs.as_mut_ptr(), std::mem::size_of::<INPUT>() as i32);
    }
}

#[cfg(not(windows))]
fn toggle_system_mute() {}

#[tauri::command]
fn start_dictation(state: State<'_, AppState>, app: AppHandle) -> Result<(), String> {
    // Only allow recording if the capsule interface is visible on screen
    if let Some(main_win) = app.get_webview_window("main") {
        let is_visible = main_win.is_visible().unwrap_or(false);
        if !is_visible {
            return Err("Capsule interface is not visible; recording disabled".to_string());
        }
    }

    let noise_deafening = {
        let conf = state.config.read().map_err(|e| e.to_string())?;
        conf.noise_deafening
    };

    #[cfg(windows)]
    if noise_deafening && !ATOMIC_SYSTEM_MUTED.load(std::sync::atomic::Ordering::Relaxed) {
        toggle_system_mute();
        ATOMIC_SYSTEM_MUTED.store(true, std::sync::atomic::Ordering::Relaxed);
    }

    let mut recorder_lock = state.recorder.lock().map_err(|e| e.to_string())?;
    if recorder_lock.is_none() {
        let device_name = {
            let conf = state.config.read().map_err(|e| e.to_string())?;
            conf.audio_device.clone()
        };
        match AudioRecorder::with_device(Some(&device_name)) {
            Ok(rec) => *recorder_lock = Some(rec),
            Err(e) => eprintln!("[AetherVoice] AudioRecorder init warning: {}", e),
        }
    }
    if let Some(recorder) = recorder_lock.as_ref() {
        recorder.start_recording();
        let _ = app.emit("dictation-state", "listening");
    }
    Ok(())
}

#[tauri::command]
async fn stop_dictation(state: State<'_, AppState>, app: AppHandle) -> Result<String, String> {
    #[cfg(windows)]
    {
        if ATOMIC_SYSTEM_MUTED.load(std::sync::atomic::Ordering::Relaxed) {
            toggle_system_mute();
            ATOMIC_SYSTEM_MUTED.store(false, std::sync::atomic::Ordering::Relaxed);
        }
    }

    let (samples, _gain) = {
        let conf = state.config.read().map_err(|e| e.to_string())?;
        let g = conf.mic_gain;
        let recorder_lock = state.recorder.lock().map_err(|e| e.to_string())?;
        if let Some(recorder) = recorder_lock.as_ref() {
            (recorder.stop_recording(g), g)
        } else {
            return Err("Audio recorder not initialized".to_string());
        }
    };

    let _ = app.emit("dictation-state", "transcribing");

    let config = {
        let lock = state.config.read().map_err(|e| e.to_string())?;
        lock.clone()
    };

    let target_whisper_model = if config.model_id.contains("medium") {
        "medium.en"
    } else if config.model_id.contains("base") {
        "base.en"
    } else {
        "turbo"
    };

    let whisper_arc = Arc::clone(&state.whisper);
    let raw_text = {
        let mut whisper_guard = whisper_arc.lock().await;
        if whisper_guard.is_none() {
            *whisper_guard = Some(WhisperEngine::new(target_whisper_model));
        } else if let Some(whisper) = whisper_guard.as_ref() {
            whisper.set_model(target_whisper_model);
        }
        if let Some(whisper) = whisper_guard.as_ref() {
            whisper.transcribe(samples).await?
        } else {
            String::new()
        }
    };

    let screen_context = if config.deep_context {
        let (app_name, title) = get_foreground_context();
        if !title.is_empty() {
            Some(format!("Active Application: {}\nWindow Title: {}", app_name, title))
        } else {
            Some(format!("Active Application: {}", app_name))
        }
    } else {
        None
    };

    let cleaned_text = refine_with_llm(&raw_text, &config, screen_context.as_deref()).await;

    if !cleaned_text.is_empty() {
        if let Err(e) = inject_text(&cleaned_text) {
            eprintln!("[AetherVoice] Text injection failed: {}", e);
        }
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

#[tauri::command]
fn get_audio_level(state: State<'_, AppState>) -> f32 {
    if let Ok(rec_lock) = state.recorder.lock() {
        if let Some(recorder) = rec_lock.as_ref() {
            return recorder.get_current_level();
        }
    }
    0.0
}

fn get_foreground_context() -> (String, String) {
    #[cfg(windows)]
    unsafe {
        use windows_sys::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowTextW};
        use windows_sys::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION};
        use windows_sys::Win32::System::ProcessStatus::GetProcessImageFileNameW;

        let hwnd = GetForegroundWindow();
        if hwnd.is_null() {
            return ("Desktop".to_string(), String::new());
        }

        // Try getting window title
        let mut title_buf = [0u16; 512];
        let len = GetWindowTextW(hwnd, title_buf.as_mut_ptr(), 512);
        let title = if len > 0 {
            String::from_utf16_lossy(&title_buf[..len as usize]).trim().to_string()
        } else {
            String::new()
        };

        // Try getting process executable name
        let mut process_id = 0u32;
        windows_sys::Win32::UI::WindowsAndMessaging::GetWindowThreadProcessId(hwnd, &mut process_id);
        let mut app_name = String::new();
        if process_id > 0 {
            let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, process_id);
            if !handle.is_null() {
                let mut img_buf = [0u16; 512];
                let img_len = GetProcessImageFileNameW(handle, img_buf.as_mut_ptr(), 512);
                windows_sys::Win32::Foundation::CloseHandle(handle);
                if img_len > 0 {
                    let full_path = String::from_utf16_lossy(&img_buf[..img_len as usize]);
                    if let Some(filename) = full_path.split('\\').last() {
                        let lower = filename.to_lowercase();
                        app_name = if lower.contains("code") { "VS Code".to_string() }
                        else if lower.contains("chrome") { "Google Chrome".to_string() }
                        else if lower.contains("msedge") { "Microsoft Edge".to_string() }
                        else if lower.contains("firefox") { "Mozilla Firefox".to_string() }
                        else if lower.contains("slack") { "Slack".to_string() }
                        else if lower.contains("discord") { "Discord".to_string() }
                        else if lower.contains("notepad") { "Notepad".to_string() }
                        else if lower.contains("cursor") { "Cursor".to_string() }
                        else if lower.contains("windsurf") { "Windsurf".to_string() }
                        else if lower.contains("antigravity") { "Antigravity IDE".to_string() }
                        else if lower.contains("word") { "Microsoft Word".to_string() }
                        else if lower.contains("teams") { "Microsoft Teams".to_string() }
                        else if lower.contains("obsidian") { "Obsidian".to_string() }
                        else { filename.replace(".exe", "") };
                    }
                }
            }
        }

        if app_name.is_empty() && !title.is_empty() {
            let title_lower = title.to_lowercase();
            app_name = if title_lower.contains("chrome") { "Google Chrome".to_string() }
            else if title_lower.contains("edge") { "Microsoft Edge".to_string() }
            else if title_lower.contains("visual studio code") || title_lower.contains("code") { "VS Code".to_string() }
            else if title_lower.contains("slack") { "Slack".to_string() }
            else if title_lower.contains("discord") { "Discord".to_string() }
            else { title.clone() };
        }

        if app_name.is_empty() {
            app_name = "Desktop".to_string();
        }

        return (app_name, title);
    }
    #[cfg(not(windows))]
    ("Desktop".to_string(), String::new())
}

#[tauri::command]
fn get_active_app() -> String {
    let (app, _) = get_foreground_context();
    app
}

#[cfg(windows)]
fn spawn_global_hotkey_listener(app: AppHandle) {
    use std::sync::atomic::{AtomicBool, Ordering};
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
        GetAsyncKeyState, VK_CONTROL, VK_F8, VK_LMENU, VK_RMENU, VK_SPACE,
    };

    static IS_KEY_DOWN: AtomicBool = AtomicBool::new(false);
    static IS_HANDS_FREE_DOWN: AtomicBool = AtomicBool::new(false);

    std::thread::spawn(move || {
        loop {
            std::thread::sleep(std::time::Duration::from_millis(15));

            let (hotkey, mode, hands_free_key) = {
                if let Some(state) = app.try_state::<AppState>() {
                    if let Ok(conf) = state.config.read() {
                        (conf.hotkey.clone(), conf.activation_mode.clone(), conf.hands_free_hotkey.clone())
                    } else {
                        ("AltRight".to_string(), "push-to-talk".to_string(), "F8".to_string())
                    }
                } else {
                    ("AltRight".to_string(), "push-to-talk".to_string(), "F8".to_string())
                }
            };

            let check_key = |key_str: &str| -> bool {
                unsafe {
                    match key_str {
                        "AltRight" => (GetAsyncKeyState(VK_RMENU as i32) as u16 & 0x8000) != 0,
                        "AltLeft" => (GetAsyncKeyState(VK_LMENU as i32) as u16 & 0x8000) != 0,
                        "F8" => (GetAsyncKeyState(VK_F8 as i32) as u16 & 0x8000) != 0,
                        "Ctrl+Space" => {
                            let ctrl = (GetAsyncKeyState(VK_CONTROL as i32) as u16 & 0x8000) != 0;
                            let space = (GetAsyncKeyState(VK_SPACE as i32) as u16 & 0x8000) != 0;
                            ctrl && space
                        }
                        "None" => false,
                        _ => false,
                    }
                }
            };

            // Check if the capsule window is visible. If not visible/toggled off, do NOT record.
            let is_capsule_visible = app.get_webview_window("main")
                .and_then(|w| w.is_visible().ok())
                .unwrap_or(false);

            if !is_capsule_visible {
                // Reset state so when capsule becomes visible again, keys aren't considered stuck down
                IS_KEY_DOWN.store(false, Ordering::Relaxed);
                IS_HANDS_FREE_DOWN.store(false, Ordering::Relaxed);
                continue;
            }

            // 1. Check primary hotkey
            let primary_pressed = check_key(&hotkey);
            let was_primary_down = IS_KEY_DOWN.load(Ordering::Relaxed);

            if primary_pressed && !was_primary_down {
                IS_KEY_DOWN.store(true, Ordering::Relaxed);
                let _ = app.emit("global-hotkey-event", serde_json::json!({ "action": "press", "mode": mode }));
            } else if !primary_pressed && was_primary_down {
                IS_KEY_DOWN.store(false, Ordering::Relaxed);
                let _ = app.emit("global-hotkey-event", serde_json::json!({ "action": "release", "mode": mode }));
            }

            // 2. Check dedicated hands-free toggle hotkey
            if !hands_free_key.is_empty() && hands_free_key != "None" && hands_free_key != hotkey {
                let hf_pressed = check_key(&hands_free_key);
                let was_hf_down = IS_HANDS_FREE_DOWN.load(Ordering::Relaxed);

                if hf_pressed && !was_hf_down {
                    IS_HANDS_FREE_DOWN.store(true, Ordering::Relaxed);
                    // Emit toggle press event
                    let _ = app.emit("global-hotkey-event", serde_json::json!({ "action": "press", "mode": "toggle" }));
                } else if !hf_pressed && was_hf_down {
                    IS_HANDS_FREE_DOWN.store(false, Ordering::Relaxed);
                }
            }
        }
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    println!("[AetherVoice] Initializing AppState...");
    let state = AppState {
        recorder: Arc::new(Mutex::new(None)),
        whisper: Arc::new(tokio::sync::Mutex::new(Some(WhisperEngine::new("turbo")))),
        config: Arc::new(RwLock::new(AppConfig::default())),
    };

    tauri::Builder::default()
        .manage(state)
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            check_model_status,
            ensure_model,
            get_installed_llm_models,
            pull_llm_model,
            delete_llm_model,
            verify_and_sync_models,
            confirm_model_installed,
            unconfirm_model,
            start_dictation,
            stop_dictation,
            test_inject,
            toggle_capsule_visibility,
            open_settings_window,
            get_user_config,
            save_user_config,
            get_user_stats,
            save_user_stats,
            get_audio_devices,
            start_dragging,
            get_audio_level,
            get_active_app,
            get_dictation_history,
            save_dictation_history,
            get_companion_state,
            save_companion_state,
            toggle_companion_window,
            open_settings
        ])
        .setup(|app| {
            let handle = app.handle().clone();

            // Initialize persistent storage (restore any wiped data from persistent backup)
            init_persistent_storage(&handle);

            // Load saved user config from disk on startup
            if let Ok(config_path) = get_config_path(&handle) {
                if config_path.exists() {
                    if let Ok(data) = std::fs::read_to_string(&config_path) {
                        if let Ok(loaded) = serde_json::from_str::<AppConfig>(&data) {
                            if let Some(state) = handle.try_state::<AppState>() {
                                if let Ok(mut lock) = state.config.write() {
                                    *lock = loaded;
                                    println!("[AetherVoice] Loaded saved user configuration.");
                                }
                            }
                        }
                    }
                }
            }

            // Perform manual model verification on startup to guarantee consistency
            let handle_verify = handle.clone();
            tauri::async_runtime::spawn(async move {
                if let Some(state_ref) = handle_verify.try_state::<AppState>() {
                    println!("[AetherVoice] Verifying installed models against user's recorded configuration...");
                    match verify_and_sync_models_internal(&handle_verify, &state_ref).await {
                        Ok(summary) => {
                            println!(
                                "[AetherVoice] Startup model verification complete: {} confirmed, {} verified on disk (Ollama online: {})",
                                summary.confirmed_models.len(),
                                summary.verified_models.len(),
                                summary.ollama_available
                            );
                        }
                        Err(e) => {
                            eprintln!("[AetherVoice] Startup model verification error: {}", e);
                        }
                    }
                }
            });

            #[cfg(windows)]
            spawn_global_hotkey_listener(handle.clone());

            let settings_i = MenuItem::with_id(app, "settings", "Settings...", true, None::<&str>)?;
            let toggle_i = MenuItem::with_id(app, "toggle", "Toggle Capsule", true, None::<&str>)?;
            let quit_i = MenuItem::with_id(app, "quit", "Quit AetherVoice", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&settings_i, &toggle_i, &quit_i])?;

            let mut tray_builder = TrayIconBuilder::new()
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
                        if let Some(state) = app.try_state::<AppState>() {
                            if let Ok(whisper_lock) = state.whisper.try_lock() {
                                if let Some(whisper_engine) = whisper_lock.as_ref() {
                                    whisper_engine.terminate();
                                }
                            }
                        }
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
                });

            if let Some(icon) = app.default_window_icon() {
                tray_builder = tray_builder.icon(icon.clone());
            }

            let _tray = tray_builder.build(app)?;

            if let Some(main_win) = app.get_webview_window("main") {
                println!("[AetherVoice] Showing capsule window on launch...");
                let _ = main_win.show();
                let _ = main_win.set_focus();
            }

            if let Some(companion_win) = app.get_webview_window("companion") {
                // Check if companion is enabled
                let is_enabled = if let Ok(state_val) = get_companion_state(handle.clone()) {
                    state_val.get("enabled").and_then(|v| v.as_bool()).unwrap_or(true)
                } else {
                    true
                };
                if is_enabled {
                    let _ = companion_win.show();
                } else {
                    let _ = companion_win.hide();
                }
            }

            println!("[AetherVoice] Setup completed successfully.");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
