import { describe, expect, it } from "vitest";

import {
  defaultWorkbenchLayoutPreferences,
  restoreWorkbenchLayoutPreferences,
} from "./layoutPreferences";

describe("workbench layout preferences", () => {
  it("accepts only the fixed bounded schema", () => {
    expect(
      restoreWorkbenchLayoutPreferences(
        JSON.stringify({
          schemaVersion: 1,
          reviewPaneWidth: 500,
          terminalDockHeight: 340,
          selectedReviewPane: "git",
        }),
      ),
    ).toEqual({
      schemaVersion: 1,
      reviewPaneWidth: 500,
      terminalDockHeight: 340,
      selectedReviewPane: "git",
    });
  });

  it.each([
    null,
    "{",
    JSON.stringify({ schemaVersion: 2 }),
    JSON.stringify({
      schemaVersion: 1,
      reviewPaneWidth: 9,
      terminalDockHeight: 340,
      selectedReviewPane: "git",
    }),
    JSON.stringify({
      schemaVersion: 1,
      reviewPaneWidth: 500,
      terminalDockHeight: 340,
      selectedReviewPane: "git",
      transcript: "not allowed",
    }),
    "x".repeat(513),
  ])("fails safely for invalid input", (raw) => {
    expect(restoreWorkbenchLayoutPreferences(raw)).toEqual(
      defaultWorkbenchLayoutPreferences,
    );
  });
});
