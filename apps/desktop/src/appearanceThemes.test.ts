import { describe, expect, it } from "vitest";

import {
  applyAppearanceTheme,
  appearanceThemes,
  appearanceThemeTokenNames,
  defaultAppearanceTheme,
  isThemeId,
  nextAppearanceTheme,
  restoreAppearanceTheme,
} from "./appearanceThemes";

describe("appearance themes", () => {
  it("keeps the closed built-in palette registry and Forge fallback", () => {
    expect(appearanceThemes.map(({ id }) => id)).toEqual([
      "forge",
      "midnight-atelier",
      "blueprint-terminal",
      "signal-noir",
      "aurora-workbench",
      "obsidian-copper",
      "monochrome-editorial",
      "pacific-night",
    ]);
    expect(defaultAppearanceTheme).toBe("forge");
    expect(
      appearanceThemes.every(
        ({ tokens }) =>
          tokens === null || tokens.length === appearanceThemeTokenNames.length,
      ),
    ).toBe(true);
    expect(isThemeId("custom-theme")).toBe(false);
    expect(restoreAppearanceTheme("custom-theme")).toBe("forge");
  });

  it("migrates the former two-state local preference without reading system state", () => {
    expect(restoreAppearanceTheme("dark")).toBe("forge");
    expect(restoreAppearanceTheme("light")).toBe("aurora-workbench");
    expect(restoreAppearanceTheme(null)).toBe("forge");
  });

  it("cycles only through the closed built-in list", () => {
    expect(nextAppearanceTheme("forge")).toBe("midnight-atelier");
    expect(nextAppearanceTheme("pacific-night")).toBe("forge");
  });

  it("applies runtime palette tokens and restores the Forge CSS fallback", () => {
    applyAppearanceTheme("midnight-atelier");

    expect(document.documentElement.dataset.theme).toBe("midnight-atelier");
    expect(document.documentElement.style.colorScheme).toBe("dark");
    expect(document.documentElement.style.getPropertyValue("--bg")).toBe(
      "#0b1020",
    );

    applyAppearanceTheme("forge");

    expect(document.documentElement.dataset.theme).toBe("forge");
    expect(document.documentElement.style.colorScheme).toBe("dark");
    expect(document.documentElement.style.getPropertyValue("--bg")).toBe("");
  });
});
