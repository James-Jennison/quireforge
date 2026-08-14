import type { ProjectWorkspaceSnapshot } from "./lib/project";
import type { TaskCatalogSnapshot } from "./lib/taskRecords";

interface HomeDashboardProps {
  projects: ProjectWorkspaceSnapshot;
  currentProject: ProjectWorkspaceSnapshot["projects"][number] | null;
  taskCatalog: TaskCatalogSnapshot;
  onNewTask: () => void;
  onOpenTaskCatalog: () => void;
  onOpenDurableSources: () => void;
  onAttachProject: () => void;
  onOpenProjects: () => void;
  onOpenSessions: () => void;
  onOpenIntegrations: () => void;
  onOpenTerminal: () => void;
}

export function HomeDashboard({
  projects,
  currentProject,
  taskCatalog,
  onNewTask,
  onOpenTaskCatalog,
  onOpenDurableSources,
  onAttachProject,
  onOpenProjects,
  onOpenSessions,
  onOpenIntegrations,
  onOpenTerminal,
}: HomeDashboardProps) {
  const visibleProjects = projects.projects
    .filter((project) => !project.archived)
    .slice(0, 3);
  const activeTaskCount = taskCatalog.tasks.filter(
    (task) => task.status === "active",
  ).length;
  const taskSummary =
    taskCatalog.state === "unavailable"
      ? "Task metadata is unavailable."
      : taskCatalog.taskCount === 0
        ? "No local tasks are recorded for this project."
        : `${activeTaskCount} active of ${taskCatalog.taskCount} local task${taskCatalog.taskCount === 1 ? "" : "s"}.`;

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

        <button
          className="home-composer"
          data-visual-region="home-composer"
          type="button"
          onClick={onNewTask}
        >
          <span>Describe a change, investigation, or review…</span>
          <strong>New task</strong>
        </button>

        <div className="home-section-heading">
          <h2>Current workspace</h2>
          <button type="button" onClick={onOpenProjects}>
            Manage projects
          </button>
        </div>

        <div className="home-projects">
          {currentProject ? (
            <button type="button" onClick={onNewTask}>
              <span aria-hidden="true">⌑</span>
              <strong>{currentProject.displayName}</strong>
              <small>
                {currentProject.directory?.state === "connected-accessible"
                  ? "Ready for a focused task"
                  : "Project needs attention before work can start"}
              </small>
              <em>
                {currentProject.directory?.git.isRepository
                  ? "Verified Git workspace"
                  : "Verified local workspace"}
              </em>
            </button>
          ) : (
            <button type="button" onClick={onAttachProject}>
              <span aria-hidden="true">+</span>
              <strong>Choose a workspace</strong>
              <small>Attach an existing local project to begin</small>
              <em>Native picker</em>
            </button>
          )}
        </div>

        <div className="home-section-heading">
          <h2>Work inventory</h2>
          <span>Local metadata only</span>
        </div>
        <div className="home-work-inventory">
          <section>
            <span aria-hidden="true">✓</span>
            <h3>Task catalogue</h3>
            <p>{taskSummary}</p>
            {taskCatalog.selectedTask && (
              <small>
                Selected: {taskCatalog.selectedTask.title} ·{" "}
                {taskCatalog.plans.length} plan
                {taskCatalog.plans.length === 1 ? "" : "s"}
              </small>
            )}
            <button type="button" onClick={onOpenTaskCatalog}>
              Open task catalogue
            </button>
          </section>
          <section>
            <span aria-hidden="true">⌑</span>
            <h3>Sources and artifacts</h3>
            <p>
              Review explicitly admitted local source records and artifact
              references. Nothing is sent or attached automatically.
            </p>
            <button type="button" onClick={onOpenDurableSources}>
              Review durable sources
            </button>
          </section>
          <section>
            <span aria-hidden="true">◌</span>
            <h3>Thread activity</h3>
            <p>
              Continue or inspect existing task history without changing the
              selected project or starting work.
            </p>
            <button type="button" onClick={onOpenSessions}>
              Open threads
            </button>
          </section>
        </div>

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
