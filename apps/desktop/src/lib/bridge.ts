import { invoke } from "@tauri-apps/api/core";

import { desktopBootstrapSchema, type DesktopBootstrap } from "./contract";

export const DESKTOP_BOOTSTRAP_COMMAND = "desktop_bootstrap";

export type InvokeFunction = (
  command: string,
  args?: Record<string, unknown>,
) => Promise<unknown>;

const invokeTauri: InvokeFunction = (command, args) =>
  invoke<unknown>(command, args);

export async function loadDesktopBootstrap(
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<DesktopBootstrap> {
  const payload = await invokeFunction(DESKTOP_BOOTSTRAP_COMMAND);
  return desktopBootstrapSchema.parse(payload);
}
