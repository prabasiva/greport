import { test, expect } from "@playwright/test";

test.describe("Calendar page", () => {
  test("loads the calendar route", async ({ page }) => {
    await page.goto("/calendar");
    // Page should load without errors
    await expect(page.locator("aside")).toBeVisible();
  });

  test("shows calendar heading", async ({ page }) => {
    await page.goto("/calendar");
    // Default mode is aggregate, so the calendar layout renders with heading
    await expect(
      page.getByRole("heading", { name: "Calendar" }),
    ).toBeVisible();
  });

  test("shows aggregate subtitle", async ({ page }) => {
    await page.goto("/calendar");
    await expect(
      page.getByText("All repositories"),
    ).toBeVisible();
  });

  test("sidebar Planning link is visible from calendar page", async ({ page }) => {
    await page.goto("/calendar");
    await expect(
      page.locator("aside").getByText("Planning"),
    ).toBeVisible();
  });
});
