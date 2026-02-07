import { test, expect } from "@playwright/test";

test.describe("Sidebar navigation", () => {
  test("renders all navigation links", async ({ page }) => {
    await page.goto("/");
    const sidebar = page.locator("aside");
    await expect(sidebar.getByText("Dashboard")).toBeVisible();
    await expect(sidebar.getByText("Issues")).toBeVisible();
    await expect(sidebar.getByText("Pull Requests")).toBeVisible();
    await expect(sidebar.getByText("Releases")).toBeVisible();
    await expect(sidebar.getByText("Planning")).toBeVisible();
    await expect(sidebar.getByText("Velocity")).toBeVisible();
    await expect(sidebar.getByText("Contributors")).toBeVisible();
    await expect(sidebar.getByText("SLA")).toBeVisible();
    await expect(sidebar.getByText("Settings")).toBeVisible();
  });

  test("navigates to Planning page", async ({ page }) => {
    await page.goto("/");
    await page.locator("aside").getByText("Planning").click();
    await expect(page).toHaveURL(/\/planning/);
  });

  test("navigates to Settings page", async ({ page }) => {
    await page.goto("/");
    await page.locator("aside").getByText("Settings").click();
    await expect(page).toHaveURL(/\/settings/);
  });

  test("shows greport logo", async ({ page }) => {
    await page.goto("/");
    await expect(page.locator("aside").getByText("greport")).toBeVisible();
  });
});
