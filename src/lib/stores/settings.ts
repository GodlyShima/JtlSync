import { DEFAULT_CONFIG } from "$lib/definitions/settings";
import type { ToolConfig } from "$lib/types";
import { writable } from "svelte/store";

function createSettingsStore() {
  const { subscribe, set, update } = writable<ToolConfig>(DEFAULT_CONFIG);

  return {
    subscribe,
    init: () => {
      if (typeof window !== "undefined") {
        const stored = localStorage.getItem("neohtop_config");
        if (stored) {
          try {
            const config = JSON.parse(stored);
            set({ ...DEFAULT_CONFIG, ...config });
          } catch (e) {
            console.error("Failed to parse stored config:", e);
            set(DEFAULT_CONFIG);
          }
        }
      }
    },
    updateConfig: (newConfig: Partial<ToolConfig>) => {
      update((config) => {
        const updated = { ...config, ...newConfig };
        if (typeof window !== "undefined") {
          localStorage.setItem("neohtop_config", JSON.stringify(updated));
        }
        return updated;
      });
    },
  };
}

export const settingsStore = createSettingsStore();
