# Refine the progress-first report layout

Status: resolved
Blocked by: None
Triage: ready-for-agent
Type: task
Owner: /root (HTML presentation and isolated browser inspection)

## Required work

- Keep the user's selected progress-first layout and the retained comparison in
  ticket 02; use a shorter heading and introduction.
- Put a collapsed Report details summary immediately after the metrics so it is
  discoverable without scrolling through the routine list.
- Group fixture notes, interpretation, stale reports and corrected reports;
  retain every exact notice in bounded, keyboard-scrollable lists.
- Preserve source/evidence context filters, immutable corrections, independent
  stage totals, routine evidence, offline loading and live refresh. Keep actual
  load/refresh errors visible above the progress overview.
- Verify the maintained report on desktop and phone, publish generated HTML and
  the new snapshot identity, and clean only the new review artifacts.

## Comments

2026-10-13: the user chose “Progress first, collapsed report details”. While the
layout session awaited its preview-server escalation, the recovery coordinator
published the earlier context-aware layout and completed the three-way prototype
comparison in ticket 02. This follow-up preserves that finished work and adds
the shorter introduction, nearby details summary and grouped notices. Reporting
code and shared observations belong to the completed recovery effort.

## Completion evidence

The user-selected progress-first layout is implemented in
`crates/pipeline/src/progress.html.in`. The metrics precede a compact collapsed
Report details row; all notes are grouped and bounded, with visible stale and
correction counts. Source provenance and actual load errors remain accessible.
Generated HTML is published and opened in the Windows browser at localhost:4185.

`../layout-refinement/report.md` records browser assertions, screenshots, file
loading, live refresh, independent stage/context totals, build validation and
exact cleanup receipts. Both scoped utility passes succeed; identified browser
cleanup reclaims 30,937,964 bytes and isolated Cargo clean removes 571 files
(146.9 MiB reported). Required evidence and source remain, with
224,959,778,816 measured available filesystem bytes. No worktree or gameplay
evidence was removed. Source and small receipts are handed to the user's
checkpoint owner for the consistent main-checkout commit; no separate Git
mutation was performed by the layout task.
