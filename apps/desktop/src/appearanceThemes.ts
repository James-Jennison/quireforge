export const appearanceThemeStorageKey = "quireforge-theme";

export const appearanceThemes = [
  {
    id: "forge",
    label: "Forge",
    description: "Warm, low-glare QuireForge default",
  },
  {
    id: "midnight-atelier",
    label: "Midnight Atelier",
    description: "Ink-blue surfaces with a violet working accent",
  },
  {
    id: "blueprint-terminal",
    label: "Blueprint Terminal",
    description: "Technical navy with clear cyan emphasis",
  },
  {
    id: "signal-noir",
    label: "Signal Noir",
    description: "Charcoal contrast with restrained coral signals",
  },
  {
    id: "aurora-workbench",
    label: "Aurora Workbench",
    description: "Bright, calm surfaces for daylight work",
  },
  {
    id: "obsidian-copper",
    label: "Obsidian & Copper",
    description: "Deep mineral surfaces with copper control color",
  },
  {
    id: "monochrome-editorial",
    label: "Monochrome Editorial",
    description: "High-clarity neutral paper and graphite",
  },
  {
    id: "pacific-night",
    label: "Pacific Night",
    description: "Ocean-dark panels with a cool teal accent",
  },
] as const;

export type ThemeId = (typeof appearanceThemes)[number]["id"];

export const defaultAppearanceTheme: ThemeId = "forge";

const themeIds = new Set<ThemeId>(appearanceThemes.map(({ id }) => id));

export function isThemeId(value: string | null): value is ThemeId {
  return value !== null && themeIds.has(value as ThemeId);
}

export function restoreAppearanceTheme(value: string | null): ThemeId {
  if (isThemeId(value)) return value;
  if (value === "dark") return "forge";
  if (value === "light") return "aurora-workbench";
  return defaultAppearanceTheme;
}

export function storedAppearanceTheme(): ThemeId {
  return restoreAppearanceTheme(
    window.localStorage.getItem(appearanceThemeStorageKey),
  );
}

export function applyAppearanceTheme(theme: ThemeId) {
  document.documentElement.dataset.theme = theme;
}

export function nextAppearanceTheme(theme: ThemeId): ThemeId {
  const currentIndex = appearanceThemes.findIndex(({ id }) => id === theme);
  return appearanceThemes[(currentIndex + 1) % appearanceThemes.length]!.id;
}
