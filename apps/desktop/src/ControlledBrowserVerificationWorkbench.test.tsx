import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

const id = "019a5800-0000-7000-8000-000000000001";
const snapshot = {
  schemaVersion: 1 as const,
  fictionalLocalOnly: true as const,
  readOnly: true as const,
  adapter: "ephemeral-webkitgtk-fixture" as const,
  state: "prepared",
  projectId: id,
  taskId: null,
  attemptId: id,
  authorizationId: id,
  target: "quireforge-fixture://verification/expected?assert=marker",
  origin: "quireforge-fixture://verification",
  assertion: "fixture-marker",
  requestDigest: "a".repeat(64),
  expiresAtMs: 1,
  evidenceDigest: null,
  visibleText: null,
  diagnostic: null,
  auditState: "reviewed; no adapter launched",
};

const operations = vi.hoisted(() => ({
  load: vi.fn(() =>
    Promise.resolve({
      ...snapshot,
      state: "closed",
      attemptId: null,
      authorizationId: null,
      projectId: null,
      target: null,
      origin: null,
      assertion: null,
      requestDigest: null,
      expiresAtMs: null,
    }),
  ),
  prepare: vi.fn(() => Promise.resolve(snapshot)),
  confirm: vi.fn(() =>
    Promise.resolve({
      ...snapshot,
      state: "verified",
      authorizationId: null,
      evidenceDigest: "b".repeat(64),
      visibleText: "fixture marker verified",
    }),
  ),
  cancel: vi.fn(() =>
    Promise.resolve({ ...snapshot, state: "cancelled", authorizationId: null }),
  ),
  revoke: vi.fn(() =>
    Promise.resolve({ ...snapshot, state: "revoked", authorizationId: null }),
  ),
}));

vi.mock("./lib/bridge", () => ({
  loadControlledBrowserVerification: operations.load,
  prepareControlledBrowserVerification: operations.prepare,
  confirmControlledBrowserVerification: operations.confirm,
  cancelControlledBrowserVerification: operations.cancel,
  revokeControlledBrowserVerification: operations.revoke,
}));

import { ControlledBrowserVerificationWorkbench } from "./ControlledBrowserVerificationWorkbench";

describe("ControlledBrowserVerificationWorkbench", () => {
  it("closes its dialog with Escape", () => {
    const onClose = vi.fn();
    render(
      <ControlledBrowserVerificationWorkbench
        projectId={id}
        onClose={onClose}
      />,
    );
    fireEvent.keyDown(window, { key: "Escape" });
    expect(onClose).toHaveBeenCalledOnce();
  });

  it("keeps fictional/local-only/read-only scope visible and prevents duplicate confirmation", async () => {
    render(
      <ControlledBrowserVerificationWorkbench
        projectId={id}
        onClose={vi.fn()}
      />,
    );
    expect(screen.getByRole("button", { name: "Close" })).toHaveFocus();
    await screen.findByText(/No real website, profile, account, credential/i);
    fireEvent.click(screen.getByRole("button", { name: "Prepare review" }));
    await waitFor(() => expect(operations.prepare).toHaveBeenCalledOnce());
    const confirm = screen.getByRole("button", { name: "Confirm once" });
    fireEvent.click(confirm);
    expect(confirm).toBeDisabled();
    await waitFor(() => expect(operations.confirm).toHaveBeenCalledOnce());
    expect(screen.getByText(/bounded fictional evidence/i)).toBeInTheDocument();
  });
});
