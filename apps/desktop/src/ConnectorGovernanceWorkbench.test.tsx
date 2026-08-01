import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { ConnectorGovernanceWorkbench } from "./ConnectorGovernanceWorkbench";
import { connectorSnapshotSchema } from "./lib/connectorGovernance";
import { taskCatalogSchema } from "./lib/taskRecords";

const projectId = "019a5900-0000-7000-8000-000000000001";
const taskId = "019a5900-0000-7000-8000-000000000002";
const operationId = "019a5900-0000-7000-8000-000000000003";
const authorizationId = "019a5900-0000-7000-8000-000000000004";
const bindingId = "019a5900-0000-7000-8000-000000000005";
const planId = "019a5900-0000-7000-8000-000000000006";
const descriptorId = "019a57c0-0000-7000-8000-000000000001";
const digest = "a".repeat(64);

const tasks = taskCatalogSchema.parse({
  schemaVersion: 1,
  state: "ready",
  tasks: [
    {
      id: taskId,
      title: "Connector task",
      status: "active",
      archived: false,
      selectedPlanId: planId,
      planCount: 1,
      updatedAtMs: 1,
      cleanupEligible: false,
    },
  ],
  selectedTask: {
    id: taskId,
    title: "Connector task",
    status: "active",
    archived: false,
    selectedPlanId: planId,
    planCount: 1,
    updatedAtMs: 1,
    cleanupEligible: false,
  },
  plans: [{ id: planId, label: "Primary", position: 0, body: "Task plan." }],
  taskCount: 1,
  payloadBytes: 1,
  warning: false,
  diagnosticCode: null,
});

function snapshot(state: string, overrides: Record<string, unknown> = {}) {
  return connectorSnapshotSchema.parse({
    schemaVersion: 1,
    fictionalLocalOnly: true,
    state,
    projectId,
    taskId,
    operationId,
    authorizationId,
    operation: "mutation",
    diagnostic: null,
    bindingId,
    descriptorId,
    descriptorVersion: 1,
    descriptorSha256: digest,
    scopeDigest: digest,
    requestDigest: digest,
    expiresAtMs: 9999999999999,
    declaredCapabilities: ["read", "mutation"],
    grantedAuthority: ["mutation"],
    auditState: "review required; no fictional mutation dispatched",
    ...overrides,
  });
}

describe("ConnectorGovernanceWorkbench", () => {
  it("keeps fictional labeling and explicit read/mutation authority distinct", async () => {
    const prepare = vi.fn().mockResolvedValue(
      snapshot("succeeded", {
        operation: "read",
        authorizationId: null,
        grantedAuthority: ["read"],
      }),
    );
    render(
      <ConnectorGovernanceWorkbench
        projectId={projectId}
        onClose={vi.fn()}
        operations={{
          catalog: () =>
            Promise.resolve(
              snapshot("ready", {
                projectId: null,
                taskId: null,
                operationId: null,
                authorizationId: null,
                operation: null,
                bindingId: null,
                scopeDigest: null,
                requestDigest: null,
                expiresAtMs: null,
                grantedAuthority: [],
              }),
            ),
          tasks: () => Promise.resolve(tasks),
          prepare,
          confirm: vi.fn(),
          cancel: vi.fn(),
          revoke: vi.fn(),
        }}
      />,
    );
    await screen.findByText(/Fictional local-only connector fixture ready/i);
    expect(screen.getByRole("button", { name: "Close" })).toHaveFocus();
    expect(
      screen.getByText(/successful fictional read never authorizes mutation/i),
    ).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Run fictional read" }));
    await waitFor(() =>
      expect(prepare).toHaveBeenCalledWith({
        taskId,
        operation: "read",
        target: "mock-object-read",
      }),
    );
    expect(screen.getByText(/Granted: read/i)).toBeInTheDocument();
  });

  it("confirms a prepared mutation once and shows an ambiguous outcome as non-retryable", async () => {
    const prepare = vi.fn().mockResolvedValue(snapshot("prepared"));
    const confirm = vi.fn().mockResolvedValue(
      snapshot("outcome-unknown", {
        authorizationId: null,
        grantedAuthority: [],
        auditState: "ambiguous fictional outcome; automatic retry prohibited",
      }),
    );
    render(
      <ConnectorGovernanceWorkbench
        projectId={projectId}
        onClose={vi.fn()}
        operations={{
          catalog: () =>
            Promise.resolve(
              snapshot("ready", {
                projectId: null,
                taskId: null,
                operationId: null,
                authorizationId: null,
                operation: null,
                bindingId: null,
                scopeDigest: null,
                requestDigest: null,
                expiresAtMs: null,
                grantedAuthority: [],
              }),
            ),
          tasks: () => Promise.resolve(tasks),
          prepare,
          confirm,
          cancel: vi.fn(),
          revoke: vi.fn(),
        }}
      />,
    );
    await screen.findByText(/fixture ready/i);
    fireEvent.click(
      screen.getByRole("button", { name: "Prepare ambiguous fixture" }),
    );
    await waitFor(() => expect(prepare).toHaveBeenCalledTimes(1));
    fireEvent.click(screen.getByRole("button", { name: "Confirm once" }));
    await waitFor(() =>
      expect(confirm).toHaveBeenCalledWith({
        taskId,
        operationId,
        authorizationId,
      }),
    );
    expect(screen.getByRole("alert")).toHaveTextContent(
      /Automatic retry is prohibited/i,
    );
    expect(screen.getByRole("button", { name: "Confirm once" })).toBeDisabled();
  });
});
