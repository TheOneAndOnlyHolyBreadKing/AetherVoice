# AetherVoice: The Ethereal Monolith

> **100% Free, Local-First, Zero-Cloud Speech-to-Text Dictation**  
> A privacy-respecting native desktop alternative to Aqua Voice built with Tauri v2, Rust, and embedded Whisper.

---

## 🌟 Key Features

- **Zero Cloud & Zero Cost**: Runs 100% locally on your machine. No monthly subscriptions, no API keys, and zero bytes transmitted over the network.
- **The Ethereal Monolith Capsule**: A floating, transparent, draggable widget with Void Obsidian glassmorphism and real-time audio visualization.
- **Aqua Voice-Style Settings & Custom Instructions**:
  - Full instructions panel to structure and format dictation output.
  - 5 Built-in presets: *Software Architect*, *Clean Dictation*, *Structured Bullets*, *Lowercase (Slack)*, and *Meeting Action Items*.
  - Interactive live test sandbox directly inside settings.
- **Personal Dictionary & Macro Replacements**: Add custom technical jargon (e.g. *Kubernetes*, *PostgreSQL*, *Tauri*) and configure spoken text replacements (e.g. `"my email"` ➔ `"dev@example.com"`).
- **Direct OS Keystroke Injection**: Native virtual typing via Win32 `SendInput` (`enigo`) directly into whatever editor or application you have focused.

---

## 🛠 Tech Stack

- **Frontend**: Vite 8.3.1 + TypeScript + Vanilla CSS (Void Obsidian `#0A0D14`, Cyan Aether `#00F2FE`, Electric Indigo `#4FACFE`).
- **Backend Shell**: Tauri v2 (`@tauri-apps/api: ^2`).
- **Core Engine**: Rust 1.85 / 2021 (`x86_64-pc-windows-gnu`).
- **Audio Subsystem**: `cpal 0.15` (16kHz linear resampling, energy silence suppression).
- **Local Model**: Whisper Large-v3-Turbo Q5_0 GGUF.
- **Input Synthesis**: `enigo 0.2` (Win32 virtual keystroke injection).

---

## 🚀 Getting Started

### Prerequisites

- [Node.js](https://nodejs.org/) (v18+)
- [Rust](https://rustup.rs/) (`stable-x86_64-pc-windows-gnu` or `msvc`)
- WinLibs MinGW-w64 toolchain (on Windows GNU target)

### Development

```bash
# Install frontend dependencies
npm install

# Run application in development mode
npm run tauri dev
```

### Production Build

```bash
npm run tauri build
```

---

## 📋 Living Project Build & Memory

For complete architectural decisions, resolved compiler gotchas, and cross-machine handoff notes, refer to [project-build.md](project-build.md).
