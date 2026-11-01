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
