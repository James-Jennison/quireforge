import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { UsagePanel } from "./UsagePanel";
import {
  codexUsageSchema,
  scaffoldCodexUsage,
  unavailableCodexUsage,
  usageResetLabel,
  type CodexUsageSnapshot,
} from "./lib/usage";

function snapshot(
  overrides: Partial<CodexUsageSnapshot> = {},
): CodexUsageSnapshot {
  return codexUsageSchema.parse({ ...scaffoldCodexUsage, ...overrides });
}

function compact(
  usage: CodexUsageSnapshot,
  state: "checking" | "native" | "preview" | "unavailable" = "native",
) {
  return render(
    <UsagePanel
      snapshot={usage}
      state={state}
      busy={state === "checking"}
      compact
      onRefresh={vi.fn()}
    />,
  );
}

describe("UsagePanel", () => {
  it("selects the general Codex weekly window and its matching reset over a short 100% window", () => {
    const usage = snapshot({
      runtimeMeters: [
        {
          ...scaffoldCodexUsage.runtimeMeters[0]!,
          windows: [
            {
              kind: "primary",
              usedPercent: 0,
              remainingPercent: 100,
              windowDurationMinutes: 300,
              resetsAt: 1_784_808_000,
            },
            {
              kind: "secondary",
              usedPercent: 1,
              remainingPercent: 99,
              windowDurationMinutes: 10_080,
              resetsAt: 1_785_612_540,
            },
          ],
        },
        scaffoldCodexUsage.runtimeMeters[1]!,
      ],
    });

    compact(usage);
    expect(screen.getByText("Usage available")).toBeInTheDocument();
    expect(screen.getByText("99%")).toBeInTheDocument();
    expect(
      screen.getByText(usageResetLabel(1_785_612_540)),
    ).toBeInTheDocument();
    expect(screen.queryByText("100%")).not.toBeInTheDocument();
  });

  it("prefers the general codex weekly meter over model meters and resolves remaining ties deterministically", () => {
    const usage = snapshot({
      runtimeMeters: [
        scaffoldCodexUsage.runtimeMeters[1]!,
        {
          ...scaffoldCodexUsage.runtimeMeters[0]!,
          limitId: "codex-z",
          windows: [
            {
              kind: "secondary",
              usedPercent: 12,
              remainingPercent: 88,
              windowDurationMinutes: 10_080,
              resetsAt: 1_785_412_800,
            },
          ],
        },
        {
          ...scaffoldCodexUsage.runtimeMeters[0]!,
          windows: [
            {
              kind: "secondary",
              usedPercent: 1,
              remainingPercent: 99,
              windowDurationMinutes: 10_080,
              resetsAt: 1_785_612_540,
            },
          ],
        },
      ],
    });
    compact(usage);
    expect(screen.getByText("99%")).toBeInTheDocument();
  });

  it("shows no numeric value when no exact general weekly window is available", () => {
    compact(
      snapshot({
        runtimeMeters: [
          {
            ...scaffoldCodexUsage.runtimeMeters[0]!,
            windows: [
              {
                kind: "primary",
                usedPercent: 20,
                remainingPercent: 80,
                windowDurationMinutes: 300,
                resetsAt: 1_784_808_000,
              },
            ],
          },
        ],
      }),
    );
    expect(screen.getByText("—")).toBeInTheDocument();
    expect(screen.getByText("No weekly window reported")).toBeInTheDocument();
    expect(screen.queryByText(/^\d+%$/u)).not.toBeInTheDocument();
  });

  it.each([
    ["unavailable", unavailableCodexUsage, "unavailable"],
    [
      "not metered",
      snapshot({
        state: "not-metered",
        sharedUsage: null,
        runtimeMeters: [],
        diagnosticCode: "no-usage-windows",
      }),
      "native",
    ],
    ["browser preview", scaffoldCodexUsage, "preview"],
    ["loading", scaffoldCodexUsage, "checking"],
  ] as const)(
    "shows no numeric percentage when %s",
    (_description, usage, state) => {
      compact(usage, state);
      expect(screen.getByText("—")).toBeInTheDocument();
      expect(screen.queryByText(/^\d+%$/u)).not.toBeInTheDocument();
    },
  );

  it("renders every reported runtime meter and window in Settings", () => {
    render(
      <UsagePanel
        snapshot={scaffoldCodexUsage}
        state="native"
        busy={false}
        onRefresh={vi.fn()}
      />,
    );
    expect(
      screen.getByRole("heading", { name: "Codex usage limits" }),
    ).toBeInTheDocument();
    expect(
      screen.getByText(/browser page may display stale data/u),
    ).toBeInTheDocument();
    expect(screen.getByText("Codex · Weekly window")).toBeInTheDocument();
    expect(
      screen.getByText("GPT-5.3-Codex-Spark · Weekly window"),
    ).toBeInTheDocument();
    expect(screen.getAllByText("Scope not verified")).toHaveLength(3);
  });
});
