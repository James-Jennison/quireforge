import {
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

const advisorDraftBridge = vi.hoisted(() => ({
  createAdvisorDraft: vi.fn(),
  decideAdvisorDraft: vi.fn(),
  dispatchAdvisorOnce: vi.fn(),
  loadAdvisorTextAttachment: vi.fn().mockResolvedValue({
    schemaVersion: 1,
    state: "empty",
    attachment: null,
    confirmationState: null,
    diagnosticCode: null,
  }),
  pickAdvisorTextAttachment: vi.fn(),
  cancelAdvisorTextAttachment: vi.fn(),
  loadAdvisorImageAttachment: vi.fn().mockResolvedValue({
    schemaVersion: 1,
    state: "empty",
    attachment: null,
    previewDataUrl: null,
    confirmationState: null,
    diagnosticCode: null,
  }),
  pickAdvisorImageAttachment: vi.fn(),
  cancelAdvisorImageAttachment: vi.fn(),
  saveAdvisorTextExport: vi.fn(),
}));

vi.mock("./lib/bridge", () => advisorDraftBridge);

import { AdvisorWorkspace } from "./AdvisorWorkspace";
import { scaffoldAdvisorConversation } from "./lib/advisorConversation";
import { scaffoldCodexAuth } from "./lib/auth";
import { scaffoldCodexRuntime } from "./lib/codex";
import {
  parseAdvisorSelectedProjectStateSnapshot,
  advisorWorkspaceSnapshotSchema,
} from "./lib/advisorWorkspace";

const advisorWorkspaceFixture = advisorWorkspaceSnapshotSchema.parse({
  schemaVersion: 1,
  conversationCount: 1,
  contextReferenceCount: 1,
  proposalCount: 1,
  contextSummaries: [
    { kind: "project-state", trust: "verified", freshness: "current" },
  ],
  proposalSummaries: [{ state: "draft", requiresExplicitApproval: true }],
});

const selectedProjectStateFixture = parseAdvisorSelectedProjectStateSnapshot({
  schemaVersion: 1,
  sourceKind: "project-state",
  selectedAtMs: 1,
  trust: "verified",
  freshness: "current",
  provenanceSource: "project-state-snapshot",
  worktree: "clean",
  diagnosticCount: 0,
});

const props = {
  availability: "native" as const,
  snapshot: advisorWorkspaceFixture,
  selectedProjectState: null,
  selectionState: "idle" as const,
  canSelectProjectState: true,
  onRequestProjectState: vi.fn(),
  onConfirmProjectState: vi.fn(),
  onCancelProjectState: vi.fn(),
  onRemoveProjectState: vi.fn(),
  auth: {
    ...scaffoldCodexAuth,
    state: "authenticated" as const,
    accountKind: "chatgpt" as const,
  },
  runtime: scaffoldCodexRuntime,
  conversation: scaffoldAdvisorConversation,
  conversationBusy: false,
  selectedProjectId: null,
  onConversationStart: vi.fn().mockResolvedValue(scaffoldAdvisorConversation),
  onConversationPoll: vi.fn(),
  onConversationInterrupt: vi.fn(),
  onDispatch: vi.fn(),
  onOpenExecution: vi.fn(),
};

describe("AdvisorWorkspace", () => {
  it("renders a managed read-only composer without executable controls", () => {
    render(
      <AdvisorWorkspace
        {...props}
        snapshot={{
          ...advisorWorkspaceFixture,
          conversationCount: 0,
          contextReferenceCount: 0,
          proposalCount: 0,
          contextSummaries: [],
          proposalSummaries: [],
        }}
      />,
    );

    expect(
      screen.getByRole("heading", {
        name: "Read-only planning, without execution.",
      }),
    ).toBeInTheDocument();
    expect(screen.getByText("No Advisor metadata yet.")).toBeInTheDocument();
    expect(
      screen.getByRole("textbox", { name: "Advisor message" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "Send to Advisor" }),
    ).toBeInTheDocument();
    expect(
      screen.getByText(/Advisor is read-only: no commands/u),
    ).toBeInTheDocument();
    expect(screen.getByRole("status")).toHaveTextContent(
      "Enter a message to send.",
    );
    expect(
      screen.queryByRole("button", { name: /approve|dispatch|terminal/u }),
    ).not.toBeInTheDocument();
  });

  it("shows only safe reference summaries for valid metadata", () => {
    render(<AdvisorWorkspace {...props} />);
    const summaries = within(
      screen.getByRole("list", { name: "Advisor metadata summaries" }),
    );
    const [contextSummary, proposalSummary] =
      summaries.getAllByRole("listitem");
    expect(contextSummary).toHaveTextContent(
      "project-state: verified, current",
    );
    expect(proposalSummary).toHaveTextContent(
      "Proposal digest: draft, explicit approval required.",
    );
    expect(
      screen.queryByText("advisor-thread-fixture-01"),
    ).not.toBeInTheDocument();
    expect(screen.queryByText(/gpt-5.6-terra/i)).not.toBeInTheDocument();
  });

  it("requires explicit confirmation before selecting a snapshot", () => {
    const onRequestProjectState = vi.fn();
    render(
      <AdvisorWorkspace
        {...props}
        onRequestProjectState={onRequestProjectState}
      />,
    );
    fireEvent.click(
      screen.getByRole("button", {
        name: "Select current Project State snapshot",
      }),
    );
    expect(onRequestProjectState).toHaveBeenCalledOnce();
  });

  it("requires a second per-send confirmation before including selected context", () => {
    const onConversationStart = vi
      .fn()
      .mockResolvedValue(scaffoldAdvisorConversation);
    render(
      <AdvisorWorkspace
        {...props}
        selectedProjectState={selectedProjectStateFixture}
        selectedProjectId="018f0000-0000-7000-8000-000000000001"
        onConversationStart={onConversationStart}
      />,
    );
    expect(
      screen.getByText(/Temporary safe summary: current, clean/u),
    ).toBeInTheDocument();
    fireEvent.change(screen.getByRole("textbox", { name: "Advisor message" }), {
      target: { value: "Prepare a safe milestone plan." },
    });
    fireEvent.click(
      screen.getByRole("checkbox", {
        name: /include the selected temporary project state summary/i,
      }),
    );
    fireEvent.click(screen.getByRole("button", { name: "Send to Advisor" }));
    expect(onConversationStart).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole("button", { name: "Confirm inclusion" }));
    expect(onConversationStart).toHaveBeenCalledWith({
      prompt: "Prepare a safe milestone plan.",
      projectId: "018f0000-0000-7000-8000-000000000001",
      attachmentId: null,
      attachmentManifestSha256: null,
      attachmentConfirmation: null,
      imageAttachmentId: null,
      imageAttachmentManifestSha256: null,
      imageAttachmentConfirmation: null,
    });
  });

  it("keeps an API-key account unavailable", () => {
    render(
      <AdvisorWorkspace
        {...props}
        auth={{ ...props.auth, accountKind: "api-key" }}
      />,
    );
    expect(
      screen.getByRole("button", { name: "Send to Advisor" }),
    ).toBeDisabled();
    expect(
      screen.getByText(
        "Advisor is unavailable until managed ChatGPT browser sign-in is complete.",
      ),
    ).toBeInTheDocument();
  });

  it("explains a managed-Codex protocol failure without exposing diagnostics", () => {
    render(
      <AdvisorWorkspace
        {...props}
        conversation={{
          ...scaffoldAdvisorConversation,
          state: "unavailable",
          diagnosticCode: "protocol-invalid",
        }}
      />,
    );

    expect(screen.getByRole("alert")).toHaveTextContent(
      "Advisor could not complete the managed Codex conversation safely. Try again later.",
    );
  });

  it("explains a thread-start rejection without exposing server diagnostics", () => {
    render(
      <AdvisorWorkspace
        {...props}
        conversation={{
          ...scaffoldAdvisorConversation,
          state: "unavailable",
          diagnosticCode: "thread-start-rejected",
        }}
      />,
    );

    expect(screen.getByRole("alert")).toHaveTextContent(
      "Advisor could not start a managed conversation with its read-only settings. Try again later.",
    );
    expect(
      screen.queryByText(/server error|rpc|raw diagnostic/i),
    ).not.toBeInTheDocument();
  });

  it("coalesces consecutive streamed assistant fragments into readable messages", () => {
    const { container } = render(
      <AdvisorWorkspace
        {...props}
        conversation={{
          ...scaffoldAdvisorConversation,
          state: "completed",
          events: [
            { type: "agent-message-delta", sequence: 1, delta: "Hello" },
            { type: "agent-message-delta", sequence: 2, delta: " world" },
            { type: "agent-message-delta", sequence: 3, delta: "." },
            {
              type: "reasoning-summary-delta",
              sequence: 4,
              delta: "Safe summary.",
            },
            { type: "agent-message-delta", sequence: 5, delta: "Next" },
            { type: "agent-message-delta", sequence: 6, delta: " message." },
          ],
        }}
      />,
    );

    const messages = container.querySelectorAll(".conversation-event__message");
    expect(messages).toHaveLength(2);
    expect(messages[0]).toHaveTextContent("Hello world.");
    expect(messages[1]).toHaveTextContent("Next message.");
    expect(screen.getByText("Reasoning summary")).toBeInTheDocument();
    expect(container.querySelector(".conversation-events")).toHaveAttribute(
      "aria-live",
      "polite",
    );
    expect(
      screen.getByRole("log", { name: "Active Advisor conversation" }),
    ).toHaveAttribute("tabindex", "0");
  });

  it("clears transient composer state and attachments when a mode reset is accepted", async () => {
    const { rerender } = render(<AdvisorWorkspace {...props} resetToken={0} />);
    const prompt = screen.getByRole("textbox", { name: "Advisor message" });
    fireEvent.change(prompt, { target: { value: "Do not transfer this." } });
    expect(prompt).toHaveValue("Do not transfer this.");

    rerender(<AdvisorWorkspace {...props} resetToken={1} />);

    await waitFor(() => expect(prompt).toHaveValue(""));
    expect(advisorDraftBridge.cancelAdvisorTextAttachment).toHaveBeenCalled();
    expect(advisorDraftBridge.cancelAdvisorImageAttachment).toHaveBeenCalled();
  });

  it("keeps the capability notice separate when a sendable message is entered", () => {
    render(<AdvisorWorkspace {...props} />);
    fireEvent.change(screen.getByRole("textbox", { name: "Advisor message" }), {
      target: { value: "Review the milestone boundary." },
    });

    expect(
      screen.getByRole("button", { name: "Send to Advisor" }),
    ).toBeEnabled();
    expect(
      screen.getByText(
        /Project State and one text\/data file or one image are optional and require confirmation/u,
      ),
    ).toBeInTheDocument();
    expect(
      screen.queryByText("Enter a message to send."),
    ).not.toBeInTheDocument();
  });

  it("requires explicit confirmation before sending one bounded text attachment", async () => {
    const attachment = {
      schemaVersion: 1,
      state: "ready" as const,
      attachment: {
        attachmentId: "018f0000-0000-7000-8000-000000000099",
        displayName: "notes.md",
        contentCategory: "text-data" as const,
        contentType: "markdown" as const,
        byteSize: 42,
        sha256: "a".repeat(64),
        projection: {
          kind: "normalized-utf8-text" as const,
          normalizedByteSize: 42,
        },
        disposal: "transient-memory-one-send" as const,
      },
      confirmationState: "confirmation-required" as const,
      diagnosticCode: null,
    };
    advisorDraftBridge.pickAdvisorTextAttachment.mockResolvedValueOnce(
      attachment,
    );
    const onConversationStart = vi
      .fn()
      .mockResolvedValue(scaffoldAdvisorConversation);
    render(
      <AdvisorWorkspace {...props} onConversationStart={onConversationStart} />,
    );
    fireEvent.click(
      screen.getByRole("button", { name: "Attach text or data file" }),
    );
    await screen.findByText(/Ready: notes.md/u);
    fireEvent.change(screen.getByRole("textbox", { name: "Advisor message" }), {
      target: { value: "Review the attached notes." },
    });
    fireEvent.click(screen.getByRole("button", { name: "Send to Advisor" }));
    expect(onConversationStart).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole("button", { name: "Confirm inclusion" }));
    expect(onConversationStart).toHaveBeenCalledWith({
      prompt: "Review the attached notes.",
      projectId: null,
      attachmentId: attachment.attachment.attachmentId,
      attachmentManifestSha256: attachment.attachment.sha256,
      attachmentConfirmation: "confirmed-for-single-send",
      imageAttachmentId: null,
      imageAttachmentManifestSha256: null,
      imageAttachmentConfirmation: null,
    });
  });

  it("labels Phase A draft approval as non-executable", () => {
    render(
      <AdvisorWorkspace
        {...props}
        targetProjectId="018f0000-0000-7000-8000-000000000002"
        conversation={{
          ...scaffoldAdvisorConversation,
          state: "completed",
          conversationId: "018f0000-0000-7000-8000-000000000003",
        }}
      />,
    );
    expect(
      screen.getByRole("textbox", { name: "Editable draft" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "Create approval draft" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "Copy draft" }),
    ).toBeInTheDocument();
    expect(
      screen.getByText(
        /cannot start Codex, run a command, or change a project/u,
      ),
    ).toBeInTheDocument();
  });

  it("binds the selected safe projection and invalidates approval when its target changes", async () => {
    advisorDraftBridge.createAdvisorDraft.mockResolvedValue({
      proposalId: "018f0000-0000-7000-8000-000000000004",
      state: "draft",
      expiresAtMs: 1771235467000,
      dispatchAvailable: false,
    });
    const conversation = {
      ...scaffoldAdvisorConversation,
      state: "completed" as const,
      conversationId: "018f0000-0000-7000-8000-000000000003",
    };
    const { rerender } = render(
      <AdvisorWorkspace
        {...props}
        conversation={conversation}
        targetProjectId="018f0000-0000-7000-8000-000000000002"
        selectedProjectId="018f0000-0000-7000-8000-000000000002"
        selectedProjectState={selectedProjectStateFixture}
      />,
    );
    fireEvent.click(
      screen.getByRole("checkbox", {
        name: /include the selected temporary project state summary/i,
      }),
    );
    fireEvent.change(screen.getByRole("textbox", { name: "Editable draft" }), {
      target: { value: "Prepare a bounded implementation plan." },
    });
    fireEvent.click(
      screen.getByRole("button", { name: "Create approval draft" }),
    );

    await waitFor(() =>
      expect(advisorDraftBridge.createAdvisorDraft).toHaveBeenCalledWith(
        expect.objectContaining({
          selectedProjectState: selectedProjectStateFixture,
        }),
      ),
    );
    expect(await screen.findByText(/Draft is draft/u)).toBeInTheDocument();

    rerender(
      <AdvisorWorkspace
        {...props}
        conversation={conversation}
        targetProjectId="018f0000-0000-7000-8000-000000000005"
        selectedProjectId="018f0000-0000-7000-8000-000000000005"
        selectedProjectState={selectedProjectStateFixture}
      />,
    );
    await waitFor(() =>
      expect(screen.queryByText(/Draft is draft/u)).not.toBeInTheDocument(),
    );
  });

  it("re-sends the complete transient binding when approving a draft", async () => {
    advisorDraftBridge.createAdvisorDraft.mockResolvedValue({
      proposalId: "018f0000-0000-7000-8000-000000000006",
      state: "draft",
      expiresAtMs: 1771235467000,
      dispatchAvailable: false,
    });
    advisorDraftBridge.decideAdvisorDraft.mockResolvedValue({
      proposalId: "018f0000-0000-7000-8000-000000000006",
      state: "approved",
      expiresAtMs: 1771235467000,
      dispatchAvailable: false,
    });
    render(
      <AdvisorWorkspace
        {...props}
        targetProjectId="018f0000-0000-7000-8000-000000000002"
        conversation={{
          ...scaffoldAdvisorConversation,
          state: "completed",
          conversationId: "018f0000-0000-7000-8000-000000000003",
        }}
      />,
    );
    fireEvent.change(screen.getByRole("textbox", { name: "Editable draft" }), {
      target: { value: "Review the bounded handoff." },
    });
    fireEvent.click(
      screen.getByRole("button", { name: "Create approval draft" }),
    );
    await screen.findByRole("button", { name: "Approve draft" });
    fireEvent.click(screen.getByRole("button", { name: "Approve draft" }));
    await waitFor(() =>
      expect(advisorDraftBridge.decideAdvisorDraft).toHaveBeenCalledWith({
        proposalId: "018f0000-0000-7000-8000-000000000006",
        decision: "approved",
        binding: {
          advisorConversationId: "018f0000-0000-7000-8000-000000000003",
          targetProjectId: "018f0000-0000-7000-8000-000000000002",
          prompt: "Review the bounded handoff.",
          selectedProjectState: null,
          declaredCapabilities: ["workspace-write"],
          requestedModel: "gpt-5.6-sol",
          requestedReasoningEffort: "low",
        },
      }),
    );
  });

  it("dispatches an approved draft once through the supplied execution boundary", async () => {
    const onDispatch = vi.fn().mockResolvedValue({
      proposalId: "018f0000-0000-7000-8000-000000000006",
      state: "started",
      executionConversationId: "018f0000-0000-7000-8000-000000000007",
    });
    advisorDraftBridge.createAdvisorDraft.mockResolvedValue({
      proposalId: "018f0000-0000-7000-8000-000000000006",
      state: "draft",
      expiresAtMs: 1771235467000,
      dispatchAvailable: false,
    });
    advisorDraftBridge.decideAdvisorDraft.mockResolvedValue({
      proposalId: "018f0000-0000-7000-8000-000000000006",
      state: "approved",
      expiresAtMs: 1771235467000,
      dispatchAvailable: true,
    });
    render(
      <AdvisorWorkspace
        {...props}
        onDispatch={onDispatch}
        targetProjectId="018f0000-0000-7000-8000-000000000002"
        conversation={{
          ...scaffoldAdvisorConversation,
          state: "completed",
          conversationId: "018f0000-0000-7000-8000-000000000003",
        }}
      />,
    );
    fireEvent.change(screen.getByRole("textbox", { name: "Editable draft" }), {
      target: { value: "Review the bounded handoff." },
    });
    fireEvent.click(
      screen.getByRole("button", { name: "Create approval draft" }),
    );
    await screen.findByRole("button", { name: "Approve draft" });
    fireEvent.click(screen.getByRole("button", { name: "Approve draft" }));
    const dispatch = await screen.findByRole("button", {
      name: "Dispatch once to execution workspace",
    });
    fireEvent.click(dispatch);
    await waitFor(() => expect(onDispatch).toHaveBeenCalledOnce());
    expect(onDispatch).toHaveBeenCalledWith(
      expect.objectContaining({
        proposalId: "018f0000-0000-7000-8000-000000000006",
      }),
    );
    expect(
      await screen.findByText(
        /approved request started in the execution workspace/i,
      ),
    ).toBeInTheDocument();
  });
});
