import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

interface ReplacementItem {
  spoken: string;
  replacement: string;
}

interface ModelCatalogItem {
  id: string;
  name: string;
  size: string;
  parameters: string;
  description: string;
  category: string;
  accuracy: number; // 0 to 100 percentage
  speed: number;    // 0 to 100 percentage
  recommended?: boolean;
  language?: string;
}

const CATALOG_MODELS: ModelCatalogItem[] = [
  {
    id: "gemma2:2b",
    name: "Google Gemma 2 2B",
    size: "1.6 GB",
    parameters: "2 Billion",
    description: "Exceptional speed and precision. Optimized by Google DeepMind for fast reasoning, sentence polishing, and structured dictation formatting.",
    category: "Balanced & Fast",
    accuracy: 94,
    speed: 97,
    recommended: true,
    language: "Multilingual"
  },
  {
    id: "llama3.2:1b",
    name: "Meta Llama 3.2 1B",
    size: "1.3 GB",
    parameters: "1 Billion",
    description: "Ultra-lightweight edge model. Boots instantly, minimal GPU/CPU RAM consumption. Perfect for low-latency verbatim restructuring.",
    category: "Ultra-Lightweight",
    accuracy: 82,
    speed: 99,
    language: "Multilingual"
  },
  {
    id: "llama3.2:3b",
    name: "Meta Llama 3.2 3B",
    size: "2.0 GB",
    parameters: "3 Billion",
    description: "State-of-the-art compact reasoning. Excellent multi-sentence context retention and deep understanding of spoken nuances and instructions.",
    category: "Balanced",
    accuracy: 92,
    speed: 91,
    language: "Multilingual"
  },
  {
    id: "qwen2.5:0.5b",
    name: "Qwen 2.5 0.5B",
    size: "398 MB",
    parameters: "0.5 Billion",
    description: "Featherweight sub-billion model. Under 400 MB download size. Blazing fast punctuation and sentence casing on any hardware.",
    category: "Featherweight",
    accuracy: 78,
    speed: 100,
    language: "Multilingual"
  },
  {
    id: "qwen2.5:1.5b",
    name: "Qwen 2.5 1.5B",
    size: "986 MB",
    parameters: "1.5 Billion",
    description: "Tailored for software engineering and technical writing. Excels at preserving programming syntax, variable names, and code snippets.",
    category: "Technical & Code",
    accuracy: 91,
    speed: 95,
    language: "Multilingual"
  },
  {
    id: "qwen2.5:3b",
    name: "Qwen 2.5 3B",
    size: "1.9 GB",
    parameters: "3 Billion",
    description: "Advanced coding & reasoning engine. Converts rambling technical thoughts into clean bullet points, API designs, and structured lists.",
    category: "Technical & Code",
    accuracy: 95,
    speed: 89,
    language: "Multilingual"
  },
  {
    id: "deepseek-r1:1.5b",
    name: "DeepSeek R1 1.5B",
    size: "1.1 GB",
    parameters: "1.5 Billion",
    description: "Distilled reasoning model with chain-of-thought comprehension. Formats intricate complex instructions with extraordinary logical consistency.",
    category: "Reasoning & Logic",
    accuracy: 93,
    speed: 93,
    language: "Multilingual"
  },
  {
    id: "mistral:7b",
    name: "Mistral 7B",
    size: "4.1 GB",
    parameters: "7 Billion",
    description: "Heavyweight flagship reasoning model with profound writing fluency, deep contextual comprehension, and advanced grammar restructuring.",
    category: "High Accuracy",
    accuracy: 98,
    speed: 76,
    language: "Multilingual"
  }
];

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
  deep_context?: boolean;
  hands_free_hotkey?: string;
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
  ],
  deep_context: true,
  hands_free_hotkey: "F8"
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
const handsFreeHotkeySelect = document.getElementById("hands-free-hotkey-select") as HTMLSelectElement | null;
const modeSelect = document.getElementById("mode-select") as HTMLSelectElement;
const modelSelect = document.getElementById("model-select") as HTMLSelectElement;
const llmModelSelect = document.getElementById("llm-model-select") as HTMLSelectElement | null;
const deepContextToggle = document.getElementById("deep-context-toggle") as HTMLInputElement | null;
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

// Models Tab & Modals
const modelsList = document.getElementById("models-list") as HTMLElement | null;
const refreshModelsBtn = document.getElementById("refresh-models-btn") as HTMLButtonElement | null;

// Modal 1: Confirm Delete
const modalConfirmDelete = document.getElementById("modal-confirm-delete") as HTMLElement | null;
const modalDeleteDesc = document.getElementById("modal-delete-desc") as HTMLElement | null;
const modalDeleteCancelBtn = document.getElementById("modal-delete-cancel-btn") as HTMLButtonElement | null;
const modalDeleteConfirmBtn = document.getElementById("modal-delete-confirm-btn") as HTMLButtonElement | null;

// Modal 2: Deleting Progress
const modalDeletingProcess = document.getElementById("modal-deleting-process") as HTMLElement | null;
const modalDeletingTitle = document.getElementById("modal-deleting-title") as HTMLElement | null;
const modalDeletingSubtitle = document.getElementById("modal-deleting-subtitle") as HTMLElement | null;
const modalDeletingBar = document.getElementById("modal-deleting-bar") as HTMLElement | null;
const modalDeletingStatusText = document.getElementById("modal-deleting-status-text") as HTMLElement | null;
const modalDeletingPercentText = document.getElementById("modal-deleting-percent-text") as HTMLElement | null;
const modalDeletingCancelBtn = document.getElementById("modal-deleting-cancel-btn") as HTMLButtonElement | null;

// Modal 3: Download Progress
const modalDownloadProcess = document.getElementById("modal-download-process") as HTMLElement | null;
const modalDownloadTitle = document.getElementById("modal-download-title") as HTMLElement | null;
const modalDownloadSubtitle = document.getElementById("modal-download-subtitle") as HTMLElement | null;
const modalDownloadBar = document.getElementById("modal-download-bar") as HTMLElement | null;
const modalDownloadStatusText = document.getElementById("modal-download-status-text") as HTMLElement | null;
const modalDownloadPercentText = document.getElementById("modal-download-percent-text") as HTMLElement | null;
const modalDownloadCancelBtn = document.getElementById("modal-download-cancel-btn") as HTMLButtonElement | null;

// Track active downloading & deleting models
const downloadingModels = new Map<string, { status: string; completed?: number; total?: number }>();
let installedModelIds: string[] = [];
let pendingDeleteModel: ModelCatalogItem | null = null;
let activeDeleteCancelled = false;
let activeDeleteInterval: number | null = null;
let activeDownloadModelId: string | null = null;
let activeDownloadCancelled = false;

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
  } else if (tabId === "models") {
    loadAndRenderModels();
  }
}

// Check installed Ollama models and render the Models Tab & Dropdown
async function fetchInstalledModels(): Promise<string[]> {
  try {
    const models = await invoke<string[]>("get_installed_llm_models");
    installedModelIds = models || [];
    return installedModelIds;
  } catch (err) {
    console.debug("Failed to query Ollama models:", err);
    installedModelIds = [];
    return [];
  }
}

// Dynamically populate the "Local Reasoning & Instructions Model" dropdown in Settings
// ONLY includes models currently installed on the machine (+ Disabled)
function syncLLMModelDropdown() {
  if (!llmModelSelect) return;

  const previousValue = currentConfig.llm_model || llmModelSelect.value || "gemma2:2b";
  llmModelSelect.innerHTML = "";

  // Filter available models that are installed on disk
  const matchedInstalled: { id: string; name: string }[] = [];

  // Match known catalog items
  CATALOG_MODELS.forEach((cat) => {
    const isInstalled = installedModelIds.some((installedName) => {
      const lower = installedName.toLowerCase();
      const catLower = cat.id.toLowerCase();
      return lower === catLower || lower.startsWith(`${catLower}:`) || lower.replace(":latest", "") === catLower;
    });

    if (isInstalled) {
      matchedInstalled.push({ id: cat.id, name: `${cat.name} (${cat.size})` });
    }
  });

  // Also include any other custom models the user has in Ollama
  installedModelIds.forEach((installedName) => {
    const alreadyMatched = matchedInstalled.some(
      (m) => m.id.toLowerCase() === installedName.toLowerCase() || installedName.toLowerCase().startsWith(m.id.toLowerCase())
    );
    if (!alreadyMatched && !installedName.includes("whisper")) {
      matchedInstalled.push({ id: installedName, name: `${installedName} (Local Model)` });
    }
  });

  if (matchedInstalled.length > 0) {
    matchedInstalled.forEach((m) => {
      const opt = document.createElement("option");
      opt.value = m.id;
      opt.textContent = m.name;
      llmModelSelect.appendChild(opt);
    });
  }

  // Always include the Disabled option
  const disabledOpt = document.createElement("option");
  disabledOpt.value = "none";
  disabledOpt.textContent = "Disabled (Verbatim Dictation Only)";
  llmModelSelect.appendChild(disabledOpt);

  // Preserve previous selection if still available, or pick the first available installed model
  const options = Array.from(llmModelSelect.options).map((o) => o.value);
  if (options.includes(previousValue)) {
    llmModelSelect.value = previousValue;
  } else if (matchedInstalled.length > 0) {
    llmModelSelect.value = matchedInstalled[0].id;
    currentConfig.llm_model = matchedInstalled[0].id;
  } else {
    llmModelSelect.value = "none";
    currentConfig.llm_model = "none";
  }
}

// Render the Models Catalog List in the Models Tab
async function loadAndRenderModels() {
  if (!modelsList) return;

  await fetchInstalledModels();
  syncLLMModelDropdown();

  modelsList.innerHTML = "";

  CATALOG_MODELS.forEach((model) => {
    const isInstalled = installedModelIds.some((installedName) => {
      const lower = installedName.toLowerCase();
      const catLower = model.id.toLowerCase();
      return lower === catLower || lower.startsWith(`${catLower}:`) || lower.replace(":latest", "") === catLower;
    });

    const isDownloading = downloadingModels.has(model.id);
    const downloadInfo = downloadingModels.get(model.id);

    const card = document.createElement("div");
    card.className = "model-catalog-card";

    let badgeClass = "available";
    let badgeText = "Available to Download";
    if (isDownloading) {
      badgeClass = "downloading";
      badgeText = "Downloading...";
    } else if (isInstalled) {
      badgeClass = "installed";
      badgeText = "Installed & Active";
    }

    let progressHtml = "";
    if (isDownloading && downloadInfo) {
      let pct = 0;
      if (downloadInfo.total && downloadInfo.total > 0 && downloadInfo.completed) {
        pct = Math.round((downloadInfo.completed / downloadInfo.total) * 100);
      }
      progressHtml = `
        <div style="font-size: 11.5px; color: var(--cyan-aether); margin-top: 4px;">
          ${downloadInfo.status || "Downloading chunks..."} ${pct > 0 ? `(${pct}%)` : ""}
        </div>
        <div class="model-progress-bar-wrap">
          <div class="model-progress-bar" style="width: ${Math.max(5, pct)}%"></div>
        </div>
      `;
    }

    card.innerHTML = `
      <div class="model-catalog-info">
        <div class="model-title-row">
          <span class="model-catalog-name">${model.name}</span>
          <span class="model-status-badge ${badgeClass}">${badgeText}</span>
          ${model.recommended ? '<span style="font-size: 11px; background: rgba(37,99,235,0.2); color: #60a5fa; border: 1px solid rgba(37,99,235,0.3); padding: 1px 7px; border-radius: 10px; font-weight: 600;">Recommended</span>' : ''}
        </div>
        <div class="model-catalog-desc">${model.description}</div>
        
        <div class="model-bottom-row">
          <div class="model-meta-item">
            <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <circle cx="12" cy="12" r="10"></circle>
              <line x1="2" y1="12" x2="22" y2="12"></line>
              <path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"></path>
            </svg>
            <span>${model.language || "Multilingual"}</span>
          </div>
          <div class="model-meta-item">
            <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"></path>
            </svg>
            <span>${model.parameters}</span>
          </div>
          <div class="model-meta-item">
            <span>🏷 ${model.category}</span>
          </div>
        </div>

        ${progressHtml}
      </div>

      <div class="model-side-metrics">
        <div class="model-meter-container">
          <div class="model-meter-row" title="${model.accuracy}% instruction & formatting fidelity">
            <span class="model-meter-label">accuracy</span>
            <div class="model-meter-bar-track">
              <div class="model-meter-bar-fill" style="width: ${model.accuracy}%"></div>
            </div>
            <span class="model-meter-val">${model.accuracy}%</span>
          </div>
          <div class="model-meter-row" title="${model.speed}% throughput efficiency">
            <span class="model-meter-label">speed</span>
            <div class="model-meter-bar-track">
              <div class="model-meter-bar-fill" style="width: ${model.speed}%"></div>
            </div>
            <span class="model-meter-val">${model.speed}%</span>
          </div>
        </div>

        <div class="model-action-footer">
          <div class="model-size-badge">
            <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <rect x="2" y="2" width="20" height="8" rx="2" ry="2"></rect>
              <rect x="2" y="14" width="20" height="8" rx="2" ry="2"></rect>
              <line x1="6" y1="6" x2="6.01" y2="6"></line>
              <line x1="6" y1="18" x2="6.01" y2="18"></line>
            </svg>
            <span>${model.size}</span>
          </div>

          <div class="model-actions-wrap">
            ${
              isDownloading
                ? `<button class="model-btn-download" disabled>
                     <span>⏳ Downloading...</span>
                   </button>`
                : isInstalled
                ? `<button class="model-btn-delete" data-model="${model.id}">
                     <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                       <path d="M3 6h18"></path>
                       <path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6"></path>
                       <path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2"></path>
                     </svg>
                     <span>Delete</span>
                   </button>`
                : `<button class="model-btn-download" data-model="${model.id}">
                     <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                       <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path>
                       <polyline points="7 10 12 15 17 10"></polyline>
                       <line x1="12" y1="15" x2="12" y2="3"></line>
                     </svg>
                     <span>Download</span>
                   </button>`
            }
          </div>
        </div>
      </div>
    `;

    // Download action
    const dlBtn = card.querySelector<HTMLButtonElement>(".model-btn-download");
    if (dlBtn && !isDownloading && !isInstalled) {
      dlBtn.addEventListener("click", () => {
        openDownloadModal(model);
      });
    }

    // Delete action
    const delBtn = card.querySelector<HTMLButtonElement>(".model-btn-delete");
    if (delBtn) {
      delBtn.addEventListener("click", () => {
        openDeleteConfirmModal(model);
      });
    }

    modelsList.appendChild(card);
  });
}

// --------------------------------------------------------------------------
// MODAL WORKFLOW 1: Confirm Delete Warning Modal
// --------------------------------------------------------------------------
function openDeleteConfirmModal(model: ModelCatalogItem) {
  pendingDeleteModel = model;
  if (modalDeleteDesc) {
    modalDeleteDesc.textContent = `Are you sure you want to remove "${model.name}" from your machine? This will permanently delete its local weights and free up ${model.size} disk space.`;
  }
  if (modalConfirmDelete) {
    modalConfirmDelete.style.display = "flex";
  }
}

function closeDeleteConfirmModal() {
  pendingDeleteModel = null;
  if (modalConfirmDelete) {
    modalConfirmDelete.style.display = "none";
  }
}

// --------------------------------------------------------------------------
// MODAL WORKFLOW 2: Deletion Progress Modal (Cannot exit out midway, can cancel)
// --------------------------------------------------------------------------
async function startDeletionProcess(model: ModelCatalogItem) {
  closeDeleteConfirmModal();

  activeDeleteCancelled = false;
  if (modalDeletingTitle) modalDeletingTitle.textContent = `Deleting ${model.name}...`;
  if (modalDeletingSubtitle) modalDeletingSubtitle.textContent = `Safely unlinking model layers and freeing up ${model.size}...`;
  if (modalDeletingBar) modalDeletingBar.style.width = "0%";
  if (modalDeletingPercentText) modalDeletingPercentText.textContent = "0%";
  if (modalDeletingStatusText) modalDeletingStatusText.textContent = "Initializing deletion...";
  if (modalDeletingCancelBtn) {
    modalDeletingCancelBtn.disabled = false;
    modalDeletingCancelBtn.textContent = "Cancel";
  }
  if (modalDeletingProcess) {
    modalDeletingProcess.style.display = "flex";
  }

  let progress = 0;
  activeDeleteInterval = window.setInterval(async () => {
    if (activeDeleteCancelled) {
      if (activeDeleteInterval) clearInterval(activeDeleteInterval);
      return;
    }

    if (progress < 85) {
      progress += 10;
      if (modalDeletingBar) modalDeletingBar.style.width = `${progress}%`;
      if (modalDeletingPercentText) modalDeletingPercentText.textContent = `${progress}%`;
      if (modalDeletingStatusText) {
        if (progress < 30) modalDeletingStatusText.textContent = "Locating model layers in local Ollama storage...";
        else if (progress < 60) modalDeletingStatusText.textContent = "Unregistering manifest and unlinking weights...";
        else modalDeletingStatusText.textContent = "Reclaiming disk blocks...";
      }
    }
  }, 140);

  try {
    await invoke("delete_llm_model", { model: model.id });

    if (activeDeleteCancelled) {
      if (activeDeleteInterval) clearInterval(activeDeleteInterval);
      if (modalDeletingProcess) modalDeletingProcess.style.display = "none";
      await loadAndRenderModels();
      return;
    }

    // Finish to 100%
    if (activeDeleteInterval) clearInterval(activeDeleteInterval);
    if (modalDeletingBar) modalDeletingBar.style.width = "100%";
    if (modalDeletingPercentText) modalDeletingPercentText.textContent = "100%";
    if (modalDeletingStatusText) modalDeletingStatusText.textContent = "Model successfully deleted.";
    if (modalDeletingCancelBtn) modalDeletingCancelBtn.disabled = true;

    if (currentConfig.llm_model === model.id) {
      currentConfig.llm_model = "none";
      await saveConfigToBackend();
    }

    setTimeout(async () => {
      if (modalDeletingProcess) modalDeletingProcess.style.display = "none";
      await loadAndRenderModels();
    }, 700);
  } catch (err) {
    if (activeDeleteInterval) clearInterval(activeDeleteInterval);
    if (!activeDeleteCancelled) {
      alert(`Deletion failed for ${model.name}: ${err}`);
    }
    if (modalDeletingProcess) modalDeletingProcess.style.display = "none";
    await loadAndRenderModels();
  }
}

function cancelDeletionProcess() {
  activeDeleteCancelled = true;
  if (activeDeleteInterval) clearInterval(activeDeleteInterval);
  if (modalDeletingStatusText) modalDeletingStatusText.textContent = "Cancelling deletion process...";
  if (modalDeletingCancelBtn) {
    modalDeletingCancelBtn.disabled = true;
    modalDeletingCancelBtn.textContent = "Cancelling...";
  }
  setTimeout(() => {
    if (modalDeletingProcess) modalDeletingProcess.style.display = "none";
    loadAndRenderModels();
  }, 400);
}

// --------------------------------------------------------------------------
// MODAL WORKFLOW 3: Download Progress Modal (Cannot exit out midway, can cancel)
// --------------------------------------------------------------------------
async function openDownloadModal(model: ModelCatalogItem) {
  activeDownloadModelId = model.id;
  activeDownloadCancelled = false;

  downloadingModels.set(model.id, { status: "Starting download..." });
  loadAndRenderModels();

  if (modalDownloadTitle) modalDownloadTitle.textContent = `Downloading ${model.name}...`;
  if (modalDownloadSubtitle) modalDownloadSubtitle.textContent = `Pulling model layers directly to disk (${model.size})`;
  if (modalDownloadBar) modalDownloadBar.style.width = "0%";
  if (modalDownloadPercentText) modalDownloadPercentText.textContent = "0%";
  if (modalDownloadStatusText) modalDownloadStatusText.textContent = "Contacting local Ollama service...";
  if (modalDownloadCancelBtn) {
    modalDownloadCancelBtn.disabled = false;
    modalDownloadCancelBtn.textContent = "Cancel Download";
  }
  if (modalDownloadProcess) {
    modalDownloadProcess.style.display = "flex";
  }

  try {
    await invoke("pull_llm_model", { model: model.id });

    if (activeDownloadCancelled) {
      if (modalDownloadProcess) modalDownloadProcess.style.display = "none";
      downloadingModels.delete(model.id);
      await loadAndRenderModels();
      return;
    }

    if (modalDownloadBar) modalDownloadBar.style.width = "100%";
    if (modalDownloadPercentText) modalDownloadPercentText.textContent = "100%";
    if (modalDownloadStatusText) modalDownloadStatusText.textContent = "Download complete and verified!";
    if (modalDownloadCancelBtn) modalDownloadCancelBtn.disabled = true;

    downloadingModels.delete(model.id);
    await loadAndRenderModels();
    await saveConfigToBackend();

    setTimeout(() => {
      if (modalDownloadProcess) modalDownloadProcess.style.display = "none";
      activeDownloadModelId = null;
    }, 700);
  } catch (err) {
    if (!activeDownloadCancelled) {
      alert(`Download failed for ${model.name}: ${err}`);
    }
    downloadingModels.delete(model.id);
    if (modalDownloadProcess) modalDownloadProcess.style.display = "none";
    activeDownloadModelId = null;
    await loadAndRenderModels();
  }
}

async function cancelDownloadProcess() {
  activeDownloadCancelled = true;
  const currentModelId = activeDownloadModelId;

  if (modalDownloadStatusText) modalDownloadStatusText.textContent = "Stopping and cancelling download...";
  if (modalDownloadCancelBtn) {
    modalDownloadCancelBtn.disabled = true;
    modalDownloadCancelBtn.textContent = "Cancelling...";
  }

  if (currentModelId) {
    downloadingModels.delete(currentModelId);
    // If Ollama already created a partial model, clean it up
    try {
      await invoke("delete_llm_model", { model: currentModelId });
    } catch (_) {
      // Ignore cleanup error if model wasn't registered yet
    }
  }

  setTimeout(() => {
    if (modalDownloadProcess) modalDownloadProcess.style.display = "none";
    activeDownloadModelId = null;
    loadAndRenderModels();
  }, 400);
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
  // Cycles dynamically through all tracked apps, displaying precisely the top 5 most frequently used
  const appCounts: Record<string, number> = { ...(statsData.appUsage || {}) };
  historyData.forEach((item) => {
    if (item.app) {
      appCounts[item.app] = (appCounts[item.app] || 0) + 1;
    }
  });

  const sortedApps = Object.entries(appCounts).sort((a, b) => b[1] - a[1]);
  // Strictly display only the first 5 most frequently used apps
  const topFiveApps = sortedApps.slice(0, 5);
  const totalTrackedSessions = topFiveApps.reduce((acc, curr) => acc + curr[1], 0);

  if (topFiveApps.length > 0 && topAppName && topAppPercentage) {
    const [topApp, count] = topFiveApps[0];
    const percentage = totalTrackedSessions > 0 ? Math.round((count / totalTrackedSessions) * 100) : 100;
    topAppName.textContent = topApp;
    topAppPercentage.textContent = `${percentage}% of dictation sessions (${count} total)`;
  } else if (topAppName && topAppPercentage) {
    topAppName.textContent = "VS Code / IDE";
    topAppPercentage.textContent = "Primary desktop editor";
  }

  // Render application breakdown list - exactly top 5 most used apps
  if (appBreakdownList) {
    appBreakdownList.innerHTML = "";
    if (topFiveApps.length > 0) {
      topFiveApps.forEach(([app, count]) => {
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

  await fetchInstalledModels();
  syncLLMModelDropdown();

  // Populate UI
  if (instructionsTextarea) instructionsTextarea.value = currentConfig.custom_instructions;
  if (hotkeySelect) hotkeySelect.value = currentConfig.hotkey;
  if (handsFreeHotkeySelect) handsFreeHotkeySelect.value = currentConfig.hands_free_hotkey || "F8";
  if (modeSelect) modeSelect.value = currentConfig.activation_mode;
  if (modelSelect) modelSelect.value = currentConfig.model_id;
  if (llmModelSelect) {
    if (currentConfig.llm_model) {
      llmModelSelect.value = currentConfig.llm_model;
    }
  }
  if (deepContextToggle) deepContextToggle.checked = currentConfig.deep_context ?? true;
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
  loadAndRenderModels();
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

  // Listen for backend model pull progress events
  try {
    await listen<{ status?: string; completed?: number; total?: number }>("model-pull-progress", (event) => {
      const payload = event.payload;
      for (const [modelId, info] of downloadingModels.entries()) {
        if (payload.status) info.status = payload.status;
        if (payload.completed) info.completed = payload.completed;
        if (payload.total) info.total = payload.total;
        downloadingModels.set(modelId, info);

        // Update modal progress bar & labels in real-time
        if (activeDownloadModelId === modelId) {
          if (payload.status && modalDownloadStatusText) {
            modalDownloadStatusText.textContent = payload.status;
          }
          if (payload.completed && payload.total && payload.total > 0) {
            const pct = Math.min(100, Math.round((payload.completed / payload.total) * 100));
            if (modalDownloadBar) modalDownloadBar.style.width = `${pct}%`;
            if (modalDownloadPercentText) modalDownloadPercentText.textContent = `${pct}%`;
          }
        }
      }
      loadAndRenderModels();
    });

    await listen<{ model: string }>("model-pull-completed", async (event) => {
      downloadingModels.delete(event.payload.model);
      if (activeDownloadModelId === event.payload.model) {
        if (modalDownloadBar) modalDownloadBar.style.width = "100%";
        if (modalDownloadPercentText) modalDownloadPercentText.textContent = "100%";
        if (modalDownloadStatusText) modalDownloadStatusText.textContent = "Download complete and verified!";
        setTimeout(() => {
          if (modalDownloadProcess) modalDownloadProcess.style.display = "none";
          activeDownloadModelId = null;
        }, 700);
      }
      await loadAndRenderModels();
      saveConfigToBackend();
    });
  } catch (err) {
    console.debug("Model pull listener error:", err);
  }

  // Modal 1: Confirm Delete Button Listeners
  modalDeleteCancelBtn?.addEventListener("click", () => {
    closeDeleteConfirmModal();
  });

  modalDeleteConfirmBtn?.addEventListener("click", () => {
    if (pendingDeleteModel) {
      startDeletionProcess(pendingDeleteModel);
    }
  });

  // Modal 2: Deletion Progress Cancel Button Listener
  modalDeletingCancelBtn?.addEventListener("click", () => {
    cancelDeletionProcess();
  });

  // Modal 3: Download Progress Cancel Button Listener
  modalDownloadCancelBtn?.addEventListener("click", () => {
    cancelDownloadProcess();
  });

  // Refresh Models button
  refreshModelsBtn?.addEventListener("click", async () => {
    refreshModelsBtn.disabled = true;
    refreshModelsBtn.textContent = "Checking...";
    await loadAndRenderModels();
    setTimeout(() => {
      if (refreshModelsBtn) {
        refreshModelsBtn.disabled = false;
        refreshModelsBtn.textContent = "↻ Refresh Local Status";
      }
    }, 600);
  });

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
    if (handsFreeHotkeySelect) currentConfig.hands_free_hotkey = handsFreeHotkeySelect.value;
    currentConfig.activation_mode = modeSelect.value;
    currentConfig.model_id = modelSelect.value;
    if (llmModelSelect) currentConfig.llm_model = llmModelSelect.value;
    if (deepContextToggle) currentConfig.deep_context = deepContextToggle.checked;
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
