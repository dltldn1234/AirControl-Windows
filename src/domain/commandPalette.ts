import type { RuntimeSnapshot } from "./runtimeStatus";

export const COMMAND_PALETTE_COMMANDS = [
  { id: "nextApplication", label: "Next app", detail: "Switch to the next eligible app window", shortcut: "1" },
  { id: "snapWindowLeft", label: "Snap left", detail: "Focus the next eligible app window and snap it left", shortcut: "2" },
  { id: "snapWindowRight", label: "Snap right", detail: "Focus the next eligible app window and snap it right", shortcut: "3" },
  { id: "playPause", label: "Play / pause", detail: "Toggle media playback", shortcut: "4" },
  { id: "volumeDown", label: "Volume down", detail: "Lower system media volume", shortcut: "5" },
  { id: "volumeUp", label: "Volume up", detail: "Raise system media volume", shortcut: "6" },
] as const;

export function canOpenCommandPalette(
  status: RuntimeSnapshot | null,
  connection: "preview" | "connected" | "error",
): boolean {
  return connection === "connected"
    && status?.camera === "running"
    && status.handTracking === "detected"
    && status.controlMode === "disabled";
}

export function isCommandPaletteOpen(status: RuntimeSnapshot | null): boolean {
  return status?.controlMode === "commandPalette";
}
