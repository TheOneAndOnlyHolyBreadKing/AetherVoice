use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex};

/// Represents an audio recorder managing input capture and resampling to 16kHz mono.
pub struct AudioRecorder {
    is_recording: Arc<AtomicBool>,
    audio_buffer: Arc<Mutex<Vec<f32>>>,
    current_level: Arc<AtomicU32>,
    _stream: Option<cpal::Stream>,
    sample_rate: u32,
}

// cpal::Stream contains raw Windows pointers (*mut ()) on some platforms,
// but AudioRecorder is strictly accessed safely behind Mutex across threads.
unsafe impl Send for AudioRecorder {}
unsafe impl Sync for AudioRecorder {}

impl AudioRecorder {
    /// Lists all available audio input devices on the system.
    pub fn list_input_devices() -> Vec<String> {
        let host = cpal::default_host();
        let mut devices = Vec::new();
        if let Ok(input_devices) = host.input_devices() {
            for d in input_devices {
                if let Ok(name) = d.name() {
                    devices.push(name);
                }
            }
        }
        devices
    }

    pub fn new() -> Result<Self, String> {
        Self::with_device(None)
    }

    pub fn with_device(device_name: Option<&str>) -> Result<Self, String> {
        let host = cpal::default_host();
        let device = if let Some(target_name) = device_name {
            if target_name.is_empty() || target_name == "Default" {
                host.default_input_device()
                    .ok_or_else(|| "No default audio input device found".to_string())?
            } else {
                let mut found = None;
                if let Ok(devices) = host.input_devices() {
                    for d in devices {
                        if let Ok(name) = d.name() {
                            if name == target_name {
                                found = Some(d);
                                break;
                            }
                        }
                    }
                }
                found.or_else(|| host.default_input_device())
                    .ok_or_else(|| format!("Audio device '{}' not found and no default available", target_name))?
            }
        } else {
            host.default_input_device()
                .ok_or_else(|| "No default audio input device found".to_string())?
        };

        let default_config = device
            .default_input_config()
            .map_err(|e| format!("Failed to get default input format: {}", e))?;

        let sample_rate = default_config.sample_rate().0;
        let channels = default_config.channels();

        let is_recording = Arc::new(AtomicBool::new(false));
        let audio_buffer = Arc::new(Mutex::new(Vec::with_capacity(16000 * 10))); // 10s initial capacity
        let current_level = Arc::new(AtomicU32::new(0));

        let buffer_clone = Arc::clone(&audio_buffer);
        let recording_flag = Arc::clone(&is_recording);
        let level_f32 = Arc::clone(&current_level);
        let level_i16 = Arc::clone(&current_level);
        let level_u16 = Arc::clone(&current_level);

        let err_fn = |err| eprintln!("Audio stream error: {}", err);

        let stream = match default_config.sample_format() {
            cpal::SampleFormat::F32 => device.build_input_stream(
                &default_config.into(),
                move |data: &[f32], _: &_| {
                    if recording_flag.load(Ordering::Relaxed) {
                        Self::process_samples_f32(data, channels, &buffer_clone, &level_f32);
                    } else {
                        level_f32.store(0, Ordering::Relaxed);
                    }
                },
                err_fn,
                None,
            ),
            cpal::SampleFormat::I16 => device.build_input_stream(
                &default_config.into(),
                move |data: &[i16], _: &_| {
                    if recording_flag.load(Ordering::Relaxed) {
                        Self::process_samples_i16(data, channels, &buffer_clone, &level_i16);
                    } else {
                        level_i16.store(0, Ordering::Relaxed);
                    }
                },
                err_fn,
                None,
            ),
            cpal::SampleFormat::U16 => device.build_input_stream(
                &default_config.into(),
                move |data: &[u16], _: &_| {
                    if recording_flag.load(Ordering::Relaxed) {
                        Self::process_samples_u16(data, channels, &buffer_clone, &level_u16);
                    } else {
                        level_u16.store(0, Ordering::Relaxed);
                    }
                },
                err_fn,
                None,
            ),
            _ => return Err("Unsupported audio sample format".to_string()),
        }
        .map_err(|e| format!("Failed to build input stream: {}", e))?;

        stream
            .play()
            .map_err(|e| format!("Failed to play stream: {}", e))?;

        Ok(Self {
            is_recording,
            audio_buffer,
            current_level,
            _stream: Some(stream),
            sample_rate,
        })
    }

    fn process_samples_f32(data: &[f32], channels: u16, buffer: &Arc<Mutex<Vec<f32>>>, level: &Arc<AtomicU32>) {
        let mut sum_sq = 0.0f32;
        let mut count = 0;
        let mut buf = match buffer.lock() {
            Ok(b) => b,
            Err(_) => return,
        };
        if channels == 1 {
            for &s in data {
                sum_sq += s * s;
                count += 1;
            }
            buf.extend_from_slice(data);
        } else {
            let ch = channels as usize;
            for frame in data.chunks_exact(ch) {
                let sum: f32 = frame.iter().sum();
                let mono = sum / ch as f32;
                sum_sq += mono * mono;
                count += 1;
                buf.push(mono);
            }
        }
        if count > 0 {
            let rms = (sum_sq / count as f32).sqrt();
            level.store(rms.to_bits(), Ordering::Relaxed);
        }
    }

    fn process_samples_i16(data: &[i16], channels: u16, buffer: &Arc<Mutex<Vec<f32>>>, level: &Arc<AtomicU32>) {
        let mut sum_sq = 0.0f32;
        let mut count = 0;
        let mut buf = match buffer.lock() {
            Ok(b) => b,
            Err(_) => return,
        };
        let ch = channels as usize;
        for frame in data.chunks_exact(ch) {
            let sum: f32 = frame.iter().map(|&s| s as f32 / 32768.0).sum();
            let mono = sum / ch as f32;
            sum_sq += mono * mono;
            count += 1;
            buf.push(mono);
        }
        if count > 0 {
            let rms = (sum_sq / count as f32).sqrt();
            level.store(rms.to_bits(), Ordering::Relaxed);
        }
    }

    fn process_samples_u16(data: &[u16], channels: u16, buffer: &Arc<Mutex<Vec<f32>>>, level: &Arc<AtomicU32>) {
        let mut sum_sq = 0.0f32;
        let mut count = 0;
        let mut buf = match buffer.lock() {
            Ok(b) => b,
            Err(_) => return,
        };
        let ch = channels as usize;
        for frame in data.chunks_exact(ch) {
            let sum: f32 = frame.iter().map(|&s| (s as f32 - 32768.0) / 32768.0).sum();
            let mono = sum / ch as f32;
            sum_sq += mono * mono;
            count += 1;
            buf.push(mono);
        }
        if count > 0 {
            let rms = (sum_sq / count as f32).sqrt();
            level.store(rms.to_bits(), Ordering::Relaxed);
        }
    }

    /// Return latest calculated RMS audio energy (0.0 to 1.0)
    pub fn get_current_level(&self) -> f32 {
        f32::from_bits(self.current_level.load(Ordering::Relaxed))
    }

    /// Start capturing audio into the buffer.
    pub fn start_recording(&self) {
        if let Ok(mut buf) = self.audio_buffer.lock() {
            buf.clear();
        }
        self.current_level.store(0, Ordering::Relaxed);
        self.is_recording.store(true, Ordering::SeqCst);
    }

    /// Stop capturing audio and return resampled 16,000Hz mono PCM samples ready for Whisper.
    pub fn stop_recording(&self, gain: f32) -> Vec<f32> {
        self.is_recording.store(false, Ordering::SeqCst);
        self.current_level.store(0, Ordering::Relaxed);

        let mut raw_samples = match self.audio_buffer.lock() {
            Ok(mut b) => std::mem::take(&mut *b),
            Err(_) => Vec::new(),
        };

        if raw_samples.is_empty() {
            return Vec::new();
        }

        // Apply mic gain
        let effective_gain = if gain <= 0.0 { 1.0 } else { gain };
        if (effective_gain - 1.0).abs() > 0.001 {
            for sample in raw_samples.iter_mut() {
                *sample = (*sample * effective_gain).clamp(-1.0, 1.0);
            }
        }

        let target_rate = 16000;
        let resampled = resample_linear(&raw_samples, self.sample_rate, target_rate);

        // Lower threshold to 0.002 to avoid eating soft speech / desktop mics
        let trimmed = trim_silence(&resampled, 0.002, 1600);
        if trimmed.is_empty() {
            resampled
        } else {
            trimmed
        }
    }

    pub fn is_recording(&self) -> bool {
        self.is_recording.load(Ordering::Relaxed)
    }
}

/// Linear interpolation resampler for high-performance audio rate conversion to 16kHz mono.
pub fn resample_linear(input: &[f32], source_rate: u32, target_rate: u32) -> Vec<f32> {
    if source_rate == target_rate || input.is_empty() {
        return input.to_vec();
    }

    let ratio = source_rate as f64 / target_rate as f64;
    let target_len = (input.len() as f64 / ratio).round() as usize;
    let mut output = Vec::with_capacity(target_len);

    for i in 0..target_len {
        let src_idx = i as f64 * ratio;
        let idx0 = src_idx.floor() as usize;
        let idx1 = (idx0 + 1).min(input.len() - 1);
        let fract = (src_idx - idx0 as f64) as f32;

        let sample = input[idx0] * (1.0 - fract) + input[idx1] * fract;
        output.push(sample);
    }

    output
}

/// Trims leading and trailing silence based on energy threshold.
pub fn trim_silence(samples: &[f32], threshold: f32, frame_size: usize) -> Vec<f32> {
    if samples.len() <= frame_size {
        return samples.to_vec();
    }

    let mut start_idx = 0;
    for chunk in samples.chunks(frame_size) {
        let energy = chunk.iter().map(|&s| s.abs()).sum::<f32>() / chunk.len() as f32;
        if energy > threshold {
            break;
        }
        start_idx += chunk.len();
    }

    let mut end_idx = samples.len();
    for chunk in samples.rchunks(frame_size) {
        let energy = chunk.iter().map(|&s| s.abs()).sum::<f32>() / chunk.len() as f32;
        if energy > threshold {
            break;
        }
        end_idx = end_idx.saturating_sub(chunk.len());
    }

    if start_idx >= end_idx {
        // Don't discard audio if energy is consistently low throughout; keep full recording
        return samples.to_vec();
    }

    let pad = frame_size;
    let actual_start = start_idx.saturating_sub(pad);
    let actual_end = (end_idx + pad).min(samples.len());

    samples[actual_start..actual_end].to_vec()
}
