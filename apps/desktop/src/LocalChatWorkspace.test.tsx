import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { LocalChatWorkspace } from "./LocalChatWorkspace";

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
    render(<LocalChatWorkspace onRun={onRun} onCancel={vi.fn()} />);
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
    expect(onRun).toHaveBeenCalledWith({ message: "Hello" });
    expect(screen.getByText("You")).toBeVisible();
    expect(screen.getByText("QuireForge")).toBeVisible();
    expect(
      screen.getByText("Enter to send · Shift+Enter for a new line"),
    ).toBeVisible();
    expect(
      screen.queryByText(/attach project|use a source|provider|terminal|git/i),
    ).toBeNull();
  });

  it("keeps Shift+Enter available for a multiline message", () => {
    const onRun = vi.fn();
    render(<LocalChatWorkspace onRun={onRun} onCancel={vi.fn()} />);
    const composer = screen.getByRole("textbox", {
      name: "Local chat message",
    });
    fireEvent.change(composer, { target: { value: "First line" } });
    fireEvent.keyDown(composer, { key: "Enter", shiftKey: true });

    expect(onRun).not.toHaveBeenCalled();
  });
});
