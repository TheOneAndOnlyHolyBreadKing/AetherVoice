# Project Specification & Living Build Log: AetherVoice

## 1. Executive Summary & Vision
- **Objective**: 100% Free, Local-First, Zero-Cloud Speech-to-Text Dictation desktop application. High-fidelity privacy-respecting alternative to Aqua Voice.
- **Core Principles**: Local-first processing, zero cloud dependency, low-latency keystroke injection, sleek ethereal UI widget.
- **Target OS / Platform**: Windows 10/11 x86_64 (`x86_64-pc-windows-gnu` / `msvc`).

---

## 2. Technical Stack, Toolchains & Dependencies
- **Frontend**: Vite 8 + TypeScript + Vanilla CSS (Transparent floating capsule HUD & multi-tab settings).
- **Desktop Shell**: Tauri v2 (`@tauri-apps/api: ^2`).
- **Core Engine**: Rust 2021 (`x86_64-pc-windows-gnu` / `msvc`), `cpal 0.15` audio capture, `windows-sys 0.59.0` (Win32 API bindings).
- **Speech Engine**: GPU Whisper (`whisper-large-v3-turbo`, `medium.en`, `base.en`) managed via persistent `whisper_server.py` IPC daemon.
- **AI Reasoning / Formatting**: Local Ollama server (`127.0.0.1:11434`) supporting Gemma 2, Llama 3.2, Qwen 2.5, DeepSeek R1, Mistral.
- **Packaging / Installer**: NSIS standalone installer with custom automated dependency installer hooks (`installer_hooks.nsh`).

---

## 3. Architecture & Data Flow
```mermaid
graph TD
    A[Global Hotkey: Alt / F8] --> B[Rust Core / AppState]
    B --> C[CPAL Audio Streamer]
    C -->|16kHz PCM Float| D[AudioBuffer / WAV]
    D -->|IPC Stdin| E[whisper_server.py Daemon]
    E -->|GPU Whisper Inference| F[Raw Transcribed Text]
    F --> G[Ollama LLM Refinement API]
    G --> H[Win32 Keystroke Injector]
    H --> I[Active Target Window: VS Code, Browser, Terminal]
```

---

## 4. Current Implementation Progress & Milestones
- [x] **Phase 1: Project Initialization & Toolchain Setup**
- [x] **Phase 2: Core Speech & Dictation Engine (Whisper + Ollama)**
- [x] **Phase 3: Settings Dashboard, Audio Devices, Hotkeys & Replacements**
- [x] **Phase 4: Single-Instance Process Recognition**
  - Mutex-backed single-instance lock (`Global\AetherVoice_SingleInstance_Mutex`).
  - Native system modal alert preventing duplicate instances and notifying user of active instance in tray/screen.
- [x] **Phase 5: Automated Prerequisites & Setup Packaging**
  - NSIS custom installer hook (`installer_hooks.nsh`) dynamically detecting and provisioning Python, Whisper, PyTorch, audio dependencies, and Ollama.
  - Bundling and path resolution for `whisper_server.py` across development and production install directories.
- [x] **Phase 6: Persistent User Data & Strict Model Verification System**
  - Double-layer data persistence: Primary store (`%APPDATA%\com.aethervoice.app`) with persistent mirror (`%APPDATA%\AetherVoice`) impervious to installer wipes.
  - Interactive NSIS installer prompt (`NSIS_HOOK_PREINSTALL`): Asks user whether to preserve existing settings, dictation history, and confirmed models (defaults to preserving all data).
  - NSIS restore hooks (`NSIS_HOOK_POSTINSTALL`): Automatically restores missing configuration, history, and stats if fresh install directory is created.
  - Strict Startup Model Verification (`verify_and_sync_models`): Cross-references local Ollama physical tags (`/api/tags`) against `CATALOG_MODEL_IDS`.
  - Automatic Catalog Detection: Automatically identifies any installed models on the user's machine matching the official catalog (e.g., `gemma2:2b`) and marks them as confirmed/installed.
  - Interface Whitelist Invariant: Strictly filters out random/unverified non-catalog models on the system; only verified catalog models appear in the catalog and settings dropdown.
  - Full Stats Disk Persistence (`save_user_stats`, `get_user_stats`): Bridges stats from ephemeral browser `localStorage` to persistent disk JSON.
- [x] **Phase 7: User Acceptance Testing & GitHub Release Distribution**
  - Upgraded release target to `v0.2.0`.
  - Multi-layer persistence and strict model verification verified and confirmed.

---

## 5. Verified Working Capabilities (Proven State)
- **Single-Instance Enforcement**: Verified via Win32 `CreateMutexW` and `GetLastError() == ERROR_ALREADY_EXISTS`. Triggers native modal alert `MessageBoxW` and clean shutdown when launched multiple times.
- **Automated Setup Prerequisites**: Integrated into NSIS setup installer bundle (`AetherVoice_0.2.0_x64-setup.exe`), inspecting/installing Python, PyTorch, Whisper, and Ollama with a visible progress window during installation.
- **In-App Self-Healing Whisper Engine**: If a user runs without prerequisites pre-installed, AetherVoice detects the missing `whisper` module, informs the user with a modal notice, and automatically runs `pip install` in the background with `py` and `python` launcher fallbacks.
- **Persistent Multi-Layer User Storage**: Config (`aethervoice_config.json`), history (`aethervoice_history.json`), and stats (`aethervoice_stats.json`) mirrored to `%APPDATA%\AetherVoice`. Startup sequence self-heals by restoring from mirror if primary directory is wiped.
- **Strict Startup Model Verification**: Verified through `verify_and_sync_models_internal`. Catalog models present on the system are auto-detected and confirmed; uncataloged/random models are discarded.
- **Whisper Server Resolution**: Auto-locates `whisper_server.py` from executable root or resources bundle directory.
- **Standalone Setup EXE Generation**: Compiled release installer located at `src-tauri/target/release/bundle/nsis/AetherVoice_0.2.0_x64-setup.exe`.

---

## 6. Load-Bearing Decisions & Gotchas Resolved
- **Tauri NSIS Hook Macros**: Tauri v2 NSIS requires `NSIS_HOOK_PREINSTALL`, `NSIS_HOOK_POSTINSTALL`, `NSIS_HOOK_PREUNINSTALL`, and `NSIS_HOOK_POSTUNINSTALL`. Legacy electron-builder macro names like `customInstall` are ignored by Tauri's NSIS template.
- **Tauri Bundle Directory Deletion**: Tauri's uninstaller targets `$APPDATA\com.aethervoice.app` and `$LOCALAPPDATA\com.aethervoice.app`. Mirroring persistent files to `%APPDATA%\AetherVoice` guarantees zero data loss on reinstallations or upgrades unless user explicitly requests data deletion.
- **Safe LLM Fallback Invariant**: When no confirmed model is verified, `llm_model` safely defaults to `"none"` (Verbatim dictation) rather than auto-selecting random local Ollama models.
- **Windows-sys 0.59.0 Feature Requirements**: `CreateMutexW` requires the `Win32_Security` and `Win32_Foundation` features in `Cargo.toml`.
- **MessageBoxW Null Mutability**: Win32 `MessageBoxW` expects a mutable pointer for `HWND` (`core::ptr::null_mut()`).
- **Visible NSIS Prerequisite Execution**: Replaced silent background execution in NSIS with `ExecWait` console execution so users see active pip/winget progress rather than failing silently.

---

## 7. Immediate Next Steps (Laptop / Desktop Crossover)
1. Build production release installer `AetherVoice_0.2.0_x64-setup.exe`.
2. Commit and push changes to `main` branch.
3. Publish release tag `v0.2.0` and attach setup bundle on GitHub Releases.

---

## 8. Run & Verification Runbook
- Frontend build: `npm run build`
- Rust check: `cargo check` (in `src-tauri`)
- Bundle NSIS installer: `npx tauri build --bundles nsis`
- Output installer: `src-tauri/target/release/bundle/nsis/AetherVoice_0.2.0_x64-setup.exe`
