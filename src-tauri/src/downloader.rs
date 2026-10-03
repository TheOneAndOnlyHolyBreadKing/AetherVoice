use futures_util::StreamExt;
use reqwest::Client;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Emitter};

pub const DEFAULT_MODEL_URL: &str =
    "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3-turbo-q5_0.bin";
pub const DEFAULT_MODEL_FILENAME: &str = "ggml-large-v3-turbo-q5_0.bin";

/// Returns the models directory inside AppData: `%APPDATA%/com.aethervoice.app/models/`
pub fn get_models_dir(app: &AppHandle) -> Result<PathBuf, String> {
    use tauri::Manager;
    let app_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to resolve AppData directory: {}", e))?;
    let models_dir = app_dir.join("models");
    if !models_dir.exists() {
        std::fs::create_dir_all(&models_dir)
            .map_err(|e| format!("Failed to create models directory: {}", e))?;
    }
    Ok(models_dir)
}

/// Checks if the default Whisper model exists locally
pub fn is_model_installed(models_dir: &Path) -> bool {
    models_dir.join(DEFAULT_MODEL_FILENAME).exists()
}

/// Downloads the model with chunk-based progress reporting sent via the `download-progress` event
pub async fn download_model(
    app: AppHandle,
    models_dir: &Path,
    model_url: &str,
) -> Result<PathBuf, String> {
    let client = Client::new();
    let response = client
        .get(model_url)
        .send()
        .await
        .map_err(|e| format!("Network error while initiating model download: {}", e))?;

    if !response.status().is_success() {
        return Err(format!(
            "Failed to download model, HTTP Status: {}",
            response.status()
        ));
    }

    let total_size = response.content_length().unwrap_or(0);
    let target_path = models_dir.join(DEFAULT_MODEL_FILENAME);
    let temp_path = models_dir.join(format!("{}.part", DEFAULT_MODEL_FILENAME));

    let mut file = File::create(&temp_path)
        .map_err(|e| format!("Failed to create temporary download file: {}", e))?;
    let mut downloaded: u64 = 0;
    let mut stream = response.bytes_stream();

    while let Some(chunk_result) = stream.next().await {
        let chunk =
            chunk_result.map_err(|e| format!("Error while downloading model stream: {}", e))?;
        file.write_all(&chunk)
            .map_err(|e| format!("Failed to write chunk to disk: {}", e))?;

        downloaded += chunk.len() as u64;

        if total_size > 0 {
            let progress = (downloaded as f64 / total_size as f64) * 100.0;
            let _ = app.emit("download-progress", progress);
        }
    }

    file.flush()
        .map_err(|e| format!("Failed to flush model file: {}", e))?;
    drop(file);

    // Atomically rename temporary file to destination
    std::fs::rename(&temp_path, &target_path)
        .map_err(|e| format!("Failed to finalize model file: {}", e))?;

    let _ = app.emit("download-progress", 100.0);
    Ok(target_path)
}
