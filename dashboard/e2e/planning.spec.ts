import { test, expect } from "@playwright/test";

test.describe("Planning page", () => {
  test("loads the planning route", async ({ page }) => {
    await page.goto("/planning");
    // Page should load without errors
    await expect(page.locator("aside")).toBeVisible();
  });

  test("shows planning heading", async ({ page }) => {
    await page.goto("/planning");
    // Default mode is aggregate, so the planning layout renders with heading
    await expect(
      page.getByRole("heading", { name: "Planning" }),
    ).toBeVisible();
  });

  test("shows view switcher", async ({ page }) => {
    await page.goto("/planning");
    await expect(page.getByText("Calendar View")).toBeVisible();
    await expect(page.getByText("Release Plan")).toBeVisible();
  });

  test("can navigate from planning to settings", async ({ page }) => {
    await page.goto("/planning");
    await page.locator("aside").getByText("Settings").click();
    await expect(page).toHaveURL(/\/settings/);
  });
});
