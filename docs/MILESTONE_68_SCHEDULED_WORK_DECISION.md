# M68 Decision: Local or External Scheduled Work

## Decision

No scheduled or recurring execution is authorized. Existing scheduled-work
presentation remains read-only discovery; it neither creates schedules nor runs
tasks.

## Required future gate

Any proposal must define schedule identity, owner consent, exact action scope,
destination and payload binding, time zone and missed-run semantics, bounded
retry, cancellation, revocation, expiry, audit, recovery, notification, and
rollback. It must distinguish local reminders from any action that accesses a
project, runtime, browser, connector, provider, filesystem, or network.

Scheduler dependencies, package impact, service ownership, and crash behavior
require separate review. This decision creates no timer, background process,
service, credential access, external action, or deployment.
