import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { ChatWorkspace } from "./ChatWorkspace";
import { scaffoldCodexAuth } from "./lib/auth";
import { scaffoldChatConversation } from "./lib/chat";

describe("ChatWorkspace", () => {
  it("submits only through a managed ChatGPT account", () => {
    const onStart = vi.fn().mockResolvedValue(scaffoldChatConversation);
    render(
      <ChatWorkspace
        auth={{
          ...scaffoldCodexAuth,
          state: "authenticated",
          accountKind: "chatgpt",
        }}
        snapshot={scaffoldChatConversation}
        busy={false}
        onStart={onStart}
        onPoll={vi.fn()}
        onInterrupt={vi.fn()}
      />,
    );

    fireEvent.change(screen.getByRole("textbox", { name: "Chat message" }), {
      target: { value: "Explain this failure." },
    });
    fireEvent.click(screen.getByRole("button", { name: "Send" }));

    expect(onStart).toHaveBeenCalledWith({ prompt: "Explain this failure." });
    expect(screen.getByText(/no attached directory/i)).toBeInTheDocument();
  });

  it("does not offer an API-key fallback", () => {
    render(
      <ChatWorkspace
        auth={{
          ...scaffoldCodexAuth,
          state: "authenticated",
          accountKind: "api-key",
        }}
        snapshot={scaffoldChatConversation}
        busy={false}
        onStart={vi.fn()}
        onPoll={vi.fn()}
        onInterrupt={vi.fn()}
      />,
    );

    expect(screen.getByRole("button", { name: "Send" })).toBeDisabled();
    expect(screen.getByRole("status")).toHaveTextContent(
      /managed ChatGPT browser sign-in/i,
    );
  });
});
