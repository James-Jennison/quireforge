import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

const dynamicBridge = vi.hoisted(() => ({
  clearDynamicAnalysis: vi.fn(),
  loadDynamicAnalysis: vi.fn(),
  pickDynamicAnalysis: vi.fn(),
  runDynamicAnalysis: vi.fn(),
}));

vi.mock("./lib/bridge", () => dynamicBridge);

import { DynamicAnalysisWorkspace } from "./DynamicAnalysisWorkspace";
import { scaffoldDynamicAnalysis } from "./lib/dynamicAnalysis";

const manifest = {
  runId: "018f2c2d-6a14-7e4f-8d67-16b3c71a9988",
  displayName: "probe",
  byteSize: 64,
  sha256: "a".repeat(64),
  elfType: "executable" as const,
  staticRuntime: true as const,
  maxMemoryBytes: 512 * 1024 * 1024,
  maxWallTimeMs: 30_000,
};

describe("DynamicAnalysisWorkspace", () => {
  it("requires explicit confirmation and presents only the safe manifest", async () => {
    dynamicBridge.loadDynamicAnalysis.mockResolvedValue({
      ...scaffoldDynamicAnalysis,
      state: "ready",
      manifest,
    });
    dynamicBridge.clearDynamicAnalysis.mockResolvedValue(
      scaffoldDynamicAnalysis,
    );
    render(<DynamicAnalysisWorkspace />);

    await screen.findByText("probe");
    expect(screen.queryByText("/tmp/probe")).not.toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "Run isolated analysis" }),
    ).toBeDisabled();
    fireEvent.click(screen.getByRole("checkbox"));
    expect(
      screen.getByRole("button", { name: "Run isolated analysis" }),
    ).toBeEnabled();
    fireEvent.click(screen.getByRole("button", { name: "Clear" }));
    await waitFor(() =>
      expect(dynamicBridge.clearDynamicAnalysis).toHaveBeenCalledOnce(),
    );
  });

  it("explains the bounded unsupported-runtime policy without exposing a path", async () => {
    dynamicBridge.loadDynamicAnalysis.mockResolvedValue({
      ...scaffoldDynamicAnalysis,
      state: "unavailable",
      diagnosticCode: "unsupported-runtime",
    });
    render(<DynamicAnalysisWorkspace />);
    expect(await screen.findByRole("status")).toHaveTextContent(
      /without a program interpreter/i,
    );
    expect(screen.queryByText("/tmp/dynamic")).not.toBeInTheDocument();
  });
});
