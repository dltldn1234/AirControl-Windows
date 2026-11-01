import { expect, test } from "@playwright/test";

test("dashboard clearly separates preview from the desktop runtime", async ({ page }) => {
  await page.goto("/");

  await expect(page.getByRole("heading", { level: 1, name: "Overview" })).toBeVisible();
  await expect(page.getByText("Desktop runtime is not connected")).toBeVisible();
  await expect(page.getByRole("button", { name: /Start control/ })).toBeDisabled();
});

test("sidebar opens a feature page with an explicit implementation state", async ({ page }) => {
  await page.goto("/");
  await page.getByRole("button", { name: "Camera & tracking" }).click();

  await expect(page.getByRole("heading", { level: 1, name: "Camera & tracking" })).toBeVisible();
  await expect(page.getByText("IN DEVELOPMENT")).toBeVisible();
});
