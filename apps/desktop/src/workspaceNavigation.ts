export type PrimaryWorkspaceRoute =
  | "home"
  | "advisor"
  | "conversation"
  | "projects"
  | "project-state"
  | "sessions"
  | "scheduled"
  | "integrations"
  | "files"
  | "changes"
  | "worktrees"
  | "terminal";

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
  group: "main" | "workspace";
  description: string;
}

export const workspaceNavigation: readonly WorkspaceNavigationItem[] = [
  {
    route: "home",
    label: "Home",
    icon: "grid",
    group: "main",
    description: "Dashboard and starting workspace",
  },
  {
    route: "advisor",
    label: "Advisor",
    icon: "grid",
    group: "main",
    description: "Plan with a managed, read-only Advisor",
  },
  {
    route: "conversation",
    label: "New task",
    icon: "plus",
    group: "main",
    description: "Create a focused Codex task",
  },
  {
    route: "projects",
    label: "Projects",
    icon: "folder",
    group: "main",
    description: "Select and manage local projects",
  },
  {
    route: "project-state",
    label: "Project state",
    icon: "git",
    group: "workspace",
    description: "Inspect normalized repository evidence",
  },
  {
    route: "sessions",
    label: "Threads",
    icon: "thread",
    group: "main",
    description: "Review and continue task history",
  },
  {
    route: "scheduled",
    label: "Scheduled",
    icon: "clock",
    group: "main",
    description: "Review discovered scheduled work",
  },
  {
    route: "integrations",
    label: "Integrations",
    icon: "blocks",
    group: "main",
    description: "Manage supported tools and connections",
  },
  {
    route: "files",
    label: "Files",
    icon: "folder",
    group: "workspace",
    description: "Preview files from the active project",
  },
  {
    route: "changes",
    label: "Changes",
    icon: "git",
    group: "workspace",
    description: "Review source-control changes",
  },
  {
    route: "worktrees",
    label: "Worktrees",
    icon: "git",
    group: "workspace",
    description: "Create and manage project worktrees",
  },
  {
    route: "terminal",
    label: "Terminal",
    icon: "terminal",
    group: "workspace",
    description: "Use the integrated project terminal",
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
  route: "home",
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
