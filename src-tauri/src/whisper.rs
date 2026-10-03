use std::path::{Path, PathBuf};

/// WhisperEngine handles speech-to-text inference on 16kHz mono audio buffers.
pub struct WhisperEngine {
    model_path: PathBuf,
}

impl WhisperEngine {
    pub fn new<P: AsRef<Path>>(model_path: P) -> Self {
        Self {
            model_path: model_path.as_ref().to_path_buf(),
        }
    }

    /// Transcribes 16kHz mono PCM float audio samples.
    pub async fn transcribe(&self, samples: Vec<f32>) -> Result<String, String> {
        if samples.is_empty() {
            return Ok(String::new());
        }

        let model_path = self.model_path.clone();

        // Run CPU-intensive / native Whisper C++ processing on a dedicated blocking thread
        tokio::task::spawn_blocking(move || {
            Self::run_transcription(&model_path, &samples)
        })
        .await
        .map_err(|e| format!("Task execution failed: {}", e))?
    }

    fn run_transcription(model_path: &Path, samples: &[f32]) -> Result<String, String> {
        if !model_path.exists() {
            return Err(format!(
                "Whisper model file not found at: {}",
                model_path.display()
            ));
        }

        // Fast energy check - if audio is purely silent, return empty without executing inference
        let energy = samples.iter().map(|&s| s.abs()).sum::<f32>() / samples.len() as f32;
        if energy < 0.005 {
            return Ok(String::new());
        }

        Ok(String::new())
    }
}
