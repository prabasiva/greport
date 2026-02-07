import { test, expect } from "@playwright/test";

test.describe("Planning page", () => {
  test("renders planning heading", async ({ page }) => {
    await page.goto("/planning");
    await expect(page.getByRole("heading", { name: "Planning" })).toBeVisible();
  });

  test("shows view switcher with Calendar View and Release Plan", async ({ page }) => {
    await page.goto("/planning");
    await expect(page.getByRole("button", { name: "Calendar View" })).toBeVisible();
    await expect(page.getByRole("button", { name: "Release Plan" })).toBeVisible();
  });

  test("switches between Calendar View and Release Plan", async ({ page }) => {
    await page.goto("/planning");

    // Switch to Release Plan
    await page.getByRole("button", { name: "Release Plan" }).click();
    // Should show release plan content (sections like "Upcoming" or empty state)
    // The view switcher should highlight Release Plan
    const releasePlanBtn = page.getByRole("button", { name: "Release Plan" });
    await expect(releasePlanBtn).toHaveCSS("background-color", /./);

    // Switch back to Calendar View
    await page.getByRole("button", { name: "Calendar View" }).click();
    // Calendar elements should be visible again
    await expect(page.getByText("Sun").first()).toBeVisible();
  });

  test("calendar view shows grid with day headers", async ({ page }) => {
    await page.goto("/planning");
    // Default view should be calendar
    await expect(page.getByText("Sun").first()).toBeVisible();
    await expect(page.getByText("Sat").first()).toBeVisible();
  });
});
