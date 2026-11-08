// Playwright configuration for the browser end-to-end tests (ADR-0032: existing tools
// instead of an own browser driver). Run from this directory:
//   npm ci && npx playwright install --with-deps firefox chromium webkit
//   ORDO_BIN=../../target/debug/ordo ORDO_APP_DIST=../../platform/ordo-app/dist npx playwright test
import { defineConfig, devices } from "@playwright/test";

const viewport = { width: 1200, height: 1000 };

export default defineConfig({
  testDir: ".",
  testMatch: "*.spec.ts",
  timeout: 10 * 60_000,
  expect: { timeout: 15_000 },
  // One scenario per browser, one after the other: each starts its own server.
  workers: 1,
  fullyParallel: false,
  reporter: process.env.CI ? [["list"], ["html", { open: "never" }]] : "list",
  use: { trace: "retain-on-failure", screenshot: "only-on-failure" },
  projects: [
    { name: "firefox", use: { ...devices["Desktop Firefox"], viewport } },
    { name: "chromium", use: { ...devices["Desktop Chrome"], viewport } },
    { name: "webkit", use: { ...devices["Desktop Safari"], viewport } },
  ],
});
