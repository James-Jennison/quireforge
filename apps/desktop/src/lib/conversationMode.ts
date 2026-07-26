import { z } from "zod";

import type { CodexAuthSnapshot } from "./auth";

export const conversationModeSchema = z.enum(["chat", "codex"]);
export type ConversationMode = z.infer<typeof conversationModeSchema>;

export const chatAuthenticationStateSchema = z.enum([
  "ready",
  "sign-in-required",
  "sign-in-pending",
  "unavailable",
]);
export type ChatAuthenticationState = z.infer<
  typeof chatAuthenticationStateSchema
>;

/**
 * Derives only capability readiness. It intentionally does not expose account
 * identifiers, plan details, credentials, browser state, or any token value.
 */
export function managedChatAuthenticationState(
  auth: CodexAuthSnapshot,
): ChatAuthenticationState {
  if (auth.state === "authenticated" && auth.accountKind === "chatgpt") {
    return "ready";
  }
  if (auth.state === "login-pending" && auth.pendingMethod === "browser") {
    return "sign-in-pending";
  }
  if (auth.state === "unauthenticated") return "sign-in-required";
  return "unavailable";
}

export const conversationModeCapabilitySchema = z
  .object({
    mode: conversationModeSchema,
    requiresAttachedProject: z.boolean(),
    allowsNativeActions: z.boolean(),
    allowsTerminal: z.boolean(),
    allowsGit: z.boolean(),
    allowsWorktrees: z.boolean(),
    allowsIntegrations: z.boolean(),
    requiresManagedChatGptAuth: z.boolean(),
  })
  .strict();

export type ConversationModeCapability = z.infer<
  typeof conversationModeCapabilitySchema
>;

export const chatAuthenticationSnapshotSchema = z
  .object({
    schemaVersion: z.literal(1),
    state: chatAuthenticationStateSchema,
    capabilities: z.tuple([
      conversationModeCapabilitySchema,
      conversationModeCapabilitySchema,
    ]),
  })
  .strict()
  .superRefine((snapshot, context) => {
    const [chat, codex] = snapshot.capabilities;
    if (chat.mode !== "chat") {
      context.addIssue({
        code: "custom",
        message: "Chat authentication must expose the Chat capability profile first",
        path: ["capabilities", 0, "mode"],
      });
    }
    if (codex.mode !== "codex") {
      context.addIssue({
        code: "custom",
        message: "Chat authentication must expose the Codex capability profile second",
        path: ["capabilities", 1, "mode"],
      });
    }
  });

export type ChatAuthenticationSnapshot = z.infer<
  typeof chatAuthenticationSnapshotSchema
>;

export const conversationModeCapabilities: Record<
  ConversationMode,
  ConversationModeCapability
> = {
  chat: {
    mode: "chat",
    requiresAttachedProject: false,
    allowsNativeActions: false,
    allowsTerminal: false,
    allowsGit: false,
    allowsWorktrees: false,
    allowsIntegrations: false,
    requiresManagedChatGptAuth: true,
  },
  codex: {
    mode: "codex",
    requiresAttachedProject: true,
    allowsNativeActions: true,
    allowsTerminal: true,
    allowsGit: true,
    allowsWorktrees: true,
    allowsIntegrations: true,
    requiresManagedChatGptAuth: false,
  },
};
