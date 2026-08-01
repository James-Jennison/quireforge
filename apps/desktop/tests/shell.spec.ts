import AxeBuilder from "@axe-core/playwright";
import { expect, test } from "@playwright/test";

import integrationCatalogFixture from "../fixtures/integration-catalog.json" with { type: "json" };
import integrationControlFixture from "../fixtures/integration-control.json" with { type: "json" };
import integrationMutationFixture from "../fixtures/integration-mutation.json" with { type: "json" };
import filePreviewFixture from "../fixtures/file-preview.json" with { type: "json" };
import conversationAttachmentFixture from "../fixtures/conversation-attachments.json" with { type: "json" };
import usageFixture from "../fixtures/codex-usage.json" with { type: "json" };
import projectStateFixtures from "../fixtures/project-state.json" with { type: "json" };
import taskCatalogFixture from "../fixtures/task-catalog.json" with { type: "json" };

const nativeIntegrationCatalog = {
  ...integrationCatalogFixture,
  capabilities: integrationCatalogFixture.capabilities.map((capability) =>
    [
      "connector.authorize",
      "plugin.install",
      "plugin.remove",
      "marketplace.configure",
      "skill.configure",
      "mcp.authorize",
      "scheduled-task.catalog",
    ].includes(capability.id)
      ? {
          ...capability,
          availability: "ready",
          implementation: "ready",
          diagnosticCode: null,
        }
      : capability,
  ),
};

const modelSelectionFixture = {
  schemaVersion: 1,
  availability: "ready",
  effective: {
    modelId: "gpt-5.6-sol",
    reasoningEffort: "high",
  },
  pending: null,
  policy: {
    ownership: "manual",
    userLocked: false,
    allowedModelIds: [],
    reasoningCeiling: null,
  },
  diagnosticCode: null,
} as const;

const recommendationSelectionFixture = {
  ...modelSelectionFixture,
  pending: {
    choice: {
      modelId: "gpt-5.6-terra",
      reasoningEffort: "high",
    },
    provenance: "codex",
    application: "recommendation",
    rationale: "Use the bounded lower-latency option for the next turn.",
    requestedAtMs: 1_700_000_001_500,
  },
  policy: {
    ...modelSelectionFixture.policy,
    ownership: "recommend",
  },
} as const;

const repositoryStateFixture = {
  schemaVersion: 1,
  state: projectStateFixtures.activeMilestone,
  git: {
    upstream: "origin/feat/milestone-24c-project-state-workspace",
    detached: false,
    stagedCount: 0,
    unstagedCount: 0,
    untrackedCount: 0,
    mergeInProgress: false,
    rebaseInProgress: false,
    cherryPickInProgress: false,
    bisectInProgress: false,
    shallow: false,
  },
  evidence: {
    packages: [],
    validations: [
      {
        version: 1,
        id: "frontend-tests",
        family: "frontend-tests",
        status: "passed",
        sourceCommit: "0123456789abcdef0123456789abcdef01234567",
        evidencePath: "target/validation-summary.json",
        operation: "pnpm-test",
        timestamp: "2026-07-26T00:00:00Z",
        freshness: "current",
      },
    ],
    handoff: null,
  },
  diagnostics: [
    {
      id: "tracking-freshness-unknown",
      severity: "info",
      affectedField: "repository.remoteHead",
      sourceRef: null,
      explanation: "Remote tracking freshness was not requested.",
      approvalRequired: false,
      recommendedAction: "Inspect existing tracking evidence if needed.",
    },
  ],
} as const;

const advisorWorkspaceFixture = {
  schemaVersion: 1,
  conversationCount: 1,
  contextReferenceCount: 1,
  proposalCount: 1,
  contextSummaries: [
    { kind: "project-state", trust: "verified", freshness: "current" },
  ],
  proposalSummaries: [{ state: "draft", requiresExplicitApproval: true }],
} as const;

const advisorProjectStateFixture = {
  schemaVersion: 1,
  sourceKind: "project-state",
  selectedAtMs: 1,
  trust: "verified",
  freshness: "current",
  provenanceSource: "project-state-snapshot",
  worktree: "clean",
  diagnosticCount: 0,
} as const;

const nativeResponses = {
  desktop_bootstrap: {
    schemaVersion: 1,
    product: {
      name: "QuireForge",
      tagline: "Build boldly. Work locally.",
      description: "An unofficial native Linux workspace for Codex",
      identifier: "io.github.codeframe78.QuireForge",
      executable: "quireforge",
      version: "0.0.0",
    },
    capabilities: [
      {
        id: "desktop-foundation",
        label: "Desktop foundation",
        state: "ready",
        milestone: 3,
      },
      {
        id: "codex-runtime",
        label: "Codex runtime adapter",
        state: "ready",
        milestone: 4,
      },
      {
        id: "codex-auth",
        label: "Codex authentication",
        state: "ready",
        milestone: 5,
      },
      {
        id: "project-attachments",
        label: "Local project attachments",
        state: "ready",
        milestone: 6,
      },
      {
        id: "conversation-runtime",
        label: "Native conversation runtime",
        state: "ready",
        milestone: 7,
      },
      {
        id: "integrated-terminal",
        label: "Integrated terminal",
        state: "ready",
        milestone: 12,
      },
      {
        id: "integration-center",
        label: "Integration Center",
        state: "ready",
        milestone: 14,
      },
      {
        id: "safe-file-previews",
        label: "Safe file previews",
        state: "ready",
        milestone: 15,
      },
      {
        id: "conversation-attachments",
        label: "Conversation image attachments",
        state: "ready",
        milestone: 15,
      },
      {
        id: "desktop-integration",
        label: "Reviewed desktop integration",
        state: "ready",
        milestone: 15,
      },
      {
        id: "scheduled-task-catalog",
        label: "Read-only scheduled task catalog",
        state: "ready",
        milestone: 17,
      },
      {
        id: "agent-model-selection",
        label: "Policy-bounded next-turn selection",
        state: "ready",
        milestone: 18,
      },
    ],
  },
  codex_runtime_probe: {
    schemaVersion: 1,
    adapterVersion: "codex-app-server-v2",
    availability: "ready",
    backend: "app-server-stdio",
    cliVersion: "0.144.6",
    capabilities: [
      { id: "runtime-probe", state: "ready", route: "cli" },
      { id: "app-server-stdio", state: "ready", route: "app-server" },
      { id: "model-discovery", state: "ready", route: "app-server" },
      { id: "normalized-events", state: "ready", route: "native" },
      { id: "conversation-runtime", state: "ready", route: "app-server" },
    ],
    models: [
      {
        id: "gpt-5.6-sol",
        displayName: "GPT-5.6-Sol",
        isDefault: true,
        defaultReasoningEffort: "low",
        supportedReasoningEfforts: ["low", "medium", "high", "xhigh", "max"],
      },
      {
        id: "gpt-5.6-terra",
        displayName: "GPT-5.6 Terra",
        isDefault: false,
        defaultReasoningEffort: "medium",
        supportedReasoningEfforts: ["medium", "high"],
      },
    ],
    diagnosticCode: null,
  },
  codex_auth_status: {
    schemaVersion: 1,
    state: "authenticated",
    accountKind: "chatgpt",
    pendingMethod: null,
    handoff: null,
    diagnosticCode: null,
  },
  codex_usage_status: usageFixture,
  codex_usage_refresh: usageFixture,
  file_preview_pick: {
    ...filePreviewFixture,
    projectId: "018f0000-0000-7000-8000-000000000001",
  },
  file_preview_open: null,
  file_preview_cancel: true,
  conversation_notify: {
    schemaVersion: 1,
    status: "foreground",
  },
  conversation_attachment_status: {
    schemaVersion: 1,
    state: "empty",
    projectId: "018f0000-0000-7000-8000-000000000001",
    attachments: [],
    diagnosticCode: null,
  },
  conversation_attachment_pick: {
    ...conversationAttachmentFixture,
    projectId: "018f0000-0000-7000-8000-000000000001",
  },
  conversation_attachment_cancel: {
    schemaVersion: 1,
    state: "empty",
    projectId: "018f0000-0000-7000-8000-000000000001",
    attachments: [],
    diagnosticCode: null,
  },
  integration_catalog_read: nativeIntegrationCatalog,
  integration_catalog_refresh: nativeIntegrationCatalog,
  integration_control_preview: integrationControlFixture.preview,
  integration_control_confirm: integrationControlFixture.result,
  integration_control_open_browser: {
    ...integrationControlFixture.result,
    state: "pending",
    browserHandoffAvailable: false,
  },
  integration_control_status: {
    ...integrationControlFixture.result,
    state: "completed",
    actionId: null,
    browserHandoffAvailable: false,
    catalogRefreshRequired: true,
  },
  integration_mutation_preview: integrationMutationFixture.preview,
  integration_mutation_confirm: integrationMutationFixture.result,
  project_workspace_status: {
    schemaVersion: 1,
    state: "ready",
    projects: [
      {
        id: "018f0000-0000-7000-8000-000000000001",
        displayName: "QuireForge",
        archived: false,
        directory: {
          associationId: "018f0000-0000-7000-8000-000000000002",
          displayPath: "~/work/quireforge",
          resolvedDisplayPath: "/mnt/work/quireforge",
          state: "connected-accessible",
          expectedAccess: "read-write",
          isPrimary: true,
          git: { isRepository: true, isLinkedWorktree: false },
          hasAgentsGuidance: true,
          hasCodexConfig: false,
        },
      },
    ],
    pendingAttachment: null,
    diagnosticCode: null,
  },
  task_catalog_status: taskCatalogFixture,
  task_plan_select: taskCatalogFixture,
  advisor_snapshot_read: advisorWorkspaceFixture,
  advisor_project_state_snapshot_read: advisorProjectStateFixture,
  advisor_conversation_status: {
    schemaVersion: 1,
    mode: "advisor",
    state: "empty",
    conversationId: null,
    projectStateIncluded: false,
    events: [],
    diagnosticCode: null,
  },
  advisor_conversation_start: {
    schemaVersion: 1,
    mode: "advisor",
    state: "running",
    conversationId: "018f0000-0000-7000-8000-000000000061",
    projectStateIncluded: false,
    events: [],
    diagnosticCode: null,
  },
  advisor_conversation_interrupt: {
    schemaVersion: 1,
    mode: "advisor",
    state: "interrupted",
    conversationId: "018f0000-0000-7000-8000-000000000061",
    projectStateIncluded: false,
    events: [],
    diagnosticCode: null,
  },
  advisor_text_attachment_status: {
    schemaVersion: 1,
    state: "empty",
    attachment: null,
    confirmationState: null,
    diagnosticCode: null,
  },
  advisor_image_attachment_status: {
    schemaVersion: 1,
    state: "empty",
    attachment: null,
    previewDataUrl: null,
    confirmationState: null,
    diagnosticCode: null,
  },
  advisor_document_attachment_status: {
    schemaVersion: 1,
    state: "empty",
    attachment: null,
    confirmationState: null,
    diagnosticCode: null,
  },
  advisor_archive_attachment_status: {
    schemaVersion: 1,
    state: "empty",
    attachment: null,
    entries: [],
    confirmationState: null,
    diagnosticCode: null,
  },
  advisor_binary_attachment_status: {
    schemaVersion: 1,
    state: "empty",
    attachment: null,
    confirmationState: null,
    diagnosticCode: null,
  },
  repository_state_read: repositoryStateFixture,
  worktree_status: {
    schemaVersion: 2,
    state: "ready",
    sourceProjectId: "018f0000-0000-7000-8000-000000000001",
    worktrees: [
      {
        projectId: "018f0000-0000-7000-8000-000000000001",
        recoveryId: null,
        displayName: "QuireForge",
        displayPath: "~/work/quireforge",
        branchName: "feature/review",
        ownership: "source",
        state: "ready",
        current: true,
      },
      {
        projectId: null,
        recoveryId: "018f0000-0000-7000-8000-000000000041",
        displayName: "feature/recoverable",
        displayPath:
          "~/.local/share/io.github.codeframe78.QuireForge/worktrees/recoverable",
        branchName: "feature/recoverable",
        ownership: "external",
        state: "ready",
        current: false,
      },
      {
        projectId: "018f0000-0000-7000-8000-000000000003",
        recoveryId: null,
        displayName: "feature/managed-cleanup",
        displayPath:
          "~/.local/share/io.github.codeframe78.QuireForge/worktrees/managed-cleanup",
        branchName: "feature/managed-cleanup",
        ownership: "managed",
        state: "ready",
        current: false,
      },
    ],
    truncated: false,
    diagnosticCode: null,
  },
  worktree_create_preview: {
    schemaVersion: 2,
    state: "ready",
    sourceProjectId: "018f0000-0000-7000-8000-000000000001",
    operation: "create",
    branchName: "feature/isolated",
    displayPath: "~/.local/share/quireforge/worktrees/isolated",
    ownership: "managed",
    destructive: false,
    confirmationId: "018f0000-0000-7000-8000-000000000040",
    diagnosticCode: null,
  },
  worktree_recover_preview: {
    schemaVersion: 2,
    state: "ready",
    sourceProjectId: "018f0000-0000-7000-8000-000000000001",
    operation: "recover",
    branchName: "feature/recoverable",
    displayPath:
      "~/.local/share/io.github.codeframe78.QuireForge/worktrees/recoverable",
    ownership: "managed",
    destructive: false,
    confirmationId: "018f0000-0000-7000-8000-000000000042",
    diagnosticCode: null,
  },
  worktree_remove_preview: {
    schemaVersion: 2,
    state: "ready",
    sourceProjectId: "018f0000-0000-7000-8000-000000000001",
    operation: "remove",
    branchName: "feature/managed-cleanup",
    displayPath:
      "~/.local/share/io.github.codeframe78.QuireForge/worktrees/managed-cleanup",
    ownership: "managed",
    destructive: true,
    confirmationId: "018f0000-0000-7000-8000-000000000043",
    diagnosticCode: null,
  },
  worktree_cancel: true,
  git_status: {
    schemaVersion: 1,
    state: "ready",
    projectId: "018f0000-0000-7000-8000-000000000001",
    branch: {
      head: "feature/review",
      upstream: "origin/feature/review",
      ahead: 1,
      behind: 0,
      detached: false,
    },
    changes: [
      {
        path: "src/App.tsx",
        previousPath: null,
        staged: null,
        worktree: "modified",
        conflict: false,
        submodule: false,
        reviewable: true,
      },
    ],
    truncated: false,
    diagnosticCode: null,
  },
  git_diff: {
    schemaVersion: 1,
    state: "ready",
    projectId: "018f0000-0000-7000-8000-000000000001",
    path: "src/App.tsx",
    area: "worktree",
    kind: "text",
    lines: [
      {
        kind: "hunk",
        oldLine: null,
        newLine: null,
        text: "@@ -1 +1 @@",
      },
      { kind: "deletion", oldLine: 1, newLine: null, text: "old line" },
      { kind: "addition", oldLine: null, newLine: 1, text: "new line" },
    ],
    truncated: false,
    diagnosticCode: null,
  },
  git_mutation_preview: {
    schemaVersion: 1,
    state: "ready",
    projectId: "018f0000-0000-7000-8000-000000000001",
    operation: "stage",
    path: "src/App.tsx",
    targets: [
      {
        path: "src/App.tsx",
        staged: null,
        worktree: "modified",
      },
    ],
    destructive: false,
    confirmationId: "018f0000-0000-7000-8000-000000000030",
    secretFindings: [],
    diagnosticCode: null,
  },
  git_mutation_confirm: {
    schemaVersion: 1,
    state: "applied",
    projectId: "018f0000-0000-7000-8000-000000000001",
    operation: "stage",
    recoveryId: null,
    workspace: {
      schemaVersion: 1,
      state: "ready",
      projectId: "018f0000-0000-7000-8000-000000000001",
      branch: {
        head: "feature/review",
        upstream: "origin/feature/review",
        ahead: 1,
        behind: 0,
        detached: false,
      },
      changes: [
        {
          path: "src/App.tsx",
          previousPath: null,
          staged: "modified",
          worktree: null,
          conflict: false,
          submodule: false,
          reviewable: true,
        },
      ],
      truncated: false,
      diagnosticCode: null,
    },
    diagnosticCode: null,
  },
  conversation_status: {
    schemaVersion: 3,
    state: "empty",
    conversationId: null,
    projectId: null,
    modelId: null,
    reasoningEffort: null,
    modelSelection: null,
    sandboxMode: null,
    approvalPolicy: null,
    pendingApproval: null,
    events: [],
    diagnosticCode: null,
  },
  conversation_active: {
    schemaVersion: 1,
    capacity: 4,
    conversations: [],
  },
  terminal_status: {
    schemaVersion: 1,
    capacity: 8,
    terminals: [],
    diagnosticCode: null,
  },
  conversation_sessions: {
    schemaVersion: 3,
    state: "ready",
    sessions: [
      {
        conversationId: "018f0000-0000-7000-8000-000000000010",
        projectId: "018f0000-0000-7000-8000-000000000001",
        parentConversationId: null,
        title: "Review lifecycle boundaries",
        modelId: "gpt-5.6-sol",
        reasoningEffort: "high",
        modelSelection: recommendationSelectionFixture,
        sandboxMode: "workspace-write",
        approvalPolicy: "on-request",
        state: "completed",
        createdAtMs: 1_700_000_000_000,
        updatedAtMs: 1_700_000_001_000,
      },
      {
        conversationId: "018f0000-0000-7000-8000-000000000011",
        projectId: "018f0000-0000-7000-8000-000000000001",
        parentConversationId: "018f0000-0000-7000-8000-000000000010",
        title: "Try the smaller adapter",
        modelId: "gpt-5.6-sol",
        reasoningEffort: "high",
        modelSelection: modelSelectionFixture,
        sandboxMode: "workspace-write",
        approvalPolicy: "on-request",
        state: "interrupted",
        createdAtMs: 1_700_000_002_000,
        updatedAtMs: 1_700_000_003_000,
      },
    ],
    diagnosticCode: null,
  },
  model_selection_update: modelSelectionFixture,
} as const;

const liveTerminalSnapshot = {
  schemaVersion: 1,
  state: "running",
  terminalId: "018f0000-0000-7000-8000-000000000050",
  projectId: "018f0000-0000-7000-8000-000000000001",
  title: "Terminal 1",
  live: true,
  columns: 100,
  rows: 30,
  output: [],
  firstSequence: 0,
  lastSequence: 0,
  truncated: false,
  hasMore: false,
  exitCode: null,
  diagnosticCode: null,
} as const;

const nativeTerminalResponses = {
  ...nativeResponses,
  terminal_status: {
    schemaVersion: 1,
    capacity: 8,
    terminals: [liveTerminalSnapshot],
    diagnosticCode: null,
  },
  terminal_poll: liveTerminalSnapshot,
  terminal_resize: liveTerminalSnapshot,
} as const;

async function installNativeFixture(
  page: import("@playwright/test").Page,
  responses: Record<string, unknown> = nativeResponses,
) {
  await page.addInitScript((responses) => {
    const hasFixtureSequence = (
      value: unknown,
    ): value is { sequence: unknown[] } =>
      typeof value === "object" &&
      value !== null &&
      "sequence" in value &&
      Array.isArray((value as Record<string, unknown>).sequence);
    const target = window as unknown as {
      __TAURI_INTERNALS__: {
        invoke: (command: string) => Promise<unknown>;
      };
    };
    target.__TAURI_INTERNALS__ = {
      invoke: (command) => {
        if (!(command in responses))
          throw new Error(`Unexpected command: ${command}`);
        const response = responses[command];
        if (hasFixtureSequence(response)) {
          const next = response.sequence.shift();
          if (next === undefined)
            throw new Error(`Exhausted fixture sequence: ${command}`);
          return Promise.resolve(structuredClone(next));
        }
        return Promise.resolve(structuredClone(response));
      },
    };
  }, responses);
}

async function openWorkspace(
  page: import("@playwright/test").Page,
  label: string,
) {
  const destination = page.getByRole("link", { name: label, exact: true });
  if ((page.viewportSize()?.width ?? 0) <= 760) {
    await page.getByRole("button", { name: "Open navigation" }).click();
  }
  await destination.click();
  const confirmation = page.getByRole("dialog", {
    name: "Confirm conversation mode change",
  });
  if (
    await confirmation
      .waitFor({ state: "visible", timeout: 750 })
      .then(() => true)
      .catch(() => false)
  ) {
    await confirmation
      .getByRole("button", { name: /Confirm (Advisor|QuireForge)/u })
      .click();
  }
  await expect(destination).toHaveAttribute("aria-current", "page");
}

async function openAccountSettings(page: import("@playwright/test").Page) {
  const account = page.getByRole("button", {
    name: "Open Codex account and connection settings",
  });
  if ((page.viewportSize()?.width ?? 0) <= 760) {
    await page.getByRole("button", { name: "Open navigation" }).click();
  }
  await account.click();
  await expect(page).toHaveURL(/#settings\/general$/u);
}

const approvalConversation = {
  schemaVersion: 3,
  state: "waiting-for-approval",
  conversationId: "018f0000-0000-7000-8000-000000000020",
  projectId: "018f0000-0000-7000-8000-000000000001",
  modelId: "gpt-5.6-sol",
  reasoningEffort: "high",
  modelSelection: modelSelectionFixture,
  sandboxMode: "workspace-write",
  approvalPolicy: "on-request",
  pendingApproval: {
    approvalId: "018f0000-0000-7000-8000-000000000021",
    activityId: "018f0000-0000-7000-8000-000000000022",
    kind: "command-execution",
    title: "Run this command?",
    reason: "The project check needs permission.",
    details: [{ label: "Command", value: "pnpm check" }],
    decisions: ["approve", "decline", "cancel"],
  },
  events: [
    {
      type: "activity",
      sequence: 1,
      activityId: "018f0000-0000-7000-8000-000000000022",
      kind: "command-execution",
      status: "started",
      title: "Run command",
      detail: "pnpm check",
      exitCode: null,
    },
    {
      type: "activity-output-delta",
      sequence: 2,
      activityId: "018f0000-0000-7000-8000-000000000022",
      delta: "Checking the desktop contract…",
    },
    {
      type: "approval-requested",
      sequence: 3,
      approvalId: "018f0000-0000-7000-8000-000000000021",
      activityId: "018f0000-0000-7000-8000-000000000022",
      kind: "command-execution",
    },
  ],
  diagnosticCode: null,
} as const;

const approvedConversation = {
  ...approvalConversation,
  state: "completed",
  pendingApproval: null,
  events: [
    ...approvalConversation.events,
    {
      type: "approval-resolved",
      sequence: 4,
      approvalId: "018f0000-0000-7000-8000-000000000021",
      resolution: "approved",
    },
    {
      type: "activity",
      sequence: 5,
      activityId: "018f0000-0000-7000-8000-000000000022",
      kind: "command-execution",
      status: "completed",
      title: "Run command",
      detail: "pnpm check",
      exitCode: 0,
    },
    { type: "lifecycle", sequence: 6, phase: "completed" },
  ],
} as const;

const localReviewBrowserId = "018f0000-0000-7000-8000-000000000101";
const localReviewBrowserItemId = "018f0000-0000-7000-8000-000000000102";
const localReviewBrowserSha = "a".repeat(64);
const localReviewBrowserSnapshot = {
  schemaVersion: 1,
  collections: [
    {
      collectionId: localReviewBrowserId,
      taskId: localReviewBrowserId,
      planId: null,
      title: "Browser review",
      state: "active",
      itemCount: 1,
      payloadBytes: 12,
      updatedAtMs: 1,
      warning: false,
      annotationCountWarning: false,
      annotationByteWarning: false,
      comparisonCountWarning: false,
    },
  ],
  selectedCollection: {
    collectionId: localReviewBrowserId,
    taskId: localReviewBrowserId,
    planId: null,
    title: "Browser review",
    state: "active",
    itemCount: 1,
    payloadBytes: 12,
    updatedAtMs: 1,
    warning: false,
    annotationCountWarning: false,
    annotationByteWarning: false,
    comparisonCountWarning: false,
  },
  items: [
    {
      itemId: localReviewBrowserItemId,
      class: "text",
      textFormat: "plain",
      sourceKind: "user-authored-text",
      state: "ready",
      title: "Browser text",
      mimeType: "text/plain",
      width: null,
      height: null,
      byteSize: 12,
      lineCount: 1,
      sha256: localReviewBrowserSha,
      createdAtMs: 1,
      annotations: [],
    },
  ],
  comparisons: [],
  collectionCount: 1,
  payloadBytes: 12,
  warning: false,
  diagnosticCode: null,
} as const;

test("Local Review uses the real shell with a deterministic path-free native fixture", async ({
  page,
}) => {
  const requests: string[] = [];
  page.on("request", (request) => requests.push(request.url()));
  await installNativeFixture(page, {
    ...nativeResponses,
    local_review_status: localReviewBrowserSnapshot,
  });
  await page.goto("/");
  await openWorkspace(page, "New task");
  await page.getByRole("button", { name: "Review panes" }).click();
  const reviewTab = page.getByRole("tab", { name: "Review", exact: true });
  await expect(reviewTab).toBeVisible();
  await reviewTab.click();
  await expect(
    page.getByRole("heading", { name: "Collections" }),
  ).toBeVisible();
  const compactReview = await page.evaluate(
    () => window.innerWidth <= 760 || window.innerHeight <= 520,
  );
  if (compactReview) {
    await expect(
      page.getByRole("button", { name: /Browser review — active/ }),
    ).toBeVisible();
  } else {
    await expect(
      page.getByRole("button", { name: /Browser text/ }),
    ).toBeVisible();
  }
  await expect(page.locator('input[type="file"]')).toHaveCount(0);
  if (compactReview) {
    await expect(
      page.getByRole("separator", { name: "Resize task evidence" }),
    ).toHaveCount(0);
  } else {
    await expect(
      page.getByRole("separator", { name: "Resize task evidence" }),
    ).toBeVisible();
  }
  expect(
    requests.every((url) => url.startsWith("http://127.0.0.1:1420/")),
  ).toBe(true);
});

test("M56 task-template application remains explicit and contained", async ({
  page,
}) => {
  const templateId = "01980a10-0000-7000-8000-000000000001";
  const taskId = "018f0000-0000-7000-8000-000000000001";
  const digest = "a".repeat(64);
  await page.emulateMedia({ reducedMotion: "reduce" });
  await installNativeFixture(page, {
    ...nativeResponses,
    task_template_catalog: {
      schemaVersion: 1,
      state: "ready",
      templates: [
        {
          id: templateId,
          title: "Feature implementation",
          purpose: "Plan a bounded feature.",
          origin: "built-in",
          state: "active",
        },
      ],
      capacity: {
        recordCount: 1,
        canonicalBytes: 100,
        warning: false,
        countLimit: 64,
        canonicalByteLimit: 2097152,
      },
      diagnosticCode: null,
    },
    task_template_inspect: {
      schemaVersion: 1,
      state: "ready",
      template: {
        id: templateId,
        title: "Feature implementation",
        purpose: "Plan a bounded feature.",
        instructions:
          "Define outcome, constraints, evidence, tests, risks, and completion criteria.",
        origin: "built-in",
        state: "active",
        version: 1,
        sha256: digest,
      },
      mutationHandle: "01980a10-0000-7000-8000-000000000004",
      diagnosticCode: null,
    },
    task_template_preview: {
      schemaVersion: 1,
      state: "ready",
      reservationId: "01980a10-0000-7000-8000-000000000005",
      bindingSha256: digest,
      expiresAtMs: 100,
      checklist: {
        templateActive: true,
        taskPlanAvailable: true,
        exactDraftRequired: true,
        confirmationRequired: true,
      },
      diagnosticCode: null,
    },
    task_template_confirm: {
      schemaVersion: 1,
      state: "ready",
      applied: true,
      cancelled: false,
      diagnosticCode: null,
    },
    task_template_cancel: {
      schemaVersion: 1,
      state: "ready",
      applied: false,
      cancelled: true,
      diagnosticCode: null,
    },
  });
  await page.goto("/");
  await openWorkspace(page, "New task");
  await page.getByRole("button", { name: "Show workbench context" }).click();
  await page.getByRole("button", { name: "Task Templates" }).click();
  await page.getByRole("button", { name: /Feature implementation/ }).click();
  const apply = page.getByRole("button", { name: "Apply to task" });
  await apply.click();
  let application = page.getByRole("dialog", {
    name: "Apply template to task",
  });
  await expect(application).toBeVisible();
  await application.press("Escape");
  await expect(application).toHaveCount(0);
  await expect(apply).toBeFocused();
  await expect(
    page.evaluate(
      () =>
        document.documentElement.scrollWidth <=
        document.documentElement.clientWidth,
    ),
  ).resolves.toBe(true);

  await apply.click();
  application = page.getByRole("dialog", { name: "Apply template to task" });
  await expect(application).toBeVisible();
  for (const viewport of [
    { width: 720, height: 900 },
    { width: 1280, height: 500 },
    { width: 640, height: 450 },
  ]) {
    await page.setViewportSize(viewport);
    await application
      .getByRole("button", { name: "Cancel" })
      .scrollIntoViewIfNeeded();
    await expect(application).toBeVisible();
    await expect(
      page.evaluate(
        () =>
          document.documentElement.scrollWidth <=
          document.documentElement.clientWidth,
      ),
    ).resolves.toBe(true);
  }
  const taskSelector = application.getByRole("combobox", {
    name: "Task",
    exact: true,
  });
  const planSelector = application.getByRole("combobox", {
    name: "Owned plan",
    exact: true,
  });
  await expect(planSelector).toBeDisabled();
  await expect(taskSelector).toBeEnabled();
  await taskSelector.selectOption(taskId);
  await expect(planSelector).toBeEnabled();
  await application
    .getByRole("button", { name: "Request native preview" })
    .click();
  await expect(
    page.getByRole("region", { name: "Authoritative application preview" }),
  ).toBeVisible();
  await application
    .getByRole("button", { name: "Review confirmation" })
    .click();
  const confirmation = page.getByRole("dialog", {
    name: "Confirm template application",
  });
  await expect(confirmation).toBeVisible();
  await expect(
    confirmation.getByRole("button", { name: "Cancel" }),
  ).toBeFocused();
  await confirmation
    .getByRole("button", { name: "Confirm application" })
    .click();
  await expect(
    page.getByRole("status").filter({ hasText: "confirmed" }),
  ).toBeVisible();
  await expect(
    page.evaluate(
      () =>
        document.documentElement.scrollWidth <=
        document.documentElement.clientWidth,
    ),
  ).resolves.toBe(true);
});

test("Local Review uses the compact overlay in a narrow desktop window", async ({
  page,
}) => {
  await page.setViewportSize({ width: 720, height: 900 });
  await installNativeFixture(page, {
    ...nativeResponses,
    local_review_status: localReviewBrowserSnapshot,
  });
  await page.goto("/");
  expect(await page.evaluate(() => window.innerWidth)).toBe(720);
  await page.getByRole("button", { name: "Open navigation" }).click();
  await page.getByRole("button", { name: "QuireForge", exact: true }).click();
  const quireForgeWorkspace = page.getByRole("menuitemradio", {
    name: /QuireForge/u,
  });
  await expect(quireForgeWorkspace).toHaveAttribute("aria-checked", "true");
  await quireForgeWorkspace.click();
  await page.keyboard.press("Escape");
  await expect(page.locator(".app-shell--mobile-navigation-open")).toHaveCount(
    0,
  );
  await openWorkspace(page, "New task");
  await page.keyboard.press("Escape");
  await expect(page.locator(".app-shell--mobile-navigation-open")).toHaveCount(
    0,
  );
  await expect(
    page.getByRole("button", { name: "Review panes" }),
  ).toBeVisible();
  await page.getByRole("button", { name: "Review panes" }).click();
  await page.getByRole("tab", { name: "Review", exact: true }).click();
  const collection = page.getByRole("button", {
    name: /Browser review — active/,
  });
  await collection.click();
  await expect(
    page.getByRole("button", { name: "Back to collections" }),
  ).toBeVisible();
  await expect(
    page.getByRole("separator", { name: "Resize task evidence" }),
  ).toHaveCount(0);
  await page.getByRole("button", { name: "Back to collections" }).click();
  await expect(collection).toBeFocused();
  expect(
    await page.evaluate(
      () => document.documentElement.scrollWidth <= window.innerWidth,
    ),
  ).toBe(true);
});

test("Local Review remains reachable in a short desktop window", async ({
  page,
}) => {
  const requests: string[] = [];
  page.on("request", (request) => requests.push(request.url()));
  await page.setViewportSize({ width: 1280, height: 500 });
  await installNativeFixture(page, {
    ...nativeResponses,
    local_review_status: localReviewBrowserSnapshot,
  });
  await page.goto("/");
  expect(await page.evaluate(() => window.innerHeight)).toBe(500);
  await openWorkspace(page, "New task");
  const reviewTrigger = page.getByRole("button", { name: "Review panes" });
  await expect(reviewTrigger).toBeVisible();
  await reviewTrigger.click();
  const reviewTab = page.getByRole("tab", { name: "Review", exact: true });
  await reviewTab.click();
  await expect(reviewTab).toHaveAttribute("aria-selected", "true");
  await expect(
    page.getByRole("heading", { name: "Collections" }),
  ).toBeVisible();
  await expect(
    page.getByRole("separator", { name: "Resize task evidence" }),
  ).toHaveCount(0);
  const reviewShell = page.getByRole("complementary", {
    name: "Task evidence",
  });
  const shellBox = await reviewShell.boundingBox();
  expect(shellBox).not.toBeNull();
  expect(shellBox!.y).toBeGreaterThanOrEqual(0);
  expect(shellBox!.y + shellBox!.height).toBeLessThanOrEqual(500);

  const collection = page.getByRole("button", {
    name: /Browser review — active/,
  });
  await collection.click();
  await expect(
    page.getByRole("button", { name: "Back to collections" }),
  ).toBeVisible();
  const items = page.getByRole("list", { name: "Review items" });
  await expect(items).toBeVisible();
  await expect(items).toHaveCSS("overflow-y", "auto");
  const item = page.getByRole("button", { name: /Browser text/ });
  await item.click();
  await expect(
    page.getByRole("button", { name: "Back to items" }),
  ).toBeVisible();
  await expect(page.getByRole("region", { name: "Annotations" })).toBeVisible();
  await page.getByRole("button", { name: "Back to items" }).click();
  await expect(item).toBeFocused();
  expect(
    await page.evaluate(
      () => document.documentElement.scrollWidth <= window.innerWidth,
    ),
  ).toBe(true);
  await expect(page.locator('input[type="file"]')).toHaveCount(0);
  expect(
    requests.every((url) => url.startsWith("http://127.0.0.1:1420/")),
  ).toBe(true);
});

test("Local Review remains usable at effective 200% desktop reflow", async ({
  page,
}) => {
  // 640 × 450 CSS pixels models a 1280 × 900 desktop window at 200% layout
  // scale. This remains ordinary desktop Chromium; it is not device emulation.
  const requests: string[] = [];
  page.on("request", (request) => requests.push(request.url()));
  await page.setViewportSize({ width: 640, height: 450 });
  await installNativeFixture(page, {
    ...nativeResponses,
    local_review_status: localReviewBrowserSnapshot,
  });
  await page.goto("/");
  expect(
    await page.evaluate(() => [window.innerWidth, window.innerHeight]),
  ).toEqual([640, 450]);

  await page.getByRole("button", { name: "Open navigation" }).click();
  await page.getByRole("button", { name: "QuireForge", exact: true }).click();
  const quireForgeWorkspace = page.getByRole("menuitemradio", {
    name: /QuireForge/u,
  });
  await expect(quireForgeWorkspace).toHaveAttribute("aria-checked", "true");
  await quireForgeWorkspace.click();
  await page.keyboard.press("Escape");
  await expect(page.locator(".app-shell--mobile-navigation-open")).toHaveCount(
    0,
  );
  await openWorkspace(page, "New task");
  await page.keyboard.press("Escape");
  await expect(page.locator(".app-shell--mobile-navigation-open")).toHaveCount(
    0,
  );

  const reviewTrigger = page.getByRole("button", { name: "Review panes" });
  await expect(reviewTrigger).toBeVisible();
  await reviewTrigger.click();
  const reviewTab = page.getByRole("tab", { name: "Review", exact: true });
  await reviewTab.click();
  await expect(reviewTab).toHaveAttribute("aria-selected", "true");
  await expect(
    page.getByRole("heading", { name: "Collections" }),
  ).toBeVisible();
  await expect(
    page.getByRole("separator", { name: "Resize task evidence" }),
  ).toHaveCount(0);

  const reviewShell = page.getByRole("complementary", {
    name: "Task evidence",
  });
  const shellBox = await reviewShell.boundingBox();
  expect(shellBox).not.toBeNull();
  expect(shellBox!.x).toBeGreaterThanOrEqual(0);
  expect(shellBox!.y).toBeGreaterThanOrEqual(0);
  expect(shellBox!.x + shellBox!.width).toBeLessThanOrEqual(640);
  expect(shellBox!.y + shellBox!.height).toBeLessThanOrEqual(450);

  const collection = page.getByRole("button", {
    name: /Browser review — active/,
  });
  await collection.click();
  await expect(
    page.getByRole("button", { name: "Back to collections" }),
  ).toBeVisible();
  const items = page.getByRole("list", { name: "Review items" });
  await expect(items).toBeVisible();
  await expect(items).toHaveCSS("overflow-y", "auto");
  const item = page.getByRole("button", { name: /Browser text/ });
  await item.click();
  const backToItems = page.getByRole("button", { name: "Back to items" });
  await expect(backToItems).toBeVisible();
  await expect(page.getByRole("region", { name: "Annotations" })).toBeVisible();
  const backBox = await backToItems.boundingBox();
  expect(backBox).not.toBeNull();
  expect(backBox!.y + backBox!.height).toBeLessThanOrEqual(450);
  await backToItems.click();
  await expect(item).toBeFocused();

  expect(
    await page.evaluate(() => {
      const shell = document.querySelector<HTMLElement>(".app-shell");
      return (
        document.documentElement.scrollWidth <= window.innerWidth &&
        (!shell || shell.scrollWidth <= shell.clientWidth)
      );
    }),
  ).toBe(true);
  await expect(page.locator('input[type="file"]')).toHaveCount(0);
  expect(
    requests.every((url) => url.startsWith("http://127.0.0.1:1420/")),
  ).toBe(true);
});

test("mock inference clears a prepared review when its bound input changes", async ({
  page,
}) => {
  const id = "018f0000-0000-7000-8000-000000000001";
  const attemptId = "019a5900-0000-7000-8000-000000000002";
  const authorizationId = "019a5900-0000-7000-8000-000000000003";
  const digest = "a".repeat(64);
  const ready = {
    schemaVersion: 1,
    mockOnly: true,
    attemptId,
    state: "ready",
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
      state: "pending",
      expiresAtTick: 3,
    },
    events: [],
    usage: null,
    evidence: [],
  } as const;
  const authorized = {
    ...ready,
    state: "authorized",
    authorization: { ...ready.authorization, state: "authorized" },
  } as const;
  const submitted = {
    ...ready,
    state: "submitted",
    authorization: { ...ready.authorization, state: "consumed" },
  } as const;
  const streaming = {
    ...submitted,
    state: "streaming",
    events: [
      {
        id,
        sequence: 1,
        kind: "text-delta",
        text: "incremental fixture output",
        structuredState: null,
        sha256: digest,
      },
    ],
  } as const;
  const cancelling = {
    ...streaming,
    state: "cancelling",
    events: [
      ...streaming.events,
      {
        id: attemptId,
        sequence: 2,
        kind: "cancellation-requested",
        text: null,
        structuredState: null,
        sha256: digest,
      },
    ],
  } as const;
  const cancelled = {
    ...cancelling,
    state: "cancelled",
    events: [
      ...cancelling.events,
      {
        id: authorizationId,
        sequence: 3,
        kind: "terminal",
        text: "cancelled",
        structuredState: null,
        sha256: digest,
      },
    ],
  } as const;
  await installNativeFixture(page, {
    ...nativeResponses,
    mock_inference_catalog: {
      schemaVersion: 1,
      profiles: [
        {
          id: "lantern-stream",
          providerLabel: "Fictional Lantern",
          endpointLabel: "Local fixture endpoint",
          modelLabel: "Lantern Text Fixture",
          adapterLabel: "registry fixture adapter",
          scenario: "streamed-text",
          descriptorSha256: digest,
          capabilityProfileSha256: digest,
        },
        {
          id: "ember-ambiguous",
          providerLabel: "Fictional Ember",
          endpointLabel: "Local fixture endpoint",
          modelLabel: "Ember Ambiguity Fixture",
          adapterLabel: "registry fixture adapter",
          scenario: "ambiguous",
          descriptorSha256: "b".repeat(64),
          capabilityProfileSha256: "b".repeat(64),
        },
      ],
    },
    mock_inference_prepare: { sequence: [ready, ready] },
    mock_inference_authorize: authorized,
    mock_inference_submit: submitted,
    mock_inference_poll: { sequence: [streaming, cancelled] },
    mock_inference_cancel: cancelling,
  });
  await page.goto("/");
  await openWorkspace(page, "New task");
  const primaryWorkspace = page.locator(
    '[data-workspace-view="conversation"] .conversation-mode-workspace',
  );
  await expect(
    primaryWorkspace.getByRole("heading", { name: "Tasks" }),
  ).toBeVisible();
  await expect(
    primaryWorkspace.getByRole("button", { name: "New task" }),
  ).toBeVisible();
  await expect(
    primaryWorkspace.getByText("Review local task records"),
  ).toBeVisible();
  await page.getByRole("button", { name: "Fictional mock inference" }).click();
  await page.setViewportSize({ width: 640, height: 450 });
  await page.evaluate(() => {
    document.documentElement.style.zoom = "2";
  });
  await expect(
    page.getByRole("heading", { name: "Fictional mock inference" }),
  ).toBeVisible();
  const input = page.getByLabel("Bounded authored input");
  await input.fill("Visible input");
  await page.getByRole("button", { name: "Prepare local mock review" }).click();
  await expect(page.getByText("Exact local review")).toBeVisible();
  await page
    .getByLabel("Fictional destination")
    .selectOption("ember-ambiguous");
  await expect(page.getByText("Exact local review")).toHaveCount(0);
  await expect(page.getByText(/reviewed binding changed/i)).toBeVisible();
  await input.fill("Changed visible input");
  await expect(page.getByText("Exact local review")).toHaveCount(0);
  await expect(page.getByText(/reviewed binding changed/i)).toBeVisible();
  await expect(
    page.getByRole("button", { name: "Authorize one local mock submission" }),
  ).toHaveCount(0);
  await input.fill("Fresh visible input");
  await page
    .getByRole("button", { name: "Prepare local mock review" })
    .press("Enter");
  await page
    .getByRole("button", { name: "Authorize one local mock submission" })
    .press("Enter");
  await page
    .getByRole("button", { name: "Submit deterministic mock" })
    .press("Enter");
  await expect(
    page.getByRole("button", {
      name: "Continue bounded local fixture stream",
    }),
  ).toBeEnabled();
  await page
    .getByRole("button", { name: "Continue bounded local fixture stream" })
    .press("Enter");
  await expect(page.getByText("incremental fixture output")).toBeVisible();
  await page.getByRole("button", { name: "Cancel" }).press("Enter");
  await expect(page.getByText("cancellation-requested")).toBeVisible();
  await page
    .getByRole("button", { name: "Continue bounded local fixture stream" })
    .press("Enter");
  await expect(page.getByText(/Mock attempt is cancelled/i)).toBeVisible();
  await page
    .getByRole("button", { name: "Prepare fresh retry or regeneration" })
    .press("Enter");
  await expect(page.getByText("Prior local attempt")).toBeVisible();
  await expect(page.getByText("Exact local review")).toHaveCount(0);
  await page.getByRole("button", { name: "Close", exact: true }).press("Enter");
  await expect(
    page.getByRole("button", { name: "Fictional mock inference" }),
  ).toBeFocused();
});

test("production New task creates one durable task for the fictional mock selector", async ({
  page,
}) => {
  const taskId = "018f0000-0000-7000-8000-000000000055";
  const planId = "018f0000-0000-7000-8000-000000000056";
  const createdCatalog = {
    ...taskCatalogFixture,
    tasks: [
      {
        id: taskId,
        title: "Untitled task",
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
      title: "Untitled task",
      status: "active",
      archived: false,
      selectedPlanId: planId,
      planCount: 1,
      updatedAtMs: 1,
      cleanupEligible: false,
    },
    plans: [
      {
        id: planId,
        label: "Primary plan",
        position: 0,
        body: "A governed local durable task.",
      },
    ],
    taskCount: 1,
  } as const;
  const emptyCatalog = {
    ...taskCatalogFixture,
    state: "empty",
    tasks: [],
    selectedTask: null,
    plans: [],
    taskCount: 0,
  } as const;

  await installNativeFixture(page, {
    ...nativeResponses,
    task_catalog_status: { sequence: [emptyCatalog, createdCatalog] },
    task_catalog_create: createdCatalog,
    mock_inference_catalog: {
      schemaVersion: 1,
      profiles: [
        {
          id: "lantern-stream",
          providerLabel: "Fictional Lantern",
          endpointLabel: "Local fixture endpoint",
          modelLabel: "Lantern Text Fixture",
          adapterLabel: "registry fixture adapter",
          scenario: "streamed-text",
          descriptorSha256: "a".repeat(64),
          capabilityProfileSha256: "a".repeat(64),
        },
      ],
    },
  });
  await page.goto("/");
  await openWorkspace(page, "New task");
  await page.getByRole("button", { name: "New task" }).click();
  await expect(page.getByText("Untitled task")).toBeVisible();
  await page.getByRole("button", { name: "Fictional mock inference" }).click();
  const selector = page.getByRole("combobox", { name: "Durable task" });
  await expect(selector).toBeEnabled();
  await expect(selector).toHaveValue(taskId);
  await expect(selector.locator("option")).toHaveText(["Untitled task"]);
});

test("mock inference exposes an ambiguous result without automatic retry", async ({
  page,
}) => {
  const id = "018f0000-0000-7000-8000-000000000021";
  const attemptId = "019a5900-0000-7000-8000-000000000022";
  const authorizationId = "019a5900-0000-7000-8000-000000000023";
  const digest = "c".repeat(64);
  const ready = {
    schemaVersion: 1,
    mockOnly: true,
    attemptId,
    state: "ready",
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
      state: "pending",
      expiresAtTick: 3,
    },
    events: [],
    usage: null,
    evidence: [],
  } as const;
  const submitted = {
    ...ready,
    state: "submitted",
    authorization: { ...ready.authorization, state: "consumed" },
  } as const;
  const ambiguous = {
    ...submitted,
    state: "ambiguous",
    events: [
      {
        id,
        sequence: 1,
        kind: "ambiguous-outcome",
        text: "no automatic retry",
        structuredState: null,
        sha256: digest,
      },
    ],
  } as const;
  await installNativeFixture(page, {
    ...nativeResponses,
    mock_inference_catalog: {
      schemaVersion: 1,
      profiles: [
        {
          id: "ember-ambiguous",
          providerLabel: "Fictional Ember",
          endpointLabel: "Local fixture endpoint",
          modelLabel: "Ember Ambiguity Fixture",
          adapterLabel: "registry fixture adapter",
          scenario: "ambiguous",
          descriptorSha256: digest,
          capabilityProfileSha256: digest,
        },
      ],
    },
    mock_inference_prepare: ready,
    mock_inference_authorize: {
      ...ready,
      state: "authorized",
      authorization: { ...ready.authorization, state: "authorized" },
    },
    mock_inference_submit: submitted,
    mock_inference_poll: ambiguous,
  });
  await page.goto("/");
  await openWorkspace(page, "New task");
  await page.getByRole("button", { name: "Fictional mock inference" }).click();
  await page.getByLabel("Bounded authored input").fill("Visible input");
  await page.getByRole("button", { name: "Prepare local mock review" }).click();
  await page
    .getByRole("button", { name: "Authorize one local mock submission" })
    .click();
  await page.getByRole("button", { name: "Submit deterministic mock" }).click();
  await page
    .getByRole("button", { name: "Continue bounded local fixture stream" })
    .click();
  await expect(page.getByText(/Mock attempt is ambiguous/i)).toBeVisible();
  await expect(page.getByText("no automatic retry")).toBeVisible();
  await expect(
    page.getByRole("button", { name: "Continue bounded local fixture stream" }),
  ).toHaveCount(0);
  await expect(
    page.getByRole("button", { name: "Prepare fresh retry or regeneration" }),
  ).toBeEnabled();
});

test("desktop preview renders the honest semantic shell", async ({ page }) => {
  const response = await page.goto("/");

  expect(response?.ok()).toBe(true);
  await expect(
    page.getByRole("heading", { name: "Welcome to QuireForge." }),
  ).toBeVisible();
  await expect(
    page.getByText(
      "Native Codex authentication is unavailable in this browser preview.",
    ),
  ).toBeVisible();
  await expect(page.getByRole("button", { name: "Try again" })).toBeEnabled();
  await expect(page.getByRole("navigation")).toHaveCount(0);
  await expect(page.getByRole("button", { name: "Start task" })).toHaveCount(0);
  await expect(page.locator("main h1")).toHaveCount(1);

  const overflow = await page.evaluate(
    () => document.documentElement.scrollWidth - window.innerWidth,
  );
  expect(overflow).toBeLessThanOrEqual(1);
});

test("authenticated home shows real usage without milestone labels", async ({
  page,
}) => {
  await installNativeFixture(page);
  await page.goto("/");

  await expect(
    page.getByRole("heading", { name: "What should we build today?" }),
  ).toBeVisible();
  await expect(
    page.getByRole("heading", { name: "Current workspace" }),
  ).toBeVisible();
  await expect(page.getByText("Verified Git workspace")).toBeVisible();
  await openAccountSettings(page);
  await expect(
    page.getByRole("heading", { name: "Codex owns authentication." }),
  ).toBeVisible();
  await expect(page.getByText("73% remaining")).toBeVisible();
  await expect(page.getByText("99% remaining")).toBeVisible();
  await expect(page.getByText(/Milestone/u)).toHaveCount(0);

  await page
    .getByRole("region", { name: "Codex usage limits" })
    .getByRole("button", { name: "Refresh", exact: true })
    .click();
  await expect(page.getByText("73% remaining")).toBeVisible();
  const accessibility = await new AxeBuilder({ page }).analyze();
  expect(accessibility.violations).toEqual([]);
});

test("every sidebar destination replaces the active workspace without page scrolling", async ({
  page,
}) => {
  await installNativeFixture(page);
  await page.goto("/");

  const destinations = [
    ["Home", "home"],
    ["Advisor", "advisor"],
    ["New task", "conversation"],
    ["Projects", "projects"],
    ["Project state", "project-state"],
    ["Threads", "sessions"],
    ["Scheduled", "scheduled"],
    ["Integrations", "integrations"],
    ["Files", "files"],
    ["Changes", "changes"],
    ["Worktrees", "worktrees"],
    ["Terminal", "terminal"],
  ] as const;

  for (const [label, route] of destinations) {
    await openWorkspace(page, label);
    await expect(
      page.locator(`[data-workspace-view="${route}"]`),
    ).toBeVisible();
    await expect(page.locator(".workspace-view:not([hidden])")).toHaveCount(1);
    expect(
      await page.evaluate(
        () =>
          document.documentElement.scrollTop +
          document.body.scrollTop +
          window.scrollY,
      ),
    ).toBe(0);
  }
});

test("Advisor presents a bounded chat-first conversation with safe summaries", async ({
  page,
}, testInfo) => {
  await installNativeFixture(page);
  await page.goto("/");

  await openWorkspace(page, "Advisor");
  await expect
    .poll(() =>
      page.evaluate(() =>
        window.localStorage.getItem(
          "quireforge-workspace-boundary-acknowledgment",
        ),
      ),
    )
    .toBe(
      JSON.stringify({
        schemaVersion: 1,
        boundaryPolicyVersion: "advisor-quireforge-boundary-v1",
        acknowledged: true,
      }),
    );
  const advisor = page.locator('[data-workspace-view="advisor"]');
  await expect(
    advisor.getByRole("heading", {
      name: "Advisor",
      exact: true,
    }),
  ).toBeVisible();
  await expect(
    advisor.getByText("Create, Learn, Explore · Read-only"),
  ).toBeVisible();
  const details = advisor.getByRole("button", {
    name: "Details",
    exact: true,
  });
  await expect(details).toHaveAttribute("aria-expanded", "false");
  await expect(
    advisor.getByRole("complementary", { name: "Advisor details" }),
  ).toHaveCount(0);
  await details.click();
  await expect(
    advisor.getByRole("complementary", { name: "Advisor details" }),
  ).toContainText("no shell, terminal, Git");
  await details.click();
  const workspaceSelector = page.getByRole("button", {
    name: "Advisor",
    exact: true,
  });
  if ((page.viewportSize()?.width ?? 0) <= 760) {
    await page.getByRole("button", { name: "Open navigation" }).click();
  }
  await workspaceSelector.click();
  const workspaceMenu = page.getByRole("menu", { name: "Choose workspace" });
  await expect(workspaceMenu.getByRole("menuitemradio")).toHaveText([
    "AdvisorCreate, learn, and explore✓",
    "QuireForgeBuild, debug, and ship",
  ]);
  await workspaceMenu
    .getByRole("menuitemradio", { name: /QuireForge/u })
    .click();
  const modeConfirmation = page.getByRole("dialog", {
    name: "Confirm conversation mode change",
  });
  await expect(modeConfirmation).toHaveCount(0);
  await expect(
    page.getByRole("button", { name: "QuireForge", exact: true }),
  ).toBeVisible();
  if ((page.viewportSize()?.width ?? 0) <= 760) {
    await page.getByRole("button", { name: "Open navigation" }).click();
  }
  await page.getByRole("button", { name: "QuireForge", exact: true }).click();
  await page.getByRole("menuitemradio", { name: /Advisor/u }).click();
  await expect(modeConfirmation).toHaveCount(0);
  await expect(workspaceSelector).toHaveAccessibleName("Advisor");
  const transcript = advisor.getByRole("log", {
    name: "Active Advisor conversation",
  });
  await expect(transcript).toHaveCSS("overflow-y", "auto");
  await expect(transcript).toHaveAttribute("tabindex", "0");
  await expect(
    advisor.getByText("project-state: verified, current"),
  ).toHaveCount(0);
  await expect(
    advisor.getByRole("textbox", { name: "Advisor message" }),
  ).toBeVisible();
  const advisorNotice = advisor
    .getByRole("note")
    .filter({ hasText: "Advisor is read-only" });
  await expect(advisorNotice).toBeVisible();
  await expect(advisor.getByRole("status")).toContainText(
    "Enter a message to send.",
  );
  const sendButton = advisor.getByRole("button", { name: "Send to Advisor" });
  await expect(advisor.locator(".conversation-composer")).toHaveCSS(
    "position",
    "relative",
  );
  await expect(sendButton).toBeDisabled();
  await advisor
    .getByRole("textbox", { name: "Advisor message" })
    .fill("Review the current boundary.");
  await expect(sendButton).toBeEnabled();
  await expect(advisor.getByText("Enter a message to send.")).toHaveCount(0);
  await expect(async () => {
    const noticeBox = await advisorNotice.boundingBox();
    const buttonBox = await sendButton.boundingBox();
    expect(noticeBox).not.toBeNull();
    expect(buttonBox).not.toBeNull();
    expect(noticeBox!.y + noticeBox!.height).toBeLessThanOrEqual(buttonBox!.y);
  }).toPass();
  await advisor
    .getByRole("button", { name: "Select current Project State snapshot" })
    .click();
  await expect(
    advisor.getByRole("dialog", { name: "Confirm Project State selection" }),
  ).toBeVisible();
  await advisor.getByRole("button", { name: "Confirm selection" }).click();
  await expect(
    advisor.getByRole("button", { name: "Remove temporary snapshot" }),
  ).toBeVisible();
  await expect(advisor.getByText(/advisor-thread|gpt-5.6/u)).toHaveCount(0);
  await expect(advisor.getByText(/\/mnt\//u)).toHaveCount(0);
  const overflow = await page.evaluate(
    () => document.documentElement.scrollWidth - window.innerWidth,
  );
  expect(overflow).toBeLessThanOrEqual(1);
  await page.screenshot({
    path: testInfo.outputPath(
      `advisor-action-row-${testInfo.project.name.includes("mobile") ? "mobile" : "desktop"}.png`,
    ),
    animations: "disabled",
  });
  // Hash-route navigation links are application controls, not skip links. The
  // inactive route target is intentionally hidden, so axe's static skip-link
  // heuristic does not apply to this routed mobile view.
  const accessibility = await new AxeBuilder({ page })
    .disableRules(["skip-link"])
    .analyze();
  expect(accessibility.violations).toEqual([]);
});

test("Advisor keeps a long transient reply reachable without forcing a reader to the latest update", async ({
  page,
}) => {
  const longReply = Array.from(
    { length: 160 },
    (_, index) =>
      `Bounded Advisor reply line ${index + 1}: review remains transient and read-only.`,
  ).join("\n");
  await installNativeFixture(page, {
    ...nativeResponses,
    advisor_conversation_start: {
      schemaVersion: 1,
      mode: "advisor",
      state: "completed",
      conversationId: "018f0000-0000-7000-8000-000000000062",
      projectStateIncluded: false,
      events: [
        {
          type: "agent-message-delta",
          sequence: 1,
          delta: longReply,
        },
      ],
      diagnosticCode: null,
    },
  });
  await page.goto("/");
  await openWorkspace(page, "Advisor");

  const advisor = page.locator('[data-workspace-view="advisor"]');
  await advisor
    .getByRole("textbox", { name: "Advisor message" })
    .fill("Review the bounded response.");
  await advisor.getByRole("button", { name: "Send to Advisor" }).click();
  const transcript = advisor.getByRole("log", {
    name: "Active Advisor conversation",
  });
  await expect(transcript).toHaveCSS("overflow-y", "auto");
  await expect
    .poll(() =>
      transcript.evaluate((node) => node.scrollHeight > node.clientHeight),
    )
    .toBe(true);

  await transcript.evaluate((node) => {
    node.scrollTop = 0;
    node.dispatchEvent(new Event("scroll", { bubbles: true }));
  });
  const jump = advisor.getByRole("button", { name: "Jump to latest" });
  await expect(jump).toBeVisible();
  await jump.click();
  await expect
    .poll(() =>
      transcript.evaluate(
        (node) => node.scrollHeight - node.scrollTop - node.clientHeight <= 1,
      ),
    )
    .toBe(true);
  await expect(jump).toHaveCount(0);

  const reply = advisor.getByText("Bounded Advisor reply line 160:");
  const composer = advisor.locator(".conversation-composer");
  await expect(async () => {
    const replyBox = await reply.boundingBox();
    const composerBox = await composer.boundingBox();
    expect(replyBox).not.toBeNull();
    expect(composerBox).not.toBeNull();
    expect(replyBox!.y + replyBox!.height).toBeLessThanOrEqual(composerBox!.y);
  }).toPass();
});

test("project state workspace presents read-only normalized evidence accessibly", async ({
  page,
}) => {
  await installNativeFixture(page);
  await page.goto("/");

  await openWorkspace(page, "Project state");
  await expect(
    page.getByRole("heading", {
      name: "Project state, without automation.",
    }),
  ).toBeVisible();
  await expect(page.getByText("Validation and packages")).toBeVisible();
  await expect(
    page.getByText("Remote tracking freshness was not requested."),
  ).toBeVisible();
  await expect(
    page.getByRole("button", { name: /approve|resolve|fetch/iu }),
  ).toHaveCount(0);
  const accessibility = await new AxeBuilder({ page }).analyze();
  expect(accessibility.violations).toEqual([]);
  const overflow = await page.evaluate(
    () => document.documentElement.scrollWidth - window.innerWidth,
  );
  expect(overflow).toBeLessThanOrEqual(1);
});

test("visual polish keeps the branded shell and composer accessible", async ({
  page,
  isMobile,
}, testInfo) => {
  await installNativeFixture(page);
  await page.goto("/");

  const homeComposer = page.locator('[data-visual-region="home-composer"]');
  await expect(homeComposer).toBeVisible();
  expect(
    await homeComposer.evaluate(
      (element) => getComputedStyle(element).borderRadius,
    ),
  ).toBe("22px");

  await openWorkspace(page, "New task");
  const composer = page.locator('[data-visual-region="conversation-composer"]');
  await expect(composer).toBeVisible();
  expect(
    await composer.evaluate(
      (element) => getComputedStyle(element).borderRadius,
    ),
  ).toBe("16px");
  await page.getByRole("textbox", { name: "Task" }).focus();
  await expect(page.getByRole("textbox", { name: "Task" })).toBeFocused();
  await expect(page.locator(".sidebar .nav-item--active")).toContainText(
    "New task",
  );

  const accessibility = await new AxeBuilder({ page }).analyze();
  expect(accessibility.violations).toEqual([]);
  const overflow = await page.evaluate(
    () => document.documentElement.scrollWidth - window.innerWidth,
  );
  expect(overflow).toBeLessThanOrEqual(1);
  await page.screenshot({
    path: testInfo.outputPath(
      `visual-polish-composer-${isMobile ? "mobile" : "desktop"}.png`,
    ),
    animations: "disabled",
  });
});

test("QuireForge workbench controls remain optional, keyboard-operable, and bounded", async ({
  page,
}) => {
  await installNativeFixture(page);
  await page.goto("/");
  await openWorkspace(page, "New task");

  await expect(
    page.getByRole("heading", { name: "Workbench context" }),
  ).toHaveCount(0);
  const actions = page.getByRole("button", { name: "⌘ Actions" });
  await actions.click();
  await expect(
    page.getByRole("dialog", { name: "Command palette" }),
  ).toBeVisible();
  await expect(
    page.getByRole("menuitem", { name: "Open task conversation" }),
  ).toBeFocused();
  await page.keyboard.press("Escape");
  await expect(actions).toBeFocused();

  await page.getByRole("button", { name: "Show workbench context" }).click();
  await expect(
    page.getByRole("heading", { name: "Workbench context" }),
  ).toBeVisible();
  await page.getByRole("tab", { name: "Problems" }).click();
  await expect(page.getByText("No problem feed available")).toBeVisible();
  await expect(
    page.getByRole("button", { name: "Open terminal dock" }),
  ).toHaveAttribute("aria-expanded", "false");
  const taskList = page.getByRole("navigation", { name: "Task list" });
  await expect(taskList).toBeVisible();
  await expect(
    taskList.getByRole("button", {
      name: /Review local task records, active, not archived, 2 plans/u,
    }),
  ).toHaveAttribute("aria-current", "page");
  const primaryPlan = page.getByRole("tab", { name: "Primary plan" });
  await primaryPlan.focus();
  await page.keyboard.press("ArrowRight");
  await expect(
    page.getByRole("tab", { name: "Alternate plan 1" }),
  ).toBeFocused();
  await expect(
    page.getByText("Plan selected. Transient task-plan state was cleared."),
  ).toBeVisible();

  const deleteTask = page.getByRole("button", { name: "Delete task" });
  await deleteTask.click();
  const confirmation = page.getByRole("dialog", {
    name: "Delete “Review local task records”?",
  });
  await expect(
    confirmation.getByRole("button", { name: "Cancel", exact: true }),
  ).toBeFocused();
  await page.keyboard.press("Escape");
  await expect(deleteTask).toBeFocused();

  const accessibility = await new AxeBuilder({ page }).analyze();
  expect(accessibility.violations).toEqual([]);
  const overflow = await page.evaluate(
    () => document.documentElement.scrollWidth - window.innerWidth,
  );
  expect(overflow).toBeLessThanOrEqual(1);
});

test("native session fixture renders grouping, tabs, and bounded controls", async ({
  page,
}) => {
  await installNativeFixture(page);
  await page.goto("/");

  await openWorkspace(page, "Threads");
  await expect(page.getByText("2 app-owned sessions.")).toBeVisible();
  await expect(
    page.getByText(/Fork of Review lifecycle boundaries/u),
  ).toBeVisible();
  await page
    .locator("#sessions")
    .getByRole("button", {
      name: /Review lifecycle boundaries.*Completed/u,
    })
    .click();
  await expect(
    page.getByRole("tab", { name: "Review lifecycle boundaries" }),
  ).toHaveAttribute("aria-selected", "true");
  await expect(page.getByLabel("Next task")).toBeVisible();
  await expect(page.getByText("Effective now")).toBeVisible();
  await expect(page.getByText("Pending next turn")).toBeVisible();
  await expect(page.getByText("Requested by Codex")).toBeVisible();
  await expect(
    page.getByText("Recommendation — never automatic"),
  ).toBeVisible();
  await page.getByRole("button", { name: "Dismiss" }).click();
  await expect(page.getByText("No change")).toBeVisible();
  await expect(
    page.getByRole("button", { name: "Resume", exact: true }),
  ).toBeDisabled();

  const results = await new AxeBuilder({ page }).analyze();
  expect(results.violations).toEqual([]);
  const overflow = await page.evaluate(
    () => document.documentElement.scrollWidth - window.innerWidth,
  );
  expect(overflow).toBeLessThanOrEqual(1);
});

test("native file preview uses the bounded shared contract", async ({
  page,
}) => {
  await installNativeFixture(page);
  await page.goto("/");

  await openWorkspace(page, "Files");
  await page.getByRole("button", { name: "Choose project file" }).click();
  await expect(
    page.getByRole("article", { name: "Preview of docs/preview.md" }),
  ).toBeVisible();
  await expect(page.getByText("48 B")).toBeVisible();
  await expect(page.locator(".file-preview-text code")).toContainText(
    "Paths remain native-only.",
  );
  await page.getByRole("button", { name: "Open with desktop app" }).click();
  await expect(
    page.getByText("Destination · System default application"),
  ).toBeVisible();
  await page.getByRole("button", { name: "Open with default app" }).click();
  await expect(
    page.getByRole("button", { name: "Opened with desktop app" }),
  ).toBeDisabled();

  const results = await new AxeBuilder({ page }).analyze();
  expect(results.violations).toEqual([]);
  const overflow = await page.evaluate(
    () => document.documentElement.scrollWidth - window.innerWidth,
  );
  expect(overflow).toBeLessThanOrEqual(1);
});

test("native conversation attachments expose only bounded draft metadata", async ({
  page,
}) => {
  await installNativeFixture(page);
  await page.goto("/");

  await openWorkspace(page, "New task");
  await page.getByRole("button", { name: "Choose images" }).click();
  await expect(page.getByText("review.png")).toBeVisible();
  await expect(page.getByText(/67 B · 1 × 1 · drag drop/u)).toBeVisible();
  await expect(
    page.getByText(/sent only with Start, Resume, or Fork/u),
  ).toBeVisible();
  await page.getByRole("button", { name: "Remove review.png" }).click();
  await expect(page.getByText("review.png")).not.toBeVisible();

  const results = await new AxeBuilder({ page }).analyze();
  expect(results.violations).toEqual([]);
  const overflow = await page.evaluate(
    () => document.documentElement.scrollWidth - window.innerWidth,
  );
  expect(overflow).toBeLessThanOrEqual(1);
});

test("native terminal fixture mounts the app-owned xterm tab", async ({
  page,
}) => {
  await installNativeFixture(page, nativeTerminalResponses);
  await page.goto("/");

  await openWorkspace(page, "Terminal");
  await expect(
    page.getByRole("heading", { name: "A real shell, rooted where you work." }),
  ).toBeVisible();
  await expect(
    page.getByRole("button", { name: /Terminal 1 Running/u }),
  ).toHaveAttribute("aria-pressed", "true");
  await expect(page.locator(".terminal-pane__viewport .xterm")).toBeVisible();
  const closeButton = page.getByRole("button", { name: "Close Terminal 1" });
  await expect(closeButton).toBeVisible();
  await expect(page.getByText(/Linux account privileges/u)).toBeVisible();

  await closeButton.click();
  const closeReview = page.getByRole("alertdialog", {
    name: "Close Terminal 1?",
  });
  await expect(closeReview).toBeVisible();
  await expect(
    page.getByRole("button", { name: "End processes and close" }),
  ).toBeFocused();
  const accessibility = await new AxeBuilder({ page }).analyze();
  expect(accessibility.violations).toEqual([]);
  await page.keyboard.press("Escape");
  await expect(closeReview).toHaveCount(0);
  await expect(closeButton).toBeFocused();

  const overflow = await page.evaluate(
    () => document.documentElement.scrollWidth - window.innerWidth,
  );
  expect(overflow).toBeLessThanOrEqual(1);
});

test("native Integration Center reviews trust before a fixed mutation", async ({
  page,
}) => {
  await installNativeFixture(page);
  await page.goto("/");

  await openWorkspace(page, "Integrations");
  await expect(
    page.getByRole("heading", { name: "Inspect trust before changing state." }),
  ).toBeVisible();
  await expect(page.getByText("5 of 5 integrations")).toBeVisible();
  await page.getByLabel("Category").selectOption("plugin");
  await expect(
    page.getByRole("heading", { name: "Fixture review plugin" }),
  ).toBeVisible();
  await expect(page.getByText(/requires separate hook trust/u)).toBeVisible();

  await page.getByRole("button", { name: "Install plugin" }).click();
  const confirmation = page.getByRole("dialog", { name: "Install plugin" });
  await expect(confirmation).toContainText(
    "Authentication, if needed, remains a separate action.",
  );
  await expect(confirmation).toContainText("Pinned plugin repository");
  await confirmation.getByRole("button", { name: "Confirm change" }).click();
  await expect(
    page.getByText(/Install plugin completed and the catalog was refreshed/u),
  ).toBeVisible();

  await page.getByLabel("Category").selectOption("mcp-server");
  await page.getByRole("button", { name: "Authorize MCP server" }).click();
  const authorization = page.getByRole("dialog", {
    name: "Authorize MCP server",
  });
  await expect(authorization).toContainText(
    "exact authorization URL returned by Codex",
  );
  await authorization.getByRole("button", { name: "Confirm action" }).click();
  await page
    .getByRole("button", { name: "Open authorization in browser" })
    .click();
  await page.getByRole("button", { name: "Check authorization" }).click();
  await expect(
    page.getByText(
      /Authorize MCP server completed and the catalog was refreshed/u,
    ),
  ).toBeVisible();
  await expect(page.getByText(/authorizationUrl/u)).toHaveCount(0);

  const results = await new AxeBuilder({ page }).analyze();
  expect(results.violations).toEqual([]);
  const overflow = await page.evaluate(
    () => document.documentElement.scrollWidth - window.innerWidth,
  );
  expect(overflow).toBeLessThanOrEqual(1);
});

test("native Scheduled catalog presents inert plugin task templates", async ({
  page,
}) => {
  await installNativeFixture(page);
  await page.goto("/");

  await openWorkspace(page, "Scheduled");

  const scheduled = page.locator("#scheduled");
  await expect(
    scheduled.getByRole("heading", {
      name: "Review task templates without handing over control.",
    }),
  ).toBeVisible();
  await expect(scheduled.getByText("Weekly review")).toBeVisible();
  await expect(scheduled.getByText("Mon, Thu at 09:30")).toBeVisible();
  await expect(scheduled.getByText("Untrusted prompt preview")).toBeVisible();
  await expect(
    scheduled.getByText(
      /cannot create, edit, enable, run, pause, or delete scheduled tasks/u,
    ),
  ).toBeVisible();
  await expect(scheduled.getByRole("button")).toHaveCount(0);

  const results = await new AxeBuilder({ page })
    .include("#scheduled")
    .analyze();
  expect(results.violations).toEqual([]);
  const overflow = await page.evaluate(
    () => document.documentElement.scrollWidth - window.innerWidth,
  );
  expect(overflow).toBeLessThanOrEqual(1);
});

test("native Git fixture reviews a diff and confirms a fixed mutation", async ({
  page,
}) => {
  await installNativeFixture(page);
  await page.goto("/");

  await openWorkspace(page, "Changes");
  await expect(
    page
      .getByLabel("Review each Git change before applying it.")
      .getByText("feature/review"),
  ).toBeVisible();
  await page.getByRole("button", { name: "Working · modified" }).click();
  await expect(
    page.getByRole("table", { name: "Diff for src/App.tsx" }),
  ).toBeVisible();
  await expect(page.getByText("new line")).toBeVisible();
  await expect(
    page.getByRole("button", { name: "Open in default editor" }),
  ).toBeEnabled();
  await page.getByRole("button", { name: "Stage" }).click();
  const confirmation = page.getByRole("dialog", {
    name: "Stage change",
  });
  await expect(confirmation).toContainText("src/App.tsx");
  await confirmation.getByRole("button", { name: "Confirm stage" }).click();
  await expect(
    page.getByText("The repository was updated and revalidated."),
  ).toBeVisible();
  await expect(
    page.getByRole("button", { name: "Staged · modified" }),
  ).toBeVisible();
  const results = await new AxeBuilder({ page }).analyze();
  expect(results.violations).toEqual([]);
  const overflow = await page.evaluate(
    () => document.documentElement.scrollWidth - window.innerWidth,
  );
  expect(overflow).toBeLessThanOrEqual(1);
});

test("native worktree fixture reviews creation, recovery, and managed cleanup", async ({
  page,
}) => {
  await installNativeFixture(page);
  await page.goto("/");

  await openWorkspace(page, "Worktrees");
  await expect(
    page.getByRole("heading", {
      name: "Give each line of work its own checkout.",
    }),
  ).toBeVisible();
  await expect(page.getByText("external checkout")).toBeVisible();
  await page.getByLabel("New branch name").fill("feature/isolated");
  await page.getByRole("button", { name: "Preview managed worktree" }).click();
  await expect(page.getByText("Create feature/isolated")).toBeVisible();
  await expect(page.getByText("Non-destructive preview")).toBeVisible();
  await page.getByRole("button", { name: "Cancel" }).click();
  await expect(page.getByText("Create feature/isolated")).toHaveCount(0);

  await page.getByRole("button", { name: "Review recovery" }).click();
  await expect(page.getByText("Recover feature/recoverable")).toBeVisible();
  await expect(
    page.getByText(/registers this retained app-managed checkout/u),
  ).toBeVisible();
  await page.getByRole("button", { name: "Cancel" }).click();

  await page.getByRole("button", { name: "Review cleanup" }).click();
  await expect(page.getByText("Destructive cleanup preview")).toBeVisible();
  await expect(page.getByText(/Its branch is preserved/u)).toBeVisible();
  await expect(
    page.getByRole("button", { name: /force|prune|delete branch/u }),
  ).toHaveCount(0);

  const results = await new AxeBuilder({ page }).analyze();
  expect(results.violations).toEqual([]);
  const overflow = await page.evaluate(
    () => document.documentElement.scrollWidth - window.innerWidth,
  );
  expect(overflow).toBeLessThanOrEqual(1);
});

test("parallel worktree monitor opens live activity and reports conflicts", async ({
  page,
}) => {
  const conflictedGit = {
    ...nativeResponses.git_status,
    changes: [
      {
        path: "src/conflicted.ts",
        previousPath: null,
        staged: "unmerged",
        worktree: "unmerged",
        conflict: true,
        submodule: false,
        reviewable: false,
      },
    ],
  };
  await installNativeFixture(page, {
    ...nativeResponses,
    conversation_status: approvalConversation,
    conversation_active: {
      schemaVersion: 1,
      capacity: 4,
      conversations: [{ ...approvalConversation, events: [] }],
    },
    conversation_poll: approvalConversation,
    git_status: conflictedGit,
  });
  await page.goto("/");

  await openWorkspace(page, "Worktrees");
  await expect(page.getByText("1 of 4 active")).toBeVisible();
  await expect(page.getByText("Approval needed")).toBeVisible();
  await expect(page.getByText("1 conflict")).toBeVisible();
  await page.getByRole("button", { name: "View live activity" }).click();
  await expect(page.getByText("Codex is waiting for approval")).toBeVisible();
  const activity = page.getByRole("button", { name: /Run command/u });
  await expect(activity).toBeVisible();
  await activity.click();
  await expect(page.getByText("Checking the desktop contract…")).toBeVisible();

  const results = await new AxeBuilder({ page }).analyze();
  expect(results.violations).toEqual([]);
  const overflow = await page.evaluate(
    () => document.documentElement.scrollWidth - window.innerWidth,
  );
  expect(overflow).toBeLessThanOrEqual(1);
});

test("native activity fixture renders bounded real-time approval detail", async ({
  page,
}) => {
  await installNativeFixture(page, {
    ...nativeResponses,
    conversation_status: approvalConversation,
    conversation_poll: approvalConversation,
    conversation_approval_decide: approvedConversation,
  });
  await page.goto("/");

  await openWorkspace(page, "New task");
  await expect(page.getByText("Codex is waiting for approval")).toBeVisible();
  await expect(
    page.getByText("The project check needs permission."),
  ).toBeVisible();
  await expect(
    page.getByRole("button", { name: "Approve once" }),
  ).toBeVisible();
  await expect(page.getByRole("button", { name: "Decline" })).toBeVisible();
  await expect(page.getByRole("button", { name: "Cancel task" })).toBeVisible();
  const activity = page.getByRole("button", { name: /Run command/u });
  await expect(activity).toHaveAttribute("aria-expanded", "false");
  await activity.click();
  await expect(activity).toHaveAttribute("aria-expanded", "true");
  await expect(page.getByText("Checking the desktop contract…")).toBeVisible();
  await expect(
    page.getByText("Approval requested for command execution."),
  ).toBeVisible();
  await expect(page.getByRole("button", { name: "Stop task" })).toBeEnabled();

  await page.getByRole("button", { name: "Approve once" }).click();
  await expect(page.getByText("Task completed")).toBeVisible();
  await expect(page.getByText("Approval approved.")).toBeVisible();
  await expect(page.getByText("Run this command?")).toHaveCount(0);

  const results = await new AxeBuilder({ page }).analyze();
  expect(results.violations).toEqual([]);
  const overflow = await page.evaluate(
    () => document.documentElement.scrollWidth - window.innerWidth,
  );
  expect(overflow).toBeLessThanOrEqual(1);
});

test("desktop preview has no automatically detectable accessibility violations", async ({
  page,
}) => {
  await page.goto("/");
  const results = await new AxeBuilder({ page }).analyze();

  expect(results.violations).toEqual([]);
});

test("keyboard users can bypass navigation and use semantic workspace links", async ({
  page,
  isMobile,
}) => {
  await installNativeFixture(page);
  await page.goto("/");

  const skipLink = page.getByRole("link", { name: "Skip to workspace" });
  await expect(skipLink).toBeAttached();
  await page.keyboard.press("Tab");
  await expect(skipLink).toBeFocused();
  await expect(skipLink).toBeVisible();
  await page.keyboard.press("Enter");
  await expect(page.getByRole("main")).toBeFocused();

  if (isMobile) return;

  const terminalLink = page.getByRole("link", { name: "Terminal" });
  await expect(terminalLink).toHaveAttribute("href", "#terminal");
  await terminalLink.focus();
  await page.keyboard.press("Enter");
  await expect(page).toHaveURL(/#terminal$/u);
  await expect(
    page.getByRole("heading", {
      name: "A real shell, rooted where you work.",
    }),
  ).toBeVisible();
});

test("reduced-motion preference disables animation and scripted smooth scrolling", async ({
  page,
}) => {
  await page.emulateMedia({ reducedMotion: "reduce" });
  await page.addInitScript(() => {
    const state = window as typeof window & {
      __quireforgeScrollBehaviors: ScrollBehavior[];
    };
    state.__quireforgeScrollBehaviors = [];
    Element.prototype.scrollIntoView = function (
      options?: boolean | ScrollIntoViewOptions,
    ) {
      if (typeof options === "object" && options.behavior) {
        state.__quireforgeScrollBehaviors.push(options.behavior);
      }
    };
  });
  await installNativeFixture(page, nativeResponses);
  await page.goto("/");

  await page.evaluate(() => {
    const probe = document.createElement("span");
    probe.className = "conversation-pulse";
    probe.dataset.testid = "motion-probe";
    document.body.append(probe);
  });
  const animationDuration = await page
    .locator('[data-testid="motion-probe"]')
    .evaluate((element) => getComputedStyle(element).animationDuration);
  const animationDurationMs = animationDuration.endsWith("ms")
    ? Number.parseFloat(animationDuration)
    : Number.parseFloat(animationDuration) * 1_000;
  expect(animationDurationMs).toBeLessThanOrEqual(0.01);

  const newThread = page.getByRole("button", {
    name: "New task",
    exact: true,
  });
  await expect(newThread).toBeEnabled();
  await newThread.click();
  await expect(page).toHaveURL(/#conversation$/u);
  const behaviors = await page.evaluate(
    () =>
      (
        window as typeof window & {
          __quireforgeScrollBehaviors: ScrollBehavior[];
        }
      ).__quireforgeScrollBehaviors,
  );
  expect(behaviors).toEqual([]);
});

test("forced-colors mode retains visible controls without horizontal overflow", async ({
  page,
}) => {
  await installNativeFixture(page);
  await page.emulateMedia({ forcedColors: "active" });
  await page.goto("/");

  const appearance = page.getByRole("button", {
    name: "Open appearance settings",
  });
  await expect(appearance).toBeVisible();
  await appearance.focus();
  const outlineStyle = await appearance.evaluate(
    (element) => getComputedStyle(element).outlineStyle,
  );
  expect(outlineStyle).not.toBe("none");
  const overflow = await page.evaluate(
    () => document.documentElement.scrollWidth - window.innerWidth,
  );
  expect(overflow).toBeLessThanOrEqual(1);
});

test("appearance picker previews, persists, and keeps every palette accessible", async ({
  page,
  isMobile,
}, testInfo) => {
  await installNativeFixture(page);
  await page.goto("/#settings/appearance");
  const picker = page.getByRole("radiogroup", { name: "QuireForge theme" });
  await expect(picker).toBeVisible();

  const themeIds = [
    "forge",
    "midnight-atelier",
    "blueprint-terminal",
    "signal-noir",
    "aurora-workbench",
    "obsidian-copper",
    "monochrome-editorial",
    "pacific-night",
  ];
  const themeLabels = [
    "Forge",
    "Midnight Atelier",
    "Blueprint Terminal",
    "Signal Noir",
    "Aurora Workbench",
    "Obsidian & Copper",
    "Monochrome Editorial",
    "Pacific Night",
  ];

  for (const [index, label] of themeLabels.entries()) {
    const option = page.getByRole("radio", { name: new RegExp(label, "u") });
    await option.click();
    await expect(page.locator("html")).toHaveAttribute(
      "data-theme",
      themeIds[index]!,
    );
    await expect(option).toHaveAttribute("aria-checked", "true");
    const contrast = await page.evaluate(() => {
      const parseHex = (value: string) => {
        const match = value.trim().match(/^#([0-9a-f]{6})$/iu);
        if (!match)
          throw new Error(`Expected an opaque hex color, got ${value}`);
        return [0, 2, 4].map(
          (start) =>
            Number.parseInt(match[1]!.slice(start, start + 2), 16) / 255,
        );
      };
      const luminance = (color: number[]) =>
        color.reduce(
          (total, component, index) =>
            total +
            (component <= 0.03928
              ? component / 12.92
              : ((component + 0.055) / 1.055) ** 2.4) *
              [0.2126, 0.7152, 0.0722][index]!,
          0,
        );
      const ratio = (foreground: string, background: string) => {
        const [lighter, darker] = [
          luminance(parseHex(foreground)),
          luminance(parseHex(background)),
        ].sort((left, right) => right - left);
        return (lighter! + 0.05) / (darker! + 0.05);
      };
      const style = getComputedStyle(document.documentElement);
      const background = style.getPropertyValue("--bg");
      return Object.fromEntries(
        ["--text", "--tm", "--accent", "--green", "--warning", "--danger"].map(
          (token) => [token, ratio(style.getPropertyValue(token), background)],
        ),
      );
    });
    for (const [token, value] of Object.entries(contrast)) {
      expect(value, `${label} ${token} contrast`).toBeGreaterThanOrEqual(4.5);
    }
  }

  const forge = page.getByRole("radio", { name: /Forge/u });
  const midnight = page.getByRole("radio", { name: /Midnight Atelier/u });
  await forge.hover();
  await expect(page.locator("html")).toHaveAttribute("data-theme", "forge");
  await page.locator(".settings-section__heading").hover();
  await expect(page.locator("html")).toHaveAttribute(
    "data-theme",
    "pacific-night",
  );
  await forge.focus();
  await page.keyboard.press("ArrowRight");
  await expect(midnight).toHaveAttribute("aria-checked", "true");
  await expect(page.locator("html")).toHaveAttribute(
    "data-theme",
    "midnight-atelier",
  );
  await page.reload();
  await expect(page.locator("html")).toHaveAttribute(
    "data-theme",
    "midnight-atelier",
  );

  const accessibility = await new AxeBuilder({ page }).analyze();
  expect(accessibility.violations).toEqual([]);
  const overflow = await page.evaluate(
    () => document.documentElement.scrollWidth - window.innerWidth,
  );
  expect(overflow).toBeLessThanOrEqual(1);
  await page.screenshot({
    path: testInfo.outputPath(
      `appearance-themes-${isMobile ? "mobile" : "desktop"}.png`,
    ),
    animations: "disabled",
  });
});

test("captures routed desktop workspace evidence", async ({
  page,
  isMobile,
}, testInfo) => {
  await installNativeFixture(page);

  if (isMobile) {
    await page.goto("/");
    await expect(
      page.getByRole("heading", { name: "What should we build today?" }),
    ).toBeVisible();
    await page.screenshot({
      path: testInfo.outputPath("after-responsive-home.png"),
      animations: "disabled",
    });
    await page.getByRole("button", { name: "Open navigation" }).click();
    await page.screenshot({
      path: testInfo.outputPath("after-responsive-navigation.png"),
      animations: "disabled",
    });
    await page.keyboard.press("Escape");
    await expect(
      page.getByRole("button", { name: "Open navigation" }),
    ).toBeVisible();
    for (const label of [
      "Scheduled",
      "Integrations",
      "Files",
      "Project state",
    ] as const) {
      await openWorkspace(page, label);
      await page.screenshot({
        path: testInfo.outputPath(
          `after-mobile-${label.toLowerCase()}-drawer.png`,
        ),
        animations: "disabled",
      });
    }
    await openAccountSettings(page);
    await page.screenshot({
      path: testInfo.outputPath("after-mobile-settings-drawer.png"),
      animations: "disabled",
    });
    return;
  }

  await page.setViewportSize({ width: 1920, height: 1080 });
  await page.goto("/");
  await expect(
    page.getByRole("heading", { name: "What should we build today?" }),
  ).toBeVisible();
  await page.screenshot({
    path: testInfo.outputPath("after-home-three-pane-1920x1080.png"),
    animations: "disabled",
  });

  await page.setViewportSize({ width: 1440, height: 900 });
  for (const label of [
    "New task",
    "Projects",
    "Project state",
    "Threads",
    "Scheduled",
    "Integrations",
    "Files",
    "Changes",
    "Worktrees",
    "Terminal",
  ]) {
    await openWorkspace(page, label);
    await page.screenshot({
      path: testInfo.outputPath(
        `after-${label.toLowerCase().replaceAll(" ", "-")}-1440x900.png`,
      ),
      animations: "disabled",
    });
  }

  await page.setViewportSize({ width: 1366, height: 768 });
  await openAccountSettings(page);
  await page.screenshot({
    path: testInfo.outputPath("after-settings-account-1366x768.png"),
    animations: "disabled",
  });

  await page.setViewportSize({ width: 720, height: 900 });
  await openWorkspace(page, "Changes");
  await page.screenshot({
    path: testInfo.outputPath("after-collapsed-changes-720x900.png"),
    animations: "disabled",
  });
});
