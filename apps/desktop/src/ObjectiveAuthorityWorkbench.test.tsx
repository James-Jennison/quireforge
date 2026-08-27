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
  it("requires a project before exposing authority choices", () => {
    render(
      <ObjectiveAuthorityWorkbench
        projectId={null}
        projectName={null}
        onClose={vi.fn()}
      />,
    );
    expect(
      screen.getByRole("heading", { name: "Choose a project first" }),
    ).toBeInTheDocument();
    expect(
      screen.queryByRole("button", { name: "Create draft objective" }),
    ).not.toBeInTheDocument();
  });

  it("uses grouped future scope and records the draft-to-revoked lifecycle", async () => {
    operations.load.mockResolvedValueOnce({
      schemaVersion: 1,
      objectives: [],
      diagnosticCode: null,
    });
    operations.create.mockResolvedValueOnce(snapshot("draft"));
    operations.activate.mockResolvedValueOnce(snapshot("active"));
    operations.revoke.mockResolvedValueOnce(snapshot("revoked"));
    render(
      <ObjectiveAuthorityWorkbench
        projectId={projectId}
        projectName="QuireForge"
        onClose={vi.fn()}
      />,
    );
    expect(await screen.findByRole("note")).toHaveTextContent(
      "Scope choices describe future work only",
    );
    expect(
      screen.getByRole("checkbox", { name: /Work on this project/u }),
    ).toBeChecked();
    expect(
      screen.getByRole("checkbox", { name: /Research and project data/u }),
    ).not.toBeChecked();
    fireEvent.click(
      screen.getByRole("checkbox", { name: /Research and project data/u }),
    );
    fireEvent.click(
      screen.getByRole("checkbox", { name: /Flag this scope for review/u }),
    );
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
      allowedLanes: [
        "work-with-code",
        "browser-workspace",
        "browser-observation",
        "connector-read",
      ],
      confirmationRequiredLanes: [
        "work-with-code",
        "browser-workspace",
        "browser-observation",
        "connector-read",
      ],
      expiresInMinutes: 60,
    });
    expect(
      screen.getByRole("heading", { name: "Current objectives" }),
    ).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Activate" }));
    await waitFor(() =>
      expect(operations.activate).toHaveBeenCalledWith({ objectiveId }),
    );
    fireEvent.click(screen.getByRole("button", { name: "Revoke" }));
    await waitFor(() =>
      expect(operations.revoke).toHaveBeenCalledWith({ objectiveId }),
    );
  });
});
