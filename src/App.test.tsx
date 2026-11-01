import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import App from "./App";

describe("AirControl desktop shell", () => {
  it("shows honest preview status and keeps system control disabled", async () => {
    render(<App />);

    expect(screen.getByText("Desktop runtime is not connected")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /Start control/ })).toBeDisabled();
  });

  it("shows the camera setup without pretending browser preview has hardware access", () => {
    render(<App />);

    fireEvent.click(screen.getByRole("button", { name: "Camera & tracking" }));

    expect(screen.getByRole("heading", { level: 1, name: "Camera & tracking" })).toBeInTheDocument();
    expect(screen.getByText("Camera preview is off")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Find cameras" })).toBeDisabled();
    expect(screen.getByRole("button", { name: "Start preview" })).toBeDisabled();
  });
});
