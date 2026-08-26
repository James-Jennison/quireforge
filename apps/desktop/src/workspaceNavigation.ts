export type PrimaryWorkspaceRoute =
  | "home"
  | "advisor"
  | "conversation"
  | "projects"
  | "studio"
  | "ledger"
  | "adapters"
  | "project-state"
  | "sessions"
  | "scheduled"
  | "integrations"
  | "files"
  | "changes"
  | "worktrees"
  | "terminal"
  | "dynamic-analysis";

export type SettingsSection =
  | "general"
  | "appearance"
  | "chat"
  | "codex"
  | "permissions-safety"
  | "models-providers"
  | "integrations"
  | "privacy-data"
  | "keyboard-shortcuts"
  | "about-updates";
export type WorkspaceRoute = PrimaryWorkspaceRoute | "settings";

export interface WorkspaceLocation {
  route: WorkspaceRoute;
  settingsSection: SettingsSection | null;
}

export interface WorkspaceNavigationItem {
  route: PrimaryWorkspaceRoute;
  label: string;
  icon: string;
  lane: "chat" | "work" | "code";
  description: string;
}

export const workspaceNavigation: readonly WorkspaceNavigationItem[] = [
  {
    route: "sessions",
    label: "Threads",
    icon: "thread",
    lane: "chat",
    description: "Browse and continue bounded local thread references",
  },
  {
    route: "home",
    label: "Overview",
    icon: "grid",
    lane: "work",
    description: "Project overview and starting workspace",
  },
  {
    route: "advisor",
    label: "Advisor",
    icon: "grid",
    lane: "chat",
    description: "Plan with a managed, read-only Advisor",
  },
  {
    route: "conversation",
    label: "New task",
    icon: "plus",
    lane: "code",
    description: "Create a focused QuireForge task",
  },
  {
    route: "projects",
    label: "Projects",
    icon: "folder",
    lane: "work",
    description: "Select and manage local projects",
  },
  {
    route: "studio",
    label: "Studio",
    icon: "folder",
    lane: "work",
    description: "Organize governed local sources and reviewed artifacts",
  },
  {
    route: "ledger",
    label: "Ledger",
    icon: "grid",
    lane: "work",
    description: "Inspect local context and authority receipts",
  },
  {
    route: "adapters",
    label: "Adapters",
    icon: "blocks",
    lane: "work",
    description: "Compare deterministic local Work destination fixtures",
  },
  {
    route: "project-state",
    label: "Project state",
    icon: "git",
    lane: "code",
    description: "Inspect normalized repository evidence",
  },
  {
    route: "scheduled",
    label: "Scheduled",
    icon: "clock",
    lane: "work",
    description: "Review discovered scheduled work",
  },
  {
    route: "integrations",
    label: "Integrations",
    icon: "blocks",
    lane: "work",
    description: "Manage supported tools and connections",
  },
  {
    route: "files",
    label: "Files",
    icon: "folder",
    lane: "code",
    description: "Preview files from the active project",
  },
  {
    route: "changes",
    label: "Changes",
    icon: "git",
    lane: "code",
    description: "Review source-control changes",
  },
  {
    route: "worktrees",
    label: "Worktrees",
    icon: "git",
    lane: "code",
    description: "Create and manage project worktrees",
  },
  {
    route: "terminal",
    label: "Terminal",
    icon: "terminal",
    lane: "code",
    description: "Use the integrated project terminal",
  },
  {
    route: "dynamic-analysis",
    label: "Isolated analysis",
    icon: "shield",
    lane: "code",
    description: "Run one static ELF in the separately installed worker",
  },
] as const;

const primaryRoutes = new Set<PrimaryWorkspaceRoute>(
  workspaceNavigation.map(({ route }) => route),
);
const settingsSections = new Set<SettingsSection>([
  "general",
  "appearance",
  "chat",
  "codex",
  "permissions-safety",
  "models-providers",
  "integrations",
  "privacy-data",
  "keyboard-shortcuts",
  "about-updates",
]);

export const defaultWorkspaceLocation: WorkspaceLocation = {
  route: "sessions",
  settingsSection: null,
};

export function parseWorkspaceHash(hash: string): WorkspaceLocation | null {
  const normalized = hash.replace(/^#\/?/u, "").replace(/\/+$/u, "");
  if (!normalized) return null;

  const [route, section, ...rest] = normalized.split("/");
  if (rest.length > 0) return null;
  if (route === "settings") {
    if (section === undefined) {
      return { route: "settings", settingsSection: "general" };
    }
    if (section === "accounts") {
      return { route: "settings", settingsSection: "general" };
    }
    if (settingsSections.has(section as SettingsSection)) {
      return {
        route: "settings",
        settingsSection: section as SettingsSection,
      };
    }
    return null;
  }

  if (
    section === undefined &&
    primaryRoutes.has(route as PrimaryWorkspaceRoute)
  )
    return { route: route as PrimaryWorkspaceRoute, settingsSection: null };
  return null;
}

export function workspaceLocationHash(location: WorkspaceLocation): string {
  if (location.route === "settings") {
    return `#settings/${location.settingsSection ?? "general"}`;
  }
  return `#${location.route}`;
}

export function workspaceLocationFor(
  route: WorkspaceRoute,
  settingsSection: SettingsSection = "general",
): WorkspaceLocation {
  return route === "settings"
    ? { route, settingsSection }
    : { route, settingsSection: null };
}

export function workspaceNavigationItem(
  route: WorkspaceRoute,
): WorkspaceNavigationItem | null {
  return (
    workspaceNavigation.find((candidate) => candidate.route === route) ?? null
  );
}
