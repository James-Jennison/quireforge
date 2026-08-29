import type { ReactNode } from "react";

interface ConversationShellProps {
  mode: "chat" | "code";
  id?: string;
  titleId: string;
  eyebrow: string;
  title: string;
  boundary: ReactNode;
  shelf?: ReactNode;
  children: ReactNode;
}

// This is deliberately presentation-only. The mode adapter owns all runtime,
// approval, project, and context decisions; sharing this shell grants none.
export function ConversationShell({
  mode,
  id = "conversation",
  titleId,
  eyebrow,
  title,
  boundary,
  shelf,
  children,
}: ConversationShellProps) {
  return (
    <section
      className="conversation-workspace conversation-shell"
      data-conversation-mode={mode}
      id={id}
      aria-labelledby={titleId}
    >
      <div className="conversation-workspace__intro">
        <div>
          <p className="eyebrow">{eyebrow}</p>
          <h1 id={titleId} data-workspace-heading tabIndex={-1}>
            {title}
          </h1>
        </div>
        {boundary}
      </div>
      {shelf}
      {children}
    </section>
  );
}
