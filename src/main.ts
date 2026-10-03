import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";

const appWindow = getCurrentWindow();

type DictationState = "idle" | "listening" | "transcribing";

const capsule = document.getElementById("capsule");
const statusLabel = document.getElementById("status-label");
const hintLabel = document.getElementById("hint-label");
const settingsBtn = document.getElementById("settings-btn");

let isRecording = false;

function updateState(state: DictationState, detail?: string) {
  if (!capsule || !statusLabel || !hintLabel) return;

  capsule.classList.remove("idle", "listening", "transcribing");

  if (state === "listening") {
    capsule.classList.add("listening");
    statusLabel.textContent = "Listening";
    hintLabel.textContent = "Release hotkey to inject";
  } else if (state === "transcribing") {
    capsule.classList.add("transcribing");
    statusLabel.textContent = "Refining";
    hintLabel.textContent = detail || "Cleaning & injecting text...";
  } else {
    capsule.classList.add("idle");
    statusLabel.textContent = "Standby";
    hintLabel.textContent = detail || "Hold Right Alt or Ctrl+Space to speak";
  }
}

async function startDictation() {
  if (isRecording) return;
  isRecording = true;
  updateState("listening");
  try {
    await invoke("start_dictation");
  } catch (err) {
    console.error("Failed to start dictation:", err);
  }
}

async function stopDictation() {
  if (!isRecording) return;
  isRecording = false;
  updateState("transcribing");
  try {
    const text = await invoke<string>("stop_dictation");
    if (text) {
      console.log("Injected text:", text);
    }
  } catch (err) {
    console.error("Failed to stop dictation:", err);
    updateState("idle");
  }
}

async function checkAndProvisionModel() {
  try {
    const isInstalled = await invoke<boolean>("check_model_status");
    if (!isInstalled) {
      updateState("transcribing", "Downloading Whisper Large-v3-Turbo...");
      await invoke("ensure_model");
      updateState("idle", "Model ready");
    }
  } catch (err) {
    console.debug("Model check or provisioning error:", err);
  }
}

window.addEventListener("DOMContentLoaded", async () => {
  // Listen for backend dictation events
  try {
    await listen<string>("dictation-state", (event) => {
      updateState(event.payload as DictationState);
    });

    await listen<number>("download-progress", (event) => {
      updateState("transcribing", `Downloading model: ${Math.round(event.payload)}%`);
    });
  } catch (err) {
    console.debug("Running without Tauri event backend:", err);
  }

  // Keyboard Push-to-Talk preview / testing (Space or Alt)
  window.addEventListener("keydown", (e) => {
    if ((e.code === "Space" && e.ctrlKey) || e.code === "AltRight") {
      if (!isRecording) {
        startDictation();
      }
    }
  });

  window.addEventListener("keyup", (e) => {
    if (e.code === "Space" || e.code === "AltRight") {
      if (isRecording) {
        stopDictation();
      }
    }
  });

  // Provision model on first launch
  checkAndProvisionModel();

  // Click on settings button to open settings window
  settingsBtn?.addEventListener("click", async (e) => {
    e.stopPropagation();
    try {
      await invoke("open_settings_window");
    } catch (err) {
      console.error("Failed to open settings window:", err);
    }
  });

  // Enable dragging the capsule anywhere on the screen
  capsule?.addEventListener("mousedown", async (e) => {
    // If the click is on the settings button, do not start dragging
    if ((e.target as HTMLElement).closest("#settings-btn")) {
      return;
    }
    if (e.button === 0) {
      try {
        await appWindow.startDragging();
      } catch {
        try {
          await invoke("start_dragging");
        } catch (err) {
          console.error("Window drag error:", err);
        }
      }
    }
  });
});
