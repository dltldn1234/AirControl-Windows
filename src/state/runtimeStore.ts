import { invoke, isTauri } from "@tauri-apps/api/core";
import { create } from "zustand";
import type { RuntimeSnapshot } from "../domain/runtimeStatus";
import type { UserPreferences } from "../domain/userPreferences";

type RuntimeConnection = "preview" | "connected" | "error";

interface RuntimeStore {
  status: RuntimeSnapshot | null;
  preferences: UserPreferences;
  connection: RuntimeConnection;
  error: string | null;
  refresh: () => Promise<void>;
  savePreferences: (preferences: UserPreferences) => Promise<boolean>;
}

const initialPreferences: UserPreferences = {
  cursorSensitivity: 1,
  cursorSmoothing: 0.4,
  pinchSensitivity: 1,
  scrollSensitivity: 1,
  reverseScrollDirection: false,
  cameraMirrored: true,
};

export const useRuntimeStore = create<RuntimeStore>((set) => ({
  status: null,
  preferences: initialPreferences,
  connection: "preview",
  error: null,
  refresh: async () => {
    if (!isTauri()) {
      set({ status: null, connection: "preview", error: null });
      return;
    }
    try {
      const [status, preferences] = await Promise.all([
        invoke<RuntimeSnapshot>("get_runtime_status"),
        invoke<UserPreferences>("load_preferences"),
      ]);
      set({ status, preferences, connection: "connected", error: null });
    } catch (error) {
      set({
        status: null,
        connection: "error",
        error: error instanceof Error ? error.message : "Runtime status is unavailable.",
      });
    }
  },
  savePreferences: async (preferences) => {
    if (!isTauri()) return false;
    try {
      const saved = await invoke<UserPreferences>("save_preferences", { preferences });
      set({ preferences: saved, error: null });
      return true;
    } catch (error) {
      set({ error: error instanceof Error ? error.message : "Preferences could not be saved." });
      return false;
    }
  },
}));
