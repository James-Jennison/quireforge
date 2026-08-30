import { fireEvent, render, screen } from "@testing-library/react";
import type { ComponentProps } from "react";
import { describe, expect, it, vi } from "vitest";

import { ChatWorkspace } from "./ChatWorkspace";
import { scaffoldCodexAuth } from "./lib/auth";
import { scaffoldChatConversation } from "./lib/chat";

function renderWorkspace(
  overrides: Partial<ComponentProps<typeof ChatWorkspace>> = {},
) {
  const onStart = vi.fn().mockResolvedValue(scaffoldChatConversation);
  const onProviderChange = vi.fn();
  render(
    <ChatWorkspace
      auth={{
        ...scaffoldCodexAuth,
        state: "authenticated",
        accountKind: "chatgpt",
      }}
      snapshot={scaffoldChatConversation}
      busy={false}
      provider={null}
      onProviderChange={onProviderChange}
      interactionProfile="direct"
      onInteractionProfileChange={vi.fn()}
      onStart={onStart}
      onPoll={vi.fn()}
      onInterrupt={vi.fn()}
      {...overrides}
    />,
  );
  return { onProviderChange, onStart };
}

describe("ChatWorkspace", () => {
  it("keeps the draft and makes no provider call before explicit selection", () => {
    const { onStart } = renderWorkspace();

    fireEvent.change(screen.getByRole("textbox", { name: "Chat message" }), {
      target: { value: "Explain this failure." },
    });

    expect(screen.getByRole("textbox", { name: "Chat message" })).toHaveValue(
      "Explain this failure.",
    );
    expect(screen.getByRole("button", { name: "Send" })).toBeDisabled();
    expect(screen.getByRole("status")).toHaveTextContent(
      /no provider connected/i,
    );
    expect(onStart).not.toHaveBeenCalled();
  });

  it("uses managed Codex only after the explicit provider choice", () => {
    const { onProviderChange } = renderWorkspace();

    fireEvent.click(screen.getByRole("button", { name: "Use managed Codex" }));

    expect(onProviderChange).toHaveBeenCalledWith("managed-codex");
  });

  it("sends a managed Codex turn with Enter", () => {
    const { onStart } = renderWorkspace({ provider: "managed-codex" });
    const composer = screen.getByRole("textbox", { name: "Chat message" });

    fireEvent.change(composer, { target: { value: "Where are we?" } });
    fireEvent.keyDown(composer, { key: "Enter" });

    expect(onStart).toHaveBeenCalledWith({
      prompt: "Where are we?",
      interactionProfile: "direct",
    });
  });

  it("keeps Shift+Enter available for a multiline chat message", () => {
    const { onStart } = renderWorkspace({ provider: "managed-codex" });
    const composer = screen.getByRole("textbox", { name: "Chat message" });

    fireEvent.change(composer, { target: { value: "First line" } });
    fireEvent.keyDown(composer, { key: "Enter", shiftKey: true });

    expect(onStart).not.toHaveBeenCalled();
  });

  it("does not offer an API-key fallback", () => {
    renderWorkspace({
      provider: "managed-codex",
      auth: {
        ...scaffoldCodexAuth,
        state: "authenticated",
        accountKind: "api-key",
      },
    });

    fireEvent.change(screen.getByRole("textbox", { name: "Chat message" }), {
      target: { value: "Explain this failure." },
    });
    expect(screen.getByRole("button", { name: "Send" })).toBeDisabled();
    expect(
      screen.getByText(/managed Codex is unavailable/i),
    ).toBeInTheDocument();
  });
});
