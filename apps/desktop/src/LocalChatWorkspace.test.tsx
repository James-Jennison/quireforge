import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { LocalChatWorkspace } from "./LocalChatWorkspace";

describe("LocalChatWorkspace", () => {
  it("submits only typed text and exposes no future capability controls", async () => {
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
    fireEvent.click(screen.getByRole("button", { name: "Send" }));
    expect(await screen.findByText("Local answer.")).toBeVisible();
    expect(onRun).toHaveBeenCalledWith({ message: "Hello" });
    expect(
      screen.queryByText(/attach project|use a source|provider|terminal|git/i),
    ).toBeNull();
  });
});
