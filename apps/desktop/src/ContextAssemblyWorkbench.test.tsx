import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { ContextAssemblyWorkbench } from "./ContextAssemblyWorkbench";

const bridge = vi.hoisted(() => ({
  acknowledgeContextAssemblyReview: vi.fn(),
  cancelContextAssembly: vi.fn(),
  confirmContextAssembly: vi.fn(),
  loadDurableSources: vi.fn().mockResolvedValue({ sources: [] }),
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
}));

vi.mock("./lib/bridge", () => bridge);

describe("ContextAssemblyWorkbench", () => {
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
});
