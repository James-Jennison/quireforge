import { fireEvent, render, screen, waitFor } from "@testing-library/react";
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
  loadAdvisorDocumentAttachment: vi.fn().mockResolvedValue({
    schemaVersion: 1,
    state: "empty",
    attachment: null,
    confirmationState: null,
    diagnosticCode: null,
  }),
  pickAdvisorDocumentAttachment: vi.fn(),
  cancelAdvisorDocumentAttachment: vi.fn(),
  loadAdvisorArchiveAttachment: vi.fn().mockResolvedValue({
    schemaVersion: 1,
    state: "empty",
    attachment: null,
    entries: [],
    confirmationState: null,
    diagnosticCode: null,
  }),
  pickAdvisorArchiveAttachment: vi.fn(),
  cancelAdvisorArchiveAttachment: vi.fn(),
  loadAdvisorBinaryAttachment: vi.fn().mockResolvedValue({
    schemaVersion: 1,
    state: "empty",
    attachment: null,
    confirmationState: null,
    diagnosticCode: null,
  }),
  pickAdvisorBinaryAttachment: vi.fn(),
  cancelAdvisorBinaryAttachment: vi.fn(),
  saveAdvisorTextExport: vi.fn(),
  loadAdvisorGeneratedArtifacts: vi.fn().mockResolvedValue({
    schemaVersion: 1,
    artifacts: [],
    diagnosticCode: null,
  }),
  createAdvisorGeneratedArtifact: vi.fn(),
  previewAdvisorGeneratedArtifact: vi.fn(),
  discardAdvisorGeneratedArtifact: vi.fn(),
  saveAdvisorGeneratedArtifact: vi.fn(),
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
  it("gives the reviewed task brief a dedicated spacious handoff control", () => {
    render(
      <AdvisorWorkspace
        {...props}
        availability="native"
        onPrepareTaskHandoff={vi.fn()}
        onOpenTaskHandoff={vi.fn()}
      />,
    );

    const brief = screen.getByRole("textbox", {
      name: "Reviewed task brief",
    });
    expect(brief).toHaveAttribute("rows", "6");
    expect(brief.closest("section")).toHaveClass("advisor-handoff");
  });

  it.each([
    ["malformed-or-unsupported-document", /malformed or unsupported/i],
    ["encrypted", /encrypted PDFs are not supported/i],
    ["active-content", /active content is not supported/i],
    ["embedded-content", /embedded content is not supported/i],
    ["external-action", /external actions are not supported/i],
  ])(
    "announces the path-free PDF %s diagnostic",
    async (diagnosticCode, text) => {
      advisorDraftBridge.loadAdvisorDocumentAttachment.mockResolvedValueOnce({
        schemaVersion: 1,
        state: "unavailable",
        attachment: null,
        confirmationState: null,
        diagnosticCode,
      });
      render(<AdvisorWorkspace {...props} />);
      expect(await screen.findByRole("alert")).toHaveTextContent(text);
      expect(screen.getByRole("alert")).not.toHaveTextContent("/mnt/");
      expect(
        screen.getByRole("button", { name: "Attach a file" }),
      ).toBeEnabled();
    },
  );

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
      screen.getByRole("heading", { name: "Advisor" }),
    ).toBeInTheDocument();
    expect(
      screen.getByText("Create, Learn, Explore · Read-only"),
    ).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Details" })).toHaveAttribute(
      "aria-expanded",
      "false",
    );
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

  it("keeps safe reference details out of the primary chat flow", () => {
    render(<AdvisorWorkspace {...props} />);
    expect(
      screen.queryByRole("list", { name: "Advisor metadata summaries" }),
    ).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Details" }));
    expect(
      screen.getByRole("complementary", { name: "Advisor details" }),
    ).toHaveTextContent(/no shell, terminal, Git/u);
    expect(
      screen.queryByText("advisor-thread-fixture-01"),
    ).not.toBeInTheDocument();
    expect(screen.queryByText(/gpt-5.6-terra/i)).not.toBeInTheDocument();
  });

  it("closes the optional details drawer with Escape and restores trigger focus", async () => {
    render(<AdvisorWorkspace {...props} />);
    const trigger = screen.getByRole("button", { name: "Details" });
    fireEvent.click(trigger);
    const drawer = screen.getByRole("complementary", {
      name: "Advisor details",
    });

    fireEvent.keyDown(drawer, { key: "Escape" });

    await waitFor(() =>
      expect(
        screen.queryByRole("complementary", { name: "Advisor details" }),
      ).not.toBeInTheDocument(),
    );
    expect(document.activeElement).toBe(trigger);
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
      documentAttachmentId: null,
      documentAttachmentManifestSha256: null,
      documentAttachmentConfirmation: null,
      archiveAttachmentId: null,
      archiveAttachmentManifestSha256: null,
      archiveAttachmentConfirmation: null,
      binaryAttachmentId: null,
      binaryAttachmentManifestSha256: null,
      binaryAttachmentConfirmation: null,
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

  it("lets a reader pause follow-latest and jump back to the newest reply", () => {
    const { container } = render(
      <AdvisorWorkspace
        {...props}
        conversation={{
          ...scaffoldAdvisorConversation,
          state: "running",
          conversationId: "018f0000-0000-7000-8000-000000000099",
          events: [
            {
              type: "agent-message-delta",
              sequence: 1,
              delta: "A bounded reply that remains in the transient viewport.",
            },
          ],
        }}
      />,
    );
    const viewport = screen.getByRole("log", {
      name: "Active Advisor conversation",
    });
    let scrollTop = 0;
    Object.defineProperties(viewport, {
      clientHeight: { configurable: true, value: 100 },
      scrollHeight: { configurable: true, value: 1000 },
      scrollTop: {
        configurable: true,
        get: () => scrollTop,
        set: (value: number) => {
          scrollTop = value;
        },
      },
    });

    scrollTop = 120;
    fireEvent.scroll(viewport);
    const jump = screen.getByRole("button", { name: "Jump to latest" });
    expect(jump).toHaveAttribute("aria-controls", "advisor-conversation-log");

    fireEvent.click(jump);
    expect(scrollTop).toBe(1000);
    expect(
      screen.queryByRole("button", { name: "Jump to latest" }),
    ).not.toBeInTheDocument();
    expect(
      container.querySelector(".conversation-composer"),
    ).toBeInTheDocument();
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
    expect(
      advisorDraftBridge.cancelAdvisorDocumentAttachment,
    ).toHaveBeenCalled();
    expect(advisorDraftBridge.cancelAdvisorBinaryAttachment).toHaveBeenCalled();
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
        /Project State and up to three existing bounded attachment projections are optional and require collection confirmation/u,
      ),
    ).toBeInTheDocument();
    expect(
      screen.queryByText("Enter a message to send."),
    ).not.toBeInTheDocument();
  });

  it("offers one bounded attachment entry that routes only to closed type pickers", () => {
    render(<AdvisorWorkspace {...props} />);

    expect(
      screen.queryByRole("button", { name: /Attach (text|PNG|PDF|ZIP|ELF)/i }),
    ).not.toBeInTheDocument();
    expect(document.querySelector('input[type="file"]')).toBeNull();

    fireEvent.click(screen.getByRole("button", { name: "Attach a file" }));
    expect(
      screen.getByRole("dialog", { name: "Choose attachment type" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "Text or data · 512 KiB" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "PNG or JPEG · 4 MiB" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "PDF document · 8 MiB" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "ZIP archive · 32 MiB" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "ELF static metadata · 32 MiB" }),
    ).toBeInTheDocument();

    fireEvent.click(
      screen.getByRole("button", { name: "PNG or JPEG · 4 MiB" }),
    );
    expect(advisorDraftBridge.pickAdvisorImageAttachment).toHaveBeenCalledTimes(
      1,
    );
    expect(advisorDraftBridge.pickAdvisorTextAttachment).not.toHaveBeenCalled();
    expect(
      advisorDraftBridge.pickAdvisorDocumentAttachment,
    ).not.toHaveBeenCalled();
    expect(
      advisorDraftBridge.pickAdvisorArchiveAttachment,
    ).not.toHaveBeenCalled();
    expect(
      advisorDraftBridge.pickAdvisorBinaryAttachment,
    ).not.toHaveBeenCalled();
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
    fireEvent.click(screen.getByRole("button", { name: "Attach a file" }));
    fireEvent.click(
      screen.getByRole("button", { name: "Text or data · 512 KiB" }),
    );
    await screen.findByText(/Ready: notes.md/u);
    fireEvent.change(screen.getByRole("textbox", { name: "Advisor message" }), {
      target: { value: "Review the attached notes." },
    });
    fireEvent.click(screen.getByRole("button", { name: "Send to Advisor" }));
    expect(onConversationStart).not.toHaveBeenCalled();
    fireEvent.click(
      await screen.findByRole("button", {
        name: "Confirm all attachments for this send",
      }),
    );
    expect(onConversationStart).toHaveBeenCalledWith({
      prompt: "Review the attached notes.",
      projectId: null,
      attachmentId: attachment.attachment.attachmentId,
      attachmentManifestSha256: attachment.attachment.sha256,
      attachmentConfirmation: "confirmed-for-single-send",
      imageAttachmentId: null,
      imageAttachmentManifestSha256: null,
      imageAttachmentConfirmation: null,
      documentAttachmentId: null,
      documentAttachmentManifestSha256: null,
      documentAttachmentConfirmation: null,
      archiveAttachmentId: null,
      archiveAttachmentManifestSha256: null,
      archiveAttachmentConfirmation: null,
      binaryAttachmentId: null,
      binaryAttachmentManifestSha256: null,
      binaryAttachmentConfirmation: null,
    });
  });

  it("shows a path-free ZIP manifest and sends only its confirmed identity", async () => {
    const archive = {
      schemaVersion: 1,
      state: "ready",
      attachment: {
        attachmentId: "018f0000-0000-7000-8000-000000000097",
        displayName: "notes.zip",
        contentCategory: "archive",
        mediaType: "zip",
        byteSize: 120,
        sha256: "b".repeat(64),
        projection: {
          kind: "archive-manifest-v1",
          schemaVersion: 1,
          discoveredEntryCount: 1,
          includedEntryCount: 1,
          omittedEntryCount: 0,
          declaredAggregateUncompressedBytes: 12,
          manifestByteSize: 80,
          truncated: false,
          warnings: [],
        },
        disposal: "transient-memory-one-send",
      },
      entries: [
        {
          name: "notes.txt",
          kind: "file",
          compressedSize: 8,
          declaredUncompressedSize: 12,
          nestedArchiveLike: false,
        },
      ],
      confirmationState: "confirmation-required",
      diagnosticCode: null,
    };
    advisorDraftBridge.loadAdvisorArchiveAttachment.mockResolvedValueOnce(
      archive,
    );
    const onConversationStart = vi
      .fn()
      .mockResolvedValue(scaffoldAdvisorConversation);
    render(
      <AdvisorWorkspace {...props} onConversationStart={onConversationStart} />,
    );
    expect(await screen.findByText(/Ready: notes\.zip/u)).toBeInTheDocument();
    expect(screen.queryByText("/tmp/notes.zip")).not.toBeInTheDocument();
    fireEvent.change(screen.getByRole("textbox", { name: "Advisor message" }), {
      target: { value: "Review this archive manifest." },
    });
    fireEvent.click(screen.getByRole("button", { name: "Send to Advisor" }));
    expect(onConversationStart).not.toHaveBeenCalled();
    fireEvent.click(
      await screen.findByRole("button", {
        name: "Confirm all attachments for this send",
      }),
    );
    expect(onConversationStart).toHaveBeenCalledWith(
      expect.objectContaining({
        archiveAttachmentId: archive.attachment.attachmentId,
        archiveAttachmentManifestSha256: archive.attachment.sha256,
        archiveAttachmentConfirmation: "confirmed-for-single-send",
      }),
    );
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
