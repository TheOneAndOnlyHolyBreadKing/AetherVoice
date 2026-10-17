import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { AetherCoreRenderer, COSMETIC_CATALOG } from "./companion_canvas";

interface CompanionStateData {
  enabled: boolean;
  scale: number;
  words_today: number;
  words_total: number;
  daily_streak: number;
  last_dictation_date: string;
  equipped_cosmetics: string[];
  unlocked_cosmetics: string[];
}

let companionState: CompanionStateData = {
  enabled: true,
  scale: 1.0,
  words_today: 0,
  words_total: 0,
  daily_streak: 1,
  last_dictation_date: new Date().toISOString().split("T")[0],
  equipped_cosmetics: ["default_halo"],
  unlocked_cosmetics: ["default_halo"],
};

const appWindow = getCurrentWindow();
let renderer: AetherCoreRenderer;

let afkTimer: number | null = null;
const AFK_TIMEOUT_MS = 3 * 60 * 1000; // 3 minutes

function resetAfkTimer() {
  if (renderer && renderer.state === "afk") {
    renderer.state = "idle";
  }
  if (afkTimer) window.clearTimeout(afkTimer);
  afkTimer = window.setTimeout(() => {
    if (renderer && renderer.state === "idle") {
      renderer.state = "afk";
    }
  }, AFK_TIMEOUT_MS);
}

function calculateLevel(wordsTotal: number): number {
  return Math.max(1, Math.floor(Math.sqrt(wordsTotal / 40)) + 1);
}

async function loadCompanionData() {
  try {
    const saved = await invoke<CompanionStateData>("get_companion_state");
    if (saved) {
      companionState = { ...companionState, ...saved };
      
      // Check date rollover for words_today
      const today = new Date().toISOString().split("T")[0];
      if (companionState.last_dictation_date !== today) {
        companionState.words_today = 0;
        companionState.last_dictation_date = today;
      }

      // Apply equipped cosmetics
      renderer.equippedCosmetics = new Set(companionState.equipped_cosmetics);
      renderer.scale = companionState.scale || 1.0;

      // Update menu badge
      const badge = document.getElementById("menu-lvl-badge");
      if (badge) {
        badge.textContent = `Lv. ${calculateLevel(companionState.words_total)}`;
      }
    }
  } catch (err) {
    console.debug("Loading default companion state:", err);
  }
}

async function saveCompanionData() {
  try {
    await invoke("save_companion_state", { state: companionState });
  } catch (err) {
    console.error("Failed to save companion state:", err);
  }
}

function checkMilestones(newWords: number) {
  companionState.words_today += newWords;
  companionState.words_total += newWords;

  COSMETIC_CATALOG.forEach((item) => {
    if (
      companionState.words_total >= item.unlockedAtWords &&
      !companionState.unlocked_cosmetics.includes(item.id)
    ) {
      companionState.unlocked_cosmetics.push(item.id);
      // Auto-equip freshly unlocked item if slot is empty
      companionState.equipped_cosmetics.push(item.id);
      renderer.equippedCosmetics.add(item.id);
    }
  });

  const badge = document.getElementById("menu-lvl-badge");
  if (badge) {
    badge.textContent = `Lv. ${calculateLevel(companionState.words_total)}`;
  }

  saveCompanionData();
}

window.addEventListener("DOMContentLoaded", async () => {
  const canvas = document.getElementById("companion-canvas") as HTMLCanvasElement;
  const menu = document.getElementById("companion-menu") as HTMLDivElement;

  renderer = new AetherCoreRenderer(canvas);
  renderer.start();

  await loadCompanionData();
  resetAfkTimer();

  // Mouse drag & interaction handling
  let isPointerDown = false;
  let startX = 0;
  let startY = 0;
  let hasDragged = false;

  canvas.addEventListener("mousedown", async (e) => {
    if (e.button === 0) {
      // Left click: Prepare drag or click
      isPointerDown = true;
      startX = e.clientX;
      startY = e.clientY;
      hasDragged = false;
      if (menu) menu.style.display = "none";
    } else if (e.button === 2) {
      // Right click: Open context menu
      e.preventDefault();
      if (menu) {
        menu.style.display = menu.style.display === "block" ? "none" : "block";
      }
    }
  });

  window.addEventListener("mousemove", async (e) => {
    if (isPointerDown && !hasDragged) {
      const dx = Math.abs(e.clientX - startX);
      const dy = Math.abs(e.clientY - startY);
      if (dx > 3 || dy > 3) {
        hasDragged = true;
        try {
          await appWindow.startDragging();
        } catch {
          await invoke("start_dragging");
        }
      }
    }
  });

  window.addEventListener("mouseup", (e) => {
    if (isPointerDown && !hasDragged && e.button === 0) {
      // It was a click / poke on the character!
      renderer.poke();
      resetAfkTimer();
    }
    isPointerDown = false;
    hasDragged = false;
  });

  // Prevent default context menu
  window.addEventListener("contextmenu", (e) => {
    e.preventDefault();
  });

  // Close context menu when clicking outside
  window.addEventListener("click", (e) => {
    if (menu && !menu.contains(e.target as Node) && e.target !== canvas) {
      menu.style.display = "none";
    }
  });

  // Context Menu Actions
  document.getElementById("menu-poke")?.addEventListener("click", () => {
    menu.style.display = "none";
    renderer.poke();
    resetAfkTimer();
  });

  document.getElementById("menu-sleep")?.addEventListener("click", () => {
    menu.style.display = "none";
    if (renderer.state === "afk") {
      renderer.state = "idle";
      resetAfkTimer();
    } else {
      renderer.state = "afk";
    }
  });

  document.getElementById("menu-wardrobe")?.addEventListener("click", async () => {
    menu.style.display = "none";
    try {
      await invoke("open_settings", { tab: "companion" });
    } catch {
      await invoke("open_settings");
    }
  });

  document.getElementById("menu-hide")?.addEventListener("click", async () => {
    menu.style.display = "none";
    companionState.enabled = false;
    await saveCompanionData();
    await appWindow.hide();
  });

  // Tauri Backend Event Listeners
  try {
    await listen<string>("dictation-state", (event) => {
      const st = event.payload;
      resetAfkTimer();

      if (st === "listening") {
        renderer.state = "listening";
      } else if (st === "transcribing") {
        renderer.triggerThrow();
      } else if (st === "idle") {
        if (renderer.state !== "throwing") {
          renderer.state = "idle";
        }
      }
    });

    await listen<number>("audio-amplitude", (event) => {
      renderer.amplitude = Math.max(0, Math.min(1.0, event.payload));
    });

    await listen<{ words: number; text: string }>("dictation-complete", (event) => {
      resetAfkTimer();
      renderer.triggerThrow();
      if (event.payload && typeof event.payload.words === "number") {
        checkMilestones(event.payload.words);
      }
    });

    await listen<CompanionStateData>("companion-settings-updated", (event) => {
      if (event.payload) {
        companionState = { ...companionState, ...event.payload };
        renderer.equippedCosmetics = new Set(companionState.equipped_cosmetics);
        renderer.scale = companionState.scale || 1.0;
      }
    });
  } catch (err) {
    console.debug("Companion running without event bus:", err);
  }
});
