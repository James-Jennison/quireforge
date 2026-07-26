export const appearanceThemeStorageKey = "quireforge-theme";
const lightThemeIds = new Set(["aurora-workbench", "monochrome-editorial"]);

export const appearanceThemeTokenNames =
  "bg surface surface-raised surface-soft border border-strong text text-muted text-subtle accent accent-contrast accent-bright accent-soft green green-soft warning warning-soft danger danger-strong danger-soft control-on-accent overlay shadow".split(
    " ",
  );

const paletteTokens = (values: string) =>
  values.split(" ").map((value) => `#${value}`);

export const appearanceThemes = [
  {
    id: "forge",
    label: "Forge",
    description: "Warm default palette",
    tokens: null,
  },
  {
    id: "midnight-atelier",
    label: "Midnight Atelier",
    description: "Violet dark palette",
    tokens: paletteTokens(
      "0b1020 11182b 17213a 202c49 e7edff1a e7edff2e edf1ff c0cae5 93a0c2 a78bfa 170d34 c4b5fd a78bfa26 7fddb1 7fddb124 f7c96f f7c96f24 ffaaa5 d75e65 d75e6526 170d34 040711c2 0000006b",
    ),
  },
  {
    id: "blueprint-terminal",
    label: "Blueprint Terminal",
    description: "Cyan technical palette",
    tokens: paletteTokens(
      "061626 0c2238 112b45 193953 e0f7ff1c e0f7ff33 eaf8ff b1d6e9 7eabc3 63c6ff 00253c 94d8ff 63c6ff26 72dfb4 72dfb424 ffcc75 ffcc7524 ffa8a3 d95f5c d95f5c26 00253c 020d17c2 000a1473",
    ),
  },
  {
    id: "signal-noir",
    label: "Signal Noir",
    description: "Charcoal coral palette",
    tokens: paletteTokens(
      "111216 191b21 22242c 2c2f39 f9f6f61a f9f6f62e f4f1f2 c5bec1 989095 ff7675 3a090b ffa7a5 ff767524 88d9b0 88d9b024 ffd075 ffd07524 ffaaa5 d45b60 d45b6026 3a090b 07080bc7 0000006e",
    ),
  },
  {
    id: "aurora-workbench",
    label: "Aurora Workbench",
    description: "Calm daylight palette",
    tokens: paletteTokens(
      "edf7f4 fbfffd ffffff dfefea 173b361f 173b3633 173b36 466762 5f7e78 087d72 f6fffc 086d64 087d721f 17764b 17764b1f 8c5c00 b2750021 a23e3d 963938 a23e3d1f f6fffc 0c211d8f 123c3524",
    ),
  },
  {
    id: "obsidian-copper",
    label: "Obsidian & Copper",
    description: "Deep copper palette",
    tokens: paletteTokens(
      "14110e 1c1814 26201a 322a22 ffebd71a ffebd72e f5ede5 c9b9aa 9e8f82 d88943 2b1607 f3b575 d8894324 91ca9a 91ca9a24 f2bf63 f2bf6324 e99b8f bf574b bf574b26 2b1607 0a0705c7 00000073",
    ),
  },
  {
    id: "monochrome-editorial",
    label: "Monochrome Editorial",
    description: "Neutral editorial palette",
    tokens: paletteTokens(
      "f0f0ed fbfbf9 ffffff e4e4e1 1d1d1d24 1d1d1d3d 202020 575757 6c6c6c 313131 ffffff 242424 3131311a 2f6b45 2f6b451c 7f5800 8b600021 9b3838 8d3030 9b38381f ffffff 0c0c0c94 00000029",
    ),
  },
  {
    id: "pacific-night",
    label: "Pacific Night",
    description: "Teal ocean palette",
    tokens: paletteTokens(
      "061b24 0d2732 143540 1c4650 e1faf81a e1faf830 e9fbf8 b8dbd6 88b6b0 45d5c5 002c29 78e6da 45d5c524 8adeaa 8adeaa24 ffd178 ffd17824 ffaaa2 d35d58 d35d5826 002c29 010f14c7 00080b75",
    ),
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
  const selectedTheme = appearanceThemes.find(({ id }) => id === theme);
  if (!selectedTheme) return;
  document.documentElement.dataset.theme = theme;
  document.documentElement.style.colorScheme = lightThemeIds.has(theme)
    ? "light"
    : "dark";
  if (selectedTheme.tokens === null) {
    for (const name of appearanceThemeTokenNames) {
      document.documentElement.style.removeProperty(`--${name}`);
    }
    return;
  }
  for (const [index, name] of appearanceThemeTokenNames.entries()) {
    document.documentElement.style.setProperty(
      `--${name}`,
      selectedTheme.tokens[index]!,
    );
  }
}

export function nextAppearanceTheme(theme: ThemeId): ThemeId {
  const currentIndex = appearanceThemes.findIndex(({ id }) => id === theme);
  return appearanceThemes[(currentIndex + 1) % appearanceThemes.length]!.id;
}
