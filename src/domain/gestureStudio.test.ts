import { describe, expect, it } from "vitest";
import { DEFAULT_GESTURE_STUDIO, gestureConflicts, updateGestureBinding, updateProfileGestureSettings } from "./gestureStudio";
import { BUILT_IN_PROFILES, type UserPreferences } from "./userPreferences";

const preferences: UserPreferences = {
  ...BUILT_IN_PROFILES[0].settings,
  cursorCalibration: { left: 0, top: 0, right: 1, bottom: 1 },
  activeProfileId: "productivity",
  profiles: BUILT_IN_PROFILES,
  autoProfileEnabled: false,
  appProfileRules: [],
};

describe("Gesture Studio settings", () => {
  it("starts with valid, distinct actions for all supported triggers", () => {
    expect(gestureConflicts(DEFAULT_GESTURE_STUDIO)).toEqual([]);
  });

  it("reports conflicting action assignments and unsupported ranges", () => {
    const duplicateAction = updateGestureBinding(DEFAULT_GESTURE_STUDIO, "middlePinch", { action: "clickAndDrag" });
    expect(gestureConflicts(duplicateAction)).toContain("clickAndDrag is assigned to multiple gestures");
    const invalidRange = updateGestureBinding(DEFAULT_GESTURE_STUDIO, "indexPinch", { sensitivity: 2 });
    expect(gestureConflicts(invalidRange)).toContain("indexPinch sensitivity is outside the supported range");
  });

  it("updates and persists bindings only for the active profile", () => {
    const changed = updateGestureBinding(DEFAULT_GESTURE_STUDIO, "indexPinch", { sensitivity: 1.3, cooldownMs: 850 });
    const next = updateProfileGestureSettings(preferences, changed);
    expect(next.gestureStudio.bindings[0]).toMatchObject({ sensitivity: 1.3, cooldownMs: 850 });
    expect(next.profiles[0].settings.gestureStudio.bindings[0]).toMatchObject({ sensitivity: 1.3, cooldownMs: 850 });
    expect(next.profiles[1].settings.gestureStudio).toEqual(BUILT_IN_PROFILES[1].settings.gestureStudio);
  });
});
