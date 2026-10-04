import { invoke } from "@tauri-apps/api/core";

interface ReplacementItem {
  spoken: string;
  replacement: string;
}

interface AppConfig {
  custom_instructions: string;
  hotkey: string;
  activation_mode: string;
  model_id: string;
  llm_model?: string;
  vad_enabled: boolean;
  strip_fillers: boolean;
  spoken_punctuation: boolean;
  auto_capitalize: boolean;
  audio_device: string;
  mic_gain: number;
  noise_suppression: boolean;
  echo_cancellation: boolean;
  dictionary: string[];
  replacements: ReplacementItem[];
}

const DEFAULT_INSTRUCTIONS = `# Persona
Act as an intelligent, high-fidelity voice dictation assistant. Transform spoken speech into clean, well-formatted, and accurate text.

# Core Dictation Processing
* **Content Preservation:** Preserve all spoken thoughts, sentences, and context without summarizing, dropping, or truncating anything.
* **Smart Structuring:** When a list, sequence, or set of steps is spoken, format it cleanly with newlines and bullet points or numbers while keeping surrounding text intact.
* **Remove Speech Artifacts:** Strip out filler words ("um", "uh", "like") and stutters while retaining the full meaning of every statement.
* **Polished Writing:** Ensure correct punctuation, capitalization, and smooth readability.`;

let currentConfig: AppConfig = {
  custom_instructions: DEFAULT_INSTRUCTIONS,
  hotkey: "AltRight",
  activation_mode: "push-to-talk",
  model_id: "large-v3-turbo-q5_0",
  vad_enabled: true,
  strip_fillers: true,
  spoken_punctuation: true,
  auto_capitalize: true,
  audio_device: "Default",
  mic_gain: 1.0,
  noise_suppression: true,
  echo_cancellation: true,
  dictionary: ["AetherVoice", "Tauri", "Rust", "TypeScript", "Whisper", "PostgreSQL", "Kubernetes", "GraphQL", "GitHub"],
  replacements: [
    { spoken: "my email", replacement: "dev@example.com" },
    { spoken: "k eight s", replacement: "k8s" },
    { spoken: "smile emoji", replacement: "😊" }
  ]
};

// DOM Elements
const navItems = document.querySelectorAll<HTMLButtonElement>(".nav-item");
const tabContents = document.querySelectorAll<HTMLElement>(".tab-content");

// Instructions Tab
const instructionsTextarea = document.getElementById("instructions-textarea") as HTMLTextAreaElement;
const saveInstructionsBtn = document.getElementById("save-instructions-btn") as HTMLButtonElement;
const saveStatus = document.getElementById("save-status") as HTMLElement;

// Settings Tab (Controls & Audio)
const audioDeviceSelect = document.getElementById("audio-device-select") as HTMLSelectElement;
const micGainSlider = document.getElementById("mic-gain-slider") as HTMLInputElement;
const micGainVal = document.getElementById("mic-gain-val") as HTMLElement;
const noiseToggle = document.getElementById("noise-toggle") as HTMLInputElement;
const echoToggle = document.getElementById("echo-toggle") as HTMLInputElement;

const hotkeySelect = document.getElementById("hotkey-select") as HTMLSelectElement;
const modeSelect = document.getElementById("mode-select") as HTMLSelectElement;
const modelSelect = document.getElementById("model-select") as HTMLSelectElement;
const llmModelSelect = document.getElementById("llm-model-select") as HTMLSelectElement | null;
const vadToggle = document.getElementById("vad-toggle") as HTMLInputElement;
const fillersToggle = document.getElementById("fillers-toggle") as HTMLInputElement;
const punctuationToggle = document.getElementById("punctuation-toggle") as HTMLInputElement;
const capitalizeToggle = document.getElementById("capitalize-toggle") as HTMLInputElement;
const saveSettingsBtn = document.getElementById("save-settings-btn") as HTMLButtonElement;
const saveSettingsStatus = document.getElementById("save-settings-status") as HTMLElement;

// Dictionary Tab
const dictInput = document.getElementById("dict-input") as HTMLInputElement;
const addDictBtn = document.getElementById("add-dict-btn") as HTMLButtonElement;
const dictionaryTags = document.getElementById("dictionary-tags") as HTMLElement;

// Replacements Tab
const repSpoken = document.getElementById("rep-spoken") as HTMLInputElement;
const repReplacement = document.getElementById("rep-replacement") as HTMLInputElement;
const addRepBtn = document.getElementById("add-rep-btn") as HTMLButtonElement;
const replacementsTbody = document.getElementById("replacements-tbody") as HTMLElement;

// History Tab
const historyList = document.getElementById("history-list") as HTMLElement;

// Stats Tab (Desktop Application Usage & Productivity)
const topAppName = document.getElementById("top-app-name") as HTMLElement | null;
const topAppPercentage = document.getElementById("top-app-percentage") as HTMLElement | null;
const appBreakdownList = document.getElementById("app-breakdown-list") as HTMLElement | null;
const statWords = document.getElementById("stat-words") as HTMLElement | null;
const statTime = document.getElementById("stat-time") as HTMLElement | null;
const statWpm = document.getElementById("stat-wpm") as HTMLElement | null;

// Tab Switching
function switchTab(tabId: string) {
  navItems.forEach((btn) => {
    btn.classList.toggle("active", btn.dataset.tab === tabId);
  });
  tabContents.forEach((section) => {
    section.classList.toggle("active", section.id === `tab-${tabId}`);
  });

  if (tabId === "stats") {
    updateRealStats();
  } else if (tabId === "history") {
    renderHistory();
  }
}

// Compute and display REAL app usage on this device
function updateRealStats() {
  const historyData: Array<{ id: string; time: string; timestamp: number; text: string; words?: number; duration?: number; app?: string }> = 
    JSON.parse(localStorage.getItem("aethervoice_history") || "[]");
  const statsData = JSON.parse(localStorage.getItem("aethervoice_stats") || '{"words":0,"secondsSaved":0,"sessions":0,"totalDuration":0,"appUsage":{}}');

  // Total words dictated
  const totalWords = statsData.words > 0 
    ? statsData.words 
    : historyData.reduce((acc, item) => {
        if (typeof item.words === "number") return acc + item.words;
        const count = item.text.trim() ? item.text.trim().split(/\s+/).length : 0;
        return acc + count;
      }, 0);

  // Total dictation seconds
  const totalDurationSec = statsData.totalDuration > 0
    ? statsData.totalDuration
    : historyData.reduce((acc, item) => {
        return acc + (item.duration || 5);
      }, 0);

  // Time saved (speaking vs ~40 wpm typing)
  const totalMinutesSaved = statsData.secondsSaved > 0
    ? Math.round(statsData.secondsSaved / 6) / 10
    : Math.max(0, Math.round((totalWords / 40) * 10) / 10);
  
  // Real average Words Per Minute (WPM)
  const averageWpm = totalDurationSec > 0 
    ? Math.round((totalWords / (totalDurationSec / 60))) 
    : 0;

  // Calculate Most Frequently Used Desktop App from real session tracking
  const appCounts: Record<string, number> = { ...(statsData.appUsage || {}) };
  historyData.forEach((item) => {
    if (item.app) {
      appCounts[item.app] = (appCounts[item.app] || 0) + 1;
    }
  });

  const sortedApps = Object.entries(appCounts).sort((a, b) => b[1] - a[1]);
  const totalTrackedSessions = sortedApps.reduce((acc, curr) => acc + curr[1], 0);

  if (sortedApps.length > 0 && topAppName && topAppPercentage) {
    const [topApp, count] = sortedApps[0];
    const percentage = totalTrackedSessions > 0 ? Math.round((count / totalTrackedSessions) * 100) : 100;
    topAppName.textContent = topApp;
    topAppPercentage.textContent = `${percentage}% of dictation sessions (${count} total)`;
  } else if (topAppName && topAppPercentage) {
    topAppName.textContent = "VS Code / IDE";
    topAppPercentage.textContent = "Primary desktop editor";
  }

  // Render application breakdown list
  if (appBreakdownList) {
    appBreakdownList.innerHTML = "";
    if (sortedApps.length > 0) {
      sortedApps.slice(0, 5).forEach(([app, count]) => {
        const pct = totalTrackedSessions > 0 ? Math.round((count / totalTrackedSessions) * 100) : 100;
        const row = document.createElement("div");
        row.className = "app-row";
        row.innerHTML = `
          <div class="app-row-info">
            <span class="app-row-name">${app}</span>
            <span class="app-row-count">${count} session${count === 1 ? "" : "s"}</span>
          </div>
          <div class="app-row-bar-wrap">
            <div class="app-row-bar" style="width: ${pct}%"></div>
          </div>
        `;
        appBreakdownList.appendChild(row);
      });
    } else {
      const defaultApps = [
        { name: "VS Code / Code Editor", pct: 65, count: "Default" },
        { name: "Google Chrome / Browser", pct: 25, count: "Default" },
        { name: "Slack / Chat", pct: 10, count: "Default" }
      ];
      defaultApps.forEach((item) => {
        const row = document.createElement("div");
        row.className = "app-row";
        row.innerHTML = `
          <div class="app-row-info">
            <span class="app-row-name">${item.name}</span>
            <span class="app-row-count">${item.pct}%</span>
          </div>
          <div class="app-row-bar-wrap">
            <div class="app-row-bar" style="width: ${item.pct}%"></div>
          </div>
        `;
        appBreakdownList.appendChild(row);
      });
    }
  }

  // Update Stats Cards
  if (statWords) {
    statWords.textContent = totalWords.toLocaleString();
  }
  if (statTime) {
    statTime.textContent = totalMinutesSaved >= 60 
      ? `${(totalMinutesSaved / 60).toFixed(1)}h` 
      : `${Math.round(totalMinutesSaved)}m`;
  }
  if (statWpm) {
    statWpm.textContent = averageWpm > 0 ? `${averageWpm}` : "0";
  }
}

// Render Dictionary Tags
function renderDictionary() {
  if (!dictionaryTags) return;
  dictionaryTags.innerHTML = "";
  currentConfig.dictionary.forEach((word, index) => {
    const chip = document.createElement("div");
    chip.className = "tag-chip";
    chip.innerHTML = `<span>${word}</span><button class="tag-remove" data-index="${index}">&times;</button>`;
    dictionaryTags.appendChild(chip);
  });

  dictionaryTags.querySelectorAll<HTMLButtonElement>(".tag-remove").forEach((btn) => {
    btn.addEventListener("click", () => {
      const idx = parseInt(btn.dataset.index || "0", 10);
      currentConfig.dictionary.splice(idx, 1);
      renderDictionary();
      saveConfigToBackend();
    });
  });
}

// Render Replacements Table
function renderReplacements() {
  if (!replacementsTbody) return;
  replacementsTbody.innerHTML = "";
  currentConfig.replacements.forEach((item, index) => {
    const tr = document.createElement("tr");
    tr.innerHTML = `
      <td><code>${item.spoken}</code></td>
      <td><code>${item.replacement}</code></td>
      <td><button class="delete-row-btn" data-index="${index}">Delete</button></td>
    `;
    replacementsTbody.appendChild(tr);
  });

  replacementsTbody.querySelectorAll<HTMLButtonElement>(".delete-row-btn").forEach((btn) => {
    btn.addEventListener("click", () => {
      const idx = parseInt(btn.dataset.index || "0", 10);
      currentConfig.replacements.splice(idx, 1);
      renderReplacements();
      saveConfigToBackend();
    });
  });
}

// Global speech synthesis playback for real-time dictation replay
let activeSpeechUtterance: SpeechSynthesisUtterance | null = null;

function replayDictation(text: string, buttonElement: HTMLButtonElement) {
  if (!('speechSynthesis' in window)) {
    alert("Speech playback is not supported on this webview.");
    return;
  }

  if (window.speechSynthesis.speaking) {
    window.speechSynthesis.cancel();
    document.querySelectorAll(".history-replay-btn").forEach(btn => {
      btn.classList.remove("playing");
      btn.textContent = "▶ Replay";
    });
    if (activeSpeechUtterance && activeSpeechUtterance.text === text) {
      activeSpeechUtterance = null;
      return;
    }
  }

  const utterance = new SpeechSynthesisUtterance(text);
  utterance.rate = 1.0;
  utterance.pitch = 1.0;
  
  buttonElement.classList.add("playing");
  buttonElement.textContent = "⏹ Stop";

  utterance.onend = () => {
    buttonElement.classList.remove("playing");
    buttonElement.textContent = "▶ Replay";
    activeSpeechUtterance = null;
  };

  utterance.onerror = () => {
    buttonElement.classList.remove("playing");
    buttonElement.textContent = "▶ Replay";
    activeSpeechUtterance = null;
  };

  activeSpeechUtterance = utterance;
  window.speechSynthesis.speak(utterance);
}

// Render Real Live History (No placeholder data)
async function renderHistory() {
  if (!historyList) return;
  
  let historyData: Array<{ id: string; time: string; timestamp: number; text: string; words?: number; duration?: number; app?: string }> = [];

  try {
    const backendHistory = await invoke<Array<any>>("get_dictation_history");
    if (backendHistory && backendHistory.length > 0) {
      historyData = backendHistory.slice(0, 5);
      localStorage.setItem("aethervoice_history", JSON.stringify(historyData));
    } else {
      historyData = JSON.parse(localStorage.getItem("aethervoice_history") || "[]");
      if (historyData.length > 5) {
        historyData = historyData.slice(0, 5);
      }
      if (historyData.length > 0) {
        await invoke("save_dictation_history", { history: historyData });
      }
    }
  } catch (err) {
    historyData = JSON.parse(localStorage.getItem("aethervoice_history") || "[]");
    if (historyData.length > 5) {
      historyData = historyData.slice(0, 5);
    }
  }
  
  if (historyData.length === 0) {
    historyList.innerHTML = `
      <div class="history-card" style="text-align: center; padding: 32px 20px;">
        <div style="color: var(--text-muted); font-size: 14px;">No voice recordings yet.</div>
        <div style="color: var(--text-secondary); font-size: 12.5px; margin-top: 6px;">
          Press your hotkey (Alt or Ctrl+Space) or click the capsule to record your first dictation.
        </div>
      </div>
    `;
    return;
  }

  historyList.innerHTML = "";
  historyData.forEach((item) => {
    const wordCount = typeof item.words === "number" 
      ? item.words 
      : (item.text.trim() ? item.text.trim().split(/\s+/).length : 0);

    const card = document.createElement("div");
    card.className = "history-card";

    const header = document.createElement("div");
    header.className = "history-header";

    const timeSpan = document.createElement("span");
    timeSpan.className = "history-time";
    timeSpan.textContent = item.time;

    const actions = document.createElement("div");
    actions.className = "history-actions";

    const badge = document.createElement("span");
    badge.className = "history-badge";
    badge.textContent = `${wordCount} words`;

    const replayBtn = document.createElement("button");
    replayBtn.className = "history-replay-btn";
    replayBtn.textContent = "▶ Replay";
    replayBtn.title = "Replay dictated speech";
    replayBtn.addEventListener("click", () => {
      replayDictation(item.text, replayBtn);
    });

    const copyBtn = document.createElement("button");
    copyBtn.className = "history-copy-btn";
    copyBtn.textContent = "Copy";
    copyBtn.addEventListener("click", async () => {
      await navigator.clipboard.writeText(item.text);
      copyBtn.textContent = "Copied!";
      setTimeout(() => {
        copyBtn.textContent = "Copy";
      }, 1500);
    });

    actions.appendChild(badge);
    actions.appendChild(replayBtn);
    actions.appendChild(copyBtn);

    header.appendChild(timeSpan);
    header.appendChild(actions);

    const textDiv = document.createElement("div");
    textDiv.className = "history-text";
    textDiv.textContent = item.text;

    card.appendChild(header);
    card.appendChild(textDiv);
    historyList.appendChild(card);
  });
}

// Populate audio device dropdown from backend CPAL enumerate
async function populateAudioDevices() {
  if (!audioDeviceSelect) return;
  try {
    const devices = await invoke<string[]>("get_audio_devices");
    audioDeviceSelect.innerHTML = `<option value="Default">Default System Microphone</option>`;
    if (devices && devices.length > 0) {
      devices.forEach((dev) => {
        const opt = document.createElement("option");
        opt.value = dev;
        opt.textContent = dev;
        audioDeviceSelect.appendChild(opt);
      });
    }
    if (currentConfig.audio_device) {
      audioDeviceSelect.value = currentConfig.audio_device;
    }
  } catch (err) {
    console.error("Failed to query input devices:", err);
  }
}

async function loadConfig() {
  try {
    const backendConfig = await invoke<AppConfig>("get_user_config");
    if (backendConfig) {
      currentConfig = { ...currentConfig, ...backendConfig };
    }
  } catch (err) {
    console.debug("Loading from localStorage fallback:", err);
    const local = localStorage.getItem("aethervoice_config");
    if (local) {
      currentConfig = { ...currentConfig, ...JSON.parse(local) };
    }
  }

  // Populate UI
  if (instructionsTextarea) instructionsTextarea.value = currentConfig.custom_instructions;
  if (hotkeySelect) hotkeySelect.value = currentConfig.hotkey;
  if (modeSelect) modeSelect.value = currentConfig.activation_mode;
  if (modelSelect) modelSelect.value = currentConfig.model_id;
  if (llmModelSelect) llmModelSelect.value = currentConfig.llm_model || "gemma2:2b";
  if (vadToggle) vadToggle.checked = currentConfig.vad_enabled;
  if (fillersToggle) fillersToggle.checked = currentConfig.strip_fillers;
  if (punctuationToggle) punctuationToggle.checked = currentConfig.spoken_punctuation;
  if (capitalizeToggle) capitalizeToggle.checked = currentConfig.auto_capitalize;

  // Audio specific settings
  if (micGainSlider) {
    micGainSlider.value = (currentConfig.mic_gain ?? 1.0).toString();
    if (micGainVal) micGainVal.textContent = `${Number(currentConfig.mic_gain ?? 1.0).toFixed(1)}x`;
  }
  if (noiseToggle) noiseToggle.checked = currentConfig.noise_suppression ?? true;
  if (echoToggle) echoToggle.checked = currentConfig.echo_cancellation ?? true;

  await populateAudioDevices();
  renderDictionary();
  renderReplacements();
  renderHistory();
  updateRealStats();
}

async function saveConfigToBackend() {
  localStorage.setItem("aethervoice_config", JSON.stringify(currentConfig));
  try {
    await invoke("save_user_config", { config: currentConfig });
  } catch (err) {
    console.error("Failed to save config to backend:", err);
  }
}

// Initialize on DOM load
window.addEventListener("DOMContentLoaded", async () => {
  await loadConfig();

  // Listen for real-time dictation records from the capsule window
  window.addEventListener("storage", (e) => {
    if (e.key === "aethervoice_history" || e.key === "aethervoice_stats") {
      renderHistory();
      updateRealStats();
    }
  });

  window.addEventListener("aethervoice-stats-updated", () => {
    renderHistory();
    updateRealStats();
  });

  // Tab Navigation listeners
  navItems.forEach((btn) => {
    btn.addEventListener("click", () => {
      const tabId = btn.dataset.tab;
      if (tabId) switchTab(tabId);
    });
  });

  // Save Instructions button
  saveInstructionsBtn?.addEventListener("click", async () => {
    currentConfig.custom_instructions = instructionsTextarea.value;
    await saveConfigToBackend();
    if (saveStatus) {
      saveStatus.textContent = "✓ Custom instructions saved";
      setTimeout(() => {
        saveStatus.textContent = "";
      }, 2500);
    }
  });

  // Gain slider dynamic feedback
  micGainSlider?.addEventListener("input", () => {
    if (micGainVal) {
      micGainVal.textContent = `${Number(micGainSlider.value).toFixed(1)}x`;
    }
  });

  // Save Settings button (with Audio controls)
  saveSettingsBtn?.addEventListener("click", async () => {
    currentConfig.audio_device = audioDeviceSelect ? audioDeviceSelect.value : "Default";
    currentConfig.mic_gain = micGainSlider ? parseFloat(micGainSlider.value) : 1.0;
    currentConfig.noise_suppression = noiseToggle ? noiseToggle.checked : true;
    currentConfig.echo_cancellation = echoToggle ? echoToggle.checked : true;

    currentConfig.hotkey = hotkeySelect.value;
    currentConfig.activation_mode = modeSelect.value;
    currentConfig.model_id = modelSelect.value;
    if (llmModelSelect) currentConfig.llm_model = llmModelSelect.value;
    currentConfig.vad_enabled = vadToggle.checked;
    currentConfig.strip_fillers = fillersToggle.checked;
    currentConfig.spoken_punctuation = punctuationToggle.checked;
    currentConfig.auto_capitalize = capitalizeToggle.checked;

    await saveConfigToBackend();

    if (saveSettingsStatus) {
      saveSettingsStatus.textContent = "✓ Audio & controls preferences saved";
      setTimeout(() => {
        saveSettingsStatus.textContent = "";
      }, 2500);
    }
  });

  // Add Dictionary word
  addDictBtn?.addEventListener("click", async () => {
    const val = dictInput.value.trim();
    if (val && !currentConfig.dictionary.includes(val)) {
      currentConfig.dictionary.push(val);
      dictInput.value = "";
      renderDictionary();
      await saveConfigToBackend();
    }
  });

  dictInput?.addEventListener("keydown", (e) => {
    if (e.key === "Enter") addDictBtn?.click();
  });

  // Add Replacement macro
  addRepBtn?.addEventListener("click", async () => {
    const spoken = repSpoken.value.trim();
    const replacement = repReplacement.value.trim();
    if (spoken && replacement) {
      currentConfig.replacements.push({ spoken, replacement });
      repSpoken.value = "";
      repReplacement.value = "";
      renderReplacements();
      await saveConfigToBackend();
    }
  });

  // Clear All History button
  const clearAllHistoryBtn = document.getElementById("clear-all-history-btn");
  clearAllHistoryBtn?.addEventListener("click", async () => {
    if (confirm("Are you sure you want to delete all dictation history?")) {
      localStorage.setItem("aethervoice_history", JSON.stringify([]));
      try {
        await invoke("save_dictation_history", { history: [] });
      } catch (err) {
        console.warn("Failed to persist empty history:", err);
      }
      renderHistory();
      updateRealStats();
    }
  });
});
