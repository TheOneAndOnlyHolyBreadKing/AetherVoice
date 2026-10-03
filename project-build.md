# Project Specification: AetherVoice (Native Cross-Platform Dictation)

## 1. Executive Summary & Vision
AetherVoice is a 100% free, local-first, zero-cloud speech-to-text dictation utility built as an alternative to Aqua Voice.
- **Zero Cost & Offline:** No API keys, subscriptions, or external cloud relays.
- **Target OS:** Windows 10/11 (`x86_64`) and macOS (Apple Silicon & Intel).
- **End-User Distribution:** One-click setup wizard (`AetherVoice-Setup.exe` via NSIS) that auto-configures shortcuts and runs without manual dependency installations or terminal prompts.

---

## 2. Technical Stack & Dependencies

```
┌────────────────────────────────────────────────────────┐
│               Frontend Shell (Tauri 2.0)               │
│        Floating Capsule UI / Settings / Downloader     │
└───────────────────────────┬────────────────────────────┘
                            │ Tauri IPC Commands
┌───────────────────────────▼────────────────────────────┐
│                    Rust Core Engine                    │
│  ├─ Global Push-to-Talk (tauri-plugin-global-shortcut) │
│  ├─ Native Audio Capture (cpal)                        │
│  ├─ Endpoint & Speech Detection (Silero VAD ONNX)      │
│  └─ Virtual Keyboard Input (enigo / Win32 SendInput)   │
└─────────────┬────────────────────────────┬─────────────┘
              ▼                            ▼
     Embedded STT Engine            Text Refinement Layer
     • whisper.cpp via whisper-rs   • Embedded GGUF SLM (llama-cpp-rs)
     • Whisper Large-v3-Turbo Q5_0  • Fast Regex & Rule Engine
```

### Core Technologies
- **App Framework:** Tauri v2 (Rust backend, HTML5/CSS3/TypeScript frontend).
- **Speech-to-Text Engine:** `whisper-rs` (C++ bindings to `whisper.cpp` compiled directly into the binary; Metal acceleration on macOS, AVX2/DirectML on Windows).
- **Default STT Model:** `whisper-large-v3-turbo` (quantized to Q5_0, ~550 MB – 800 MB GGUF).
- **Audio Capture & VAD:** `cpal` (WASAPI on Windows, CoreAudio on macOS) with Silero VAD ONNX for silence suppression.
- **Text Injection:** Native OS input synthesis via `enigo` (`SendInput` for Windows, `CGEvent` for macOS).
- **Packaging:** NSIS bundler configured to install in `currentUser` mode (no Admin UAC prompts required).

---

## 3. Architecture & Data Flow

```
[User holds Push-to-Talk Hotkey] (Default: Right Alt or Ctrl+Space)
│
▼
[1. Audio Capture Pipeline]
• cpal streams PCM audio (16kHz, 16-bit mono) into memory buffer
• Silero VAD monitors speech frames and drops ambient silence
│
[User releases Push-to-Talk Hotkey]
│
▼
[2. Local STT Processing]
• Buffer passes to embedded whisper-rs instance
• Model: ggml-large-v3-turbo-q5_0.bin
• Latency target: <250ms for a 5-second audio chunk
│
▼
[3. Refinement & Formatting Pass]
• Fast regex substitution (e.g., "new line" -> "\n", "period" -> ".")
• Context pass: Strips vocal fillers ("um", "uh", "you know") and applies auto-casing
│
▼
[4. OS Insertion]
• Rust invokes enigo to simulate immediate keyboard keystrokes directly into the active cursor
```

---

## 4. First-Launch Auto-Provisioning Flow

To keep the initial installer tiny (~15–20 MB) and avoid distributing multi-gigabyte setup files:

1. User installs via `AetherVoice-Setup.exe` and launches `AetherVoice.exe`.
2. Application checks `tauri::path::BaseDirectory::AppData` for `models/ggml-large-v3-turbo-q5_0.bin`.
3. If not present:
   - UI displays a clean progress bar modal: *"Preparing local AI dictation engine..."*
   - Rust background worker downloads the model directly from Hugging Face Hub (zero developer bandwidth cost).
4. Once verified, the app minimizes directly to the OS system tray.

---

## 5. Implementation Tasks & Roadmap

### Phase 1: Project Scaffolding
- Initialize Tauri v2 workspace with standard Rust toolchain.
- Configure `Cargo.toml` with dependencies: `tauri`, `tauri-plugin-global-shortcut`, `whisper-rs`, `cpal`, `enigo`, `tokio`, `reqwest`.
- Set up Windows NSIS packaging configuration in `tauri.conf.json` with `installMode: "currentUser"`.

### Phase 2: Audio Capture & Speech Detection
- Implement `cpal` input stream listener with 16kHz resampler.
- Add circular buffer to record only while the global hotkey is held.
- Integrate Silero VAD to detect audio cutoffs and trim blank margins.

### Phase 3: Whisper Integration
- Write `whisper-rs` wrapper to initialize context from local `.bin` / `.gguf` file.
- Implement async transcription pipeline delivering raw text strings back to Tauri state.
- Create automated download helper with chunk-based progress reporting sent via Tauri events to the frontend.

### Phase 4: Text Refinement & Keystroke Injection
- Write deterministic cleanup module for common punctuation and spoken macros.
- Hook `enigo` to type cleaned text into the active focused window.
- Implement permission pre-flight checks (notably macOS Accessibility API prompts, Windows non-admin input injection).

### Phase 5: UI/UX & Packaging
- Build the floating glassmorphic tray capsule UI matching the "Ethereal Monolith" design spec.
- Test compilation with `npm run tauri build` to output the final Windows `.exe` setup package.
