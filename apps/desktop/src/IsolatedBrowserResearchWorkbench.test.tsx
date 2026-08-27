import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

const id = "019a5800-0000-7000-8000-000000000001";
const snapshot = {
  schemaVersion: 1 as const,
  isolated: true as const,
  readOnly: true as const,
  adapter: "ephemeral-webkitgtk-research" as const,
  state: "prepared",
  projectId: id,
  taskId: null,
  attemptId: id,
  authorizationId: id,
  target: "https://google.com/" as const,
  origin: "https://google.com" as const,
  requestDigest: "a".repeat(64),
  expiresAtMs: 1,
  observationLimit: 512,
  observedAtMs: null,
  contentDigest: null,
  observedBytes: null,
  diagnostic: null,
  auditState: "reviewed; no browser launched",
};

const operations = vi.hoisted(() => ({
  load: vi.fn(() =>
    Promise.resolve({
      ...snapshot,
      state: "closed",
      projectId: null,
      attemptId: null,
      authorizationId: null,
      target: null,
      origin: null,
      requestDigest: null,
      expiresAtMs: null,
      observationLimit: null,
    }),
  ),
  prepare: vi.fn(() => Promise.resolve(snapshot)),
  confirm: vi.fn(() =>
    Promise.resolve({
      ...snapshot,
      state: "origin_drift",
      authorizationId: null,
      diagnostic: "redirect or origin drift blocked observation",
      auditState: "origin drift blocked; no observation retained",
    }),
  ),
  cancel: vi.fn(() => Promise.resolve({ ...snapshot, state: "cancelled" })),
  revoke: vi.fn(() => Promise.resolve({ ...snapshot, state: "revoked" })),
}));

vi.mock("./lib/bridge", () => ({
  loadBrowserResearch: operations.load,
  prepareBrowserResearch: operations.prepare,
  confirmBrowserResearch: operations.confirm,
  cancelBrowserResearch: operations.cancel,
  revokeBrowserResearch: operations.revoke,
}));

import { IsolatedBrowserResearchWorkbench } from "./IsolatedBrowserResearchWorkbench";

describe("IsolatedBrowserResearchWorkbench", () => {
  it("keeps Google research separate from chat and requires one explicit confirmation", async () => {
    render(
      <IsolatedBrowserResearchWorkbench projectId={id} onClose={vi.fn()} />,
    );
    expect(screen.getByRole("button", { name: "Close" })).toHaveFocus();
    await screen.findByText(/separate from Local Chat/i);
    fireEvent.click(
      screen.getByRole("button", { name: "Prepare Google review" }),
    );
    await waitFor(() => expect(operations.prepare).toHaveBeenCalledOnce());
    const confirm = screen.getByRole("button", { name: "Confirm once" });
    fireEvent.click(confirm);
    expect(confirm).toBeDisabled();
    await waitFor(() => expect(operations.confirm).toHaveBeenCalledOnce());
    expect(screen.getByRole("alert")).toHaveTextContent(/terminal/i);
  });
});
