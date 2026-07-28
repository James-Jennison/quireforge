# Milestone 41 — Advisor Conversation Usability

Milestone 41 refines only the bounded transient Advisor presentation. It makes
the Advisor log the independently scrollable portion of the chat surface,
keeps follow-latest only while the reader remains at the newest content, offers
an accessible **Jump to latest** control after manual scrollback, and keeps the
anchored composer from obscuring the final reply. The existing optional Advisor
details surface is closed by default, independently scrollable, and usable as a
desktop drawer or narrow overlay.

## Boundaries preserved

- Advisor prompts, replies, attachments, selected Project State, approval
  drafts, dispatch receipts, and completion reports retain their existing
  transient and bounded contracts.
- No app-server request shape, attachment type, source path, raw content,
  project context, terminal, Git, approval, dispatch, execution, connector,
  provider, or authentication capability changes.
- The details drawer displays only existing safe transient context and
  capability information. It is not transcript or file persistence.
- This milestone adds no dependency, transport, generic upload, file type, or
  authority.

## Accessibility and responsive contract

- The conversation log remains keyboard-focusable with native scrolling keys.
- Scrolling away from the end stops forced follow-latest. **Jump to latest** is
  a labelled control that restores the current end position.
- The details drawer starts closed; its close control and Escape return focus
  to the Details trigger.
- Transcript, drawer, and bounded composer retain intended independent
  scrolling at desktop, narrow widths, constrained heights, and 200% zoom.
- The composer stays visible in the chat surface without covering the final
  reply or causing horizontal application overflow.

## Package gate

The M41 candidate is `0.1.0-beta.36`. It requires full source, contract,
accessibility, desktop/narrow Playwright, production-bundle, Debian-only
pinned-container provenance/ABI, lifecycle, installed-smoke, and visible-launch
validation before evidence can be recorded.
