import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { UsagePanel } from "./UsagePanel";
import {
  codexUsageSchema,
  scaffoldCodexUsage,
  selectSidebarUsageWindow,
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
  it("summarizes the weekly window and its matching reset time, not the first window", () => {
    const usage = snapshot({
      meters: [
        {
          ...scaffoldCodexUsage.meters[0]!,
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
              usedPercent: 68,
              remainingPercent: 32,
              windowDurationMinutes: 10_080,
              resetsAt: 1_785_412_800,
            },
          ],
        },
      ],
    });

    compact(usage);

    expect(screen.getByText("32%")).toBeInTheDocument();
    expect(
      screen.getByText(usageResetLabel(1_785_412_800)),
    ).toBeInTheDocument();
    expect(screen.queryByText("100%")).not.toBeInTheDocument();
    expect(
      screen.queryByText(usageResetLabel(1_784_808_000)),
    ).not.toBeInTheDocument();
  });

  it("prioritizes an exact weekly duration regardless of input order", () => {
    const meter = scaffoldCodexUsage.meters[0]!;
    const selected = selectSidebarUsageWindow([
      { meter, window: meter.windows[0]! },
      { meter, window: meter.windows[1]! },
    ]);

    expect(selected?.window.windowDurationMinutes).toBe(10_080);
  });

  it("uses the longest reported duration when no weekly window exists", () => {
    const usage = snapshot({
      meters: [
        {
          ...scaffoldCodexUsage.meters[0]!,
          windows: [
            {
              kind: "primary",
              usedPercent: 90,
              remainingPercent: 10,
              windowDurationMinutes: 60,
              resetsAt: 1_784_808_000,
            },
            {
              kind: "secondary",
              usedPercent: 45,
              remainingPercent: 55,
              windowDurationMinutes: 1_440,
              resetsAt: 1_785_412_800,
            },
          ],
        },
      ],
    });

    compact(usage);
    expect(screen.getByText("55%")).toBeInTheDocument();
  });

  it.each([
    ["unavailable", unavailableCodexUsage, "unavailable"],
    [
      "not metered",
      snapshot({
        state: "not-metered",
        meters: [],
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
      expect(screen.queryByText(/\d+%/u)).not.toBeInTheDocument();
    },
  );

  it("renders every valid Codex-reported meter and window in the full panel", () => {
    const usage = snapshot({
      meters: [
        scaffoldCodexUsage.meters[0]!,
        {
          label: "Reviews",
          limitId: "codex_other",
          windows: [
            {
              kind: "primary",
              usedPercent: 20,
              remainingPercent: 80,
              windowDurationMinutes: 60,
              resetsAt: null,
            },
          ],
          limited: false,
        },
      ],
    });

    render(
      <UsagePanel
        snapshot={usage}
        state="native"
        busy={false}
        onRefresh={vi.fn()}
      />,
    );

    expect(screen.getAllByText(/remaining$/u)).toHaveLength(3);
    expect(screen.getByText("Codex · 5-hour window")).toBeInTheDocument();
    expect(screen.getByText("Codex · Weekly window")).toBeInTheDocument();
    expect(screen.getByText("Reviews · 1-hour window")).toBeInTheDocument();
  });
});
