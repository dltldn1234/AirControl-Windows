import type { ControlProfile, UserPreferences } from "./userPreferences";

export type GestureId = "indexPinch" | "middlePinch" | "twoFingerScroll" | "threeFingerSwipe";
export type GestureAction = "clickAndDrag" | "click" | "drag" | "rightClick" | "scroll" | "switchApplication" | "disabled";

export interface GestureBinding {
  gesture: GestureId;
  action: GestureAction;
  sensitivity: number;
  cooldownMs: number;
}

export interface GestureStudioSettings {
  bindings: GestureBinding[];
}

export const DEFAULT_GESTURE_STUDIO: GestureStudioSettings = {
  bindings: [
    { gesture: "indexPinch", action: "clickAndDrag", sensitivity: 1, cooldownMs: 350 },
    { gesture: "middlePinch", action: "rightClick", sensitivity: 1, cooldownMs: 450 },
    { gesture: "twoFingerScroll", action: "scroll", sensitivity: 1, cooldownMs: 250 },
    { gesture: "threeFingerSwipe", action: "switchApplication", sensitivity: 1, cooldownMs: 900 },
  ],
};

const allowedActions: Record<GestureId, GestureAction[]> = {
  indexPinch: ["clickAndDrag", "click", "drag", "disabled"],
  middlePinch: ["rightClick", "disabled"],
  twoFingerScroll: ["scroll", "disabled"],
  threeFingerSwipe: ["switchApplication", "disabled"],
};

export function gestureConflicts(settings: GestureStudioSettings): string[] {
  const seenGestures = new Set<GestureId>();
  const seenActions = new Set<GestureAction>();
  const conflicts: string[] = [];
  for (const binding of settings.bindings) {
    if (seenGestures.has(binding.gesture)) conflicts.push(`${binding.gesture} is assigned more than once`);
    seenGestures.add(binding.gesture);
    if (!allowedActions[binding.gesture]?.includes(binding.action)) {
      conflicts.push(`${binding.action} is not supported by ${binding.gesture}`);
    }
    if (binding.action !== "disabled" && seenActions.has(binding.action)) {
      conflicts.push(`${binding.action} is assigned to multiple gestures`);
    }
    if (binding.action !== "disabled") seenActions.add(binding.action);
    if (!Number.isFinite(binding.sensitivity) || binding.sensitivity < 0.6 || binding.sensitivity > 1.4) {
      conflicts.push(`${binding.gesture} sensitivity is outside the supported range`);
    }
    if (!Number.isInteger(binding.cooldownMs) || binding.cooldownMs < 100 || binding.cooldownMs > 2_000) {
      conflicts.push(`${binding.gesture} cooldown is outside the supported range`);
    }
  }
  for (const gesture of Object.keys(allowedActions) as GestureId[]) {
    if (!seenGestures.has(gesture)) conflicts.push(`${gesture} has no action assigned`);
  }
  return conflicts;
}

export function updateProfileGestureSettings(
  preferences: UserPreferences,
  gestureStudio: GestureStudioSettings,
): UserPreferences {
  const profiles = preferences.profiles.map((profile) => profile.id === preferences.activeProfileId
    ? { ...profile, settings: { ...profile.settings, gestureStudio } }
    : profile);
  return { ...preferences, gestureStudio, profiles };
}

export function updateGestureBinding(
  settings: GestureStudioSettings,
  gesture: GestureId,
  patch: Partial<GestureBinding>,
): GestureStudioSettings {
  return {
    bindings: settings.bindings.map((binding) => binding.gesture === gesture
      ? { ...binding, ...patch }
      : binding),
  };
}

export function profileGestureSettings(profile: ControlProfile): GestureStudioSettings {
  return profile.settings.gestureStudio;
}
