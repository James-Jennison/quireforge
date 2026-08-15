import { render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { ContextAuthorityLedger } from "./ContextAuthorityLedger";

const bridge = vi.hoisted(() => ({ loadContextAuthorityLedger: vi.fn() }));
vi.mock("./lib/bridge", () => bridge);

describe("ContextAuthorityLedger", () => {
  it("shows content-free receipt lifecycle metadata only", async () => {
    bridge.loadContextAuthorityLedger.mockResolvedValueOnce({
      schemaVersion: 1,
      projectId: "019fbee6-476f-71b0-853c-f067657aa69c",
      diagnostic: null,
      entries: [
        {
          recordKind: "context-bundle",
          recordId: "019fbee6-476f-71b0-853c-f067657aa69b",
          projectId: "019fbee6-476f-71b0-853c-f067657aa69c",
          taskId: null,
          state: "closed",
          bundleDigest:
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
          itemCount: 2,
          expiresAtMs: 2,
          createdAtMs: 1,
          completedAtMs: 2,
          auditOutcome: "closed",
        },
        {
          recordKind: "artifact-reference",
          recordId: "019fbee6-476f-71b0-853c-f067657aa69d",
          projectId: "019fbee6-476f-71b0-853c-f067657aa69c",
          taskId: null,
          state: "active",
          bundleDigest:
            "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789",
          itemCount: 0,
          expiresAtMs: 0,
          createdAtMs: 3,
          completedAtMs: null,
          auditOutcome: "active",
        },
      ],
    });
    render(
      <ContextAuthorityLedger projectId="019fbee6-476f-71b0-853c-f067657aa69c" />,
    );
    await waitFor(() =>
      expect(screen.getByText(/2 selected items/)).toBeInTheDocument(),
    );
    expect(screen.getByText(/cannot transfer content/i)).toBeInTheDocument();
    expect(screen.getByText("artifact-reference: active")).toBeInTheDocument();
  });
});
