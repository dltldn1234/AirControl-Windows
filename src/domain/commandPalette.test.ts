import { describe, expect, it } from "vitest";
import type { RuntimeSnapshot } from "./runtimeStatus";
import { canOpenCommandPalette, COMMAND_PALETTE_COMMANDS, isCommandPaletteOpen } from "./commandPalette";

const ready: RuntimeSnapshot = {
  camera: "running",
  handTracking: "detected",
  controlMode: "disabled",
  activeGesture: "none",
  framesPerSecond: 10,
  activeProfileId: null,
};

describe("Air Command Palette availability", () => {
  it("requires a connected desktop, live camera, full pose, and no active mode", () => {
    expect(canOpenCommandPalette(ready, "connected")).toBe(true);
    expect(canOpenCommandPalette(ready, "preview")).toBe(false);
    expect(canOpenCommandPalette({ ...ready, camera: "stopped" }, "connected")).toBe(false);
    expect(canOpenCommandPalette({ ...ready, handTracking: "palmDetected" }, "connected")).toBe(false);
    expect(canOpenCommandPalette({ ...ready, controlMode: "cursor" }, "connected")).toBe(false);
  });

  it("keeps the action list explicit and detects the exclusive mode", () => {
    expect(COMMAND_PALETTE_COMMANDS.map(({ id }) => id)).toEqual([
      "nextApplication",
      "snapWindowLeft",
      "snapWindowRight",
      "playPause",
      "volumeDown",
      "volumeUp",
    ]);
    expect(isCommandPaletteOpen({ ...ready, controlMode: "commandPalette" })).toBe(true);
    expect(isCommandPaletteOpen(ready)).toBe(false);
  });
});
