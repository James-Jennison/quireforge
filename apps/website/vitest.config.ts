import { defineConfig } from "vitest/config";
import { localTestWorkerCap } from "../../scripts/local-test-workers.mjs";

const maxWorkers = localTestWorkerCap("QUIRE_FORGE_VITEST_MAX_WORKERS", 4);

export default defineConfig({
  test: {
    include: ["src/**/*.test.ts"],
    ...(maxWorkers === undefined ? {} : { maxWorkers }),
  },
});
