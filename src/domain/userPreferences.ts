export interface UserPreferences {
  cursorSensitivity: number;
  cursorSmoothing: number;
  pinchSensitivity: number;
  windowSensitivity: number;
  scrollSensitivity: number;
  reverseScrollDirection: boolean;
  rightClickEnabled: boolean;
  cameraMirrored: boolean;
  appSwitchEnabled: boolean;
  activeProfileId: string;
  profiles: ControlProfile[];
}

export type ProfileSettings = Omit<UserPreferences, "activeProfileId" | "profiles">;

export interface ControlProfile {
  id: string;
  name: string;
  builtIn: boolean;
  settings: ProfileSettings;
}

const defaultProfileSettings: ProfileSettings = {
  cursorSensitivity: 1,
  cursorSmoothing: 0.4,
  pinchSensitivity: 1,
  windowSensitivity: 1,
  scrollSensitivity: 1,
  reverseScrollDirection: false,
  rightClickEnabled: true,
  cameraMirrored: true,
  appSwitchEnabled: false,
};

export const BUILT_IN_PROFILES: ControlProfile[] = [
  { id: "productivity", name: "Productivity", builtIn: true, settings: { ...defaultProfileSettings } },
  {
    id: "presentation",
    name: "Presentation",
    builtIn: true,
    settings: { ...defaultProfileSettings, cursorSmoothing: 0.55, appSwitchEnabled: true },
  },
  { id: "custom", name: "Custom", builtIn: true, settings: { ...defaultProfileSettings, cursorSensitivity: 1.2 } },
];

export function isValidProfileName(name: string, profiles: ControlProfile[], exceptId?: string): boolean {
  const normalized = name.trim();
  return normalized.length > 0
    && normalized.length <= 32
    && !profiles.some((profile) => profile.id !== exceptId && profile.name.trim().toLocaleLowerCase() === normalized.toLocaleLowerCase());
}

export function withProfileSettings(preferences: UserPreferences, profile: ControlProfile): UserPreferences {
  return { ...preferences, ...profile.settings, activeProfileId: profile.id };
}

function currentProfileSettings(preferences: UserPreferences): ProfileSettings {
  return {
    cursorSensitivity: preferences.cursorSensitivity,
    cursorSmoothing: preferences.cursorSmoothing,
    pinchSensitivity: preferences.pinchSensitivity,
    windowSensitivity: preferences.windowSensitivity,
    scrollSensitivity: preferences.scrollSensitivity,
    reverseScrollDirection: preferences.reverseScrollDirection,
    rightClickEnabled: preferences.rightClickEnabled,
    cameraMirrored: preferences.cameraMirrored,
    appSwitchEnabled: preferences.appSwitchEnabled,
  };
}

export function createControlProfile(preferences: UserPreferences, name: string, id: string): UserPreferences {
  if (!isValidProfileName(name, preferences.profiles)) throw new Error("Use a unique profile name with 1–32 characters.");
  if (!id.trim() || preferences.profiles.some((profile) => profile.id === id)) throw new Error("A unique profile identifier is required.");
  const profile: ControlProfile = { id, name: name.trim(), builtIn: false, settings: currentProfileSettings(preferences) };
  return withProfileSettings({ ...preferences, profiles: [...preferences.profiles, profile] }, profile);
}

export function renameControlProfile(preferences: UserPreferences, id: string, name: string): UserPreferences {
  const profile = preferences.profiles.find((item) => item.id === id);
  if (!profile || profile.builtIn) throw new Error("Built-in profiles cannot be renamed.");
  if (!isValidProfileName(name, preferences.profiles, id)) throw new Error("Use a unique profile name with 1–32 characters.");
  return { ...preferences, profiles: preferences.profiles.map((item) => item.id === id ? { ...item, name: name.trim() } : item) };
}

export function deleteControlProfile(preferences: UserPreferences, id: string): UserPreferences {
  const profile = preferences.profiles.find((item) => item.id === id);
  if (!profile || profile.builtIn) throw new Error("Built-in profiles cannot be deleted.");
  const profiles = preferences.profiles.filter((item) => item.id !== id);
  if (preferences.activeProfileId !== id) return { ...preferences, profiles };
  const fallback = profiles.find((item) => item.id === "productivity");
  if (!fallback) throw new Error("The Productivity profile is unavailable.");
  return withProfileSettings({ ...preferences, profiles }, fallback);
}

export function restoreControlProfile(preferences: UserPreferences, id: string): UserPreferences {
  const profile = preferences.profiles.find((item) => item.id === id);
  const preset = BUILT_IN_PROFILES.find((item) => item.id === id);
  if (!profile?.builtIn || !preset) throw new Error("Only built-in profiles can be restored.");
  const restored = { ...profile, settings: { ...preset.settings } };
  const next = { ...preferences, profiles: preferences.profiles.map((item) => item.id === id ? restored : item) };
  return preferences.activeProfileId === id ? withProfileSettings(next, restored) : next;
}
