import type {
  ConversationEvent,
  ConversationSnapshot,
} from "../lib/conversation";
import type { FilePreviewSnapshot } from "../lib/filePreview";
import type { GitDiffSnapshot, GitWorkspaceSnapshot } from "../lib/git";
import type {
  GeneratedArtifactPreview,
  GeneratedArtifactSnapshot,
} from "../lib/advisorGeneratedArtifact";

export const reviewPaneIds = [
  "files",
  "diff",
  "git",
  "preview",
  "activity",
  "approval",
] as const;

export type ReviewPaneId = (typeof reviewPaneIds)[number];

export interface ReviewPaneData {
  projectId: string | null;
  projectName: string | null;
  filePreview: FilePreviewSnapshot;
  conversation: ConversationSnapshot;
  conversationEvents: ConversationEvent[];
  loadGitStatus: (projectId: string) => Promise<GitWorkspaceSnapshot>;
  loadGitDiff: (request: {
    projectId: string;
    path: string;
    area: "staged" | "worktree";
  }) => Promise<GitDiffSnapshot>;
  loadArtifacts: () => Promise<GeneratedArtifactSnapshot>;
  previewArtifact: (request: {
    artifactId: string;
    manifestSha256: string;
  }) => Promise<GeneratedArtifactPreview>;
}
