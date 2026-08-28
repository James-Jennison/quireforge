import { useEffect, useId, useRef, useState } from "react";

import {
  cancelBrowserResearch,
  confirmBrowserResearch,
  loadBrowserResearch,
  prepareBrowserResearch,
  revokeBrowserResearch,
} from "./lib/bridge";
import type { BrowserResearchSnapshot } from "./lib/browserResearch";

const target = "https://google.com/";
const origin = "https://google.com";

export function IsolatedBrowserResearchWorkbench({
  projectId,
  onClose,
}: {
  projectId: string | null;
  onClose: () => void;
}) {
  const titleId = useId();
  const closeRef = useRef<HTMLButtonElement>(null);
  const [snapshot, setSnapshot] = useState<BrowserResearchSnapshot | null>(
    null,
  );
  const [notice, setNotice] = useState("Loading isolated research status…");
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    closeRef.current?.focus();
    void loadBrowserResearch()
      .then(setSnapshot)
      .then(() =>
        setNotice(
          "No browser request has occurred. This review is separate from Local Chat and has no account, cookie, upload, download, form, connector, or agent access.",
        ),
      )
      .catch(() =>
        setNotice("Browser research is unavailable; no request occurred."),
      );
  }, []);

  useEffect(() => {
    const closeOnEscape = (event: KeyboardEvent) => {
      if (event.key === "Escape") onClose();
    };
    window.addEventListener("keydown", closeOnEscape);
    return () => window.removeEventListener("keydown", closeOnEscape);
  }, [onClose]);

  const apply = async (action: () => Promise<BrowserResearchSnapshot>) => {
    setBusy(true);
    try {
      const next = await action();
      setSnapshot(next);
      setNotice(next.diagnostic ?? next.auditState);
    } catch {
      setNotice("Browser research is unavailable; no request occurred.");
    } finally {
      setBusy(false);
    }
  };

  const canConfirm =
    snapshot?.state === "prepared" &&
    snapshot.attemptId &&
    snapshot.authorizationId;
  const terminal = [
    "origin_drift",
    "prompt_injection",
    "timed_out",
    "incompatible",
    "failed",
    "cancelled",
    "revoked",
    "expired",
  ].includes(snapshot?.state ?? "");

  return (
    <section
      className="mock-inference-workbench"
      role="dialog"
      aria-modal="true"
      aria-labelledby={titleId}
    >
      <header className="mock-inference-workbench__header">
        <div>
          <p className="eyebrow">Isolated read-only browser research</p>
          <h2 id={titleId}>Google research review</h2>
        </div>
        <button ref={closeRef} type="button" onClick={onClose}>
          Close
        </button>
      </header>
      <p role="status">{notice}</p>
      <p>
        Approved target: <code>{target}</code>. Approved origin:{" "}
        <code>{origin}</code>.
      </p>
      <p>
        The research profile is ephemeral and JavaScript-disabled. Confirmation
        is one-use and expires. Redirects, origin drift, prompt injection, and
        timeouts stop observation; no page text is shown or retained.
      </p>
      <div className="mock-inference-workbench__actions">
        <button
          type="button"
          disabled={!projectId || busy}
          onClick={() =>
            void apply(() =>
              prepareBrowserResearch({
                projectId,
                taskId: null,
                target,
                origin,
                observationLimit: 512,
              }),
            )
          }
        >
          Prepare Google review
        </button>
        <button
          type="button"
          disabled={!canConfirm || busy}
          onClick={() =>
            void apply(() =>
              confirmBrowserResearch({
                attemptId: snapshot!.attemptId,
                authorizationId: snapshot!.authorizationId,
              }),
            )
          }
        >
          Confirm once
        </button>
        <button
          type="button"
          disabled={!snapshot?.attemptId || busy}
          onClick={() =>
            void apply(() =>
              cancelBrowserResearch({ attemptId: snapshot!.attemptId }),
            )
          }
        >
          Cancel
        </button>
        <button
          type="button"
          disabled={!snapshot?.attemptId || busy}
          onClick={() =>
            void apply(() =>
              revokeBrowserResearch({ attemptId: snapshot!.attemptId }),
            )
          }
        >
          Revoke
        </button>
      </div>
      {snapshot?.state === "observed" && (
        <p>
          Observation completed with bounded provenance only; page content is
          unavailable.
        </p>
      )}
      {terminal && (
        <p role="alert">
          This result is terminal. Automatic retry is prohibited; a future
          attempt requires a new review and confirmation.
        </p>
      )}
    </section>
  );
}
