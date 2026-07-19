import { invoke } from "@tauri-apps/api/core";

import { codexRuntimeSchema, type CodexRuntimeSnapshot } from "./codex";
import { desktopBootstrapSchema, type DesktopBootstrap } from "./contract";

export const CODEX_RUNTIME_PROBE_COMMAND = "codex_runtime_probe";
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

export async function loadCodexRuntime(
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<CodexRuntimeSnapshot> {
  const payload = await invokeFunction(CODEX_RUNTIME_PROBE_COMMAND);
  return codexRuntimeSchema.parse(payload);
}
