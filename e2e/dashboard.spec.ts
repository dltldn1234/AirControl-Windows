import { expect, test } from "@playwright/test";

test("dashboard clearly separates preview from the desktop runtime", async ({ page }) => {
  await page.goto("/");

  await expect(page.getByRole("heading", { level: 1, name: "Overview" })).toBeVisible();
  await expect(page.getByText("Desktop runtime is not connected")).toBeVisible();
  await expect(page.getByRole("button", { name: /Start control/ })).toBeDisabled();
});

test("camera page clearly separates native capture from browser preview", async ({ page }) => {
  await page.goto("/");
  await page.getByRole("button", { name: "Camera & tracking" }).click();

  await expect(page.getByRole("heading", { level: 1, name: "Camera & tracking" })).toBeVisible();
  await expect(page.getByRole("heading", { name: "Camera source" })).toBeVisible();
  await expect(page.getByRole("button", { name: "Find cameras" })).toBeDisabled();
  await expect(page.getByText("Open AirControl as a desktop app to access its camera service.")).toBeVisible();
});
