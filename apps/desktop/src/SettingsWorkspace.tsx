import type { KeyboardEvent as ReactKeyboardEvent } from "react";

import { appearanceThemes, type ThemeId } from "./appearanceThemes";
import type { CodexAuthSnapshot } from "./lib/auth";
import type { CodexUsageSnapshot } from "./lib/usage";
import { UsagePanel } from "./UsagePanel";
import type { SettingsSection } from "./workspaceNavigation";

type AuthViewState = CodexAuthSnapshot["state"] | "checking" | "preview";
type UsageViewState = "checking" | "native" | "preview" | "unavailable";
interface SettingsWorkspaceProps {
  section: SettingsSection;
  auth: CodexAuthSnapshot;
  authState: AuthViewState;
  authBusy: boolean;
  authActionError: boolean;
  confirmLogout: boolean;
  usage: CodexUsageSnapshot;
  usageState: UsageViewState;
  usageBusy: boolean;
  theme: ThemeId;
  productName: string;
  productVersion: string;
  bridgeLabel: string;
  runtimeLabel: string;
  cliVersion: string | null;
  onSectionChange: (section: SettingsSection) => void;
  onRefreshAuth: () => void;
  onRefreshUsage: () => void;
  onRequestLogout: () => void;
  onConfirmLogout: () => void;
  onCancelLogout: () => void;
  onThemeChange: (theme: ThemeId) => void;
  onThemePreview: (theme: ThemeId) => void;
  onThemePreviewEnd: () => void;
}

const sections: readonly {
  id: SettingsSection;
  label: string;
  description: string;
}[] = [
  {
    id: "general",
    label: "General",
    description: "Application defaults and managed account status",
  },
  {
    id: "appearance",
    label: "Appearance",
    description: "Local QuireForge display preferences",
  },
  {
    id: "chat",
    label: "Chat",
    description: "No-project conversation capability and account readiness",
  },
  {
    id: "codex",
    label: "Codex",
    description: "Attached-project task controls and native execution boundary",
  },
  {
    id: "permissions-safety",
    label: "Permissions & safety",
    description: "Reviewed approval and sandbox policy boundaries",
  },
  {
    id: "models-providers",
    label: "Models & providers",
    description: "Runtime-reported model availability and provider limits",
  },
  {
    id: "integrations",
    label: "Integrations",
    description: "Supported Codex integrations and connection status",
  },
  {
    id: "privacy-data",
    label: "Privacy & data",
    description: "Local metadata, credential ownership, and data boundaries",
  },
  {
    id: "keyboard-shortcuts",
    label: "Keyboard shortcuts",
    description: "Discoverable keyboard navigation and focus behavior",
  },
  {
    id: "about-updates",
    label: "About & updates",
    description: "Version, runtime, and product boundaries",
  },
];

const foundationSections: Readonly<
  Partial<
    Record<SettingsSection, { eyebrow: string; title: string; detail: string }>
  >
> = {
  chat: {
    eyebrow: "Chat",
    title: "A separate, no-project conversation capability.",
    detail:
      "Chat requires the managed ChatGPT browser sign-in owned by the supported Codex runtime. API keys, browser cookies, external tokens, and consumer ChatGPT session reuse are not accepted.",
  },
  codex: {
    eyebrow: "Codex",
    title: "Attached projects remain an explicit execution boundary.",
    detail:
      "Codex conversations require a verified attached project. Model, reasoning, sandbox, and approval controls remain native-owned and project-scoped.",
  },
  "permissions-safety": {
    eyebrow: "Permissions & safety",
    title: "Capability changes stay visible and reviewable.",
    detail:
      "QuireForge will not auto-approve commands, file changes, or permissions. A Chat conversation never inherits project execution authority.",
  },
  "models-providers": {
    eyebrow: "Models & providers",
    title: "Runtime capability is reported, not configured with secrets.",
    detail:
      "The installed Codex runtime remains the only approved account boundary. OpenAI API or project keys are separate and are not a substitute for managed ChatGPT account access.",
  },
  integrations: {
    eyebrow: "Integrations",
    title: "Existing Codex integrations stay opt-in.",
    detail:
      "This foundation does not install, authorize, or enable integrations. Any future mode-specific availability must be explicitly advertised by the native capability contract.",
  },
  "privacy-data": {
    eyebrow: "Privacy & data",
    title: "Credentials remain with Codex; QuireForge stores bounded metadata.",
    detail:
      "Passwords, API keys, access tokens, browser cookies, account identifiers, and raw provider responses never enter QuireForge settings or local metadata.",
  },
  "keyboard-shortcuts": {
    eyebrow: "Keyboard shortcuts",
    title: "Keyboard access is part of every workspace boundary.",
    detail:
      "Settings navigation, mode selection, confirmations, and recovery states will preserve visible focus, semantic labels, and predictable escape behavior.",
  },
};

function accountKindLabel(accountKind: CodexAuthSnapshot["accountKind"]) {
  switch (accountKind) {
    case "chatgpt":
      return "ChatGPT account through Codex";
    case "api-key":
      return "API key managed by Codex";
    case "managed-provider":
      return "Managed provider through Codex";
    default:
      return "Codex-managed provider";
  }
}

function authStateLabel(state: AuthViewState) {
  switch (state) {
    case "authenticated":
      return "Connected";
    case "not-required":
      return "No additional sign-in required";
    case "checking":
      return "Checking connection";
    case "login-pending":
      return "Sign-in pending";
    case "unauthenticated":
      return "Reconnect required";
    case "preview":
      return "Native connection unavailable";
    default:
      return "Connection unavailable";
  }
}

export function SettingsWorkspace({
  section,
  auth,
  authState,
  authBusy,
  authActionError,
  confirmLogout,
  usage,
  usageState,
  usageBusy,
  theme,
  productName,
  productVersion,
  bridgeLabel,
  runtimeLabel,
  cliVersion,
  onSectionChange,
  onRefreshAuth,
  onRefreshUsage,
  onRequestLogout,
  onConfirmLogout,
  onCancelLogout,
  onThemeChange,
  onThemePreview,
  onThemePreviewEnd,
}: SettingsWorkspaceProps) {
  function handleThemeKeyDown(
    event: ReactKeyboardEvent<HTMLButtonElement>,
    candidate: ThemeId,
  ) {
    const currentIndex = appearanceThemes.findIndex(
      ({ id }) => id === candidate,
    );
    const direction =
      event.key === "ArrowRight" || event.key === "ArrowDown"
        ? 1
        : event.key === "ArrowLeft" || event.key === "ArrowUp"
          ? -1
          : 0;
    if (direction === 0) return;

    event.preventDefault();
    const nextIndex =
      (currentIndex + direction + appearanceThemes.length) %
      appearanceThemes.length;
    const nextTheme = appearanceThemes[nextIndex]!.id;
    onThemeChange(nextTheme);
    event.currentTarget.parentElement
      ?.querySelector<HTMLButtonElement>(`[data-theme-option="${nextTheme}"]`)
      ?.focus();
  }

  return (
    <section
      className="settings-workspace"
      id="settings"
      aria-labelledby="settings-title"
    >
      <aside
        className="settings-workspace__navigation"
        aria-label="Settings sections"
      >
        <div>
          <p className="eyebrow">QuireForge settings</p>
          <h1 id="settings-title" data-workspace-heading tabIndex={-1}>
            Settings
          </h1>
          <p>Local preferences and supported Codex connections.</p>
        </div>
        <nav aria-label="Settings">
          {sections.map((candidate) => (
            <button
              type="button"
              key={candidate.id}
              className={
                candidate.id === section
                  ? "settings-nav-item settings-nav-item--active"
                  : "settings-nav-item"
              }
              aria-current={candidate.id === section ? "page" : undefined}
              onClick={() => onSectionChange(candidate.id)}
            >
              <strong>{candidate.label}</strong>
              <span>{candidate.description}</span>
            </button>
          ))}
        </nav>
      </aside>

      <div className="settings-workspace__content">
        {section === "general" && (
          <div
            className="settings-section"
            aria-labelledby="settings-general-title"
          >
            <div className="settings-section__heading">
              <p className="eyebrow">General</p>
              <h2 id="settings-general-title">Codex owns authentication.</h2>
              <p>
                QuireForge receives a bounded connection state. It never reads,
                stores, or displays account identifiers, email addresses,
                passwords, API keys, or authentication tokens.
              </p>
            </div>

            <article className="settings-card settings-account-card">
              <div className="settings-card__heading">
                <div
                  className={`settings-connection-state settings-connection-state--${authState}`}
                  aria-hidden="true"
                />
                <div>
                  <span>Codex connection</span>
                  <h3>{authStateLabel(authState)}</h3>
                  <p>
                    {authState === "authenticated"
                      ? accountKindLabel(auth.accountKind)
                      : "Connection state reported by the local Codex runtime"}
                  </p>
                </div>
              </div>

              <dl className="settings-facts">
                <div>
                  <dt>Remote owner</dt>
                  <dd>Codex</dd>
                </div>
                <div>
                  <dt>QuireForge access</dt>
                  <dd>Normalized status only</dd>
                </div>
                <div>
                  <dt>CLI</dt>
                  <dd>{cliVersion ?? "Not detected"}</dd>
                </div>
              </dl>

              <div className="settings-card__actions">
                <button
                  className="auth-button auth-button--quiet"
                  type="button"
                  disabled={authBusy}
                  onClick={onRefreshAuth}
                >
                  Refresh connection
                </button>
                {authState === "authenticated" && !confirmLogout && (
                  <button
                    className="auth-button auth-button--danger"
                    type="button"
                    disabled={authBusy}
                    onClick={onRequestLogout}
                  >
                    Sign out of Codex
                  </button>
                )}
                {authState === "authenticated" && confirmLogout && (
                  <div
                    className="logout-confirmation"
                    role="group"
                    aria-label="Confirm Codex sign out"
                  >
                    <button
                      className="auth-button auth-button--danger"
                      type="button"
                      disabled={authBusy}
                      onClick={onConfirmLogout}
                    >
                      Confirm sign out
                    </button>
                    <button
                      className="auth-button auth-button--quiet"
                      type="button"
                      disabled={authBusy}
                      onClick={onCancelLogout}
                    >
                      Keep signed in
                    </button>
                  </div>
                )}
              </div>

              {authActionError && (
                <p className="auth-error" role="alert">
                  The native authentication action did not complete. QuireForge
                  did not change or retain your credentials.
                </p>
              )}
            </article>

            <UsagePanel
              snapshot={usage}
              state={usageState}
              busy={usageBusy}
              onRefresh={onRefreshUsage}
            />

            <div className="settings-boundary-note">
              <strong>Remote account controls are not available here.</strong>
              <p>
                Billing, plan, profile, and private ChatGPT account data remain
                outside QuireForge. This view contains local application
                preferences and only the connection operations supported by the
                installed Codex runtime.
              </p>
            </div>
          </div>
        )}

        {section === "appearance" && (
          <div
            className="settings-section"
            aria-labelledby="settings-appearance-title"
          >
            <div className="settings-section__heading">
              <p className="eyebrow">Appearance</p>
              <h2 id="settings-appearance-title">
                Make the local workspace comfortable.
              </h2>
              <p>
                Appearance preferences belong to QuireForge and never alter
                Codex configuration or account state.
              </p>
            </div>

            <article className="settings-card">
              <div className="settings-card__heading">
                <div>
                  <span>Color theme</span>
                  <h3>Application appearance</h3>
                  <p>Stored locally for this QuireForge installation.</p>
                </div>
              </div>
              <div
                className="theme-options"
                role="radiogroup"
                aria-label="QuireForge theme"
                onPointerLeave={onThemePreviewEnd}
                onBlur={(event) => {
                  const nextTarget =
                    event.relatedTarget instanceof Node
                      ? event.relatedTarget
                      : null;
                  if (!event.currentTarget.contains(nextTarget)) {
                    onThemePreviewEnd();
                  }
                }}
              >
                {appearanceThemes.map((candidate) => (
                  <button
                    key={candidate.id}
                    className={
                      candidate.id === theme
                        ? "theme-option theme-option--active"
                        : "theme-option"
                    }
                    type="button"
                    role="radio"
                    aria-checked={candidate.id === theme}
                    tabIndex={candidate.id === theme ? 0 : -1}
                    data-theme-option={candidate.id}
                    onFocus={() => onThemePreview(candidate.id)}
                    onPointerEnter={() => onThemePreview(candidate.id)}
                    onKeyDown={(event) =>
                      handleThemeKeyDown(event, candidate.id)
                    }
                    onClick={() => onThemeChange(candidate.id)}
                  >
                    <span
                      className={`theme-option__preview theme-option__preview--${candidate.id}`}
                      aria-hidden="true"
                    />
                    <strong>{candidate.label}</strong>
                    <small>{candidate.description}</small>
                  </button>
                ))}
              </div>
            </article>
          </div>
        )}

        {section === "about-updates" && (
          <div
            className="settings-section"
            aria-labelledby="settings-about-updates-title"
          >
            <div className="settings-section__heading">
              <p className="eyebrow">About & updates</p>
              <h2 id="settings-about-updates-title">{productName}</h2>
              <p>Build boldly. Work locally.</p>
            </div>

            <article className="settings-card">
              <dl className="settings-facts settings-facts--stacked">
                <div>
                  <dt>Version</dt>
                  <dd>v{productVersion}</dd>
                </div>
                <div>
                  <dt>Native boundary</dt>
                  <dd>{bridgeLabel}</dd>
                </div>
                <div>
                  <dt>Codex adapter</dt>
                  <dd>{runtimeLabel}</dd>
                </div>
                <div>
                  <dt>Product</dt>
                  <dd>Unofficial native Linux workspace for Codex</dd>
                </div>
              </dl>
            </article>

            <div className="settings-boundary-note">
              <strong>Independent and unofficial.</strong>
              <p>
                QuireForge is not made, endorsed, supported, or distributed by
                OpenAI. Attached projects remain in place, and Codex continues
                to own its authentication, configuration, and session data.
              </p>
            </div>
          </div>
        )}

        {foundationSections[section] && (
          <div
            className="settings-section"
            aria-labelledby={`settings-${section}-title`}
          >
            <div className="settings-section__heading">
              <p className="eyebrow">{foundationSections[section].eyebrow}</p>
              <h2 id={`settings-${section}-title`}>
                {foundationSections[section].title}
              </h2>
              <p>{foundationSections[section].detail}</p>
            </div>
            <div className="settings-boundary-note">
              <strong>Foundation in progress.</strong>
              <p>
                This section exposes the approved ownership boundary before it
                offers any new control. Unsupported actions remain unavailable
                instead of being simulated.
              </p>
            </div>
          </div>
        )}
      </div>
    </section>
  );
}
