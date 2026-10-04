# AetherVoice: The Ethereal Monolith

> **100% Free, Local-First, Zero-Cloud Speech-to-Text Dictation**  
> A privacy-respecting native desktop alternative to Aqua Voice built with Tauri v2, Rust, GPU-accelerated Whisper, and local Ollama reasoning.

---

## 🌟 Key Features

- **Zero Cloud & Zero Cost**: Runs 100% locally on your hardware. No subscriptions, no API keys, and zero telemetry or audio transmitted over the network.
- **The Ethereal Monolith Capsule**: A floating, transparent, draggable widget with Void Obsidian glassmorphism and dynamic vocal-intensity audio visualization.
- **Local Speech Recognition & Intelligence**:
  - High-fidelity transcription powered by local GPU Whisper (Large-v3-Turbo, Medium, Base).
  - Optional local neural reasoning and formatting via Ollama (Gemma 2, Llama 3, Qwen 2.5, Phi-3, Mistral).
  - **Deep Context Mode**: Intelligently adapts punctuation, terminology, and formatting to the active application on your screen.
  - **Noise-Deafening Mode**: Automatically mutes background system playback while recording and instantly restores it upon completion.
- **Hands-Free & Push-to-Talk Modes**:
  - Hold down your hotkey (`Alt` or `Ctrl+Space`) to dictate like a walkie-talkie.
  - Toggle hands-free dictation using dedicated hotkeys (e.g. `F8`) to talk continuously without holding down keys.
- **Aqua Voice-Style Settings & Custom Instructions**: Full instructions panel to structure and format dictation output according to your workflow.
- **Personal Dictionary & Macro Replacements**: Add custom technical jargon and configure spoken text replacements (e.g., `"my email"` ➔ `"dev@example.com"`).
- **Direct OS Keystroke & Clipboard Injection**: Native virtual typing via Win32 keystrokes directly into any active editor, browser, or terminal.

---

## 🛠 Tech Stack

- **Frontend**: Vite 8 + TypeScript + Vanilla CSS
- **Desktop Shell**: Tauri v2 (`@tauri-apps/api: ^2`)
- **Core Engine**: Rust 2021 (`x86_64-pc-windows-gnu` / `msvc`)
- **Speech Engine**: GPU Whisper (`whisper-large-v3-turbo`, `medium.en`, `base.en`)
- **Audio Subsystem**: `cpal 0.15` (16kHz linear resampling, silence energy filtering)
- **Local LLM Integration**: Ollama API (`127.0.0.1:11434`) with automatic tag detection and model download manager

---

## 🚀 Getting Started

### Prerequisites

- [Node.js](https://nodejs.org/) (v18+)
- [Rust](https://rustup.rs/)
- [Python 3.10+](https://python.org/) with PyTorch and OpenAI Whisper (for local GPU transcription)
- Optional: [Ollama](https://ollama.com/) running locally for intelligent formatting and reasoning

### Development

```bash
# Install frontend dependencies
npm install

# Run application in development mode
npm run tauri dev
```

### Production Build

```bash
npm run build
```

---

## 📦 Releases

Download the latest standalone setup installer (`AetherVoice_0.1.0_x64-setup.exe`) from the [GitHub Releases](https://github.com/TheOneAndOnlyHolyBreadKing/AetherVoice/releases) page.
