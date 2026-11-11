// Playwright configuration of the field-near test (field/run.sh, dsn~field-test~1): the
// browsers run in the tablet containers, the test connects to them itself.
import { defineConfig } from "@playwright/test";

export default defineConfig({
  testDir: "field",
  testMatch: "*.test.ts",
  timeout: 15 * 60_000,
  expect: { timeout: 20_000 },
  workers: 1,
  reporter: "list",
});
