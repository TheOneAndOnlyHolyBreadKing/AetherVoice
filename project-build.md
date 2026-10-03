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
  - Implemented Custom Instructions with 5 preset templates: Software Architect, Clean Dictation, Structured Bullets, Lowercase (Slack), and Meeting Action Items.
  - Added interactive Live Test Sandbox directly inside settings.
  - Added system tray menu with "Settings...", "Toggle Capsule", and "Quit".

- [x] **Phase 5: Floating Capsule Physics & Visual Cleanup**
  - Enabled desktop-wide dragging anywhere on the capsule (`start_dragging` IPC + `getCurrentWindow().startDragging()`).
  - Added `core:window:allow-start-dragging` permission in `capabilities/default.json`.
  - Added `pointer-events: none` on inner labels/icons so dragging is never intercepted.
  - Completely removed outer dark drop shadows (`box-shadow: none !important`) for a crisp, transparent float.

- [ ] **Phase 6: whisper-rs native build & NSIS installer bundling**
  - Link embedded `whisper-rs` C++ library with CPU/GPU acceleration.
  - Build final standalone release installer with `npm run tauri build`.

---

## 5. Verified Working Capabilities (Proven State)
1. **Frontend Compilation**: `npm run build` bundles client cleanly in < 1 second.
2. **Backend Compilation**: `cargo check` and `cargo build` pass with exit code 0.
3. **Desktop Dragging**: Grabbing anywhere on the floating capsule moves it across the desktop.
4. **Settings Window**: Clicking the gear icon opens the full Aqua Voice-style settings panel with custom instructions.
5. **Config & IPC**: `get_user_config` and `save_user_config` persist settings to JSON.

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

---

## 7. Immediate Next Steps (For Laptop / Desktop Crossover)
1. **Launch & Test Application**: Run `npm run tauri dev` in the project root.
2. **Test Floating Capsule**: Click and drag the capsule across screens, verify no dark box shadow.
3. **Test Settings Panel**: Click gear icon, edit custom instructions, switch presets, test phrases in sandbox, and hit Save.
4. **Model Provisioning**: Download default `ggml-large-v3-turbo-q5_0.bin` via the in-app progress bar.

---

## 8. Run & Verification Runbook
- Start Dev Server: `npm run tauri dev`
- Test Frontend Build: `npm run build`
- Test Backend Check: `cargo check` (inside `src-tauri`)
- Test Backend Build: `cargo build` (inside `src-tauri`)
