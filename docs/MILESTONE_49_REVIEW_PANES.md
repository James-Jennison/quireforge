# Milestone 49 — QuireForge Review Panes

Status: complete. Package-source commit
`f1a44324859faa2ed43f24ab60db12b58e6c6836`.

M49 adds a closed-by-default review-shell overlay to the dominant task
conversation. The shell itself is lazy loaded; each Files, Diff, Git, Preview,
Activity, and Approval implementation is its own lazy module. Closing the
shell unmounts the active pane and restores focus to its trigger. It creates no
timer, subscription, polling loop, durable record, or new native command.

| Pane | Existing typed source | Read-only boundary |
| --- | --- | --- |
| Files | `FilePreviewSnapshot` | Existing safe selected-file metadata only; no picker, opener, read, or write. |
| Diff | `git_status` then `git_diff` | One existing bounded changed-file diff after explicit pane open; no mutation. |
| Git | `git_status` | Existing branch/status summary after explicit pane open; no refresh loop or mutation. |
| Preview | M48 `advisor_generated_artifact_snapshot` and preview | Bounded generated text only after explicit pane open/selection. |
| Activity | existing normalized `ConversationEvent` state | Existing bounded task activity only. |
| Approval | existing `ConversationSnapshot.pendingApproval` | Existing proposal details only; no decision or dispatch action. |

Unavailable, empty, loading, failure, and truncation outcomes are explicitly
rendered. The shell uses semantic tabs and a labelled tabpanel, keeps focus
inside ordinary keyboard flow, and contains no live region beyond short
operation status text. Narrow layouts make the overlay full-width with bounded
insets. No pane imports terminal, worktree, browser, connector, Advisor
dispatch, approval-decision, Git-mutation, or project-write capabilities.

## Beta.44 package evidence

The pinned Ubuntu 22.04 container produced the Debian-only set in
`target/ubuntu-22.04/release/packages/`:

| Artifact | Bytes | SHA-256 |
| --- | ---: | --- |
| `quireforge_0.1.0.beta.44_amd64.deb` | 5,510,604 | `e205e345f66f50b4ab95dca9c11074796f1633a0d3ed0bd70036141b047858be` |
| `quireforge-sandboxd_0.1.0.beta.44_amd64.deb` | 3,233,532 | `30c708ee96d8312d6b0621ed642436af80347c1aea0046a651e9bfd199d327d5` |

The clean package source was `f1a44324859faa2ed43f24ab60db12b58e6c6836`.
Container lifecycle and visible launch passed. Both binaries require maximum
`GLIBC_2.34`, within the Ubuntu 22.04 `GLIBC_2.35` ceiling. The restricted
`sudo -n /usr/local/sbin/quireforge-validate-deb` wrapper installed the desktop
package and confirmed `quireforge 0.1.0~beta.44`; an isolated Xvfb launch of
`/usr/bin/quireforge` passed. The production bundle measured 194,943 bytes
startup entry, 315,548 bytes application shell, 955,122 bytes JavaScript, and
111,159 bytes CSS, within the 256 KiB / 448 KiB / 1.5 MiB / 160 KiB temporary
ceilings.
