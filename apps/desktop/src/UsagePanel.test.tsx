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
  it("does not promote a 99% weekly Codex meter into shared usage", () => {
    const usage = snapshot({
      runtimeMeters: [
        {
          ...scaffoldCodexUsage.runtimeMeters[0]!,
          windows: [
            {
              kind: "secondary",
              usedPercent: 1,
              remainingPercent: 99,
              windowDurationMinutes: 10_080,
              resetsAt: 1_785_412_800,
            },
          ],
        },
      ],
    });

    compact(usage);

    expect(screen.getByText("—")).toBeInTheDocument();
    expect(screen.getByText("View in ChatGPT")).toBeInTheDocument();
    expect(screen.queryByText("99%")).not.toBeInTheDocument();
  });

  it("does not promote a 100% weekly model meter into shared usage", () => {
    compact(
      snapshot({
        runtimeMeters: [
          {
            ...scaffoldCodexUsage.runtimeMeters[0]!,
            label: "GPT-5.3-Codex-Spark",
            limitId: "gpt-5.3-codex-spark",
            scope: "unknown",
            windows: [
              {
                kind: "secondary",
                usedPercent: 0,
                remainingPercent: 100,
                windowDurationMinutes: 10_080,
                resetsAt: 1_785_412_800,
              },
            ],
          },
        ],
      }),
    );

    expect(screen.getByText("—")).toBeInTheDocument();
    expect(screen.queryByText("100%")).not.toBeInTheDocument();
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
    "shows no numeric shared percentage when %s",
    (_description, usage, state) => {
      compact(usage, state);

      expect(screen.getByText("—")).toBeInTheDocument();
      expect(screen.queryByText(/^\d+%$/u)).not.toBeInTheDocument();
    },
  );

  it("renders all runtime meters with their reported percentages, reset times, and unverified scope", () => {
    const usage = snapshot({
      runtimeMeters: [
        scaffoldCodexUsage.runtimeMeters[0]!,
        {
          label: "Reviews",
          limitId: "codex_other",
          scope: "unknown",
          windows: [
            {
              kind: "primary",
              usedPercent: 20,
              remainingPercent: 80,
              windowDurationMinutes: 60,
              resetsAt: 1_784_808_000,
            },
          ],
          limited: false,
        },
      ],
    });

    render(
      <UsagePanel snapshot={usage} state="native" busy={false} onRefresh={vi.fn()} />,
    );

    expect(screen.getByRole("heading", { name: "Codex runtime limits" })).toBeInTheDocument();
    expect(screen.getByText(/may not match the shared usage balance/u)).toBeInTheDocument();
    expect(screen.getAllByText("Scope not verified")).toHaveLength(3);
    expect(screen.getByText("Codex · 5-hour window")).toBeInTheDocument();
    expect(screen.getByText("Codex · Weekly window")).toBeInTheDocument();
    expect(screen.getByText("Reviews · 1-hour window")).toBeInTheDocument();
    expect(screen.getAllByText(usageResetLabel(1_784_808_000))).toHaveLength(2);
    expect(screen.getAllByText(/remaining$/u)).toHaveLength(3);
    expect(
      screen.getByRole("progressbar", {
        name: "Codex runtime meter Weekly window remaining",
      }),
    ).toBeInTheDocument();
  });

  it("can display a future explicitly verified shared usage value", () => {
    const usage = snapshot({
      sharedUsage: { remainingPercent: 32, resetsAt: 1_785_412_800 },
    });

    compact(usage);

    expect(screen.getByText("32%")).toBeInTheDocument();
    expect(screen.getByText(usageResetLabel(1_785_412_800))).toBeInTheDocument();
  });
});
