import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";

const appWindow = getCurrentWindow();

type DictationState = "idle" | "listening" | "transcribing";

const capsule = document.getElementById("capsule");
const dismissBtn = document.getElementById("dismiss-btn");
const stopBtn = document.getElementById("stop-btn");

let isRecording = false;
let audioPollInterval: number | null = null;
let recordingStartTime: number = 0;

function updateState(state: DictationState, _detail?: string) {
  if (!capsule) return;

  capsule.classList.remove("idle", "listening", "transcribing");

  if (state === "listening") {
    capsule.classList.add("listening");
  } else if (state === "transcribing") {
    capsule.classList.add("transcribing");
  } else {
    capsule.classList.add("idle");
  }
}

// Base standby heights for the 12 waveform bars
const STANDBY_HEIGHTS = [3, 5, 8, 12, 16, 18, 18, 15, 10, 6, 4, 2];
const waveformBars = Array.from({ length: 12 }, (_, i) => 
  document.querySelector(`.bar-${i + 1}`) as HTMLElement | null
);

function resetWaveform() {
  waveformBars.forEach((bar, idx) => {
    if (bar) {
      bar.style.height = `${STANDBY_HEIGHTS[idx]}px`;
    }
  });
}

function startVisualizer() {
  if (audioPollInterval) clearInterval(audioPollInterval);

  audioPollInterval = window.setInterval(async () => {
    if (!isRecording) {
      if (audioPollInterval) clearInterval(audioPollInterval);
      audioPollInterval = null;
      resetWaveform();
      return;
    }

    try {
      const rawLevel = await invoke<number>("get_audio_level");
      // rawLevel is RMS (0.0 to ~1.0). Scale to give a responsive bouncy visual
      const boost = Math.min(1.0, Math.pow(rawLevel * 4.5, 0.75));

      waveformBars.forEach((bar, idx) => {
        if (!bar) return;
        const base = STANDBY_HEIGHTS[idx];
        const centerFactor = 1.0 - Math.abs(idx - 5.5) / 6.0; // 0.1 to 1.0
        // Dynamic jitter for natural organic frequency movement
        const jitter = (Math.sin(Date.now() / 80 + idx * 0.9) * 0.25 + 0.75);
        const dynamicHeight = Math.max(
          2.5,
          Math.min(22, base * (1 - boost * 0.5) + (22 * boost * centerFactor * jitter))
        );
        bar.style.height = `${dynamicHeight.toFixed(1)}px`;
      });
    } catch {
      // ignore when polling fails
    }
  }, 40);
}

function stopVisualizer() {
  if (audioPollInterval) {
    clearInterval(audioPollInterval);
    audioPollInterval = null;
  }
  resetWaveform();
}

function recordSessionStats(text: string, durationSec: number, targetApp: string = "Desktop") {
  try {
    const trimmed = text.trim();
    const words = trimmed ? trimmed.split(/\s+/).length : 0;
    
    // Update live dictation history in localStorage
    const history = JSON.parse(localStorage.getItem("aethervoice_history") || "[]");
    const now = new Date();
    const timeStr = now.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
    const fullDateStr = now.toLocaleDateString([], { month: "short", day: "numeric" });
    
    history.unshift({
      id: Date.now().toString(),
      time: `${fullDateStr} · ${timeStr}`,
      timestamp: Date.now(),
      text: trimmed,
      words: words,
      duration: Math.round(durationSec),
      app: targetApp
    });

    // Keep last 100 entries
    if (history.length > 100) history.pop();
    localStorage.setItem("aethervoice_history", JSON.stringify(history));

    // Update cumulative stats including per-app dictation frequency
    const stats = JSON.parse(localStorage.getItem("aethervoice_stats") || '{"words":0,"secondsSaved":0,"sessions":0,"totalDuration":0,"appUsage":{}}');
    stats.words += words;
    stats.sessions += 1;
    stats.totalDuration += durationSec;
    // An average person speaks ~150 wpm and types ~40 wpm; speaking saves typing time
    stats.secondsSaved += Math.round(words * 1.0 + durationSec * 0.3);
    
    if (!stats.appUsage) stats.appUsage = {};
    stats.appUsage[targetApp] = (stats.appUsage[targetApp] || 0) + 1;

    localStorage.setItem("aethervoice_stats", JSON.stringify(stats));

    // Dispatch custom event in case settings window is open in same webview context
    window.dispatchEvent(new Event("aethervoice-stats-updated"));
  } catch (err) {
    console.error("Failed to record session stats:", err);
  }
}

async function startDictation() {
  if (isRecording) return;
  isRecording = true;
  recordingStartTime = Date.now();
  updateState("listening");
  startVisualizer();
  try {
    await invoke("start_dictation");
  } catch (err) {
    console.error("Failed to start dictation:", err);
    stopVisualizer();
  }
}

async function stopDictation() {
  if (!isRecording) return;
  isRecording = false;
  const durationSec = Math.max(1, (Date.now() - recordingStartTime) / 1000);
  stopVisualizer();
  updateState("transcribing");

  let activeApp = "Desktop";
  try {
    activeApp = await invoke<string>("get_active_app");
  } catch {
    activeApp = "Desktop";
  }

  try {
    const text = await invoke<string>("stop_dictation");
    if (text) {
      console.log("Injected text:", text);
      recordSessionStats(text, durationSec, activeApp);
    } else {
      // Record session even if empty silence
      recordSessionStats("Dictated audio (silence / no speech detected)", durationSec, activeApp);
    }
  } catch (err) {
    console.error("Failed to stop dictation:", err);
  } finally {
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

    // Global system-wide hotkey listener (Right Alt, Ctrl+Space, F8, etc.)
    await listen<{ action: "press" | "release"; mode: string }>("global-hotkey-event", async (event) => {
      const { action, mode } = event.payload;
      if (mode === "toggle") {
        if (action === "press") {
          if (!isRecording) {
            await startDictation();
          } else {
            await stopDictation();
          }
        }
      } else {
        // Push-to-talk (hold to speak)
        if (action === "press") {
          if (!isRecording) {
            await startDictation();
          }
        } else if (action === "release") {
          if (isRecording) {
            await stopDictation();
          }
        }
      }
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

  // Dismiss button: minimize or hide the bubble to system tray
  dismissBtn?.addEventListener("click", async (e) => {
    e.stopPropagation();
    try {
      await appWindow.hide();
    } catch {
      try {
        await invoke("toggle_capsule_visibility");
      } catch (err) {
        console.error("Failed to hide window:", err);
      }
    }
  });

  // Stop button: finishes dictation / toggles state
  stopBtn?.addEventListener("click", async (e) => {
    e.stopPropagation();
    if (isRecording) {
      await stopDictation();
    } else {
      await startDictation();
    }
  });

  // Center voice region: click to toggle speech recording
  document.querySelector(".bubble-center")?.addEventListener("click", async () => {
    // If not dragging, single click starts or stops recording
    if (!isRecording) {
      await startDictation();
    } else {
      await stopDictation();
    }
  });

  // Double click anywhere on bubble opens the settings window
  capsule?.addEventListener("dblclick", async (e) => {
    e.stopPropagation();
    try {
      await invoke("open_settings_window");
    } catch (err) {
      console.error("Failed to open settings window:", err);
    }
  });

  // Enable dragging the bubble anywhere on the screen
  capsule?.addEventListener("mousedown", async (e) => {
    if ((e.target as HTMLElement).closest(".bubble-action-btn")) {
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
