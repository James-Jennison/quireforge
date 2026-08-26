import { z } from "zod";

export const threadStatusSchema = z.enum(["none", "unread", "needsDecision"]);
export type ThreadStatus = z.infer<typeof threadStatusSchema>;

export const threadTreeSnapshotSchema = z
  .object({
    schemaVersion: z.literal(1),
    state: z.enum(["empty", "ready"]),
    threads: z
      .array(
        z
          .object({
            id: z.string().min(1).max(128),
            title: z.string().min(1).max(256),
            projectLabel: z.string().min(1).max(256).nullable(),
            status: threadStatusSchema,
          })
          .strict(),
      )
      .max(256),
  })
  .strict();

export type ThreadTreeSnapshot = z.infer<typeof threadTreeSnapshotSchema>;

export interface ThreadTreeSource {
  conversationId: string;
  title: string | null;
  projectLabel: string | null;
}

export interface ThreadTreeGroup {
  label: string;
  status: ThreadStatus;
  threads: ThreadTreeSnapshot["threads"];
}

const threadTitleFallback = "Untitled thread";

export function statusForThread(value: unknown): ThreadStatus {
  return threadStatusSchema.safeParse(value).data ?? "none";
}

export function aggregateThreadStatus(
  statuses: readonly unknown[],
): ThreadStatus {
  const closed = statuses.map(statusForThread);
  if (closed.includes("needsDecision")) return "needsDecision";
  if (closed.includes("unread")) return "unread";
  return "none";
}

export function projectThreadTree(
  sources: readonly ThreadTreeSource[],
  viewedThreadIds: ReadonlySet<string>,
): ThreadTreeSnapshot {
  const threads = sources.map((source) => ({
    id: source.conversationId,
    title: source.title ?? threadTitleFallback,
    projectLabel: source.projectLabel,
    // M69B has no native pending-decision event. A thread is unread only until
    // this app session has opened the existing bounded reference.
    status: viewedThreadIds.has(source.conversationId) ? "none" : "unread",
  }));

  return threadTreeSnapshotSchema.parse({
    schemaVersion: 1,
    state: threads.length === 0 ? "empty" : "ready",
    threads,
  });
}

export function groupThreadTree(
  snapshot: ThreadTreeSnapshot,
): readonly ThreadTreeGroup[] {
  const groups = new Map<string, ThreadTreeSnapshot["threads"]>();
  for (const thread of snapshot.threads) {
    const label = thread.projectLabel ?? "No project";
    const group = groups.get(label) ?? [];
    group.push(thread);
    groups.set(label, group);
  }
  return [...groups.entries()].map(([label, threads]) => ({
    label,
    status: aggregateThreadStatus(threads.map((thread) => thread.status)),
    threads,
  }));
}
