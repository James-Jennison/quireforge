import { useEffect, useState, type ReactNode } from "react";

import brandMark from "../../../assets/brand/quireforge-app-icon.svg";
import { loadDesktopBootstrap } from "./lib/bridge";
import { scaffoldBootstrap, type DesktopBootstrap } from "./lib/contract";

import "./styles.css";

type BridgeState = "connecting" | "native" | "preview";
type Theme = "light" | "dark";

interface AppProps {
  loadBootstrap?: () => Promise<DesktopBootstrap>;
}

const navigation = [
  { label: "Workspace", milestone: 3, icon: "grid" },
  { label: "Threads", milestone: 7, icon: "thread" },
  { label: "Integrations", milestone: 14, icon: "blocks" },
] as const;

function initialTheme(): Theme {
  const stored = window.localStorage.getItem("quireforge-theme");
  if (stored === "light" || stored === "dark") return stored;
  return window.matchMedia?.("(prefers-color-scheme: light)").matches
    ? "light"
    : "dark";
}

function Glyph({ name }: { name: string }) {
  const paths: Record<string, ReactNode> = {
    grid: (
      <>
        <rect x="3" y="3" width="7" height="7" rx="2" />
        <rect x="14" y="3" width="7" height="7" rx="2" />
        <rect x="3" y="14" width="7" height="7" rx="2" />
        <rect x="14" y="14" width="7" height="7" rx="2" />
      </>
    ),
    thread: (
      <>
        <path d="M6 7.5h12M6 12h8M6 16.5h5" />
        <path d="M4 3.5h16a2 2 0 0 1 2 2v11a2 2 0 0 1-2 2h-8l-5 3v-3H4a2 2 0 0 1-2-2v-11a2 2 0 0 1 2-2Z" />
      </>
    ),
    blocks: (
      <>
        <path d="m8 3 4 2.3v4.6L8 12.2 4 9.9V5.3L8 3ZM16 11.8l4 2.3v4.6L16 21l-4-2.3v-4.6l4-2.3Z" />
        <path d="m16 3 4 2.3v4.6l-4 2.3-4-2.3V5.3L16 3ZM8 11.8l4 2.3v4.6L8 21l-4-2.3v-4.6l4-2.3Z" />
      </>
    ),
    plus: <path d="M12 5v14M5 12h14" />,
    folder: (
      <path d="M3 6.5A2.5 2.5 0 0 1 5.5 4H10l2 2h6.5A2.5 2.5 0 0 1 21 8.5v8A2.5 2.5 0 0 1 18.5 19h-13A2.5 2.5 0 0 1 3 16.5v-10Z" />
    ),
    terminal: (
      <>
        <path d="m5 7 4 5-4 5M11 17h8" />
        <rect x="2.5" y="3.5" width="19" height="17" rx="3" />
      </>
    ),
    shield: (
      <>
        <path d="M12 2.8 20 6v5.7c0 4.5-3.1 8-8 9.5-4.9-1.5-8-5-8-9.5V6l8-3.2Z" />
        <path d="m8.5 12 2.2 2.2 4.8-5" />
      </>
    ),
    check: <path d="m5 12 4.2 4.2L19 6.5" />,
    chevron: <path d="m9 18 6-6-6-6" />,
  };

  return (
    <svg
      aria-hidden="true"
      className="glyph"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeLinecap="round"
      strokeLinejoin="round"
      strokeWidth="1.7"
    >
      {paths[name]}
    </svg>
  );
}

function StatusDot({ state }: { state: BridgeState }) {
  return (
    <span className={`status-dot status-dot--${state}`} aria-hidden="true" />
  );
}

export default function App({
  loadBootstrap = loadDesktopBootstrap,
}: AppProps) {
  const [bootstrap, setBootstrap] =
    useState<DesktopBootstrap>(scaffoldBootstrap);
  const [bridgeState, setBridgeState] = useState<BridgeState>("connecting");
  const [theme, setTheme] = useState<Theme>(initialTheme);

  useEffect(() => {
    let active = true;
    void loadBootstrap()
      .then((result) => {
        if (!active) return;
        setBootstrap(result);
        setBridgeState("native");
      })
      .catch(() => {
        if (active) setBridgeState("preview");
      });

    return () => {
      active = false;
    };
  }, [loadBootstrap]);

  useEffect(() => {
    document.documentElement.dataset.theme = theme;
    window.localStorage.setItem("quireforge-theme", theme);
  }, [theme]);

  const bridgeLabel = {
    connecting: "Checking native bridge",
    native: "Native IPC verified",
    preview: "Browser preview",
  }[bridgeState];

  return (
    <div className="app-shell">
      <aside className="sidebar">
        <div className="brand-lockup">
          <img src={brandMark} alt="" className="brand-mark" />
          <div>
            <strong>{bootstrap.product.name}</strong>
            <span>Linux workspace</span>
          </div>
        </div>

        <button className="primary-action" type="button" disabled>
          <Glyph name="plus" />
          New thread
          <span className="button-milestone">M7</span>
        </button>

        <nav className="primary-nav" aria-label="Primary navigation">
          <p className="nav-label">Workbench</p>
          {navigation.map((item, index) => (
            <button
              className={`nav-item ${index === 0 ? "nav-item--active" : ""}`}
              type="button"
              aria-current={index === 0 ? "page" : undefined}
              disabled={index !== 0}
              key={item.label}
            >
              <Glyph name={item.icon} />
              <span>{item.label}</span>
              {index !== 0 && (
                <span className="nav-milestone">M{item.milestone}</span>
              )}
            </button>
          ))}
        </nav>

        <div className="project-panel">
          <div className="project-icon">
            <Glyph name="folder" />
          </div>
          <div>
            <strong>No project attached</strong>
            <span>Direct local directories arrive in Milestone 6.</span>
          </div>
        </div>

        <div className="sidebar-footer">
          <div className="bridge-status" role="status" aria-live="polite">
            <StatusDot state={bridgeState} />
            <span>{bridgeLabel}</span>
          </div>
          <span className="version">v{bootstrap.product.version}</span>
        </div>
      </aside>

      <main className="workspace">
        <header className="topbar">
          <div className="breadcrumb" aria-label="Current location">
            <span>QuireForge</span>
            <Glyph name="chevron" />
            <strong>Workspace</strong>
          </div>
          <div className="topbar-actions">
            <span className="foundation-badge">
              <Glyph name="shield" />
              Milestone 3 foundation
            </span>
            <button
              className="theme-toggle"
              type="button"
              aria-label={`Use ${theme === "dark" ? "light" : "dark"} theme`}
              onClick={() => setTheme(theme === "dark" ? "light" : "dark")}
            >
              <span className="theme-toggle__track" aria-hidden="true">
                <span className="theme-toggle__thumb" />
              </span>
            </button>
          </div>
        </header>

        <div className="workspace-scroll">
          <section className="hero" aria-labelledby="workspace-title">
            <div className="hero-copy">
              <p className="eyebrow">
                <span /> Native Linux foundation
              </p>
              <h1 id="workspace-title">A quiet place for ambitious work.</h1>
              <p className="hero-description">
                The QuireForge desktop shell is wired to a small, validated Rust
                boundary. Codex sessions and local projects will arrive through
                their documented interfaces in the milestones ahead.
              </p>
              <div className="hero-actions">
                <button className="secondary-action" type="button" disabled>
                  <Glyph name="folder" />
                  Attach a local project
                  <span>M6</span>
                </button>
                <a className="text-link" href="#foundation">
                  Inspect foundation
                  <Glyph name="chevron" />
                </a>
              </div>
            </div>

            <div
              className="hero-visual"
              aria-label="QuireForge foundation status"
            >
              <div className="visual-glow" />
              <div className="terminal-card">
                <div className="terminal-card__bar">
                  <div className="window-dots" aria-hidden="true">
                    <span />
                    <span />
                    <span />
                  </div>
                  <span>quireforge / foundation</span>
                  <Glyph name="terminal" />
                </div>
                <div className="terminal-card__body">
                  <p>
                    <span className="prompt">›</span> verify desktop boundary
                  </p>
                  <div className="verification-line">
                    <Glyph name="check" />
                    <div>
                      <strong>Identity contract</strong>
                      <span>io.github.codeframe78.QuireForge</span>
                    </div>
                    <em>verified</em>
                  </div>
                  <div className="verification-line">
                    <Glyph name="check" />
                    <div>
                      <strong>Typed IPC fixture</strong>
                      <span>desktop_bootstrap · schema v1</span>
                    </div>
                    <em>verified</em>
                  </div>
                  <div className="verification-line verification-line--planned">
                    <span className="planned-ring" />
                    <div>
                      <strong>Codex process adapter</strong>
                      <span>supported app-server interface</span>
                    </div>
                    <em>Milestone 4</em>
                  </div>
                </div>
              </div>
            </div>
          </section>

          <section
            className="foundation"
            id="foundation"
            aria-labelledby="foundation-title"
          >
            <div className="section-heading">
              <div>
                <p className="eyebrow">Implementation map</p>
                <h2 id="foundation-title">Foundation, with honest edges.</h2>
              </div>
              <p>
                Each surface reports what exists now and what remains planned.
                Nothing here fabricates a Codex session, integration, or
                project.
              </p>
            </div>

            <div className="capability-grid">
              {bootstrap.capabilities.map((capability, index) => (
                <article className="capability-card" key={capability.id}>
                  <div className="capability-card__top">
                    <span
                      className={`capability-number capability-number--${index}`}
                    >
                      0{index + 1}
                    </span>
                    <span
                      className={`state-badge state-badge--${capability.state}`}
                    >
                      {capability.state}
                    </span>
                  </div>
                  <h3>{capability.label}</h3>
                  <p>
                    {capability.state === "ready"
                      ? "Tauri, React, strict TypeScript, and a validated native contract."
                      : capability.id === "codex-runtime"
                        ? "Version probing, process lifecycle, and normalized events through supported Codex interfaces."
                        : "Explicit directory selection, identity verification, and in-place local work."}
                  </p>
                  <footer>
                    <span>Milestone {capability.milestone}</span>
                    <Glyph
                      name={capability.state === "ready" ? "check" : "chevron"}
                    />
                  </footer>
                </article>
              ))}
            </div>
          </section>

          <section className="boundary-note" aria-label="Security boundary">
            <div className="boundary-icon">
              <Glyph name="shield" />
            </div>
            <div>
              <strong>Local by design. Narrow by default.</strong>
              <p>
                The frontend cannot spawn arbitrary processes or read arbitrary
                files. QuireForge metadata stays separate from Codex
                credentials, configuration, sessions, and connector
                authorization.
              </p>
            </div>
            <span>0 broad plugin permissions</span>
          </section>

          <footer className="product-footer">
            <span>{bootstrap.product.tagline}</span>
            <p>
              QuireForge is an unofficial community project. It is not made,
              endorsed, supported, or distributed by OpenAI.
            </p>
          </footer>
        </div>
      </main>
    </div>
  );
}
