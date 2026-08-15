import { useEffect, useState } from "react";
import { loadMockInferenceCatalog } from "./lib/bridge";
import type { MockInferenceCatalog } from "./lib/mockInference";

export function WorkAdapterCatalog({
  loadCatalog = loadMockInferenceCatalog,
}: {
  loadCatalog?: () => Promise<MockInferenceCatalog>;
}) {
  const [catalog, setCatalog] = useState<MockInferenceCatalog | null>(null);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const selected =
    catalog?.profiles.find((profile) => profile.id === selectedId) ?? null;

  useEffect(() => {
    let active = true;
    void loadCatalog()
      .then((next) => {
        if (!active) return;
        setCatalog(next);
        setSelectedId(next.profiles[0]?.id ?? null);
      })
      .catch(() => active && setCatalog(null));
    return () => {
      active = false;
    };
  }, [loadCatalog]);

  return (
    <section className="task-template-workbench" aria-label="Work adapters">
      <header className="task-template-workbench__header">
        <div>
          <p className="eyebrow">Local fixture catalogue</p>
          <h1 className="work-route-title">Provider-neutral Work adapters</h1>
        </div>
      </header>
      <p>
        Compare deterministic local destination descriptors before a governed
        review. Selecting one is local UI state only: it does not connect,
        transmit, retain input, request credentials, or grant authority.
      </p>
      {!catalog ? (
        <p role="status">
          Local adapter fixtures are unavailable; no connection was attempted.
        </p>
      ) : (
        <div className="work-adapter-catalog">
          <label htmlFor="work-adapter">Deterministic fixture</label>
          <select
            id="work-adapter"
            value={selectedId ?? ""}
            onChange={(event) => setSelectedId(event.target.value)}
          >
            {catalog.profiles.map((profile) => (
              <option key={profile.id} value={profile.id}>
                {profile.providerLabel} · {profile.modelLabel} ·{" "}
                {profile.scenario}
              </option>
            ))}
          </select>
          {selected && (
            <section
              className="work-adapter-catalog__detail"
              aria-label="Selected fixture details"
            >
              <p className="eyebrow">Selected local fixture</p>
              <h2>{selected.adapterLabel}</h2>
              <dl>
                <div>
                  <dt>Scenario</dt>
                  <dd>{selected.scenario}</dd>
                </div>
                <div>
                  <dt>Retention</dt>
                  <dd>Transient local fixture</dd>
                </div>
                <div>
                  <dt>Descriptor</dt>
                  <dd>
                    <code title={selected.descriptorSha256}>
                      {selected.descriptorSha256.slice(0, 12)}…
                      {selected.descriptorSha256.slice(-8)}
                    </code>
                  </dd>
                </div>
              </dl>
            </section>
          )}
        </div>
      )}
      <p className="context-note">
        Privacy and retention: no live provider exists here. Any later attempt
        remains subject to the existing reviewed, expiring, digest-bound local
        fixture contract.
      </p>
    </section>
  );
}
