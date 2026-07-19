import { describe, expect, it, vi } from "vitest";

import { DESKTOP_BOOTSTRAP_COMMAND, loadDesktopBootstrap } from "./bridge";
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
});
