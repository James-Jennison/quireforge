import { describe, expect, it } from "vitest";

import {
  aggregateThreadStatus,
  groupThreadTree,
  projectThreadTree,
  statusForThread,
  threadTreeSnapshotSchema,
} from "./threadTree";

describe("thread tree projection", () => {
  it("uses only bounded thread presentation fields", () => {
    const parsed = threadTreeSnapshotSchema.safeParse({
      schemaVersion: 1,
      state: "ready",
      threads: [
        {
          id: "thread-1",
          title: "Review local boundaries",
          projectLabel: "QuireForge",
          status: "unread",
          path: "/private/source",
        },
      ],
    });
    expect(parsed.success).toBe(false);
  });

  it("marks an existing reference unread until this app session opens it", () => {
    const snapshot = projectThreadTree(
      [
        {
          conversationId: "thread-1",
          title: "Review local boundaries",
          projectLabel: "QuireForge",
        },
      ],
      new Set(),
    );
    expect(snapshot.threads[0]?.status).toBe("unread");
    expect(
      projectThreadTree(
        [
          {
            conversationId: "thread-1",
            title: "Review local boundaries",
            projectLabel: "QuireForge",
          },
        ],
        new Set(["thread-1"]),
      ).threads[0]?.status,
    ).toBe("none");
  });

  it("fails unknown statuses closed and preserves a real decision in a folder", () => {
    expect(statusForThread("loading")).toBe("none");
    expect(aggregateThreadStatus(["unread", "needsDecision"])).toBe(
      "needsDecision",
    );
    const [group] = groupThreadTree(
      threadTreeSnapshotSchema.parse({
        schemaVersion: 1,
        state: "ready",
        threads: [
          {
            id: "thread-1",
            title: "A",
            projectLabel: "QuireForge",
            status: "unread",
          },
          {
            id: "thread-2",
            title: "B",
            projectLabel: "QuireForge",
            status: "needsDecision",
          },
        ],
      }),
    );
    expect(group?.status).toBe("needsDecision");
  });
});
