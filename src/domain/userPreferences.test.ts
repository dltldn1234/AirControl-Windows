import { describe, expect, it } from "vitest";
import { BUILT_IN_PROFILES, isValidProfileName, withProfileSettings, type UserPreferences } from "./userPreferences";

const preferences: UserPreferences = {
  ...BUILT_IN_PROFILES[0].settings,
  activeProfileId: "productivity",
  profiles: BUILT_IN_PROFILES,
};

describe("control profile helpers", () => {
  it("provides named presets while leaving system control disabled by default", () => {
    expect(BUILT_IN_PROFILES.map(({ name }) => name)).toEqual(["Productivity", "Presentation", "Custom"]);
    expect(preferences.activeProfileId).toBe("productivity");
  });

  it("applies a selected profile without changing unrelated profile definitions", () => {
    const selected = withProfileSettings(preferences, BUILT_IN_PROFILES[1]);
    expect(selected.activeProfileId).toBe("presentation");
    expect(selected.appSwitchEnabled).toBe(true);
    expect(selected.profiles).toEqual(preferences.profiles);
  });

  it("validates profile names and rejects case-insensitive duplicates", () => {
    expect(isValidProfileName("Travel", BUILT_IN_PROFILES)).toBe(true);
    expect(isValidProfileName(" presentation ", BUILT_IN_PROFILES)).toBe(false);
    expect(isValidProfileName(" ", BUILT_IN_PROFILES)).toBe(false);
    expect(isValidProfileName("x".repeat(33), BUILT_IN_PROFILES)).toBe(false);
    expect(isValidProfileName("Presentation", BUILT_IN_PROFILES, "presentation")).toBe(true);
  });
});
