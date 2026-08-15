# M66 Context and Authority Ledger

## Purpose

M66 makes existing local governance evidence inspectable in the Work lane. The
ledger is a read-only, project-scoped projection; it does not create a source,
context bundle, artifact reference, connector operation, browser attempt, or
authority grant.

## Projection and privacy boundary

The native snapshot is bounded to 64 records and exposes only: record kind and
UUID, project/task association, lifecycle state, SHA-256 digest, selected-item
count, expiry/completion/creation timestamps, and the latest content-free audit
outcome. It reads existing M55 durable-source, M57 fictional-connector, M58
controlled-browser, M60 context-bundle, and M65 artifact-reference records.
M63 local-runtime use remains represented only through its governed M60 receipt;
M64 provides the shared lane shell and introduces no ledger record of its own.

The projection never contains source or artifact bytes, labels, paths,
transcripts, URLs, provider context, credentials, browser data, local-runtime
output, request payloads, or authority tokens. It rejects unknown bridge fields
and returns a diagnostic-only empty snapshot if local storage cannot be read.

## Authority

The Ledger is neither ambient memory nor a transfer mechanism. It cannot
prepare, review, confirm, dispatch, retry, recover, delete, upload, save,
execute, or authorize anything. Existing M55/M57/M58/M60/M63/M64/M65 ownership,
confirmation, expiry, revocation, digest-binding, audit, and fail-closed rules
remain unchanged.

## Evidence

Focused storage coverage proves content-free context and durable-source mapping.
Strict TypeScript bridge coverage rejects a content-bearing field, and the
desktop component coverage verifies content-free lifecycle rendering across
record kinds. Full validation evidence is recorded only after the integrated
M66--M68 completion gates run.

The beta.79 accessibility candidate preserves this contract while restoring the
Ledger's route-local H1-to-H2 structure. Desktop and mobile end-to-end Axe
coverage exercises bounded Ledger metadata and confirms the route remains lazy;
no receipt content or authority was added.
