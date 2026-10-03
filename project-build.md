# Project Specification & Living Build Log: AetherVoice

## 1. Executive Summary & Vision
- **Objective**: AetherVoice is a 100% free, local-first, zero-cloud speech-to-text dictation utility built as a privacy-respecting alternative to Aqua Voice.
- **Core Principles**: Zero Cost, Zero API Keys, 100% Local Inference, Whisper Large-v3-Turbo GGUF execution, Zero Cloud Relays.
- **Target OS**: Windows 10/11 (`x86_64`) and macOS.
- **Design Brand Identity**: "The Ethereal Monolith" — Void Obsidian (`#0A0D14`), Cyan Aether (`#00F2FE`), Deep Electric Indigo (`#4FACFE`), and Slate Vapor (`#64748B`).

---

## 2. Technical Stack, Toolchains & Dependencies
- **Shell & UI Framework**: Tauri v2 (`@tauri-apps/api: ^2`, `@tauri-apps/plugin-opener: ^2`).
- **Frontend**: Vite 8.3.1 + TypeScript + Vanilla CSS (No Tailwind ad-hoc bloat).
- **Core Backend**: Rust 1.85 / 2021 edition (`stable-x86_64-pc-windows-gnu`).
- **Native Toolchain**: WinLibs MinGW-w64 (GCC 14.2, Clang 19, CMake, Binutils) in `%LOCALAPPDATA%\Programs\mingw64\bin`.
- **Audio Capture Subsystem**: `cpal 0.15` (Windows WASAPI loop, 16kHz linear resampling, silence trimming).
- **Input Injection Engine**: `enigo 0.2` (Win32 `SendInput` virtual keystroke synthesis).
- **Text Refinement Engine**: Custom regex substitution, user macro replacements, dictionary casing enforcement, and style directives (Software Architect, Bullets, Lowercase).
- **Persistence**: `%APPDATA%/com.aethervoice.app/aethervoice_config.json` via Tauri IPC (`get_user_config` / `save_user_config`).

---

## 3. Architecture & Data Flow

```text
[User Holds Hotkey: Right Alt / Ctrl+Space]
  │
  ▼
[AudioRecorder (CPAL)] ──── 16kHz PCM Buffer ───► [Silero / Energy Silence Trimmer]
  │
[User Releases Hotkey]
  │
  ▼
[Whisper Engine] ────────── Local GGUF Inference (ggml-large-v3-turbo-q5_0.bin)
  │
  ▼
[Text Refinement Layer] ─── User Instructions + Replacements + Dictionary
  │
  ▼
[Native Input Injector] ─── Win32 SendInput Keystroke Typing into Active Window
```

---

## 4. Current Implementation Progress & Milestones

- [x] **Phase 1: Toolchain Setup & Windows MinGW Environment**
  - Configured WinLibs MinGW-w64 toolchain in user path.
  - Set default Rust target to `x86_64-pc-windows-gnu`.
  - Added Win32 `GetShortPathNameW` path sanitizer in `build.rs` to fix `windres` unquoted space compiler errors on Windows.

- [x] **Phase 2: Core Audio & Dictation Engine**
  - Built `AudioRecorder` with `cpal` streaming at 16kHz mono.
  - Implemented `unsafe impl Send for AudioRecorder` and `Sync` to satisfy Tauri thread bounds.
  - Built `downloader.rs` for Hugging Face streaming model download with progress reporting.
  - Built `whisper.rs` thread-safe inference runner.
  - Built `injector.rs` using `enigo` for keystroke synthesis.

- [x] **Phase 3: Brand Identity & UI Shell ("The Ethereal Monolith")**
  - Integrated custom icon (`src-tauri/icons/icon.ico`) and brand logo (`src/assets/logo.png`).
  - Styled floating draggable capsule widget (`src/styles.css`, `index.html`, `src/main.ts`).
  - Added animated sound visualizer bars and active listening states.

- [x] **Phase 4: Aqua Voice-Style Settings & Custom Instructions Panel**
  - Multi-window configuration: Added dedicated 960x650 `settings` window in `tauri.conf.json`.
  - Created `settings.html`, `src/settings.css`, and `src/settings.ts`.
  - Integrated 8 navigation sections: Settings, Dictionary, Replacements, Instructions, History, Offline Engine, Stats, About.
  - Simplified Instructions tab by removing preset templates and sandbox per direct design requirements.
  - Added system tray menu with "Settings...", "Toggle Capsule", and "Quit".

- [x] **Phase 5: Audio Hardware Device Management & Arc Usage Gauge**
  - **Blue Dot Brand Icon**: Replaced generic image logo in the settings sidebar with the glowing ethereal blue dot icon matching the Aqua Voice bubble design.
  - **Audio Hardware Device Selection**: Integrated CPAL device enumeration (`AudioRecorder::list_input_devices`) exposed via `get_audio_devices` Tauri command.
  - **Microphone & Processing Settings**: Added audio input device selector, microphone input gain slider (0.5x – 2.5x), background noise suppression toggle, and acoustic echo cancellation toggle.
  - **Top Arc Usage Gauge for Stats**: Implemented an SVG dynamic Arc Usage gauge in the Stats tab displaying real-time neural STT throughput, peak buffer capacity, and local CPU/GPU load.

- [x] **Phase 5: Floating Capsule Physics & Visual Cleanup**
  - Enabled desktop-wide dragging anywhere on the capsule (`start_dragging` IPC + `getCurrentWindow().startDragging()`).
  - Added `core:window:allow-start-dragging` permission in `capabilities/default.json`.
  - Added `pointer-events: none` on inner labels/icons so dragging is never intercepted.
  - Completely removed outer dark drop shadows (`box-shadow: none !important`) for a crisp, transparent float.

- [x] **Phase 7: Local GPU Whisper Turbo & Gemma 2 2B LLM Pipeline**
  - Integrated local Whisper Large-v3-Turbo Python bridge with GPU/CUDA acceleration.
  - Implemented local Gemma 2 2B post-processing via Ollama on `127.0.0.1:11434` with 24-hour VRAM pinning (`keep_alive: "24h"`).
  - Built model selector dropdown in Settings allowing dynamic switching between Gemma 2, other models, or disabled.
  - Added live hardware telemetry: Top active application tracking via Win32 `GetForegroundWindow` + `GetProcessImageFileNameW`.

- [x] **Phase 8: Aqua Voice-Fidelity Real-time Dictation & Native Windows Injection**
  - **Native Windows Clipboard Paste**: Solved `enigo` newline drop bug by implementing native Win32 `OpenClipboard`, `SetClipboardData(CF_UNICODETEXT)`, and synthesized `Ctrl+V` SendInput keystrokes in `injector.rs` with SendInput Unicode fallback.
  - **Aqua Voice Few-Shot Prompt Architecture**: Redesigned LLM prompt in `refinement.rs` with few-shot in-context examples, strict 100% content preservation, verbatim list formatting with blank lines, and absolute refusal to converse or answer questions.
  - **Prompt Auto-Migration**: Added automatic upgrade in `lib.rs` and `settings.ts` migrating legacy software architect instructions to the high-fidelity Aqua Voice dictation assistant.
  - **Anti-Slop UI Compliance**: Enforced Agency OS Windows Desktop App standards (no blue glows, no boxes around logos, solid one-color buttons, flat crisp borders).

---

## 5. Verified Working Capabilities (Proven State)
1. **Frontend Compilation**: `npm run build` bundles client cleanly in < 1 second.
2. **Backend Compilation**: `cargo check` and `cargo build` pass cleanly with GCC 16.2.0 MinGW on Windows.
3. **Standalone Production / Debug Bundling**: `npx tauri build --debug` successfully generates standalone installers (`.exe` and `.msi`).
4. **Desktop Dragging**: Grabbing anywhere on the floating capsule moves it across the desktop.
5. **Settings Window**: Dedicated settings window and custom instructions panel with presets.
6. **Config & IPC**: `get_user_config` and `save_user_config` persist settings to JSON.

---

## 6. Load-Bearing Decisions & Gotchas Resolved

- **Windows Space Paths & `windres` Preprocessor Bug**:
  - *Problem*: Path `C:\Projects\Software Apps\laptop and desktop\AetherVoice` causes `cc1.exe` to fail on unquoted spaces.
  - *Fix*: `build.rs` calls Win32 `GetShortPathNameW` to map `OUT_DIR` and `current_dir` to 8.3 short paths.
- **Export Ordinal Overflow (`export ordinal too large: 137401`)**:
  - *Problem*: Tauri template defaulted `crate-type = ["staticlib", "cdylib", "rlib"]`. Windows PE DLLs strictly enforce 16-bit export tables (<65,536 symbols).
  - *Fix*: Set `crate-type = ["rlib"]` in `src-tauri/Cargo.toml`.
- **CPAL Raw Pointer Bounds**:
  - *Problem*: `cpal::Stream` contains raw pointers on Windows preventing `tauri::manage`.
  - *Fix*: Explicitly implemented `unsafe impl Send for AudioRecorder` and `Sync`.
- **Window Dragging in Tauri v2**:
  - *Problem*: `startDragging()` is disabled unless `core:window:allow-start-dragging` is declared in capabilities.
  - *Fix*: Added permission in `capabilities/default.json` and added fallback `start_dragging` IPC command.
- **Tauri v2 Embedded Dist Window URL Routing**:
  - *Problem*: Standalone bundles require explicit window routes, otherwise webviews fail to load.
  - *Fix*: Declared `"url": "/index.html"` explicitly in `tauri.conf.json`.
- **Tray Icon Initialization Safety**:
  - *Problem*: Unchecked `app.default_window_icon().unwrap()` causes panics if runtime window icons are asynchronously resolved.
  - *Fix*: Wrapped icon configuration in `if let Some(icon) = app.default_window_icon()`.
- **Eager Audio Device Blocking Startup**:
  - *Problem*: Synchronous `AudioRecorder::new()` and `stream.play()` in `run()` caused CPAL to block during app initialization if an audio device or stream is not immediately ready.
  - *Fix*: Made `AudioRecorder` lazy, initializing on demand when `start_dictation` is triggered.
- **Aqua Voice Floating Bubble Architecture**:
  - *Problem*: Standard window title bar and borders gave a rectangular window appearance instead of a floating pill.
  - *Fix*: Set `decorations: false`, `transparent: true`, `alwaysOnTop: true`, and implemented a 3-segment floating bubble:
    - Left: Circular dismiss button (`×`) to minimize/hide to system tray.
    - Center: Glowing pulsating blue orb + 12-bar dynamic voice visualizer (draggable across screen, click to record).
    - Right: Circular red stop button (`■`) to complete speech capture and trigger transcription/refinement.
    - Double click on bubble opens the settings dashboard.

---

## 7. Immediate Next Steps (For Laptop / Desktop Crossover)
1. **Interactive Desktop Launch**:
   - Double-click [`aethervoice.exe`](file:///c:/My%20Projects/apps/AetherVoice/src-tauri/target/debug/aethervoice.exe) directly on your desktop (File Explorer is opened and highlighting it).
   - Or run the updated installer: [`AetherVoice_0.1.0_x64-setup.exe`](file:///c:/My%20Projects/apps/AetherVoice/src-tauri/target/debug/bundle/nsis/AetherVoice_0.1.0_x64-setup.exe).
2. **Bubble Interaction**:
   - Click and drag the center segment anywhere on your screens.
   - Click center or red stop button to test recording state.
   - Click `×` to hide to tray, or double click the bubble to open the full settings panel.

---

## 8. Run & Verification Runbook
- Start Dev Server (with persistent MinGW/Cargo PATH): `npm run dev:desktop`
- Test Frontend Build: `npm run build`
- Test Backend Build: `cargo build` (inside `src-tauri`)
- Direct Binary: `src-tauri/target/debug/aethervoice.exe`
- Standalone Installer: `src-tauri/target/debug/bundle/nsis/AetherVoice_0.1.0_x64-setup.exe`
