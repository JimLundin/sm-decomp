# Issue tracker: Local Markdown

Issues and specs live as Markdown files in `.scratch/`.

## Files

- One feature per directory: `.scratch/<feature-slug>/`.
- Spec: `.scratch/<feature-slug>/spec.md`. A spec may point to an existing
  authoritative document instead of duplicating it.
- Tickets: `.scratch/<feature-slug>/issues/<NN>-<slug>.md`, numbered from
  `01` in dependency order, one file per ticket.
- Put `Status:` and `Blocked by:` near the top of each ticket.
- Append discussion under `## Comments`.

## Work status and dependencies

- New tickets use `Status: open`. Triage role strings are defined in
  `triage-labels.md` and recorded separately as `Triage:`.
- List blocker numbers in `Blocked by: NN, NN`, or use `Blocked by: None`.
- A ticket is ready when it is open and every blocker is resolved.
- Claim a ticket by setting `Status: claimed` before implementation.
- Resolve by appending implementation, verification and commit/evidence pointers
  under `## Completion evidence`, completing `run-cleanup.md` and recording its
  result there, then setting `Status: resolved`.
- Resolve a spec once every required ticket is resolved; append its integration
  branch and verification evidence. Local resolution requires no PR.
- Existing tickets with `Status: done` remain completed and satisfy blockers.

## Skill operations

When asked to publish, create the appropriate local file. When asked to fetch,
read the referenced file including its comments and completion evidence.

For wayfinding, use `.scratch/<effort>/map.md` with one child ticket per issue
file. Record each child's `Type:` as `research`, `prototype`, `grilling` or
`task`. Choose the first open, unblocked, unclaimed ticket by number. Resolve
research questions under `## Answer`, set `Status: resolved`, and append a
context pointer to the map's Decisions-so-far.
