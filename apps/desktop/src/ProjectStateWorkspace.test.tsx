import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { ProjectStateWorkspace } from "./ProjectStateWorkspace";
import {
  repositoryStateReadSnapshotSchema,
  scaffoldRepositoryStateSnapshot,
} from "./lib/repositoryState";

const representativeSnapshot = repositoryStateReadSnapshotSchema.parse({
  ...scaffoldRepositoryStateSnapshot,
  evidence: {
    packages: [
      {
        manifestVersion: 1,
        kind: "deb",
        sourceCommit: "0123456789abcdef0123456789abcdef01234567",
        artifactPath: "target/packages/quireforge.deb",
        filename: "quireforge.deb",
        cleanSource: true,
        checksum: "a".repeat(64),
        checksumFile: "a".repeat(64),
        localVerified: true,
        localPresent: true,
        declaredSize: 42,
        targetOs: "ubuntu-2204",
        architecture: "x86_64",
        maxGlibc: "2.34",
        desktopEntry: "passed",
        icon: "passed",
        install: "passed",
        upgrade: "passed",
        removal: "passed",
        launch: "passed",
        smoke: "passed",
        freshness: "current",
      },
    ],
    validations: [
      {
        version: 1,
        id: "rust-tests",
        family: "rust-tests",
        status: "passed",
        sourceCommit: "0123456789abcdef0123456789abcdef01234567",
        evidencePath: "target/validation-summary.json",
        operation: "cargo-test",
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
});

describe("ProjectStateWorkspace", () => {
  it("renders normalized evidence without exposing mutation controls", () => {
    const onRefresh = vi.fn();
    render(
      <ProjectStateWorkspace
        availability="native"
        projectName="QuireForge"
        snapshot={representativeSnapshot}
        busy={false}
        onRefresh={onRefresh}
      />,
    );

    expect(
      screen.getByRole("heading", {
        name: "Project state, without automation.",
      }),
    ).toBeInTheDocument();
    expect(screen.getByText("Validation and packages")).toBeInTheDocument();
    expect(
      screen.getByText(/rust-tests: passed, current/u),
    ).toBeInTheDocument();
    expect(
      screen.getByText(/deb: current, locally verified/u),
    ).toBeInTheDocument();
    expect(
      screen.getByText(/Remote tracking freshness was not requested/u),
    ).toBeInTheDocument();
    expect(
      screen.queryByRole("button", { name: /approve|resolve|fetch/u }),
    ).not.toBeInTheDocument();

    fireEvent.click(
      screen.getByRole("button", { name: "Refresh local evidence" }),
    );
    expect(onRefresh).toHaveBeenCalledOnce();
  });

  it("renders honest idle and browser-preview states", () => {
    const { rerender } = render(
      <ProjectStateWorkspace
        availability="idle"
        projectName={null}
        snapshot={null}
        busy={false}
        onRefresh={() => undefined}
      />,
    );
    expect(
      screen.getByText(
        "Select an attached project to inspect its normalized state.",
      ),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "Refresh local evidence" }),
    ).toBeDisabled();

    rerender(
      <ProjectStateWorkspace
        availability="preview"
        projectName="QuireForge"
        snapshot={null}
        busy={false}
        onRefresh={() => undefined}
      />,
    );
    expect(
      screen.getByText(/cannot read an attached native repository/u),
    ).toBeInTheDocument();
  });
});
