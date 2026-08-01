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
      capabilityProfileSha256: digest,
    },
    {
      id: "ember-failure",
      providerLabel: "Fictional Lantern",
      endpointLabel: "Local fixture endpoint",
      modelLabel: "Lantern Text Fixture",
      adapterLabel: "registry fixture adapter",
      scenario: "failure",
      descriptorSha256: digest,
      capabilityProfileSha256: digest,
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
    {
      id: "019a5900-0000-7000-8000-000000000005",
      title: "Second durable task",
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
          poll: vi.fn(),
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

  it.each([
    ["input", "Bounded authored input", "Changed visible input"],
    ["task", "Durable task", "019a5900-0000-7000-8000-000000000005"],
    ["destination", "Fictional destination", "ember-failure"],
  ])(
    "invalidates the visible review when the bound %s changes",
    async (_, label, value) => {
      const prepare = vi.fn().mockResolvedValue(snapshot("ready"));
      render(
        <MockInferenceWorkbench
          onClose={vi.fn()}
          operations={{
            catalog: () => Promise.resolve(catalog),
            tasks: () => Promise.resolve(taskCatalog),
            prepare,
            authorize: vi.fn(),
            submit: vi.fn(),
            cancel: vi.fn(),
            poll: vi.fn(),
          }}
        />,
      );
      await screen.findByText(/ready for an explicit review/i);
      const input = screen.getByLabelText("Bounded authored input");
      fireEvent.change(input, { target: { value: "First visible input" } });
      fireEvent.click(
        screen.getByRole("button", { name: "Prepare local mock review" }),
      );
      await screen.findByText("Exact local review");
      if (label === "Bounded authored input") {
        fireEvent.change(input, { target: { value } });
      } else {
        fireEvent.change(screen.getByLabelText(label), { target: { value } });
      }
      expect(screen.queryByText("Exact local review")).not.toBeInTheDocument();
      expect(screen.getByText(/reviewed binding changed/i)).toBeInTheDocument();
      expect(
        screen.queryByRole("button", {
          name: "Authorize one local mock submission",
        }),
      ).not.toBeInTheDocument();
    },
  );

  it("renders bounded streaming progress and keeps cancellation confirmation separate", async () => {
    const authorize = vi.fn().mockResolvedValue(snapshot("authorized"));
    const submit = vi.fn().mockResolvedValue(snapshot("submitted"));
    const cancel = vi.fn().mockResolvedValue(
      snapshot("cancelling", [
        {
          id,
          sequence: 1,
          kind: "cancellation-requested",
          text: null,
          structuredState: null,
          sha256: digest,
        },
      ]),
    );
    const poll = vi.fn().mockResolvedValue(
      snapshot("cancelled", [
        {
          id,
          sequence: 1,
          kind: "cancellation-requested",
          text: null,
          structuredState: null,
          sha256: digest,
        },
        {
          id: attemptId,
          sequence: 2,
          kind: "terminal",
          text: "cancelled",
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
          prepare: vi.fn().mockResolvedValue(snapshot("ready")),
          authorize,
          submit,
          cancel,
          poll,
        }}
      />,
    );
    await screen.findByText(/ready for an explicit review/i);
    fireEvent.change(screen.getByLabelText("Bounded authored input"), {
      target: { value: "Visible input" },
    });
    fireEvent.click(
      screen.getByRole("button", { name: "Prepare local mock review" }),
    );
    await screen.findByText("Exact local review");
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
    expect(
      screen.getByRole("button", {
        name: "Continue bounded local fixture stream",
      }),
    ).toBeEnabled();
    fireEvent.click(screen.getByRole("button", { name: "Cancel" }));
    await waitFor(() => expect(cancel).toHaveBeenCalledTimes(1));
    expect(screen.getByText(/cancellation-requested/i)).toBeInTheDocument();
    fireEvent.click(
      screen.getByRole("button", {
        name: "Continue bounded local fixture stream",
      }),
    );
    await waitFor(() => expect(poll).toHaveBeenCalledTimes(1));
    expect(screen.getAllByText(/cancelled/i).length).toBeGreaterThan(0);
  });

  it("retains only prior evidence before a fresh retry review", async () => {
    render(
      <MockInferenceWorkbench
        onClose={vi.fn()}
        operations={{
          catalog: () => Promise.resolve(catalog),
          tasks: () => Promise.resolve(taskCatalog),
          prepare: vi.fn().mockResolvedValue(snapshot("ready")),
          authorize: vi.fn(),
          submit: vi.fn(),
          cancel: vi.fn(),
          poll: vi.fn(),
        }}
      />,
    );
    await screen.findByText(/ready for an explicit review/i);
    fireEvent.change(screen.getByLabelText("Bounded authored input"), {
      target: { value: "Visible input" },
    });
    fireEvent.click(
      screen.getByRole("button", { name: "Prepare local mock review" }),
    );
    await screen.findByText("Exact local review");
    fireEvent.click(
      screen.getByRole("button", {
        name: "Prepare fresh retry or regeneration",
      }),
    );
    expect(screen.getByText("Prior local attempt")).toBeInTheDocument();
    expect(
      screen.getByText(
        /no lease, authorization, event sequence, or result is reused/i,
      ),
    ).toBeInTheDocument();
  });
});
