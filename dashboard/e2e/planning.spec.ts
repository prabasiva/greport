import { test, expect } from "@playwright/test";

test.describe("Planning page", () => {
  test("loads the planning route", async ({ page }) => {
    await page.goto("/planning");
    // Page should load without errors
    await expect(page.locator("aside")).toBeVisible();
  });

  test("shows no-repo-selected state without a backend", async ({ page }) => {
    await page.goto("/planning");
    // Without a backend API and no repo selected, the page shows the empty state
    await expect(
      page.getByText("No repository selected"),
    ).toBeVisible();
  });

  test("shows prompt to select a repository", async ({ page }) => {
    await page.goto("/planning");
    await expect(
      page.getByText("Select a repository using the selector in the header"),
    ).toBeVisible();
  });

  test("can navigate from planning to settings", async ({ page }) => {
    await page.goto("/planning");
    await page.locator("aside").getByText("Settings").click();
    await expect(page).toHaveURL(/\/settings/);
  });
});
