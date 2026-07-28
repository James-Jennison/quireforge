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

The clean implementation commit is
`eee6a9ac7e3393fd7dcd73a2c4304894c70839d4`. Focused Advisor workspace tests
passed; full source validation passed with `268` desktop and `7` website unit
tests, `251` Rust tests (with `3` expected ignores), and the required
formatting, lint, type, production-build, distribution, and Tauri build gates.
Desktop and narrow Playwright coverage passed `48/48`; website Playwright
coverage passed `8/8`.

The pinned Ubuntu 22.04 Debian-only `0.1.0-beta.36` release set is bound to
that exact clean commit:

- `quireforge_0.1.0.beta.36_amd64.deb` — SHA-256
  `9c72140b28e92ae2b764406b7886f545bef86c852165a5d71bb8a9d560ea889b`;
- `quireforge-sandboxd_0.1.0.beta.36_amd64.deb` — SHA-256
  `d361d4b171a0157070cee74f99eb15179aa36d7c0bbc0f35bd1a9889b6eef82f`.

Release-set, metadata, checksum, container lifecycle/smoke, restricted host
installation, and installed-host visible-launch validation passed. The release
manifest records clean pinned-container provenance and a highest shipped
`GLIBC_2.34`, within the Ubuntu 22.04 `GLIBC_2.35` ceiling. No release or
deployment was performed.
