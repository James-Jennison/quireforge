import { defineConfig, devices } from "@playwright/test";
import { localTestWorkerCap } from "../../scripts/local-test-workers.mjs";

const workers = localTestWorkerCap("QUIRE_FORGE_PLAYWRIGHT_WORKERS", 2);

export default defineConfig({
  testDir: "./tests",
  outputDir: "./test-results",
  reporter: process.env.CI ? "github" : "list",
  ...(workers === undefined ? {} : { workers }),
  use: {
    baseURL: "http://127.0.0.1:4321",
    ...(process.env.PLAYWRIGHT_CHROMIUM_EXECUTABLE
      ? {
          launchOptions: {
            executablePath: process.env.PLAYWRIGHT_CHROMIUM_EXECUTABLE,
          },
        }
      : {}),
    trace: "retain-on-failure",
    screenshot: "only-on-failure",
  },
  webServer: {
    command: "pnpm preview --host 127.0.0.1 --port 4321",
    url: "http://127.0.0.1:4321",
    reuseExistingServer: !process.env.CI,
    timeout: 60_000,
  },
  projects: [
    {
      name: "chromium-desktop",
      use: { ...devices["Desktop Chrome"] },
    },
    {
      name: "chromium-mobile",
      use: { ...devices["Pixel 7"] },
    },
  ],
});
