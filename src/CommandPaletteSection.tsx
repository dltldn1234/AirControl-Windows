import { Command, Keyboard, MousePointer2, Music2, Volume1 } from "lucide-react";
import { invoke, isTauri } from "@tauri-apps/api/core";
import { useState, type KeyboardEvent } from "react";
import type { RuntimeSnapshot } from "./domain/runtimeStatus";
import { canOpenCommandPalette, COMMAND_PALETTE_COMMANDS, isCommandPaletteOpen } from "./domain/commandPalette";

const icons = [MousePointer2, Command, Command, Music2, Volume1, Volume1];

export function CommandPaletteSection({
  connection,
  status,
}: {
  connection: "preview" | "connected" | "error";
  status: RuntimeSnapshot | null;
}) {
  const [error, setError] = useState<string | null>(null);
  const active = isCommandPaletteOpen(status);
  const canOpen = canOpenCommandPalette(status, connection);

  async function toggle() {
    if (!isTauri() || (!active && !canOpen)) return;
    try {
      await invoke("set_command_palette_control", { enabled: !active });
      setError(null);
    } catch (reason) {
      setError(reason instanceof Error ? reason.message : String(reason));
    }
  }

  async function execute(commandId: string) {
    if (!isTauri() || !active) return;
    try {
      await invoke("execute_palette_command", { commandId });
      setError(null);
    } catch (reason) {
      setError(reason instanceof Error ? reason.message : String(reason));
    }
  }

  function onKeyDown(event: KeyboardEvent<HTMLElement>) {
    if (event.key === "Escape" && active) {
      event.preventDefault();
      void toggle();
    }
  }

  return (
    <section className="command-palette-panel" aria-label="Air Command Palette" onKeyDown={onKeyDown}>
      <div className="settings-intro">
        <span className="planned-icon"><Command size={20} aria-hidden="true" /></span>
        <div>
          <h2>Air Command Palette</h2>
          <p>Open the palette, hover a command with your index finger, then pinch briefly to run it. Hold a fist to cancel.</p>
        </div>
        <button type="button" className={active ? "secondary-button" : "primary-button"} disabled={!active && !canOpen} onClick={() => void toggle()}>
          {active ? "Close palette" : "Open palette"}
        </button>
      </div>
      {connection !== "connected" && <p className="command-palette-note">Open the Windows desktop app to use hand selection and native commands.</p>}
      {connection === "connected" && !active && !canOpen && <p className="command-palette-note">Stop other control modes, start the camera, and wait for all 21 hand landmarks before opening.</p>}
      <div className="command-palette-grid" role="group" aria-label="Available commands">
        {COMMAND_PALETTE_COMMANDS.map((command, index) => {
          const Icon = icons[index] ?? Command;
          const selected = status?.paletteHoveredIndex === index;
          return (
            <button
              type="button"
              key={command.id}
              className={`command-palette-item ${selected ? "selected" : ""}`}
              disabled={!active}
              aria-pressed={selected}
              onClick={() => void execute(command.id)}
            >
              <span className="command-palette-icon"><Icon size={17} aria-hidden="true" /></span>
              <span className="command-palette-copy"><strong>{command.label}</strong><small>{command.detail}</small></span>
              <kbd>{command.shortcut}</kbd>
            </button>
          );
        })}
      </div>
      <div className="command-palette-status" role="status" aria-live="polite">
        <Keyboard size={14} aria-hidden="true" />
        <span>{active ? "Hover a tile, pinch to run, or press Escape to close." : "System commands remain inactive until you open the palette."}</span>
        {status?.paletteMessage && <strong>{status.paletteMessage}</strong>}
      </div>
      {error && <p className="camera-message error" role="alert">{error}</p>}
    </section>
  );
}
