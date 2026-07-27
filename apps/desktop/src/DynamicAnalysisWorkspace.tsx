import { useEffect, useRef, useState } from "react";

import {
  clearDynamicAnalysis,
  loadDynamicAnalysis,
  pickDynamicAnalysis,
  runDynamicAnalysis,
} from "./lib/bridge";
import {
  scaffoldDynamicAnalysis,
  type DynamicAnalysisSnapshot,
} from "./lib/dynamicAnalysis";

const messages: Record<string, string> = {
  "worker-unavailable":
    "The separately installed isolated-analysis worker is unavailable. The AppImage never includes or starts it.",
  "unsupported-runtime":
    "Only static ELF64 x86_64 files without a program interpreter are supported.",
  "unsupported-type":
    "Select one static ELF64 x86_64 executable or static-PIE shared object.",
  "invalid-signature": "The selected file is not a valid ELF signature.",
  "source-too-large": "The selected file exceeds the 32 MiB analysis limit.",
};

export function DynamicAnalysisWorkspace() {
  const [snapshot, setSnapshot] = useState<DynamicAnalysisSnapshot>(
    scaffoldDynamicAnalysis,
  );
  const [confirmed, setConfirmed] = useState(false);
  const confirmationRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    void loadDynamicAnalysis()
      .then(setSnapshot)
      .catch(() =>
        setSnapshot({
          ...scaffoldDynamicAnalysis,
          state: "unavailable",
          diagnosticCode: "worker-unavailable",
        }),
      );
  }, []);
  useEffect(() => {
    if (snapshot.state === "ready") confirmationRef.current?.focus();
  }, [snapshot.manifest?.runId, snapshot.state]);

  const manifest = snapshot.manifest;
  return (
    <section
      className="workspace-panel dynamic-analysis"
      aria-labelledby="dynamic-analysis-title"
    >
      <header className="workspace-panel__header">
        <div>
          <span>QuireForge only</span>
          <h1 id="dynamic-analysis-title">Isolated ELF analysis</h1>
          <p>
            One static ELF64 x86_64 sample. No project, terminal, network, or
            Advisor access.
          </p>
        </div>
      </header>
      <div className="dynamic-analysis__limits" role="note">
        32 MiB · 1 vCPU · 512 MiB · 30 seconds · no network · metadata-only
        result
      </div>
      {snapshot.diagnosticCode && (
        <p className="form-error" role="status">
          {messages[snapshot.diagnosticCode] ??
            "The sample could not be prepared safely."}
        </p>
      )}
      {manifest ? (
        <div
          className="dynamic-analysis__manifest"
          aria-label="Selected isolated-analysis sample"
        >
          <strong>{manifest.displayName}</strong>
          <span>
            Static {manifest.elfType} · {manifest.byteSize.toLocaleString()}{" "}
            bytes
          </span>
          <span>SHA-256 {manifest.sha256.slice(0, 12)}…</span>
        </div>
      ) : (
        <p>
          Select a sample explicitly. Nothing is sent to Advisor or a project.
        </p>
      )}
      {snapshot.result && (
        <div className="dynamic-analysis__result" role="status">
          <strong>Result: {snapshot.result.outcome}</strong>
          <span>
            {snapshot.result.elapsedMs} ms · guest{" "}
            {snapshot.result.guestStarted ? "started" : "not started"}
          </span>
        </div>
      )}
      <div className="workspace-actions">
        <button
          type="button"
          onClick={() =>
            void pickDynamicAnalysis().then((next) => {
              setConfirmed(false);
              setSnapshot(next);
            })
          }
        >
          Select static ELF64
        </button>
        <button
          type="button"
          disabled={!manifest}
          onClick={() =>
            void clearDynamicAnalysis().then((next) => {
              setConfirmed(false);
              setSnapshot(next);
            })
          }
        >
          Clear
        </button>
      </div>
      {manifest && (
        <div className="dynamic-analysis__confirmation">
          <label>
            <input
              ref={confirmationRef}
              type="checkbox"
              checked={confirmed}
              onChange={(event) => setConfirmed(event.currentTarget.checked)}
            />{" "}
            I understand this runs one selected static sample in a separately
            installed isolated worker.
          </label>
          <button
            type="button"
            disabled={!confirmed}
            onClick={() =>
              void runDynamicAnalysis({
                runId: manifest.runId,
                sha256: manifest.sha256,
                confirmed,
              }).then((next) => {
                setConfirmed(false);
                setSnapshot(next);
              })
            }
          >
            Run isolated analysis
          </button>
        </div>
      )}
    </section>
  );
}
