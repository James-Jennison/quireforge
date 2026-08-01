import { useEffect, useId, useRef, useState } from "react";
import {
  cancelControlledBrowserVerification,
  confirmControlledBrowserVerification,
  loadControlledBrowserVerification,
  prepareControlledBrowserVerification,
  revokeControlledBrowserVerification,
} from "./lib/bridge";
import type { BrowserVerificationSnapshot } from "./lib/controlledBrowserVerification";

export function ControlledBrowserVerificationWorkbench({
  projectId,
  onClose,
}: {
  projectId: string | null;
  onClose: () => void;
}) {
  const titleId = useId();
  const closeRef = useRef<HTMLButtonElement>(null);
  const [snapshot, setSnapshot] = useState<BrowserVerificationSnapshot | null>(
    null,
  );
  const [notice, setNotice] = useState("Loading fictional local verification…");
  const [busy, setBusy] = useState(false);
  useEffect(() => {
    closeRef.current?.focus();
    void loadControlledBrowserVerification()
      .then(setSnapshot)
      .then(() =>
        setNotice(
          "Fictional, deterministic, local-only, read-only verification. No real website, profile, account, credential, connector, provider, or mutation is involved.",
        ),
      )
      .catch(() =>
        setNotice(
          "Browser verification is unavailable; no operation occurred.",
        ),
      );
  }, []);
  const apply = async (action: () => Promise<BrowserVerificationSnapshot>) => {
    setBusy(true);
    try {
      const next = await action();
      setSnapshot(next);
      setNotice(
        next.diagnostic
          ? `Verification rejected: ${next.diagnostic.replaceAll("-", " ")}.`
          : next.auditState,
      );
    } catch {
      setNotice("Browser verification is unavailable; no operation occurred.");
    } finally {
      setBusy(false);
    }
  };
  const canConfirm =
    snapshot?.state === "prepared" &&
    snapshot.attemptId &&
    snapshot.authorizationId;
  return (
    <section
      className="mock-inference-workbench"
      role="dialog"
      aria-modal="true"
      aria-labelledby={titleId}
    >
      <header className="mock-inference-workbench__header">
        <div>
          <p className="eyebrow">Fictional local-only browser verification</p>
          <h2 id={titleId}>Read-only verification review</h2>
        </div>
        <button ref={closeRef} type="button" onClick={onClose}>
          Close
        </button>
      </header>
      <p role="status">{notice}</p>
      <p>
        Project scope: {projectId ?? "unavailable"}. Target:{" "}
        <code>quireforge-fixture://verification/expected</code>. Assertion:
        fixture marker.
      </p>
      <p>
        Preparation and review do not launch the adapter. Confirmation is
        digest-bound, expires, and can be used once. Evidence never becomes a
        durable source or provider context automatically.
      </p>
      <div className="mock-inference-workbench__actions">
        <button
          type="button"
          disabled={!projectId || busy}
          onClick={() =>
            void apply(() =>
              prepareControlledBrowserVerification({
                projectId,
                taskId: null,
                target:
                  "quireforge-fixture://verification/expected?assert=marker",
                assertion: "fixture-marker",
              }),
            )
          }
        >
          Prepare review
        </button>
        <button
          type="button"
          disabled={!canConfirm || busy}
          onClick={() =>
            void apply(() =>
              confirmControlledBrowserVerification({
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
              cancelControlledBrowserVerification({
                attemptId: snapshot!.attemptId,
              }),
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
              revokeControlledBrowserVerification({
                attemptId: snapshot!.attemptId,
              }),
            )
          }
        >
          Revoke
        </button>
      </div>
      {snapshot?.evidenceDigest && (
        <p>
          Verification completed with bounded fictional evidence:{" "}
          {snapshot.visibleText}.
        </p>
      )}
      {[
        "ambiguous",
        "timed_out",
        "redirect_blocked",
        "origin_drift",
        "quarantined",
        "incompatible",
      ].includes(snapshot?.state ?? "") && (
        <p role="alert">
          The result is terminal. Automatic retry is prohibited; a future
          attempt requires a fresh review.
        </p>
      )}
    </section>
  );
}
