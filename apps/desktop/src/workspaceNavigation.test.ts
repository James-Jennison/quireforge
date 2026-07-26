import { describe, expect, it } from "vitest";

import {
  parseWorkspaceHash,
  workspaceLocationFor,
  workspaceLocationHash,
  workspaceNavigation,
} from "./workspaceNavigation";

describe("workspace navigation", () => {
  it("defines every visible sidebar destination once", () => {
    expect(workspaceNavigation.map(({ route }) => route)).toEqual([
      "home",
      "advisor",
      "conversation",
      "projects",
      "project-state",
      "sessions",
      "scheduled",
      "integrations",
      "files",
      "changes",
      "worktrees",
      "terminal",
    ]);
    expect(new Set(workspaceNavigation.map(({ route }) => route)).size).toBe(
      workspaceNavigation.length,
    );
  });

  it("round-trips primary and settings destinations", () => {
    for (const item of workspaceNavigation) {
      const location = workspaceLocationFor(item.route);
      expect(parseWorkspaceHash(workspaceLocationHash(location))).toEqual(
        location,
      );
    }
    expect(parseWorkspaceHash("#settings/general")).toEqual({
      route: "settings",
      settingsSection: "general",
    });
    expect(parseWorkspaceHash("#settings/accounts")).toEqual({
      route: "settings",
      settingsSection: "general",
    });
    expect(parseWorkspaceHash("#/settings/appearance/")).toEqual({
      route: "settings",
      settingsSection: "appearance",
    });
  });

  it("fails closed for unknown or ambiguous hashes", () => {
    expect(parseWorkspaceHash("")).toBeNull();
    expect(parseWorkspaceHash("#unknown")).toBeNull();
    expect(parseWorkspaceHash("#terminal/extra")).toBeNull();
    expect(parseWorkspaceHash("#settings/billing")).toBeNull();
  });
});
