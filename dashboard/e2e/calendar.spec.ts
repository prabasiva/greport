import { test, expect } from "@playwright/test";

test.describe("Calendar page", () => {
  test("loads the calendar route", async ({ page }) => {
    await page.goto("/calendar");
    // Page should load without errors
    await expect(page.locator("aside")).toBeVisible();
  });

  test("shows no-repo-selected state without a backend", async ({ page }) => {
    await page.goto("/calendar");
    // Without a backend API and no repo selected, the page shows the empty state
    await expect(
      page.getByText("No repository selected"),
    ).toBeVisible();
  });

  test("shows prompt to select a repository", async ({ page }) => {
    await page.goto("/calendar");
    await expect(
      page.getByText("Select a repository using the selector in the header"),
    ).toBeVisible();
  });

  test("sidebar Planning link is visible from calendar page", async ({ page }) => {
    await page.goto("/calendar");
    await expect(
      page.locator("aside").getByText("Planning"),
    ).toBeVisible();
  });
});
