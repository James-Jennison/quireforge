import type { ProjectWorkspaceSnapshot } from "./lib/project";

interface HomeDashboardProps {
  projects: ProjectWorkspaceSnapshot;
  onNewTask: () => void;
  onAttachProject: () => void;
  onOpenProjects: () => void;
  onOpenSessions: () => void;
  onOpenIntegrations: () => void;
  onOpenTerminal: () => void;
}

export function HomeDashboard({
  projects,
  onNewTask,
  onAttachProject,
  onOpenProjects,
  onOpenSessions,
  onOpenIntegrations,
  onOpenTerminal,
}: HomeDashboardProps) {
  const visibleProjects = projects.projects
    .filter((project) => !project.archived)
    .slice(0, 3);

  return (
    <section className="home-dashboard" id="home" aria-labelledby="home-title">
      <div className="home-dashboard__main">
        <div className="home-welcome">
          <p className="eyebrow">QuireForge home</p>
          <h1 id="home-title" data-workspace-heading tabIndex={-1}>
            What should we build today?
          </h1>
          <p>
            Start a focused Codex task inside a verified local project. Your
            files stay where they are, and every execution uses the project’s
            reviewed working directory.
          </p>
        </div>

        <button className="home-composer" type="button" onClick={onNewTask}>
          <span>Describe a change, investigation, or review…</span>
          <strong>New task</strong>
        </button>

        <div className="home-section-heading">
          <h2>Projects</h2>
          <button type="button" onClick={onOpenProjects}>
            View all
          </button>
        </div>

        <div className="home-projects">
          {visibleProjects.map((project) => (
            <button type="button" onClick={onOpenProjects} key={project.id}>
              <span aria-hidden="true">⌑</span>
              <strong>{project.displayName}</strong>
              <small>
                {project.directory?.state === "connected-accessible"
                  ? "Ready for local work"
                  : "Needs attention"}
              </small>
              <em>
                {project.directory?.git.isRepository
                  ? "Git project"
                  : "Local project"}
              </em>
            </button>
          ))}
          {visibleProjects.length === 0 && (
            <button type="button" onClick={onAttachProject}>
              <span aria-hidden="true">+</span>
              <strong>Attach your first project</strong>
              <small>Choose an existing local directory</small>
              <em>Native picker</em>
            </button>
          )}
        </div>

        <div className="home-section-heading">
          <h2>Quick actions</h2>
        </div>
        <div className="home-actions">
          <button type="button" onClick={onAttachProject}>
            <span aria-hidden="true">＋</span>
            <strong>Attach project</strong>
            <small>Work in an existing folder</small>
          </button>
          <button type="button" onClick={onOpenSessions}>
            <span aria-hidden="true">◌</span>
            <strong>Resume a thread</strong>
            <small>Continue recent local work</small>
          </button>
          <button type="button" onClick={onOpenIntegrations}>
            <span aria-hidden="true">⌘</span>
            <strong>Integrations</strong>
            <small>Review connected tools</small>
          </button>
          <button type="button" onClick={onOpenTerminal}>
            <span aria-hidden="true">›_</span>
            <strong>Open terminal</strong>
            <small>Start in a verified project</small>
          </button>
        </div>
      </div>
    </section>
  );
}
