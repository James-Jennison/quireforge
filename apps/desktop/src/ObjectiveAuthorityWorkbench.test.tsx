import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

const projectId = "019fbee6-476f-71b0-853c-f067657aa69c";
const objectiveId = "019fbee6-476f-71b0-853c-f067657aa69d";

const operations = vi.hoisted(() => ({
  load: vi.fn(),
  create: vi.fn(),
  activate: vi.fn(),
  revoke: vi.fn(),
}));

vi.mock("./lib/bridge", () => ({
  loadObjectiveAuthority: operations.load,
  createObjectiveAuthority: operations.create,
  activateObjectiveAuthority: operations.activate,
  revokeObjectiveAuthority: operations.revoke,
}));

import { ObjectiveAuthorityWorkbench } from "./ObjectiveAuthorityWorkbench";

const snapshot = (state: "draft" | "active" | "revoked") => ({
  schemaVersion: 1 as const,
  objectives: [
    {
      id: objectiveId,
      projectId,
      title: "Review browser authority",
      objective: "Plan the supervised browser capability.",
      allowedLanes: ["browser-workspace", "browser-observation"],
      confirmationRequiredLanes: ["browser-observation"],
      state,
      createdAtMs: 1,
      activatedAtMs: state === "draft" ? null : 2,
      expiresAtMs: 3_600_001,
      revokedAtMs: state === "revoked" ? 3 : null,
    },
  ],
  diagnosticCode: null,
});

describe("ObjectiveAuthorityWorkbench", () => {
  it("keeps future lanes visibly locked and records the draft-to-revoked lifecycle without starting a capability", async () => {
    operations.load.mockResolvedValueOnce({
      schemaVersion: 1,
      objectives: [],
      diagnosticCode: null,
    });
    operations.create.mockResolvedValueOnce(snapshot("draft"));
    operations.activate.mockResolvedValueOnce(snapshot("active"));
    operations.revoke.mockResolvedValueOnce(snapshot("revoked"));

    render(
      <ObjectiveAuthorityWorkbench projectId={projectId} onClose={vi.fn()} />,
    );

    expect(await screen.findByRole("note")).toHaveTextContent(
      "Lane selections describe future scope only",
    );
    expect(
      screen.getAllByText(
        "Locked — no capability executes from this selection. This lane requires its own approval when available.",
      ),
    ).toHaveLength(8);
    expect(
      screen.getAllByText(
        "This only highlights a future Action Card. It never lowers or skips that lane's approval requirement.",
      ),
    ).toHaveLength(8);

    fireEvent.change(screen.getByLabelText("Title"), {
      target: { value: "Review browser authority" },
    });
    fireEvent.change(screen.getByLabelText("Objective"), {
      target: { value: "Plan the supervised browser capability." },
    });
    fireEvent.click(
      screen.getByRole("button", { name: "Create draft objective" }),
    );
    await waitFor(() => expect(operations.create).toHaveBeenCalledOnce());
    expect(operations.create).toHaveBeenCalledWith({
      projectId,
      title: "Review browser authority",
      objective: "Plan the supervised browser capability.",
      allowedLanes: ["work-with-code"],
      confirmationRequiredLanes: [],
      expiresInMinutes: 60,
    });
    expect(
      screen.getByText(
        "browser-workspace: locked future scope only; this lane requires its own approval when available.",
      ),
    ).toBeInTheDocument();
    expect(
      screen.getByText(
        "browser-observation: locked future scope only; this lane requires its own approval when available.",
      ),
    ).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Activate" }));
    await waitFor(() =>
      expect(operations.activate).toHaveBeenCalledWith({ objectiveId }),
    );
    fireEvent.click(screen.getByRole("button", { name: "Revoke" }));
    await waitFor(() =>
      expect(operations.revoke).toHaveBeenCalledWith({ objectiveId }),
    );
    expect(screen.getByText(/revoked · expires/u)).toBeInTheDocument();
    expect(operations.load).toHaveBeenCalledWith({ projectId });
  });
});
