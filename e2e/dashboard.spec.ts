import { expect, test } from "@playwright/test";

test("dashboard clearly separates preview from the desktop runtime", async ({ page }) => {
  await page.goto("/");

  await expect(page.getByRole("heading", { level: 1, name: "Overview" })).toBeVisible();
  await expect(page.getByText("Desktop runtime is not connected")).toBeVisible();
  await expect(page.getByRole("button", { name: /Start cursor control/ })).toBeDisabled();
});

test("camera page clearly separates native capture from browser preview", async ({ page }) => {
  await page.goto("/");
  await page.getByRole("button", { name: "Camera & tracking" }).click();

  await expect(page.getByRole("heading", { level: 1, name: "Camera & tracking" })).toBeVisible();
  await expect(page.getByRole("heading", { name: "Camera source" })).toBeVisible();
  await expect(page.getByRole("button", { name: "Find cameras" })).toBeDisabled();
  await expect(page.getByText("Local hand inference is waiting for a camera frame.")).toBeVisible();
});

test("window controls stay unavailable in browser preview mode", async ({ page }) => {
  await page.goto("/");
  await page.getByRole("button", { name: "Window control" }).click();

  await expect(page.getByRole("heading", { name: "Application windows" })).toBeVisible();
  await expect(page.getByText("Window enumeration and controls are available in the Windows desktop app.")).toBeVisible();
  await expect(page.getByRole("button", { name: "Start hand move" })).toBeDisabled();
});

test("gesture app switching is an explicit desktop-only preference", async ({ page }) => {
  await page.goto("/");
  await page.getByRole("button", { name: "Settings" }).click();

  const appSwitch = page.getByRole("checkbox", { name: "Gesture app switching" });
  await expect(appSwitch).not.toBeChecked();
  await expect(appSwitch).toBeDisabled();
  await expect(page.getByText(/horizontal three-finger swipe/)).toBeVisible();
});

test("settings explain and display the safe default cursor calibration range", async ({ page }) => {
  await page.goto("/");
  await page.getByRole("button", { name: "Settings" }).click();

  await expect(page.getByText("Cursor movement range")).toBeVisible();
  await expect(page.getByRole("slider", { name: "Calibration left edge" })).toHaveValue("0");
  await expect(page.getByRole("slider", { name: "Calibration right edge" })).toHaveValue("1");
  await expect(page.getByRole("button", { name: "Reset range" })).toBeDisabled();
});

test("control profiles are visible but unavailable without the native runtime", async ({ page }) => {
  await page.goto("/");
  await page.getByRole("button", { name: "Settings" }).click();

  await expect(page.getByText("Control profiles")).toBeVisible();
  await expect(page.getByRole("button", { name: "Productivity, built-in preset" })).toBeDisabled();
  await expect(page.getByRole("button", { name: "Presentation, built-in preset" })).toBeDisabled();
  await expect(page.getByRole("button", { name: "Custom, built-in preset" })).toBeDisabled();
  await expect(page.getByRole("button", { name: "Save current as profile" })).toBeDisabled();
});

test("application profile mappings remain opt-in and disabled in browser preview", async ({ page }) => {
  await page.goto("/");
  await page.getByRole("button", { name: "Settings" }).click();

  await expect(page.getByText("Application profiles")).toBeVisible();
  await expect(page.getByRole("checkbox", { name: /Automatically select a profile/ })).not.toBeChecked();
  await expect(page.getByRole("checkbox", { name: /Automatically select a profile/ })).toBeDisabled();
  await expect(page.getByRole("textbox", { name: "Windows executable name" })).toBeDisabled();
  await expect(page.getByRole("button", { name: "Add or update" })).toBeDisabled();
  await expect(page.getByText("No application mappings yet.")).toBeVisible();
});

test("Air Keyboard preview exposes the layout without sending system key events", async ({ page }) => {
  await page.goto("/?surface=keyboard");

  await expect(page.getByRole("main", { name: "Air Keyboard" })).toBeVisible();
  await expect(page.getByRole("button", { name: "Toggle English or Korean key labels" })).toBeDisabled();
  await expect(page.getByRole("button", { name: "Stop Air Keyboard" })).toBeVisible();
  await expect(page.getByText("Hand tracking paused · waiting for a confident hand pose")).toBeVisible();
});
