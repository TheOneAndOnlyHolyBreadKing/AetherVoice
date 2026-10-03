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
  vad_enabled: boolean;
  strip_fillers: boolean;
  spoken_punctuation: boolean;
  auto_capitalize: boolean;
  dictionary: string[];
  replacements: ReplacementItem[];
}

const TEMPLATES: Record<string, string> = {
  architect: `# Persona
Act as an expert software architect and prompt engineer. Transform raw spoken input into clear, structured, and precise instructions optimized for AI-driven software development.

# Core Dictation Processing
* **Remove Speech Artifacts:** Instantly strip out filler words ("um", "uh", "like"), conversational pleasantries, stutters, and self-corrections (retain only the final corrected thought).
* **Technical Translation:** Map non-technical or casual phrases to standard developer vocabulary (e.g., convert "place to hold user stuff" to "database schema for user profiles," or "button to send" to "form submission handler").
* **Infer Technical Context:** Explicitly define implied edge cases, type requirements, error handling, and architectural standards based on the spoken intent.`,

  clean: `# Persona
High-fidelity clean speech recognition.

# Core Dictation Processing
* Strip vocal hesitations and filler words ("um", "uh", "you know").
* Retain precise syntax and punctuation.`,

  bullets: `# Persona
Structured task architect and technical note organizer.

# Core Dictation Processing
* Convert spoken thoughts into formatted markdown bullet points.
* Strip fillers and organize ideas by priority.`,

  slack: `# Persona
Fast casual messaging for Slack & Discord.

# Core Dictation Processing
* Use all lowercase in Slack and chat applications.
* Strip conversational pleasantries and trailing noise.`,

  meeting: `# Persona
Executive meeting assistant.

# Core Dictation Processing
* Format transcriptions with clear headings and bulleted action items.
* Group by decisions made, action items, and next steps.`
};

let currentConfig: AppConfig = {
  custom_instructions: TEMPLATES.architect,
  hotkey: "AltRight",
  activation_mode: "push-to-talk",
  model_id: "large-v3-turbo-q5_0",
  vad_enabled: true,
  strip_fillers: true,
  spoken_punctuation: true,
  auto_capitalize: true,
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
const resetInstructionsBtn = document.getElementById("reset-instructions-btn") as HTMLButtonElement;
const saveStatus = document.getElementById("save-status") as HTMLElement;
const presetChips = document.querySelectorAll<HTMLButtonElement>(".preset-chip");
const sandboxInput = document.getElementById("sandbox-input") as HTMLInputElement;
const sandboxRunBtn = document.getElementById("sandbox-run-btn") as HTMLButtonElement;
const sandboxOutput = document.getElementById("sandbox-output") as HTMLElement;

// Settings Tab
const hotkeySelect = document.getElementById("hotkey-select") as HTMLSelectElement;
const modeSelect = document.getElementById("mode-select") as HTMLSelectElement;
const modelSelect = document.getElementById("model-select") as HTMLSelectElement;
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

// Tab Switching
function switchTab(tabId: string) {
  navItems.forEach((btn) => {
    btn.classList.toggle("active", btn.dataset.tab === tabId);
  });
  tabContents.forEach((section) => {
    section.classList.toggle("active", section.id === `tab-${tabId}`);
  });
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

// Render Sample History
function renderHistory() {
  if (!historyList) return;
  const historyData = JSON.parse(localStorage.getItem("aethervoice_history") || "[]");
  if (historyData.length === 0) {
    historyList.innerHTML = `
      <div class="history-card">
        <div class="history-header">
          <span class="history-time">Recent Sample</span>
          <button class="history-copy-btn" onclick="navigator.clipboard.writeText('Database schema for user profiles and a form submission handler.')">Copy</button>
        </div>
        <div class="history-text">Database schema for user profiles and a form submission handler.</div>
      </div>
    `;
    return;
  }

  historyList.innerHTML = historyData
    .map(
      (item: { time: string; text: string }) => `
      <div class="history-card">
        <div class="history-header">
          <span class="history-time">${item.time}</span>
          <button class="history-copy-btn" onclick="navigator.clipboard.writeText('${item.text.replace(/'/g, "\\'")}')">Copy</button>
        </div>
        <div class="history-text">${item.text}</div>
      </div>
    `
    )
    .join("");
}

// Simulate Test Output locally
function simulateRefinement(raw: string): string {
  let text = raw.trim();
  if (!text) return "";

  // 1. Replacements
  currentConfig.replacements.forEach((rep) => {
    if (rep.spoken.trim()) {
      const re = new RegExp(`\\b${rep.spoken.trim()}\\b`, "gi");
      text = text.replace(re, rep.replacement);
    }
  });

  // 2. Technical translation rules
  const instructions = (instructionsTextarea.value || currentConfig.custom_instructions).toLowerCase();
  if (instructions.includes("technical translation") || instructions.includes("software architect")) {
    text = text.replace(/\bplace to hold user stuff\b/gi, "database schema for user profiles");
    text = text.replace(/\bbutton to send\b/gi, "form submission handler");
    text = text.replace(/\bdata base\b/gi, "database");
    text = text.replace(/\bfront end\b/gi, "frontend");
    text = text.replace(/\bback end\b/gi, "backend");
  }

  // 3. Filler stripping
  if (currentConfig.strip_fillers || instructions.includes("remove speech artifacts")) {
    text = text.replace(/\b(um|uh|erm|ah|you know|like so)\b/gi, "");
  }

  // 4. Spoken punctuation
  if (currentConfig.spoken_punctuation) {
    text = text.replace(/\b(new line|next line)\b/gi, "\n");
    text = text.replace(/\bnew paragraph\b/gi, "\n\n");
    text = text.replace(/\bperiod\b/gi, ".");
    text = text.replace(/\bcomma\b/gi, ",");
    text = text.replace(/\bquestion mark\b/gi, "?");
    text = text.replace(/\bexclamation mark\b/gi, "!");
    text = text.replace(/\bcolon\b/gi, ":");
  }

  // Spacing
  text = text.replace(/\s+([.,!?:;])/g, "$1").replace(/[ \t]+/g, " ");

  // 5. User Dictionary casing
  currentConfig.dictionary.forEach((w) => {
    if (w.trim()) {
      const re = new RegExp(`\\b${w.trim()}\\b`, "gi");
      text = text.replace(re, w.trim());
    }
  });

  // 6. Formatting style
  if (instructions.includes("all lowercase") || instructions.includes("lowercase in slack")) {
    return text.trim().toLowerCase();
  }

  if (instructions.includes("bullet") || instructions.includes("list")) {
    const parts = text.split(/[.\n]/).map((s) => s.trim()).filter(Boolean);
    return parts.map((s) => `• ${s.charAt(0).toUpperCase() + s.slice(1)}`).join("\n");
  }

  // Capitalize
  return text.replace(/(^\s*|\.\s*)([a-z])/g, (_, p1, p2) => p1 + p2.toUpperCase()).trim();
}

async function loadConfig() {
  try {
    const backendConfig = await invoke<AppConfig>("get_user_config");
    if (backendConfig) {
      currentConfig = backendConfig;
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
  if (vadToggle) vadToggle.checked = currentConfig.vad_enabled;
  if (fillersToggle) fillersToggle.checked = currentConfig.strip_fillers;
  if (punctuationToggle) punctuationToggle.checked = currentConfig.spoken_punctuation;
  if (capitalizeToggle) capitalizeToggle.checked = currentConfig.auto_capitalize;

  renderDictionary();
  renderReplacements();
  renderHistory();
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

  // Tab Navigation listeners
  navItems.forEach((btn) => {
    btn.addEventListener("click", () => {
      const tabId = btn.dataset.tab;
      if (tabId) switchTab(tabId);
    });
  });

  // Preset Template Chips
  presetChips.forEach((chip) => {
    chip.addEventListener("click", () => {
      presetChips.forEach((c) => c.classList.remove("active"));
      chip.classList.add("active");
      const key = chip.dataset.preset || "architect";
      if (TEMPLATES[key]) {
        instructionsTextarea.value = TEMPLATES[key];
        currentConfig.custom_instructions = TEMPLATES[key];
      }
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

  // Reset to default template
  resetInstructionsBtn?.addEventListener("click", () => {
    instructionsTextarea.value = TEMPLATES.architect;
    currentConfig.custom_instructions = TEMPLATES.architect;
  });

  // Sandbox Live Test button
  sandboxRunBtn?.addEventListener("click", () => {
    const inputVal = sandboxInput.value;
    const output = simulateRefinement(inputVal);
    if (sandboxOutput) sandboxOutput.textContent = output;
  });

  sandboxInput?.addEventListener("keydown", (e) => {
    if (e.key === "Enter") {
      sandboxRunBtn?.click();
    }
  });

  // Save Settings button
  saveSettingsBtn?.addEventListener("click", async () => {
    currentConfig.hotkey = hotkeySelect.value;
    currentConfig.activation_mode = modeSelect.value;
    currentConfig.model_id = modelSelect.value;
    currentConfig.vad_enabled = vadToggle.checked;
    currentConfig.strip_fillers = fillersToggle.checked;
    currentConfig.spoken_punctuation = punctuationToggle.checked;
    currentConfig.auto_capitalize = capitalizeToggle.checked;
    await saveConfigToBackend();

    if (saveSettingsStatus) {
      saveSettingsStatus.textContent = "✓ Preferences saved";
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
});
