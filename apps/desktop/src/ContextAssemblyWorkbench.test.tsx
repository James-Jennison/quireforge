import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { ContextAssemblyWorkbench } from "./ContextAssemblyWorkbench";

const bridge = vi.hoisted(() => ({
  acknowledgeContextAssemblyReview: vi.fn(),
  cancelContextAssembly: vi.fn(),
  cancelContextAssemblyLocalRuntime: vi.fn(),
  confirmContextAssembly: vi.fn(),
  loadDurableSources: vi.fn().mockResolvedValue({ sources: [] }),
  loadContextAssemblyLocalRuntimeAvailability: vi.fn().mockResolvedValue({
    schemaVersion: 1,
    localOnly: true,
    available: true,
    diagnostic: null,
  }),
  loadLocalReview: vi.fn(),
  loadTaskCatalog: vi.fn().mockResolvedValue({ tasks: [] }),
  prepareContextAssembly: vi.fn().mockResolvedValue({
    schemaVersion: 1,
    fictionalLocalOnly: true,
    sink: "fictional-local-context-sink-v1",
    state: "prepared",
    projectId: "019fbee6-476f-71b0-853c-f067657aa69c",
    taskId: null,
    bundleId: "019fbee6-476f-71b0-853c-f067657aa69b",
    authorizationId: null,
    bundleDigest:
      "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
    expiresAtMs: 1,
    items: [],
    totalBytes: 12,
    estimatedTokens: 3,
    exclusions: [],
    auditState: "prepared",
    diagnostic: null,
  }),
  reviewContextAssembly: vi.fn(),
  revokeContextAssembly: vi.fn(),
  runContextAssemblyLocalRuntime: vi.fn(),
}));

vi.mock("./lib/bridge", () => bridge);

describe("ContextAssemblyWorkbench", () => {
  it("keeps the one-time local action unavailable until runtime preflight completes", async () => {
    let resolveAvailability!: (value: {
      schemaVersion: 1;
      localOnly: true;
      available: true;
      diagnostic: null;
    }) => void;
    bridge.loadContextAssemblyLocalRuntimeAvailability.mockImplementationOnce(
      () =>
        new Promise((resolve) => {
          resolveAvailability = resolve;
        }),
    );
    const confirmed = {
      schemaVersion: 1,
      fictionalLocalOnly: true,
      sink: "fictional-local-context-sink-v1",
      state: "awaiting_confirmation",
      projectId: "019fbee6-476f-71b0-853c-f067657aa69c",
      taskId: null,
      bundleId: "019fbee6-476f-71b0-853c-f067657aa69b",
      authorizationId: "019fbee6-476f-71b0-853c-f067657aa69a",
      bundleDigest:
        "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
      expiresAtMs: 1,
      items: [],
      totalBytes: 12,
      estimatedTokens: 3,
      exclusions: [],
      auditState: "review acknowledged",
      diagnostic: null,
    } as const;
    bridge.prepareContextAssembly.mockResolvedValueOnce({
      ...confirmed,
      state: "prepared",
      authorizationId: null,
    });
    bridge.reviewContextAssembly.mockResolvedValueOnce({
      ...confirmed,
      state: "awaiting_review",
      authorizationId: null,
    });
    bridge.acknowledgeContextAssemblyReview.mockResolvedValueOnce(confirmed);

    render(
      <ContextAssemblyWorkbench
        projectId="019fbee6-476f-71b0-853c-f067657aa69c"
        onClose={() => undefined}
      />,
    );
    fireEvent.change(screen.getByLabelText(/explicit user instruction/i), {
      target: { value: "Summarize the reviewed request" },
    });
    fireEvent.click(screen.getByRole("button", { name: /prepare review/i }));
    await screen.findByRole("button", { name: /review prepared bundle/i });
    fireEvent.click(
      screen.getByRole("button", { name: /review prepared bundle/i }),
    );
    await screen.findByRole("button", { name: /acknowledge exact review/i });
    fireEvent.click(
      screen.getByRole("button", { name: /acknowledge exact review/i }),
    );

    const run = await screen.findByRole("button", {
      name: /run once with local-only model/i,
    });
    expect(run).toBeDisabled();
    expect(
      screen.getByText(/checking local runtime availability/i),
    ).toBeInTheDocument();
    expect(bridge.runContextAssemblyLocalRuntime).not.toHaveBeenCalled();

    resolveAvailability({
      schemaVersion: 1,
      localOnly: true,
      available: true,
      diagnostic: null,
    });
    await waitFor(() => expect(run).toBeEnabled());
  });

  it("shows a bounded unavailable state when runtime preflight fails", async () => {
    bridge.loadContextAssemblyLocalRuntimeAvailability.mockRejectedValueOnce(
      new Error("native availability unavailable"),
    );
    const confirmed = {
      schemaVersion: 1,
      fictionalLocalOnly: true,
      sink: "fictional-local-context-sink-v1",
      state: "awaiting_confirmation",
      projectId: "019fbee6-476f-71b0-853c-f067657aa69c",
      taskId: null,
      bundleId: "019fbee6-476f-71b0-853c-f067657aa69b",
      authorizationId: "019fbee6-476f-71b0-853c-f067657aa69a",
      bundleDigest:
        "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
      expiresAtMs: 1,
      items: [],
      totalBytes: 12,
      estimatedTokens: 3,
      exclusions: [],
      auditState: "review acknowledged",
      diagnostic: null,
    } as const;
    bridge.prepareContextAssembly.mockResolvedValueOnce({
      ...confirmed,
      state: "prepared",
      authorizationId: null,
    });
    bridge.reviewContextAssembly.mockResolvedValueOnce({
      ...confirmed,
      state: "awaiting_review",
      authorizationId: null,
    });
    bridge.acknowledgeContextAssemblyReview.mockResolvedValueOnce(confirmed);

    render(
      <ContextAssemblyWorkbench
        projectId="019fbee6-476f-71b0-853c-f067657aa69c"
        onClose={() => undefined}
      />,
    );
    fireEvent.change(screen.getByLabelText(/explicit user instruction/i), {
      target: { value: "Summarize the reviewed request" },
    });
    fireEvent.click(screen.getByRole("button", { name: /prepare review/i }));
    await screen.findByRole("button", { name: /review prepared bundle/i });
    fireEvent.click(
      screen.getByRole("button", { name: /review prepared bundle/i }),
    );
    await screen.findByRole("button", { name: /acknowledge exact review/i });
    fireEvent.click(
      screen.getByRole("button", { name: /acknowledge exact review/i }),
    );

    const run = await screen.findByRole("button", {
      name: /run once with local-only model/i,
    });
    await waitFor(() => expect(run).toBeDisabled());
    expect(
      screen.getByText(/local runtime availability could not be verified/i),
    ).toBeInTheDocument();
    expect(
      screen.queryByText(/checking local runtime availability/i),
    ).not.toBeInTheDocument();
    expect(bridge.runContextAssemblyLocalRuntime).not.toHaveBeenCalled();
  });

  it("shows the bounded running state until the one local attempt resolves", async () => {
    let complete!: (value: {
      schemaVersion: 1;
      localOnly: true;
      state: "cancelled";
      output: null;
      diagnostic: "cancelled";
      inputTokenLimit: 4096;
      outputTokenLimit: 512;
      deadlineSeconds: 60;
      memoryCeilingMib: 6144;
    }) => void;
    bridge.runContextAssemblyLocalRuntime.mockImplementationOnce(
      () =>
        new Promise((resolve) => {
          complete = resolve;
        }),
    );
    const confirmed = {
      schemaVersion: 1,
      fictionalLocalOnly: true,
      sink: "fictional-local-context-sink-v1",
      state: "awaiting_confirmation",
      projectId: "019fbee6-476f-71b0-853c-f067657aa69c",
      taskId: null,
      bundleId: "019fbee6-476f-71b0-853c-f067657aa69b",
      authorizationId: "019fbee6-476f-71b0-853c-f067657aa69a",
      bundleDigest:
        "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
      expiresAtMs: 1,
      items: [],
      totalBytes: 12,
      estimatedTokens: 3,
      exclusions: [],
      auditState: "review acknowledged",
      diagnostic: null,
    };
    bridge.prepareContextAssembly.mockResolvedValueOnce({
      ...confirmed,
      state: "prepared",
      authorizationId: null,
    });
    bridge.reviewContextAssembly.mockResolvedValueOnce({
      ...confirmed,
      state: "awaiting_review",
      authorizationId: null,
    });
    bridge.acknowledgeContextAssemblyReview.mockResolvedValueOnce(confirmed);

    render(
      <ContextAssemblyWorkbench
        projectId="019fbee6-476f-71b0-853c-f067657aa69c"
        onClose={() => undefined}
      />,
    );
    fireEvent.change(screen.getByLabelText(/explicit user instruction/i), {
      target: { value: "Summarize the reviewed request" },
    });
    fireEvent.click(screen.getByRole("button", { name: /prepare review/i }));
    await screen.findByRole("button", { name: /review prepared bundle/i });
    fireEvent.click(
      screen.getByRole("button", { name: /review prepared bundle/i }),
    );
    await screen.findByRole("button", { name: /acknowledge exact review/i });
    fireEvent.click(
      screen.getByRole("button", { name: /acknowledge exact review/i }),
    );
    await screen.findByRole("button", {
      name: /run once with local-only model/i,
    });
    fireEvent.click(
      screen.getByRole("button", { name: /run once with local-only model/i }),
    );

    expect(screen.getByLabelText(/local runtime result/i)).toHaveTextContent(
      /local-only attempt: running/i,
    );
    expect(screen.getByText(/no automatic retry/i)).toBeInTheDocument();
    expect(screen.getByLabelText(/local runtime result/i)).toHaveTextContent(
      /maximum 6144 MiB memory/i,
    );
    expect(screen.getByRole("button", { name: "Close" })).toBeDisabled();
    bridge.cancelContextAssemblyLocalRuntime.mockResolvedValueOnce(false);
    fireEvent.click(
      screen.getByRole("button", { name: /request cancellation/i }),
    );
    await waitFor(() =>
      expect(bridge.cancelContextAssemblyLocalRuntime).toHaveBeenCalledWith({
        bundleId: confirmed.bundleId,
      }),
    );
    expect(
      screen.getByText(
        /cancellation could not be requested; the one local-only attempt remains bounded/i,
      ),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: /request cancellation/i }),
    ).toBeEnabled();

    bridge.cancelContextAssemblyLocalRuntime.mockResolvedValueOnce(true);
    fireEvent.click(
      screen.getByRole("button", { name: /request cancellation/i }),
    );
    expect(
      screen.getByRole("button", { name: /cancellation requested/i }),
    ).toBeDisabled();
    await waitFor(() =>
      expect(
        screen.getByText(
          /cancellation requested for the one local-only attempt/i,
        ),
      ).toBeInTheDocument(),
    );

    complete({
      schemaVersion: 1,
      localOnly: true,
      state: "cancelled",
      output: null,
      diagnostic: "cancelled",
      inputTokenLimit: 4096,
      outputTokenLimit: 512,
      deadlineSeconds: 60,
      memoryCeilingMib: 6144,
    });
    await waitFor(() =>
      expect(screen.getByLabelText(/local runtime result/i)).toHaveTextContent(
        /local-only attempt: cancelled/i,
      ),
    );
    expect(screen.getByText(/local runtime: cancelled/i)).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Cancel" })).toBeDisabled();
    expect(screen.getByRole("button", { name: "Revoke" })).toBeDisabled();
    expect(screen.getByRole("button", { name: "Close" })).toBeEnabled();
  });

  it("labels the local-only authority boundary and starts with no selected source", () => {
    render(
      <ContextAssemblyWorkbench
        projectId="019fbee6-476f-71b0-853c-f067657aa69c"
        projectLabel="Release review"
        onClose={() => undefined}
      />,
    );
    expect(
      screen.getByRole("heading", { name: /governed context review/i }),
    ).toBeInTheDocument();
    expect(
      screen.getByText(/nothing is selected by default/i),
    ).toBeInTheDocument();
    expect(
      screen.getByText(/Project scope: Release review/i),
    ).toBeInTheDocument();
    expect(
      screen.queryByText("019fbee6-476f-71b0-853c-f067657aa69c"),
    ).not.toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: /prepare review/i }),
    ).toBeDisabled();
  });

  it("does not consume an acknowledged review when the local model is unavailable", async () => {
    bridge.runContextAssemblyLocalRuntime.mockClear();
    bridge.loadContextAssemblyLocalRuntimeAvailability.mockResolvedValueOnce({
      schemaVersion: 1,
      localOnly: true,
      available: false,
      diagnostic: "model-unavailable",
    });
    bridge.loadContextAssemblyLocalRuntimeAvailability.mockResolvedValueOnce({
      schemaVersion: 1,
      localOnly: true,
      available: true,
      diagnostic: null,
    });
    const confirmed = {
      schemaVersion: 1,
      fictionalLocalOnly: true,
      sink: "fictional-local-context-sink-v1",
      state: "awaiting_confirmation",
      projectId: "019fbee6-476f-71b0-853c-f067657aa69c",
      taskId: null,
      bundleId: "019fbee6-476f-71b0-853c-f067657aa69b",
      authorizationId: "019fbee6-476f-71b0-853c-f067657aa69a",
      bundleDigest:
        "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
      expiresAtMs: 1,
      items: [],
      totalBytes: 12,
      estimatedTokens: 3,
      exclusions: [],
      auditState: "review acknowledged",
      diagnostic: null,
    };
    bridge.prepareContextAssembly.mockResolvedValueOnce({
      ...confirmed,
      state: "prepared",
      authorizationId: null,
    });
    bridge.reviewContextAssembly.mockResolvedValueOnce({
      ...confirmed,
      state: "awaiting_review",
      authorizationId: null,
    });
    bridge.acknowledgeContextAssemblyReview.mockResolvedValueOnce(confirmed);

    render(
      <ContextAssemblyWorkbench
        projectId="019fbee6-476f-71b0-853c-f067657aa69c"
        onClose={() => undefined}
      />,
    );
    fireEvent.change(screen.getByLabelText(/explicit user instruction/i), {
      target: { value: "Summarize the reviewed request" },
    });
    fireEvent.click(screen.getByRole("button", { name: /prepare review/i }));
    await screen.findByRole("button", { name: /review prepared bundle/i });
    fireEvent.click(
      screen.getByRole("button", { name: /review prepared bundle/i }),
    );
    await screen.findByRole("button", { name: /acknowledge exact review/i });
    fireEvent.click(
      screen.getByRole("button", { name: /acknowledge exact review/i }),
    );
    await waitFor(() =>
      expect(
        screen.getByText(/no reviewed bundle can be consumed/i),
      ).toBeInTheDocument(),
    );
    expect(
      screen.getByRole("button", { name: /run once with local-only model/i }),
    ).toBeDisabled();
    expect(bridge.runContextAssemblyLocalRuntime).not.toHaveBeenCalled();

    const availabilityCalls =
      bridge.loadContextAssemblyLocalRuntimeAvailability.mock.calls.length;
    fireEvent.click(
      screen.getByRole("button", {
        name: /recheck local runtime availability/i,
      }),
    );
    await waitFor(() =>
      expect(
        bridge.loadContextAssemblyLocalRuntimeAvailability,
      ).toHaveBeenCalledTimes(availabilityCalls + 1),
    );
    await waitFor(() =>
      expect(
        screen.getByRole("button", { name: /run once with local-only model/i }),
      ).toBeEnabled(),
    );
    expect(bridge.runContextAssemblyLocalRuntime).not.toHaveBeenCalled();
  });

  it("runs one confirmed review in the local-only view and retains only its bounded result", async () => {
    const confirmed = {
      schemaVersion: 1,
      fictionalLocalOnly: true,
      sink: "fictional-local-context-sink-v1",
      state: "awaiting_confirmation",
      projectId: "019fbee6-476f-71b0-853c-f067657aa69c",
      taskId: null,
      bundleId: "019fbee6-476f-71b0-853c-f067657aa69b",
      authorizationId: "019fbee6-476f-71b0-853c-f067657aa69a",
      bundleDigest:
        "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
      expiresAtMs: 1,
      items: [],
      totalBytes: 12,
      estimatedTokens: 3,
      exclusions: [],
      auditState: "review acknowledged",
      diagnostic: null,
    };
    bridge.prepareContextAssembly.mockResolvedValueOnce({
      ...confirmed,
      state: "prepared",
      authorizationId: null,
    });
    bridge.reviewContextAssembly.mockResolvedValueOnce({
      ...confirmed,
      state: "awaiting_review",
      authorizationId: null,
    });
    bridge.acknowledgeContextAssemblyReview.mockResolvedValueOnce(confirmed);
    bridge.runContextAssemblyLocalRuntime.mockResolvedValueOnce({
      schemaVersion: 1,
      localOnly: true,
      state: "completed",
      output: "Local response",
      diagnostic: null,
      inputTokenLimit: 4096,
      outputTokenLimit: 512,
      deadlineSeconds: 60,
      memoryCeilingMib: 6144,
    });

    render(
      <ContextAssemblyWorkbench
        projectId="019fbee6-476f-71b0-853c-f067657aa69c"
        onClose={() => undefined}
      />,
    );
    fireEvent.change(screen.getByLabelText(/explicit user instruction/i), {
      target: { value: "Summarize the reviewed request" },
    });
    fireEvent.click(screen.getByRole("button", { name: /prepare review/i }));
    await waitFor(() =>
      expect(
        screen.getByRole("button", { name: /review prepared bundle/i }),
      ).toBeEnabled(),
    );
    fireEvent.click(
      screen.getByRole("button", { name: /review prepared bundle/i }),
    );
    await waitFor(() =>
      expect(
        screen.getByRole("button", { name: /acknowledge exact review/i }),
      ).toBeEnabled(),
    );
    fireEvent.click(
      screen.getByRole("button", { name: /acknowledge exact review/i }),
    );
    await waitFor(() =>
      expect(
        screen.getByRole("button", { name: /run once with local-only model/i }),
      ).toBeEnabled(),
    );
    fireEvent.click(
      screen.getByRole("button", { name: /run once with local-only model/i }),
    );

    await waitFor(() =>
      expect(
        screen.getByLabelText(/local runtime result/i),
      ).toBeInTheDocument(),
    );
    expect(bridge.runContextAssemblyLocalRuntime).toHaveBeenCalledWith({
      bundleId: confirmed.bundleId,
      authorizationId: confirmed.authorizationId,
      bundleDigest: confirmed.bundleDigest,
    });
    expect(screen.getByText("Local response")).toBeInTheDocument();
    expect(screen.getByLabelText(/local runtime result/i)).toHaveTextContent(
      /CPU-only; one attempt; maximum\s*4096 input tokens/i,
    );
  });

  it("keeps a bridge failure visible without offering a retry", async () => {
    const confirmed = {
      schemaVersion: 1,
      fictionalLocalOnly: true,
      sink: "fictional-local-context-sink-v1",
      state: "awaiting_confirmation",
      projectId: "019fbee6-476f-71b0-853c-f067657aa69c",
      taskId: null,
      bundleId: "019fbee6-476f-71b0-853c-f067657aa69b",
      authorizationId: "019fbee6-476f-71b0-853c-f067657aa69a",
      bundleDigest:
        "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
      expiresAtMs: 1,
      items: [],
      totalBytes: 12,
      estimatedTokens: 3,
      exclusions: [],
      auditState: "review acknowledged",
      diagnostic: null,
    };
    bridge.prepareContextAssembly.mockResolvedValueOnce({
      ...confirmed,
      state: "prepared",
      authorizationId: null,
    });
    bridge.reviewContextAssembly.mockResolvedValueOnce({
      ...confirmed,
      state: "awaiting_review",
      authorizationId: null,
    });
    bridge.acknowledgeContextAssemblyReview.mockResolvedValueOnce(confirmed);
    bridge.runContextAssemblyLocalRuntime.mockRejectedValueOnce(
      new Error("native command unavailable"),
    );

    render(
      <ContextAssemblyWorkbench
        projectId="019fbee6-476f-71b0-853c-f067657aa69c"
        onClose={() => undefined}
      />,
    );
    fireEvent.change(screen.getByLabelText(/explicit user instruction/i), {
      target: { value: "Summarize the reviewed request" },
    });
    fireEvent.click(screen.getByRole("button", { name: /prepare review/i }));
    await screen.findByRole("button", { name: /review prepared bundle/i });
    fireEvent.click(
      screen.getByRole("button", { name: /review prepared bundle/i }),
    );
    await screen.findByRole("button", { name: /acknowledge exact review/i });
    fireEvent.click(
      screen.getByRole("button", { name: /acknowledge exact review/i }),
    );
    await screen.findByRole("button", {
      name: /run once with local-only model/i,
    });
    fireEvent.click(
      screen.getByRole("button", { name: /run once with local-only model/i }),
    );

    await waitFor(() =>
      expect(screen.getByLabelText(/local runtime result/i)).toHaveTextContent(
        /local-only attempt: failed/i,
      ),
    );
    expect(
      screen.getByText(/local runtime: runtime-unavailable/i),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: /run once with local-only model/i }),
    ).toBeDisabled();
  });

  it("keeps a busy-reviewed bundle available for an explicit later run", async () => {
    bridge.runContextAssemblyLocalRuntime.mockClear();
    const confirmed = {
      schemaVersion: 1,
      fictionalLocalOnly: true,
      sink: "fictional-local-context-sink-v1",
      state: "awaiting_confirmation",
      projectId: "019fbee6-476f-71b0-853c-f067657aa69c",
      taskId: null,
      bundleId: "019fbee6-476f-71b0-853c-f067657aa69b",
      authorizationId: "019fbee6-476f-71b0-853c-f067657aa69a",
      bundleDigest:
        "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
      expiresAtMs: 1,
      items: [],
      totalBytes: 12,
      estimatedTokens: 3,
      exclusions: [],
      auditState: "review acknowledged",
      diagnostic: null,
    };
    bridge.prepareContextAssembly.mockResolvedValueOnce({
      ...confirmed,
      state: "prepared",
      authorizationId: null,
    });
    bridge.reviewContextAssembly.mockResolvedValueOnce({
      ...confirmed,
      state: "awaiting_review",
      authorizationId: null,
    });
    bridge.acknowledgeContextAssemblyReview.mockResolvedValueOnce(confirmed);
    bridge.runContextAssemblyLocalRuntime
      .mockResolvedValueOnce({
        schemaVersion: 1,
        localOnly: true,
        state: "failed",
        output: null,
        diagnostic: "runtime-busy",
        inputTokenLimit: 4096,
        outputTokenLimit: 512,
        deadlineSeconds: 60,
        memoryCeilingMib: 6144,
      })
      .mockResolvedValueOnce({
        schemaVersion: 1,
        localOnly: true,
        state: "completed",
        output: "Local response after manual retry",
        diagnostic: null,
        inputTokenLimit: 4096,
        outputTokenLimit: 512,
        deadlineSeconds: 60,
        memoryCeilingMib: 6144,
      });

    render(
      <ContextAssemblyWorkbench
        projectId="019fbee6-476f-71b0-853c-f067657aa69c"
        onClose={() => undefined}
      />,
    );
    fireEvent.change(screen.getByLabelText(/explicit user instruction/i), {
      target: { value: "Summarize the reviewed request" },
    });
    fireEvent.click(screen.getByRole("button", { name: /prepare review/i }));
    await screen.findByRole("button", { name: /review prepared bundle/i });
    fireEvent.click(
      screen.getByRole("button", { name: /review prepared bundle/i }),
    );
    await screen.findByRole("button", { name: /acknowledge exact review/i });
    fireEvent.click(
      screen.getByRole("button", { name: /acknowledge exact review/i }),
    );
    const run = await screen.findByRole("button", {
      name: /run once with local-only model/i,
    });
    fireEvent.click(run);

    await waitFor(() =>
      expect(screen.getByLabelText(/local runtime result/i)).toHaveTextContent(
        /local runtime: runtime-busy/i,
      ),
    );
    expect(
      screen.getByText(
        /no local attempt started because another one is active/i,
      ),
    ).toBeInTheDocument();
    expect(run).toBeEnabled();
    expect(bridge.runContextAssemblyLocalRuntime).toHaveBeenCalledTimes(1);

    fireEvent.click(run);
    await waitFor(() =>
      expect(
        screen.getByText("Local response after manual retry"),
      ).toBeInTheDocument(),
    );
    expect(bridge.runContextAssemblyLocalRuntime).toHaveBeenCalledTimes(2);
  });

  it("invalidates a prepared bundle when its selection changes", async () => {
    render(
      <ContextAssemblyWorkbench
        projectId="019fbee6-476f-71b0-853c-f067657aa69c"
        onClose={() => undefined}
      />,
    );

    fireEvent.change(screen.getByLabelText(/explicit user instruction/i), {
      target: { value: "Review this selection" },
    });
    fireEvent.click(screen.getByRole("button", { name: /prepare review/i }));

    await waitFor(() =>
      expect(
        screen.getByRole("button", { name: /review prepared bundle/i }),
      ).toBeEnabled(),
    );
    expect(
      screen.getByLabelText(/include bounded project\/task metadata/i),
    ).toBeEnabled();

    fireEvent.click(
      screen.getByLabelText(/include bounded project\/task metadata/i),
    );

    expect(
      screen.getByRole("button", { name: /review prepared bundle/i }),
    ).toBeDisabled();
    expect(
      screen.queryByLabelText(/prepared context summary/i),
    ).not.toBeInTheDocument();
    expect(
      screen.getByText(/selection changed\. prepare a new local-only review/i),
    ).toBeInTheDocument();
  });

  it("discards stale project-scoped source results after the project changes", async () => {
    let resolveFirst: ((value: { sources: unknown[] }) => void) | undefined;
    const first = new Promise<{ sources: unknown[] }>((resolve) => {
      resolveFirst = resolve;
    });
    bridge.loadDurableSources
      .mockImplementationOnce(() => first)
      .mockResolvedValueOnce({
        sources: [
          {
            sourceId: "current-source",
            title: "Current project source",
            sourceClass: "durable-manual-text",
            byteSize: 12,
          },
        ],
      });

    const view = render(
      <ContextAssemblyWorkbench
        projectId="019fbee6-476f-71b0-853c-f067657aa69c"
        onClose={() => undefined}
      />,
    );
    view.rerender(
      <ContextAssemblyWorkbench
        projectId="019fbee6-476f-71b0-853c-f067657aa69d"
        onClose={() => undefined}
      />,
    );

    await waitFor(() =>
      expect(
        screen.getByLabelText(/current project source/i),
      ).toBeInTheDocument(),
    );
    resolveFirst?.({
      sources: [
        {
          sourceId: "stale-source",
          title: "Stale project source",
          sourceClass: "durable-manual-text",
          byteSize: 12,
        },
      ],
    });

    await waitFor(() =>
      expect(
        screen.queryByLabelText(/stale project source/i),
      ).not.toBeInTheDocument(),
    );
  });

  it("discards a stale prepared bundle when the project scope changes", async () => {
    let resolvePrepare:
      | ((
          value: Awaited<ReturnType<typeof bridge.prepareContextAssembly>>,
        ) => void)
      | undefined;
    bridge.prepareContextAssembly.mockImplementationOnce(
      () =>
        new Promise((resolve) => {
          resolvePrepare = resolve;
        }),
    );
    const view = render(
      <ContextAssemblyWorkbench
        projectId="019fbee6-476f-71b0-853c-f067657aa69c"
        onClose={() => undefined}
      />,
    );

    fireEvent.change(screen.getByLabelText(/explicit user instruction/i), {
      target: { value: "Review this selection" },
    });
    fireEvent.click(screen.getByRole("button", { name: /prepare review/i }));

    view.rerender(
      <ContextAssemblyWorkbench
        projectId="019fbee6-476f-71b0-853c-f067657aa69d"
        onClose={() => undefined}
      />,
    );
    resolvePrepare?.({
      schemaVersion: 1,
      fictionalLocalOnly: true,
      sink: "fictional-local-context-sink-v1",
      state: "prepared",
      projectId: "019fbee6-476f-71b0-853c-f067657aa69c",
      taskId: null,
      bundleId: "019fbee6-476f-71b0-853c-f067657aa69b",
      authorizationId: null,
      bundleDigest:
        "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
      expiresAtMs: 1,
      items: [],
      totalBytes: 12,
      estimatedTokens: 3,
      exclusions: [],
      auditState: "prepared",
      diagnostic: null,
    });

    await waitFor(() =>
      expect(
        screen.getByRole("button", { name: /prepare review/i }),
      ).toBeDisabled(),
    );
    expect(
      screen.queryByLabelText(/prepared context summary/i),
    ).not.toBeInTheDocument();
  });

  it("resets scope-bound selections and prepared state when the project changes", async () => {
    const view = render(
      <ContextAssemblyWorkbench
        projectId="019fbee6-476f-71b0-853c-f067657aa69c"
        onClose={() => undefined}
      />,
    );

    fireEvent.change(screen.getByLabelText(/explicit user instruction/i), {
      target: { value: "Review this selection" },
    });
    fireEvent.click(screen.getByRole("button", { name: /prepare review/i }));

    await waitFor(() =>
      expect(
        screen.getByRole("button", { name: /review prepared bundle/i }),
      ).toBeEnabled(),
    );
    expect(screen.getByLabelText(/explicit user instruction/i)).toHaveValue(
      "Review this selection",
    );

    view.rerender(
      <ContextAssemblyWorkbench
        projectId="019fbee6-476f-71b0-853c-f067657aa69d"
        onClose={() => undefined}
      />,
    );

    await waitFor(() =>
      expect(screen.getByLabelText(/explicit user instruction/i)).toHaveValue(
        "",
      ),
    );
    expect(
      screen.queryByLabelText(/prepared context summary/i),
    ).not.toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: /prepare review/i }),
    ).toBeDisabled();
    expect(
      screen.getByText(/nothing is selected by default/i),
    ).toBeInTheDocument();
  });
});
