import { describe, expect, it } from "vitest";
import { BUILT_IN_PROFILES, createControlProfile, deleteControlProfile, isValidProfileName, normalizeExecutableIdentity, removeAppProfileRule, renameControlProfile, restoreControlProfile, upsertAppProfileRule, withProfileSettings, type UserPreferences } from "./userPreferences";

const preferences: UserPreferences = {
  ...BUILT_IN_PROFILES[0].settings,
  cursorCalibration: { left: 0, top: 0, right: 1, bottom: 1 },
  activeProfileId: "productivity",
  profiles: BUILT_IN_PROFILES,
  autoProfileEnabled: false,
  appProfileRules: [],
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

  it("creates and renames a personal profile with a copy of the current settings", () => {
    const created = createControlProfile(preferences, "Travel", "profile-travel");
    expect(created.activeProfileId).toBe("profile-travel");
    expect(created.profiles[created.profiles.length - 1]?.settings).toEqual(BUILT_IN_PROFILES[0].settings);
    expect(renameControlProfile(created, "profile-travel", "Desk").profiles[created.profiles.length - 1]?.name).toBe("Desk");
    expect(() => createControlProfile(created, "travel", "another-id")).toThrow(/unique profile name/);
    expect(() => renameControlProfile(created, "productivity", "Changed preset")).toThrow(/cannot be renamed/);
  });

  it("deletes personal profiles safely and restores built-in preset values", () => {
    const custom = createControlProfile(preferences, "Travel", "profile-travel");
    const deleted = deleteControlProfile(custom, "profile-travel");
    expect(deleted.activeProfileId).toBe("productivity");
    expect(deleted.profiles.some((profile) => profile.id === "profile-travel")).toBe(false);
    expect(() => deleteControlProfile(preferences, "productivity")).toThrow(/cannot be deleted/);

    const modified = { ...preferences, cursorSensitivity: 1.8 };
    const restored = restoreControlProfile(modified, "productivity");
    expect(restored.cursorSensitivity).toBe(BUILT_IN_PROFILES[0].settings.cursorSensitivity);
    expect(() => restoreControlProfile(preferences, "profile-travel")).toThrow(/built-in/);
  });
});


describe("app-specific profile rules", () => {
  it("normalizes an executable path to a lowercase Windows filename", () => {
    expect(normalizeExecutableIdentity("C:\\Apps\\Editor.EXE")).toBe("editor.exe");
    expect(normalizeExecutableIdentity("notes.txt")).toBeNull();
  });

  it("upserts mappings by normalized executable and removes them safely", () => {
    const mapped = upsertAppProfileRule(preferences, "C:\\Apps\\Editor.EXE", "presentation");
    expect(mapped.appProfileRules).toEqual([{ executable: "editor.exe", profileId: "presentation" }]);
    const updated = upsertAppProfileRule(mapped, "editor.exe", "custom");
    expect(updated.appProfileRules).toHaveLength(1);
    expect(removeAppProfileRule(updated, "EDITOR.exe").appProfileRules).toEqual([]);
  });

  it("removes mappings when a custom profile is deleted", () => {
    const withCustom = createControlProfile(preferences, "Travel", "travel");
    const mapped = upsertAppProfileRule(withCustom, "travel.exe", "travel");
    expect(deleteControlProfile(mapped, "travel").appProfileRules).toEqual([]);
  });
});
