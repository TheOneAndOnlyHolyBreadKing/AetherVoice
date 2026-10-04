use hound::{WavSpec, WavWriter};
use serde::Deserialize;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::Mutex;

#[derive(Deserialize)]
struct TranscribeResponse {
    status: String,
    text: Option<String>,
    error: Option<String>,
}

/// WhisperEngine maintains a persistent subprocess executing native Whisper inference on GPU.
pub struct WhisperEngine {
    process: Mutex<Option<(Child, ChildStdin, BufReader<ChildStdout>)>>,
    temp_dir: PathBuf,
    model_name: Mutex<String>,
}

impl WhisperEngine {
    pub fn new(model_name: &str) -> Self {
        let temp_dir = std::env::temp_dir().join("aethervoice_audio");
        if !temp_dir.exists() {
            let _ = std::fs::create_dir_all(&temp_dir);
        }

        let engine = Self {
            process: Mutex::new(None),
            temp_dir,
            model_name: Mutex::new(model_name.to_string()),
        };

        // Spawn persistent Python whisper daemon in background
        engine.spawn_server();
        engine
    }

    pub fn set_model(&self, new_model: &str) {
        let mut cur_model = match self.model_name.lock() {
            Ok(m) => m,
            Err(_) => return,
        };
        if *cur_model != new_model {
            *cur_model = new_model.to_string();
            // Kill existing process if running so spawn_server restarts with new model
            if let Ok(mut lock) = self.process.lock() {
                if let Some((mut child, _, _)) = lock.take() {
                    let _ = child.kill();
                }
            }
            drop(cur_model);
            self.spawn_server();
        }
    }

    fn spawn_server(&self) {
        let mut lock = match self.process.lock() {
            Ok(l) => l,
            Err(_) => return,
        };

        if lock.is_some() {
            return;
        }

        let model = self.model_name.lock().map(|m| m.clone()).unwrap_or_else(|_| "turbo".to_string());
        let script_path = {
            let manifest_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("whisper_server.py");
            if manifest_path.exists() {
                manifest_path
            } else if let Ok(exe_path) = std::env::current_exe() {
                let exe_dir = exe_path.parent().unwrap_or(std::path::Path::new(""));
                let candidate1 = exe_dir.join("whisper_server.py");
                let candidate2 = exe_dir.join("resources").join("whisper_server.py");
                if candidate1.exists() {
                    candidate1
                } else if candidate2.exists() {
                    candidate2
                } else {
                    manifest_path
                }
            } else {
                manifest_path
            }
        };
        println!("[WhisperEngine] Launching whisper daemon with model '{}': {:?}", model, script_path);

        let mut py_binary = "python";
        // Check if python or py launcher exists
        if Command::new("python").arg("--version").output().is_err() {
            if Command::new("py").arg("--version").output().is_ok() {
                py_binary = "py";
            }
        }

        // Verify if whisper package is installed; if not, attempt quick self-heal pip install
        let whisper_check = Command::new(py_binary)
            .args(&["-c", "import whisper"])
            .output();

        if let Ok(check_out) = whisper_check {
            if !check_out.status.success() {
                println!("[WhisperEngine] Whisper not found in Python environment. Auto-installing prerequisites...");
                #[cfg(windows)]
                {
                    use windows_sys::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONINFORMATION, MB_OK, MB_SYSTEMMODAL};
                    let title: Vec<u16> = "AetherVoice - One-Time Setup\0".encode_utf16().collect();
                    let message: Vec<u16> = "AetherVoice is preparing the Whisper speech engine on your system.\n\nInstalling openai-whisper and PyTorch in the background now. Please wait a moment...\0"
                        .encode_utf16()
                        .collect();
                    unsafe {
                        MessageBoxW(core::ptr::null_mut(), message.as_ptr(), title.as_ptr(), MB_OK | MB_ICONINFORMATION | MB_SYSTEMMODAL);
                    }
                }

                let _ = Command::new(py_binary)
                    .args(&["-m", "pip", "install", "openai-whisper", "torch", "torchaudio", "sounddevice", "numpy"])
                    .status();
            }
        }

        let mut cmd = Command::new(py_binary);
        cmd.arg(&script_path)
            .arg(&model)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());

        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            cmd.creation_flags(CREATE_NO_WINDOW);
        }

        match cmd.spawn() {
            Ok(mut child) => {
                let stdin = match child.stdin.take() {
                    Some(s) => s,
                    None => return,
                };
                let stdout = match child.stdout.take() {
                    Some(s) => s,
                    None => return,
                };
                let mut reader = BufReader::new(stdout);

                // Wait for the READY handshake from whisper_server
                let mut line = String::new();
                if let Ok(_) = reader.read_line(&mut line) {
                    if line.trim() == "READY" {
                        println!("[WhisperEngine] Speech recognition engine is online and READY.");
                    }
                }

                *lock = Some((child, stdin, reader));
            }
            Err(e) => {
                eprintln!("[WhisperEngine] Failed to spawn Whisper server: {}", e);
            }
        }
    }

    /// Transcribes 16kHz mono PCM float audio samples.
    pub async fn transcribe(&self, samples: Vec<f32>) -> Result<String, String> {
        if samples.is_empty() {
            return Ok(String::new());
        }

        // Fast energy check: if audio has virtually 0 RMS, skip inference
        let energy = samples.iter().map(|&s| s.abs()).sum::<f32>() / samples.len() as f32;
        if energy < 0.001 {
            return Ok(String::new());
        }

        let temp_dir = self.temp_dir.clone();
        
        // Write audio to temporary 16kHz mono 16-bit WAV file
        let wav_path = temp_dir.join(format!("rec_{}.wav", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()));
        
        {
            let spec = WavSpec {
                channels: 1,
                sample_rate: 16000,
                bits_per_sample: 16,
                sample_format: hound::SampleFormat::Int,
            };
            let mut writer = WavWriter::create(&wav_path, spec)
                .map_err(|e| format!("Failed to create WAV file: {}", e))?;
            for &s in &samples {
                let clamped = s.clamp(-1.0, 1.0);
                let val = (clamped * 32767.0) as i16;
                writer.write_sample(val).map_err(|e| e.to_string())?;
            }
            writer.finalize().map_err(|e| e.to_string())?;
        }

        // Send file to daemon over IPC
        let response = {
            let mut lock = self.process.lock().map_err(|e| e.to_string())?;
            if lock.is_none() {
                drop(lock);
                self.spawn_server();
                lock = self.process.lock().map_err(|e| e.to_string())?;
            }

            if let Some((_, stdin, reader)) = lock.as_mut() {
                let path_str = wav_path.to_string_lossy().to_string();
                writeln!(stdin, "{}", path_str).map_err(|e| format!("Failed writing to whisper stdin: {}", e))?;
                stdin.flush().map_err(|e| format!("Failed flushing whisper stdin: {}", e))?;

                let mut resp_line = String::new();
                reader.read_line(&mut resp_line).map_err(|e| format!("Failed reading whisper stdout: {}", e))?;
                resp_line
            } else {
                return Err("Whisper subprocess is unavailable".to_string());
            }
        };

        // Clean up temporary wav file
        let _ = std::fs::remove_file(&wav_path);

        match serde_json::from_str::<TranscribeResponse>(&response) {
            Ok(parsed) => {
                if parsed.status == "ok" {
                    Ok(parsed.text.unwrap_or_default())
                } else {
                    Err(parsed.error.unwrap_or_else(|| "Unknown whisper error".to_string()))
                }
            }
            Err(e) => Err(format!("Invalid whisper response JSON '{}': {}", response.trim(), e)),
        }
    }

    /// Cleanly kills the whisper daemon child process.
    pub fn terminate(&self) {
        if let Ok(mut lock) = self.process.lock() {
            if let Some((mut child, mut stdin, _)) = lock.take() {
                // Try soft quit first
                let _ = writeln!(stdin, "QUIT");
                let _ = stdin.flush();
                // Forcibly kill to guarantee no lingering background task
                let _ = child.kill();
                let _ = child.wait();
            }
        }
    }
}

impl Drop for WhisperEngine {
    fn drop(&mut self) {
        self.terminate();
    }
}
