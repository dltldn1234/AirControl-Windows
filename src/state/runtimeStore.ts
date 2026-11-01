import { invoke, isTauri } from "@tauri-apps/api/core";
import { create } from "zustand";
import type { CameraDevice } from "../domain/camera";
import type { CameraState, HandTrackingState, RuntimeSnapshot } from "../domain/runtimeStatus";
import { BUILT_IN_PROFILES, type UserPreferences } from "../domain/userPreferences";
import { DEFAULT_GESTURE_STUDIO } from "../domain/gestureStudio";

type RuntimeConnection = "preview" | "connected" | "error";

interface RuntimeStore {
  status: RuntimeSnapshot | null;
  preferences: UserPreferences;
  connection: RuntimeConnection;
  error: string | null;
  refresh: () => Promise<void>;
  savePreferences: (preferences: UserPreferences) => Promise<boolean>;
  listCameras: () => Promise<CameraDevice[] | null>;
  startCamera: (deviceId: string) => Promise<boolean>;
  stopCamera: () => Promise<boolean>;
  setCursorControl: (enabled: boolean) => Promise<boolean>;
  setGestureTestMode: (enabled: boolean) => Promise<boolean>;
  setRuntimeStatus: (status: RuntimeSnapshot) => void;
  setCameraState: (camera: CameraState) => void;
  setHandTrackingState: (state: HandTrackingState) => void;
}

const initialPreferences: UserPreferences = {
  cursorSensitivity: 1,
  cursorSmoothing: 0.4,
  pinchSensitivity: 1,
  windowSensitivity: 1,
  scrollSensitivity: 1,
  reverseScrollDirection: false,
  rightClickEnabled: true,
  cameraMirrored: true,
  appSwitchEnabled: false,
  cursorCalibration: { left: 0, top: 0, right: 1, bottom: 1 },
  activeProfileId: "productivity",
  profiles: BUILT_IN_PROFILES,
  autoProfileEnabled: false,
  appProfileRules: [],
  gestureStudio: DEFAULT_GESTURE_STUDIO,
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
  listCameras: async () => {
    if (!isTauri()) return null;
    try {
      const cameras = await invoke<CameraDevice[]>("list_cameras");
      return cameras;
    } catch (error) {
      set({ error: error instanceof Error ? error.message : String(error) });
      return null;
    }
  },
  startCamera: async (deviceId) => {
    if (!isTauri()) return false;
    try {
      await invoke("start_camera", { deviceId });
      set({ error: null });
      return true;
    } catch (error) {
      set({ error: error instanceof Error ? error.message : String(error) });
      return false;
    }
  },
  stopCamera: async () => {
    if (!isTauri()) return false;
    try {
      await invoke("stop_camera");
      set({ error: null });
      return true;
    } catch (error) {
      set({ error: error instanceof Error ? error.message : String(error) });
      return false;
    }
  },
  setCursorControl: async (enabled) => {
    if (!isTauri()) return false;
    try {
      await invoke("set_cursor_control", { enabled });
      set({ error: null });
      return true;
    } catch (error) {
      set({ error: error instanceof Error ? error.message : String(error) });
      return false;
    }
  },
  setGestureTestMode: async (enabled) => {
    if (!isTauri()) return false;
    try {
      await invoke("set_gesture_test_mode", { enabled });
      set({ error: null });
      return true;
    } catch (error) {
      set({ error: error instanceof Error ? error.message : String(error) });
      return false;
    }
  },
  setRuntimeStatus: (status) => set({ status }),
  setCameraState: (camera) => set((state) => ({
    status: state.status ? { ...state.status, camera } : null,
  })),
  setHandTrackingState: (handTracking) => set((state) => ({
    status: state.status ? { ...state.status, handTracking } : null,
  })),
}));
