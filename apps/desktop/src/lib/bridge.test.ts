import { describe, expect, it, vi } from "vitest";

import { scaffoldCodexRuntime } from "./codex";
import {
  CODEX_RUNTIME_PROBE_COMMAND,
  DESKTOP_BOOTSTRAP_COMMAND,
  loadCodexRuntime,
  loadDesktopBootstrap,
} from "./bridge";
import { scaffoldBootstrap } from "./contract";

describe("desktop bridge", () => {
  it("invokes the one typed bootstrap command", async () => {
    const invokeFunction = vi.fn().mockResolvedValue(scaffoldBootstrap);

    await expect(loadDesktopBootstrap(invokeFunction)).resolves.toEqual(
      scaffoldBootstrap,
    );
    expect(invokeFunction).toHaveBeenCalledWith(DESKTOP_BOOTSTRAP_COMMAND);
  });

  it("does not pass malformed native data into the UI", async () => {
    const invokeFunction = vi.fn().mockResolvedValue({ schemaVersion: 1 });

    await expect(loadDesktopBootstrap(invokeFunction)).rejects.toThrow();
  });

  it("invokes and validates the normalized Codex runtime probe", async () => {
    const invoke = vi.fn().mockResolvedValue(scaffoldCodexRuntime);

    await expect(loadCodexRuntime(invoke)).resolves.toEqual(
      scaffoldCodexRuntime,
    );
    expect(invoke).toHaveBeenCalledWith(CODEX_RUNTIME_PROBE_COMMAND);
  });
});
