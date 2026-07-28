# Milestone 44 — Unified Single Attachment Entry

Status: implementation candidate for `0.1.0-beta.41`.

## Scope

Advisor's composer now presents one compact **Attach a file** entry. It opens a
bounded type chooser and invokes exactly one of the existing native pickers for
text/data, PNG/JPEG, PDF, ZIP, or static ELF inspection. Native code remains
the sole authority for each picker filter, type decision, validation, manifest,
expiry, claim, confirmation, and disposal behavior.

## Boundaries

- Exactly one pending attachment is permitted in the composer.
- The tray has no browser file input, generic uploader, drag-and-drop surface,
  collection, or new supported type.
- The existing typed commands and Advisor conversation transport are unchanged.
- The existing one-use confirmation, claim, expiry, cancellation, and disposal
  rules remain type-specific and unchanged.
- The change grants no project, terminal, Git, worktree, browser, dispatch,
  execution, connector, or Advisor authority.

## Validation and package evidence

The focused UI test proves that the single entry exposes only the five closed
type choices and routes the image choice to its existing typed picker. Full
source and pinned-Ubuntu package evidence will be recorded only after the
clean implementation commit is validated.
