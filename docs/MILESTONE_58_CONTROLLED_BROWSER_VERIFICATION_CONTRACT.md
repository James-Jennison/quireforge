# M58 — Controlled Browser Verification Contract and Implementation Plan

Status: ratified contract. The approved fictional/local-only/read-only beta.61
implementation candidate is in validation; package and installed-host
acceptance remain required before M58 is complete. The contract continues to
exclude real browser targets, sessions, credentials, automation, and network
activity.

M58 defines a narrow future ability to verify one explicitly reviewed browser-visible assertion. It is not general browsing, research retrieval, OAuth, browser automation, a web agent, a connector, generic MCP, a native tool, provider transport, or an external-action route. Projects and durable tasks remain authoritative; a browser target, page, session, result, and evidence record are subordinate references.

## Purpose and non-authority

The future user problem is bounded confirmation of a fact visible at one reviewed target, for example whether a deterministic local page contains a named marker. Page content is untrusted evidence, never an instruction, authority grant, or provider prompt. The application Tauri/WebKitGTK webview is the renderer for QuireForge itself; it is not an external-browser capability. The existing Codex-owned, native-controlled authentication handoff is likewise outside M58 and grants no page observation or browser-session authority.

Descriptor or capability metadata describes a possible verifier only. It does not select a session, launch or attach a browser, permit a URL, navigate, observe a page, retain evidence, admit a source, include context, transmit to a provider, interact, mutate, automate, or use a credential.

## Independent authority categories

Each category requires its own future typed policy decision and cannot imply another: session selection; launch/attachment; URL/origin scope; navigation; redirects; tabs/windows; observation; DOM inspection; screenshots; downloads; uploads; clipboard; form entry; clicks; authentication; cookies/storage; credential use; cross-origin navigation; external application launch; file chooser use; browser-mediated mutation; connected services; provider transmission; M55 admission; context inclusion; automation; generic MCP; and native filesystem, shell, terminal, Git, cloud, or deployment operations.

Passive observation is distinct from scrolling to reveal already reviewed content and from expanding an explicitly non-mutating disclosure. Link clicks, form entry/submission, account changes, purchases, messages, uploads, downloads, deletion, and every other external effect are interaction or mutation, not verification. The initial slice permits none of them. A later browser-mediated mutation contract must be separately ratified; M58 verification cannot be extended into one.

## Initial future vertical slice

The smallest authorized *future implementation proposal* is a fictional, deterministic, local-only target. It has one native-owned ephemeral browser instance with a new empty profile; no existing profile, signed-in session, cookie, storage, credential, password manager, client certificate, OAuth, CAPTCHA, manual takeover, account switching, download, upload, file chooser, network target, or external application launch. It may verify one exact local fixture URL and a single typed assertion, with optional project and task scope.

The controller must bind canonical URL/origin, assertion, expected fixture identity, project/task, descriptor version, policy version, timeout, and evidence limits into a request digest. Preparation performs no launch or navigation. Review shows the local-only/fictional label, project/task, exact target summary, assertion, bounds, expiry, and exclusions. Confirmation is explicit, digest-bound, expiring, and one-use. It permits one bounded launch, navigation, and read-only observation. Cancellation, expiry, denial, replay, revocation, drift, incompatibility, quarantine, timeout, ambiguity, or exit fails closed and creates no success claim or automatic retry.

## Closed lifecycle

Only the native controller changes state. Invalid transitions fail closed, consume or invalidate pending authorization as applicable, create no browser authority, and emit a bounded audit event.

| State | Entry and allowed progression | Authorization / process / evidence / recovery |
| --- | --- | --- |
| `proposed` | A typed local fixture request is formed; may prepare or close. | No authorization or process; no evidence. Edit creates a new proposal. |
| `prepared` | Native validation and digest binding passed; may await confirmation, cancel, expire, revoke, become incompatible/quarantined, or close. | No process; only bounded review projection. Fresh preparation is required after invalidation. |
| `awaiting_confirmation` | Review is displayed; may confirm, deny, cancel, expire, revoke, drift, or close. | One pending non-callable handle; no process or evidence. |
| `confirmed` | Native consumed a matching one-use authorization; may launch or enter a terminal failure. | Authorization is no longer reusable; no process yet. |
| `launching` | Native creates only the owned ephemeral fixture process; may navigate, cancel, time out, fail, or become ambiguous. | One tracked process group only; no success evidence. Cleanup joins it. |
| `navigating` | Exact target load is in progress; may observe, redirect-block, drift, timeout, cancel, fail, ambiguous, or close. | Process may exist; navigation-chain audit only. |
| `observing` | Allowed local page is loaded; may verify, fail verification, cancel, timeout, drift, ambiguous, or close. | Process may exist; bounded provisional evidence only. |
| `verified` | Assertion and complete bounded evidence succeeded. | Terminal; process must already be closed; immutable evidence/audit may persist. |
| `verification_failed` | Assertion is false or required target/evidence is missing. | Terminal; no success evidence; process closed. Fresh proposal only. |
| `cancelled`, `denied`, `expired`, `revoked` | User/policy terminal outcomes. | Non-callable; process closed; audit only, no success evidence. |
| `redirect_blocked`, `origin_drift`, `timed_out` | Scope or bounded-time rule failed. | Non-callable terminal; process closed; no automatic retry. |
| `ambiguous` | Exit, crash, incomplete observation, or storage/audit uncertainty prevents a complete result. | Non-callable terminal; process closed; no durable success claim; fresh review only. |
| `quarantined`, `incompatible` | Integrity, descriptor/runtime compatibility, or policy failure occurred. | Non-callable terminal; process closed; recovery requires later native policy. |
| `closed` | Cleanup after any terminal state is complete. | No process or usable authorization. Terminal evidence is retained only under its approved rule. |

Required audit events are `proposed`, `prepared`, `reviewed`, `confirmed`, `launch_started`, `navigation_observed`, `terminal`, and `cleanup_completed`. They contain opaque IDs, state, bounded reason code, timestamps, request/evidence digests, and project/task references only—never page content, secrets, cookies, raw URLs, browser-profile paths, or diagnostics.

## Target, navigation, and session policy

The first slice allows only its exact native-served fictional fixture scheme and canonical path; no network scheme is allowed. A later real-network proposal must separately choose schemes, DNS policy, proxy policy, certificate handling, and address restrictions.

Canonicalization must reject malformed URLs and normalize scheme/host case, default ports, dot segments, percent encoding, and IDN to a comparison-safe ASCII/punycode form before digesting. It must bind origin and, where requested, path and query; fragments never expand scope. `file:`, `data:`, `javascript:`, blob, custom, privileged, and external-protocol URLs are blocked. Popups, new windows/tabs, external protocol launches, mixed content, captive portals, certificate errors, authentication challenges, and unexpected redirects are blocked and terminal.

Redirects are zero in the first slice. A later policy must set a small numeric limit, record every canonical hop, require explicit same-origin rules, block cross-origin redirects unless separately bound, and consider DNS rebinding, loopback/private/link-local addresses, hostname resolution changes, and confusable IDN origins. Neither a redirect nor a successful load broadens URL, origin, session, or interaction authority.

No session may be selected or attached in the first slice. Existing profiles, cookies, local/session storage, saved forms, password managers, certificates, OAuth state, and ambient sign-in are neither inspected nor reused. An auth wall, CAPTCHA, expired session, duplicate/ambiguous identity, or account switch is a bounded terminal failure. Manual user takeover is excluded.

## Observation, evidence, and retention

Native normalization may retain only the configured bounded evidence projection: final canonical URL/origin, title, timestamp, runtime identity, navigation chain, typed assertion/result, content digest, and a tightly bounded selected visible-text, DOM-property, accessibility-tree, or screenshot projection. Every field has byte/count limits, UTF-8 validation, truncation flags, and redaction for secrets, personal data, and unsafe diagnostic text. Hidden DOM content is not evidence merely because it is inspectable. Screenshot evidence, if later selected, is a separately bounded capture class and must not include unreviewed browser chrome, other windows, or profiles.

Evidence is project-bound and task-bound only when the task is selected. It is not automatically an M55 durable source, provider context, provider transmission, connector result, or MCP resource. M55 admission remains a fresh reviewed operation; context inclusion and provider transmission each remain separate destination-specific decisions. A later implementation must define retention/deletion and preserve only content-free audit linkage after deletion.

## Failure, ambiguity, and cleanup

Every potentially blocking phase has a bounded native timeout. Launch or attachment failure, unavailable portal/desktop service, target load failure, renderer crash, browser exit, redirect/origin drift, certificate/auth/CAPTCHA wall, DOM drift, missing target, partial capture, storage failure, audit failure, or cancellation during any phase returns control to the UI with a non-secret reason and closes only the tracked process/process group. It must never kill by arbitrary process name.

An incomplete cleanup, incomplete evidence write, or unknown renderer outcome is `ambiguous`, never `verified`. No retry occurs automatically after timeout, crash, ambiguity, reload, remount, duplicate event, or restart. Restart finds in-flight records, invalidates their authorization, records bounded recovery evidence, performs owned-process cleanup where possible, and requires a fresh proposal. This incorporates the M55 chooser lesson: unavailable session components must produce bounded governed failure rather than freeze QuireForge.

## Threat model and controls

Page text, DOM, accessibility data, screenshots, and titles are hostile input: they cannot invoke tools, alter policy, become instructions, or enter provider context automatically. The future implementation must test and fail closed for prompt injection, hidden/overlay content, clickjacking, confusable origins, redirect/cross-origin leakage, extension interference, profile/session and credential leakage, clipboard/file-chooser/download abuse, local-network or loopback SSRF-like navigation, debugging-port exposure, stale/replayed confirmation, duplicate UI events, process recovery, audit tampering, evidence spoofing, and provider treatment of evidence as instruction.

## Future architecture and mechanism selection

The native controller owns policy evaluation, digest/authorization storage, process ownership, navigation enforcement, evidence normalization, audit, recovery, and closed Tauri commands/results. The frontend owns only bounded review and state presentation; it never supplies a raw browser command, profile path, policy decision, process ID, completion state, or evidence assertion.

The recommended first mechanism is a **native-owned, separately spawned ephemeral WebKitGTK fixture adapter**, constrained to a local deterministic fixture and no browser-profile directory. It aligns with the installed Linux/Tauri WebKitGTK runtime, keeps process ownership explicit, avoids a remote-debugging endpoint and a general automation protocol, works on pinned Ubuntu 22.04 within the existing `GLIBC_2.35` ceiling, and supports visible accessibility evidence. It must be introduced only by the later implementation approval and its packaging review.

Playwright/WebDriver/Chromium remote debugging are unsuitable for this first slice: they introduce broad automation semantics, binary/package impact, remote-control surfaces, and harder profile/process confinement. Reusing the Tauri application webview would blur renderer and browser authority. Portal APIs are not a navigation controller. A headless mechanism alone is insufficient for the required visible/accessibility acceptance, although deterministic headless-only fixture tests may supplement—not replace—visible validation.

## Required future acceptance matrix and completion criteria

Before implementation can complete, focused native, bridge, UI, fixture, packaging, and installed-host tests must prove: all lifecycle transitions; prepare without navigation; digest binding; one-use/expiry/cancel/deny/replay; duplicate UI events and reload/remount recovery; exact URL/origin binding; accepted and blocked redirects; scheme/cross-origin blocking; timeout; browser and renderer crash; tracked cleanup; restart recovery; revocation, drift, quarantine, incompatibility, and ambiguity without retry; bounded/redacted evidence; no M55 admission, provider transmission, credential/session reuse, connector/MCP authority, automation, or mutation; accessibility and governed viewport/scaling; Ubuntu 22.04 package/ABI checks; installed-host validation; and M55–M57 regression coverage.

Completion requires a fictional/local-only target, complete audit linkage, clean process exit, all stated tests, and explicit proof that no real network, credential, session, browser automation, connector, MCP, provider, filesystem, shell, Git, deployment, or mutation authority was introduced. A real target, signed-in session, interaction, download/upload, browser-mediated mutation, or provider/connector-directed browser control requires a new contract and approval.

## Ratification and remaining authority

This contract ratifies M58’s decision and makes the future fictional/local-only vertical slice executable without reopening its fundamental authority model. It does not authorize implementation. No beta number is reserved. The next genuine boundary is one comprehensive start-to-finish **M58 Controlled Browser Verification Implementation** goal, limited to this contract’s fictional, local-only, read-only slice.
