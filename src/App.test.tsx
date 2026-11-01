import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import App from "./App";

describe("AirControl desktop shell", () => {
  it("shows honest preview status and keeps system control disabled", async () => {
    render(<App />);

    expect(screen.getByText("Desktop runtime is not connected")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /Start cursor control/ })).toBeDisabled();
  });

  it("shows the camera setup without pretending browser preview has hardware access", () => {
    render(<App />);

    fireEvent.click(screen.getByRole("button", { name: "Camera & tracking" }));

    expect(screen.getByRole("heading", { level: 1, name: "Camera & tracking" })).toBeInTheDocument();
    expect(screen.getByText("Camera preview is off")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Find cameras" })).toBeDisabled();
    expect(screen.getByRole("button", { name: "Start preview" })).toBeDisabled();
  });

  it("keeps gesture app switching opt-in and unavailable in browser preview", () => {
    render(<App />);
    fireEvent.click(screen.getByRole("button", { name: "Settings" }));

    const toggle = screen.getByRole("checkbox", { name: "Gesture app switching" });
    expect(toggle).not.toBeChecked();
    expect(toggle).toBeDisabled();
    expect(screen.getByText(/horizontal three-finger swipe/)).toBeInTheDocument();
  });

  it("shows full-frame cursor calibration defaults and keeps native editing unavailable in preview", () => {
    render(<App />);
    fireEvent.click(screen.getByRole("button", { name: "Settings" }));

    expect(screen.getByRole("slider", { name: "Calibration left edge" })).toHaveValue("0");
    expect(screen.getByRole("slider", { name: "Calibration top edge" })).toHaveValue("0");
    expect(screen.getByRole("slider", { name: "Calibration right edge" })).toHaveValue("1");
    expect(screen.getByRole("slider", { name: "Calibration bottom edge" })).toHaveValue("1");
    expect(screen.getByRole("button", { name: "Reset range" })).toBeDisabled();
  });

  it("shows named profiles and never enables system control from browser preview", () => {
    render(<App />);
    fireEvent.click(screen.getByRole("button", { name: "Settings" }));

    expect(screen.getByText("Control profiles")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Productivity, built-in preset" })).toBeDisabled();
    expect(screen.getByRole("button", { name: "Presentation, built-in preset" })).toBeDisabled();
    expect(screen.getByRole("button", { name: "Custom, built-in preset" })).toBeDisabled();
    expect(screen.getByRole("button", { name: /Save current as profile/ })).toBeDisabled();
    expect(screen.getByText(/System control remains off/)).toBeInTheDocument();
  });

  it("shows the command palette action list while keeping native actions off in preview", () => {
    render(<App />);
    fireEvent.click(screen.getByRole("button", { name: "Command palette" }));

    expect(screen.getByRole("heading", { level: 1, name: "Command palette" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /Open palette/ })).toBeDisabled();
    expect(screen.getByRole("button", { name: /Next app/ })).toBeDisabled();
    expect(screen.getByRole("button", { name: /Snap left/ })).toBeDisabled();
    expect(screen.getByRole("button", { name: /Play \/ pause/ })).toBeDisabled();
    expect(screen.getByText(/Open the Windows desktop app/)).toBeInTheDocument();
  });

  it("opens Gesture Studio with profile bindings and keeps camera testing unavailable in browser preview", () => {
    render(<App />);
    fireEvent.click(screen.getByRole("button", { name: "Gesture Studio" }));

    expect(screen.getByRole("heading", { level: 1, name: "Gesture Studio" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /Index pinch.*Click and drag/ })).toHaveAttribute("aria-pressed", "true");
    expect(screen.getByRole("combobox", { name: "Gesture profile" })).toHaveValue("productivity");
    expect(screen.getByRole("button", { name: "Start test" })).toBeDisabled();
    expect(screen.getByRole("button", { name: "Find camera" })).toBeDisabled();
    expect(screen.getByRole("button", { name: "Save changes" })).toBeDisabled();
    expect(screen.getByText("Camera FPS")).toBeInTheDocument();
    expect(screen.getByText("Recent transitions")).toBeInTheDocument();
    expect(screen.getAllByText("Unavailable")).toHaveLength(2);
    expect(screen.getByText(/Open the Windows desktop app/)).toBeInTheDocument();
  });

  it("keeps Air Keyboard off in browser preview and explains its camera prerequisite", () => {
    render(<App />);
    fireEvent.click(screen.getByRole("button", { name: "Air Keyboard" }));
    expect(screen.getByRole("heading", { level: 1, name: "Air Keyboard" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Start Air Keyboard" })).toBeDisabled();
    expect(screen.getByText(/Enable the camera and wait for hand tracking/)).toBeInTheDocument();
    expect(screen.getByText(/No typed characters are logged or saved/)).toBeInTheDocument();
  });

  it("renders a separate keyboard surface with Dubeolsik legends and editing keys", () => {
    window.history.pushState({}, "", "?surface=keyboard");
    try {
      render(<App />);
      expect(screen.getByRole("main", { name: "Air Keyboard" })).toBeInTheDocument();
      expect(screen.getByRole("button", { name: "A" })).toBeInTheDocument();
      expect(screen.getByRole("button", { name: "Toggle English or Korean key labels" })).toBeInTheDocument();
      expect(screen.getByRole("button", { name: "Stop Air Keyboard" })).toBeInTheDocument();
      expect(screen.getByText(/does not report the target app’s IME state/)).toBeInTheDocument();
      expect(screen.getByRole("button", { name: "Caps" })).not.toHaveAttribute("aria-pressed");
    } finally {
      window.history.replaceState({}, "", window.location.pathname);
    }
  });
});
