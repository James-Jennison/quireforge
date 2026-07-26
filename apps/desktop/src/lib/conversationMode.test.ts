import { describe, expect, it } from "vitest";

import { scaffoldCodexAuth } from "./auth";
import {
  chatAuthenticationSnapshotSchema,
  conversationModeCapabilities,
  conversationModeCapabilitySchema,
  managedChatAuthenticationState,
} from "./conversationMode";

describe("conversation mode capability policy", () => {
  it("keeps Chat distinct from attached-project Codex capabilities", () => {
    expect(
      conversationModeCapabilitySchema.parse(conversationModeCapabilities.chat),
    ).toEqual({
      mode: "chat",
      requiresAttachedProject: false,
      allowsNativeActions: false,
      allowsTerminal: false,
      allowsGit: false,
      allowsWorktrees: false,
      allowsIntegrations: false,
      requiresManagedChatGptAuth: true,
    });
    expect(conversationModeCapabilities.codex.requiresAttachedProject).toBe(
      true,
    );
  });

  it("accepts only a managed ChatGPT account for Chat readiness", () => {
    expect(
      managedChatAuthenticationState({
        ...scaffoldCodexAuth,
        state: "authenticated",
        accountKind: "chatgpt",
      }),
    ).toBe("ready");
    expect(
      managedChatAuthenticationState({
        ...scaffoldCodexAuth,
        state: "authenticated",
        accountKind: "api-key",
      }),
    ).toBe("unavailable");
    expect(
      managedChatAuthenticationState({
        ...scaffoldCodexAuth,
        state: "login-pending",
        accountKind: null,
        pendingMethod: "browser",
        handoff: {
          verificationUrl: "https://auth.openai.com/authorize",
          userCode: null,
        },
      }),
    ).toBe("sign-in-pending");
  });

  it("rejects a reordered native capability catalog", () => {
    expect(() =>
      chatAuthenticationSnapshotSchema.parse({
        schemaVersion: 1,
        state: "ready",
        capabilities: [
          conversationModeCapabilities.codex,
          conversationModeCapabilities.chat,
        ],
      }),
    ).toThrow();
  });
});
