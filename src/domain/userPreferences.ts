import { DEFAULT_GESTURE_STUDIO, type GestureStudioSettings } from "./gestureStudio";

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
  cursorCalibration: { left: number; top: number; right: number; bottom: number };
  activeProfileId: string;
  profiles: ControlProfile[];
  autoProfileEnabled: boolean;
  appProfileRules: AppProfileRule[];
  gestureStudio: GestureStudioSettings;
}

export interface AppProfileRule {
  executable: string;
  profileId: string;
}

export type ProfileSettings = Omit<UserPreferences, "activeProfileId" | "profiles" | "cursorCalibration" | "autoProfileEnabled" | "appProfileRules">;

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
  gestureStudio: DEFAULT_GESTURE_STUDIO,
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
    gestureStudio: preferences.gestureStudio,
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
  if (preferences.activeProfileId !== id) return { ...preferences, profiles, appProfileRules: preferences.appProfileRules.filter((rule) => rule.profileId !== id) };
  const fallback = profiles.find((item) => item.id === "productivity");
  if (!fallback) throw new Error("The Productivity profile is unavailable.");
  return withProfileSettings({ ...preferences, profiles, appProfileRules: preferences.appProfileRules.filter((rule) => rule.profileId !== id) }, fallback);
}

export function restoreControlProfile(preferences: UserPreferences, id: string): UserPreferences {
  const profile = preferences.profiles.find((item) => item.id === id);
  const preset = BUILT_IN_PROFILES.find((item) => item.id === id);
  if (!profile?.builtIn || !preset) throw new Error("Only built-in profiles can be restored.");
  const restored = { ...profile, settings: { ...preset.settings } };
  const next = { ...preferences, profiles: preferences.profiles.map((item) => item.id === id ? restored : item) };
  return preferences.activeProfileId === id ? withProfileSettings(next, restored) : next;
}

export function normalizeExecutableIdentity(value: string): string | null {
  const segments = value.trim().split(/[\\/]/);
  const fileName = (segments[segments.length - 1] ?? "").toLowerCase();
  return fileName.length > 0 && fileName.length <= 260 && /^[^\\/:*?\"<>|\u0000-\u001f]+\.exe$/.test(fileName) ? fileName : null;
}

export function upsertAppProfileRule(
  preferences: UserPreferences,
  executable: string,
  profileId: string,
): UserPreferences {
  const identity = normalizeExecutableIdentity(executable);
  if (!identity) throw new Error("Enter a Windows executable name ending in .exe.");
  if (!preferences.profiles.some((profile) => profile.id === profileId)) {
    throw new Error("Choose an existing control profile.");
  }
  return {
    ...preferences,
    appProfileRules: [
      ...preferences.appProfileRules.filter((rule) => normalizeExecutableIdentity(rule.executable) !== identity),
      { executable: identity, profileId },
    ],
  };
}

export function removeAppProfileRule(preferences: UserPreferences, executable: string): UserPreferences {
  const identity = normalizeExecutableIdentity(executable);
  return identity
    ? { ...preferences, appProfileRules: preferences.appProfileRules.filter((rule) => normalizeExecutableIdentity(rule.executable) !== identity) }
    : preferences;
}
