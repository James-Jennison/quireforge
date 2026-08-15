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
          <h2>Provider-neutral Work adapters</h2>
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
        <fieldset className="work-adapter-catalog">
          <legend>Available deterministic fixtures</legend>
          {catalog.profiles.map((profile) => (
            <div key={profile.id} className="work-adapter-catalog__item">
              <input
                id={`work-adapter-${profile.id}`}
                type="radio"
                name="work-adapter"
                checked={selectedId === profile.id}
                onChange={() => setSelectedId(profile.id)}
              />
              <label htmlFor={`work-adapter-${profile.id}`}>
                <strong>{profile.adapterLabel}</strong>
                <span>
                  {profile.providerLabel} · {profile.modelLabel} ·{" "}
                  {profile.scenario}
                </span>
                <code>descriptor {profile.descriptorSha256}</code>
              </label>
            </div>
          ))}
        </fieldset>
      )}
      <p className="context-note">
        Privacy and retention: no live provider exists here. Any later attempt
        remains subject to the existing reviewed, expiring, digest-bound local
        fixture contract.
      </p>
    </section>
  );
}
