# M65 Artifact Reference Contract

## Purpose

M65 may record a user-confirmed association between a project (and optionally
one of its tasks) and an existing generated artifact. A reference is metadata
only; it is not artifact storage, a context source, or a filesystem handle.

## Stored fields and lifecycle

Each immutable active reference contains an opaque artifact UUID, its SHA-256,
closed class, bounded display label, project UUID, nullable task UUID, and
creation timestamp. No path, filename chosen by the user, source bytes,
transcript, provider data, or preview body is stored. A reference is either
`active` or `deleted`; deletion removes only the reference.

The original M48 artifact remains process-local and may expire independently.
Studio reports an unavailable original explicitly and does not retain, recover,
or recreate it. A reference never authorizes preview, save, dispatch, context
assembly, model transmission, filesystem access, or execution.

## Authority

Creation requires an explicit review of the exact project, optional task,
artifact ID, digest, class, and label, followed by confirmation. Native code
checks project/task ownership and current artifact digest before recording the
reference. Listing is project-scoped. No automatic promotion from Advisor or
Code is permitted.
