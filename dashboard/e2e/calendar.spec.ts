import { test, expect } from "@playwright/test";

test.describe("Calendar page", () => {
  test("renders calendar heading", async ({ page }) => {
    await page.goto("/calendar");
    await expect(page.getByRole("heading", { name: "Calendar" })).toBeVisible();
  });

  test("shows day-of-week headers", async ({ page }) => {
    await page.goto("/calendar");
    // Calendar grid renders day names
    await expect(page.getByText("Sun").first()).toBeVisible();
    await expect(page.getByText("Mon").first()).toBeVisible();
    await expect(page.getByText("Tue").first()).toBeVisible();
    await expect(page.getByText("Wed").first()).toBeVisible();
    await expect(page.getByText("Thu").first()).toBeVisible();
    await expect(page.getByText("Fri").first()).toBeVisible();
    await expect(page.getByText("Sat").first()).toBeVisible();
  });

  test("has navigation buttons (Prev, Today, Next)", async ({ page }) => {
    await page.goto("/calendar");
    await expect(page.getByRole("button", { name: /prev/i })).toBeVisible();
    await expect(page.getByRole("button", { name: /today/i })).toBeVisible();
    await expect(page.getByRole("button", { name: /next/i })).toBeVisible();
  });

  test("has filter toggle buttons", async ({ page }) => {
    await page.goto("/calendar");
    await expect(page.getByRole("button", { name: /issues/i })).toBeVisible();
    await expect(page.getByRole("button", { name: /milestones/i })).toBeVisible();
    await expect(page.getByRole("button", { name: /releases/i })).toBeVisible();
    await expect(page.getByRole("button", { name: /pulls|prs/i })).toBeVisible();
  });

  test("month navigation changes displayed month", async ({ page }) => {
    await page.goto("/calendar");
    // Get a reference month name
    const initialText = await page.locator("h3").first().textContent();
    // Click next
    await page.getByRole("button", { name: /next/i }).click();
    // The month heading should change
    const updatedText = await page.locator("h3").first().textContent();
    expect(updatedText).not.toBe(initialText);
  });
});
