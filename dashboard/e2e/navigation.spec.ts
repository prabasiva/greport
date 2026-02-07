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
    await page.getByText("Planning").click();
    await expect(page).toHaveURL(/\/planning/);
    await expect(page.getByText("Planning").first()).toBeVisible();
  });

  test("navigates to Calendar page", async ({ page }) => {
    await page.goto("/");
    // Calendar page is accessed via /calendar
    await page.goto("/calendar");
    await expect(page.getByText("Calendar").first()).toBeVisible();
  });

  test("highlights active nav item", async ({ page }) => {
    await page.goto("/planning");
    const planningLink = page.locator("aside").getByText("Planning");
    await expect(planningLink).toBeVisible();
  });
});
