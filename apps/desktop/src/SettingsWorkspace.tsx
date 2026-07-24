import type { CodexAuthSnapshot } from "./lib/auth";
import type { CodexUsageSnapshot } from "./lib/usage";
import { UsagePanel } from "./UsagePanel";
import type { SettingsSection } from "./workspaceNavigation";

type AuthViewState = CodexAuthSnapshot["state"] | "checking" | "preview";
type UsageViewState = "checking" | "native" | "preview";
type Theme = "light" | "dark";

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
  theme: Theme;
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
  onThemeChange: (theme: Theme) => void;
}

const sections: readonly {
  id: SettingsSection;
  label: string;
  description: string;
}[] = [
  {
    id: "accounts",
    label: "Accounts & connections",
    description: "Codex status and supported connection controls",
  },
  {
    id: "appearance",
    label: "Appearance",
    description: "Local QuireForge display preferences",
  },
  {
    id: "about",
    label: "About",
    description: "Version, runtime, and product boundaries",
  },
];

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
}: SettingsWorkspaceProps) {
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
        {section === "accounts" && (
          <div
            className="settings-section"
            aria-labelledby="settings-accounts-title"
          >
            <div className="settings-section__heading">
              <p className="eyebrow">Accounts & connections</p>
              <h2 id="settings-accounts-title">Codex owns authentication.</h2>
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
              >
                {(["dark", "light"] as const).map((candidate) => (
                  <button
                    key={candidate}
                    className={
                      candidate === theme
                        ? "theme-option theme-option--active"
                        : "theme-option"
                    }
                    type="button"
                    role="radio"
                    aria-checked={candidate === theme}
                    onClick={() => onThemeChange(candidate)}
                  >
                    <span
                      className={`theme-option__preview theme-option__preview--${candidate}`}
                      aria-hidden="true"
                    />
                    <strong>
                      {candidate === "dark" ? "Forge dark" : "Workshop light"}
                    </strong>
                    <small>
                      {candidate === "dark"
                        ? "Low-glare Linux workspace"
                        : "Bright, high-clarity workspace"}
                    </small>
                  </button>
                ))}
              </div>
            </article>
          </div>
        )}

        {section === "about" && (
          <div
            className="settings-section"
            aria-labelledby="settings-about-title"
          >
            <div className="settings-section__heading">
              <p className="eyebrow">About</p>
              <h2 id="settings-about-title">{productName}</h2>
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
      </div>
    </section>
  );
}
