import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { MockInferenceWorkbench } from "./MockInferenceWorkbench";
import {
  mockInferenceCatalogSchema,
  mockInferenceSnapshotSchema,
} from "./lib/mockInference";
import { taskCatalogSchema } from "./lib/taskRecords";

const id = "019a5900-0000-7000-8000-000000000001";
const attemptId = "019a5900-0000-7000-8000-000000000002";
const authorizationId = "019a5900-0000-7000-8000-000000000003";
const planId = "019a5900-0000-7000-8000-000000000004";
const digest = "a".repeat(64);
const catalog = mockInferenceCatalogSchema.parse({
  schemaVersion: 1,
  profiles: [
    {
      id: "lantern-stream",
      providerLabel: "Fictional Lantern",
      endpointLabel: "Local fixture endpoint",
      modelLabel: "Lantern Text Fixture",
      adapterLabel: "Lantern fixture adapter",
      scenario: "streamed-text",
      descriptorSha256: digest,
    },
  ],
});
const taskCatalog = taskCatalogSchema.parse({
  schemaVersion: 1,
  state: "ready",
  tasks: [
    {
      id,
      title: "Durable task",
      status: "active",
      archived: false,
      selectedPlanId: planId,
      planCount: 1,
      updatedAtMs: 1,
      cleanupEligible: false,
    },
  ],
  selectedTask: {
    id,
    title: "Durable task",
    status: "active",
    archived: false,
    selectedPlanId: planId,
    planCount: 1,
    updatedAtMs: 1,
    cleanupEligible: false,
  },
  plans: [
    { id: planId, label: "Primary", position: 0, body: "Visible task plan." },
  ],
  taskCount: 1,
  payloadBytes: 1,
  warning: false,
  diagnosticCode: null,
});
function snapshot(state: string, events: unknown[] = []) {
  return mockInferenceSnapshotSchema.parse({
    schemaVersion: 1,
    mockOnly: true,
    attemptId,
    state,
    diagnostic: null,
    destination: {
      providerId: id,
      endpointId: id,
      modelId: id,
      adapterId: id,
      descriptorSha256: digest,
      capabilityProfileSha256: digest,
    },
    manifest: {
      id,
      sha256: digest,
      inputSha256: digest,
      itemCount: 1,
      inputCharCount: 12,
      exclusions: ["ambient-context"],
      retention: "transient-local-mock",
      expiresAtTick: 3,
      state: "ready",
    },
    lease: {
      credentialReferenceId: id,
      leaseId: id,
      accountReference: "fictional-account-reference",
      scopes: ["mock-inference-submit"],
      state: "issued",
      expiresAtTick: 3,
    },
    authorization: {
      id: authorizationId,
      bindingSha256: digest,
      state:
        state === "ready"
          ? "pending"
          : state === "authorized"
            ? "authorized"
            : "consumed",
      expiresAtTick: 3,
    },
    events,
    usage: null,
    evidence: [],
  });
}

describe("MockInferenceWorkbench", () => {
  it("requires an explicit prepare, authorization, and submission while keeping mock labeling visible", async () => {
    const prepare = vi.fn().mockResolvedValue(snapshot("ready"));
    const authorize = vi.fn().mockResolvedValue(snapshot("authorized"));
    const submit = vi.fn().mockResolvedValue(
      snapshot("completed", [
        {
          id,
          sequence: 1,
          kind: "text-delta",
          text: "fixture output",
          structuredState: null,
          sha256: digest,
        },
      ]),
    );
    render(
      <MockInferenceWorkbench
        onClose={vi.fn()}
        operations={{
          catalog: () => Promise.resolve(catalog),
          tasks: () => Promise.resolve(taskCatalog),
          prepare,
          authorize,
          submit,
          cancel: vi.fn(),
        }}
      />,
    );
    await screen.findByText(/Fictional local mock inference is ready/i);
    fireEvent.change(screen.getByLabelText("Bounded authored input"), {
      target: { value: "Visible input" },
    });
    fireEvent.click(
      screen.getByRole("button", { name: "Prepare local mock review" }),
    );
    await waitFor(() => expect(prepare).toHaveBeenCalledTimes(1));
    expect(
      screen.getByText(/Only this visible text is selected/i),
    ).toBeInTheDocument();
    fireEvent.click(
      screen.getByRole("button", {
        name: "Authorize one local mock submission",
      }),
    );
    await waitFor(() => expect(authorize).toHaveBeenCalledTimes(1));
    fireEvent.click(
      screen.getByRole("button", { name: "Submit deterministic mock" }),
    );
    await waitFor(() => expect(submit).toHaveBeenCalledTimes(1));
    expect(screen.getByText(/fixture output/i)).toBeInTheDocument();
  });
});
