import {
  usageResetLabel,
  type CodexUsageSnapshot,
  type CodexUsageWindow,
} from "./lib/usage";

interface UsagePanelProps {
  snapshot: CodexUsageSnapshot;
  state: "checking" | "native" | "preview" | "unavailable";
  busy: boolean;
  compact?: boolean;
  onRefresh: () => void;
}

function windowLabel(window: CodexUsageWindow): string {
  const minutes = window.windowDurationMinutes;
  if (minutes === 10_080) return "Weekly window";
  if (minutes && minutes % 1_440 === 0) return `${minutes / 1_440}-day window`;
  if (minutes && minutes % 60 === 0) return `${minutes / 60}-hour window`;
  if (minutes) return `${minutes}-minute window`;
  return window.kind === "primary" ? "Primary window" : "Secondary window";
}

function scopeLabel(scope: CodexUsageSnapshot["runtimeMeters"][number]["scope"]): string {
  switch (scope) {
    case "shared-account":
      return "Shared account scope";
    case "model":
      return "Model scope";
    case "client":
      return "Client scope";
    case "unknown":
      return "Scope not verified";
  }
}

const sharedUsageExplanation =
  "The Codex runtime does not currently expose a verified shared account-usage value. Open ChatGPT Usage settings for the authoritative balance.";

export function UsagePanel({
  snapshot,
  state,
  busy,
  compact = false,
  onRefresh,
}: UsagePanelProps) {
  const windows = snapshot.runtimeMeters.flatMap((meter) =>
    meter.windows.map((window) => ({ meter, window })),
  );
  const sharedUsage =
    state === "native" && snapshot.state === "ready"
      ? snapshot.sharedUsage
      : null;

  if (compact) {
    const status = sharedUsage
      ? usageResetLabel(sharedUsage.resetsAt)
      : state === "checking"
        ? "Checking Codex usage"
        : state === "preview"
          ? "Native usage unavailable"
          : "View in ChatGPT";

    return (
      <div
        className="usage-compact"
        aria-label={
          sharedUsage
            ? `Shared usage ${sharedUsage.remainingPercent}% remaining`
            : "Shared usage unavailable"
        }
        title={sharedUsage ? undefined : sharedUsageExplanation}
      >
        <div>
          <span>Shared usage</span>
          <strong>{sharedUsage ? `${sharedUsage.remainingPercent}%` : "—"}</strong>
        </div>
        <small>{status}</small>
      </div>
    );
  }

  return (
    <section className="usage-panel" aria-labelledby="usage-title">
      <div className="usage-panel__heading">
        <div>
          <span>Account</span>
          <h2 id="usage-title">Codex runtime limits</h2>
        </div>
        <button
          type="button"
          disabled={busy || state !== "native"}
          onClick={onRefresh}
        >
          Refresh
        </button>
      </div>

      {state === "checking" && <p>Checking Codex runtime limits…</p>}
      {state === "preview" && (
        <p>Native usage information is unavailable in browser preview.</p>
      )}
      {state === "unavailable" && (
        <p>
          Codex runtime limits are currently unavailable. QuireForge will not
          estimate the remaining amount.
        </p>
      )}
      {state === "native" && snapshot.state === "not-metered" && (
        <p>No metered window reported by the Codex runtime.</p>
      )}
      {state === "native" && snapshot.state === "unavailable" && (
        <p>
          Codex runtime limits are currently unavailable. QuireForge will not
          estimate the remaining amount.
        </p>
      )}

      {state === "native" && snapshot.state === "ready" && (
        <>
          <p className="usage-panel__notice">
            These meters are reported by the Codex runtime and may not match the
            shared usage balance shown in ChatGPT Settings.
          </p>
          {windows.length === 0 ? (
            <p>No Codex runtime meters were reported.</p>
          ) : (
            <div className="usage-panel__meters">
              {windows.map(({ meter, window }) => (
                <article
                  className="usage-meter"
                  key={`${meter.limitId}-${window.kind}`}
                >
                  <div className="usage-meter__copy">
                    <span>
                      {meter.label} · {windowLabel(window)}
                    </span>
                    <small>Meter ID: {meter.limitId}</small>
                    <small>{scopeLabel(meter.scope)}</small>
                    <strong>{window.remainingPercent}% remaining</strong>
                    <small>{usageResetLabel(window.resetsAt)}</small>
                  </div>
                  <div
                    className="usage-meter__bar"
                    role="progressbar"
                    aria-label={`${meter.label} runtime meter ${windowLabel(window)} remaining`}
                    aria-valuemin={0}
                    aria-valuemax={100}
                    aria-valuenow={window.remainingPercent}
                  >
                    <span style={{ width: `${window.remainingPercent}%` }} />
                  </div>
                  {meter.limited && (
                    <em>Codex reports that this limit has been reached.</em>
                  )}
                </article>
              ))}
            </div>
          )}
        </>
      )}

      <small className="usage-panel__note">
        QuireForge displays only Codex-reported values and does not calculate,
        estimate, predict, or infer overall quota.
      </small>
    </section>
  );
}
