import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { LocalChatWorkspace } from "./LocalChatWorkspace";

const actionCard = {
  schemaVersion: 1 as const,
  cardId: "8d844bb4-af03-47be-a40b-1f216ef4ee5b",
  action: "attach-project" as const,
  state: "prepared" as const,
  dataScope: "none" as const,
  execution: "not-authorized" as const,
  receiptId: null,
  expiresAtMs: 1_800_000_000_000,
};

describe("LocalChatWorkspace", () => {
  it("sends with Enter and renders a natural local conversation", async () => {
    const onRun = vi.fn().mockResolvedValue({
      schemaVersion: 1,
      localOnly: true,
      state: "completed",
      output: "Local answer.",
      diagnostic: null,
      inputTokenLimit: 4096,
      outputTokenLimit: 512,
      deadlineSeconds: 60,
      memoryCeilingMib: 6144,
    });
    render(
      <LocalChatWorkspace
        onRun={onRun}
        onCancel={vi.fn()}
        onPrepareActionCard={vi.fn()}
        onApproveActionCard={vi.fn()}
        onRevokeActionCard={vi.fn()}
        onOpenLinkedProjectChat={vi.fn()}
      />,
    );
    fireEvent.change(
      screen.getByRole("textbox", { name: "Local chat message" }),
      { target: { value: "Hello" } },
    );
    fireEvent.keyDown(
      screen.getByRole("textbox", { name: "Local chat message" }),
      {
        key: "Enter",
      },
    );
    expect(await screen.findByText("Local answer.")).toBeVisible();
    expect(onRun).toHaveBeenCalledWith({
      message: "Hello",
      interactionProfile: "direct",
    });
    expect(screen.getByText("You")).toBeVisible();
    expect(screen.getByText("QuireForge")).toBeVisible();
    expect(screen.getByText("Local and ephemeral")).toBeVisible();
    expect(screen.getByRole("button", { name: "Actions" })).toBeVisible();
  });

  it("keeps Shift+Enter available for a multiline message", () => {
    const onRun = vi.fn();
    render(
      <LocalChatWorkspace
        onRun={onRun}
        onCancel={vi.fn()}
        onPrepareActionCard={vi.fn()}
        onApproveActionCard={vi.fn()}
        onRevokeActionCard={vi.fn()}
        onOpenLinkedProjectChat={vi.fn()}
      />,
    );
    const composer = screen.getByRole("textbox", {
      name: "Local chat message",
    });
    fireEvent.change(composer, { target: { value: "First line" } });
    fireEvent.keyDown(composer, { key: "Enter", shiftKey: true });

    expect(onRun).not.toHaveBeenCalled();
  });

  it("sends the selected conversation style with the local-only turn", async () => {
    const onRun = vi.fn().mockResolvedValue({
      schemaVersion: 1,
      localOnly: true,
      state: "completed",
      output: "A warmer answer.",
      diagnostic: null,
      inputTokenLimit: 4096,
      outputTokenLimit: 512,
      deadlineSeconds: 60,
      memoryCeilingMib: 6144,
    });
    const onInteractionProfileChange = vi.fn();
    render(
      <LocalChatWorkspace
        onRun={onRun}
        onCancel={vi.fn()}
        onPrepareActionCard={vi.fn()}
        onApproveActionCard={vi.fn()}
        onRevokeActionCard={vi.fn()}
        onOpenLinkedProjectChat={vi.fn()}
        interactionProfile="conversational"
        onInteractionProfileChange={onInteractionProfileChange}
      />,
    );

    expect(screen.getByRole("radio", { name: "Conversational" })).toBeChecked();
    fireEvent.click(screen.getByRole("radio", { name: "Direct" }));
    expect(onInteractionProfileChange).toHaveBeenCalledWith("direct");

    fireEvent.change(
      screen.getByRole("textbox", { name: "Local chat message" }),
      {
        target: { value: "Hello again" },
      },
    );
    fireEvent.click(screen.getByRole("button", { name: "Send" }));

    await screen.findByText("A warmer answer.");
    expect(onRun).toHaveBeenCalledWith({
      message: "Hello again",
      interactionProfile: "conversational",
    });
  });

  it("prepares and approves a visible non-executing action card", async () => {
    const onPrepareActionCard = vi.fn().mockResolvedValue(actionCard);
    const onApproveActionCard = vi.fn().mockResolvedValue({
      ...actionCard,
      state: "approved",
      receiptId: "10b576ce-71c7-4bc7-a738-bfcaefce0f03",
    });
    render(
      <LocalChatWorkspace
        onRun={vi.fn()}
        onCancel={vi.fn()}
        onPrepareActionCard={onPrepareActionCard}
        onApproveActionCard={onApproveActionCard}
        onRevokeActionCard={vi.fn()}
        onOpenLinkedProjectChat={vi.fn()}
      />,
    );

    fireEvent.click(screen.getByRole("button", { name: "Actions" }));
    expect(
      screen.getByText(
        "Prepare a visible proposal. Nothing will run from this menu.",
      ),
    ).toBeVisible();
    fireEvent.click(screen.getByRole("menuitem", { name: "Attach a project" }));
    expect(
      await screen.findByRole("heading", { name: "Attach a project" }),
    ).toBeVisible();
    expect(onPrepareActionCard).toHaveBeenCalledWith({
      action: "attach-project",
    });
    expect(
      screen.getByText(
        /No project, source, artifact, code, provider, or tool data/,
      ),
    ).toBeVisible();

    fireEvent.click(screen.getByRole("button", { name: "Approve for later" }));
    expect(
      await screen.findByText(
        "Approved for a later capability-specific step. No action has run.",
      ),
    ).toBeVisible();
    expect(onApproveActionCard).toHaveBeenCalledWith({
      cardId: actionCard.cardId,
    });
  });

  it("dismisses only the Actions picker with Escape or an outside pointer", () => {
    render(
      <LocalChatWorkspace
        onRun={vi.fn()}
        onCancel={vi.fn()}
        onPrepareActionCard={vi.fn()}
        onApproveActionCard={vi.fn()}
        onRevokeActionCard={vi.fn()}
        onOpenLinkedProjectChat={vi.fn()}
      />,
    );

    fireEvent.click(screen.getByRole("button", { name: "Actions" }));
    expect(screen.getByRole("menu")).toBeVisible();
    fireEvent.keyDown(window, { key: "Escape" });
    expect(screen.queryByRole("menu")).not.toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Actions" }));
    fireEvent.pointerDown(document.body);
    expect(screen.queryByRole("menu")).not.toBeInTheDocument();
  });

  it("keeps a prepared Action Card until its explicit resolution", async () => {
    const onPrepareActionCard = vi.fn().mockResolvedValue(actionCard);
    const onApproveActionCard = vi.fn();
    const onRevokeActionCard = vi.fn();
    render(
      <LocalChatWorkspace
        onRun={vi.fn()}
        onCancel={vi.fn()}
        onPrepareActionCard={onPrepareActionCard}
        onApproveActionCard={onApproveActionCard}
        onRevokeActionCard={onRevokeActionCard}
        onOpenLinkedProjectChat={vi.fn()}
      />,
    );

    fireEvent.click(screen.getByRole("button", { name: "Actions" }));
    fireEvent.click(screen.getByRole("menuitem", { name: "Attach a project" }));
    expect(
      await screen.findByRole("heading", { name: "Attach a project" }),
    ).toBeVisible();

    fireEvent.keyDown(window, { key: "Escape" });
    expect(
      screen.getByRole("heading", { name: "Attach a project" }),
    ).toBeVisible();
    expect(onApproveActionCard).not.toHaveBeenCalled();
    expect(onRevokeActionCard).not.toHaveBeenCalled();
  });

  it("offers an explicit path to the linked project conversation", () => {
    const onOpenLinkedProjectChat = vi.fn();
    render(
      <LocalChatWorkspace
        onRun={vi.fn()}
        onCancel={vi.fn()}
        onPrepareActionCard={vi.fn()}
        onApproveActionCard={vi.fn()}
        onRevokeActionCard={vi.fn()}
        onOpenLinkedProjectChat={onOpenLinkedProjectChat}
      />,
    );

    fireEvent.click(screen.getByText("About Chat & Cowork"));
    expect(
      screen.getByText(/Local runtime · Project context not attached/u),
    ).toBeVisible();
    fireEvent.click(screen.getByText("Chat options"));
    fireEvent.click(
      screen.getByRole("button", { name: "Open Code conversation" }),
    );
    expect(onOpenLinkedProjectChat).toHaveBeenCalledOnce();
  });

  it("offers separate Google research without giving the chat runtime browser authority", () => {
    const onOpenBrowserResearch = vi.fn();
    render(
      <LocalChatWorkspace
        onRun={vi.fn()}
        onCancel={vi.fn()}
        onPrepareActionCard={vi.fn()}
        onApproveActionCard={vi.fn()}
        onRevokeActionCard={vi.fn()}
        onOpenLinkedProjectChat={vi.fn()}
        onOpenBrowserResearch={onOpenBrowserResearch}
      />,
    );

    fireEvent.click(screen.getByText("Chat options"));
    fireEvent.click(
      screen.getByRole("button", { name: "Research Google (read only)" }),
    );
    expect(onOpenBrowserResearch).toHaveBeenCalledOnce();
    expect(
      screen.getByText(/Chat & Cowork does not receive browser access/i),
    ).toBeVisible();
  });
});
