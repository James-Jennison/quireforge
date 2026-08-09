import { defineConfig } from "vitest/config";
import react from "@vitejs/plugin-react";
import { localTestWorkerCap } from "../../scripts/local-test-workers.mjs";

const maxWorkers = localTestWorkerCap("QUIRE_FORGE_VITEST_MAX_WORKERS", 4);

export default defineConfig({
  plugins: [react()],
  test: {
    environment: "jsdom",
    include: ["src/**/*.test.{ts,tsx}"],
    setupFiles: ["./src/test/setup.ts"],
    ...(maxWorkers === undefined ? {} : { maxWorkers }),
  },
});
