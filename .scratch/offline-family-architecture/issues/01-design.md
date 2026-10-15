# Design the Offline family module

Status: resolved
Blocked by: None
Triage: ready-for-agent
Type: grilling

## Objective

Reach a shared design for candidate 01, using current source facts and the
user's answers. See [design tree](../design.md) and [spec](../spec.md).

## Comments

- 2026-10-15: user selected candidate 01. Round 1 asks about the deliverable and
  packaging. A read-only explorer is checking exact current dependencies and
  admission semantics. Current-only evidence policy is settled already.
- No project tests/builds, verification runs, source mutations or worktrees
  are needed for this design round. There are no run artifacts to clean up.
- Source refresh completed at 7702289. CLI/family independence is achievable
  inside pipeline; both ordinary and native verifier inventories require the
  ownership change. Exact dependency facts and proposed proof are in design.md.
- Round 1 answered: prioritize structural reorganization; defer the CLI
  independence proof. Choose the packaging with the clearest separation using
  engineering judgment. Three design alternatives are being compared.
- Three read-only design agents have returned. The [comparison and
  recommendation](../proposal.md) chooses `offline-verification`,
  `tool-execution` and `decomp-source`, a simple ordinary call and an opaque
  prepared Native handle. [Implementation and verification](../implementation.md)
  specify exact source groups, caller migration, current behavior and cleanup.
- Recorded the existing Recovered C source concept in `GLOSSARY.md` as required
  by the selected architecture/domain-modeling skills. No implementation or
  production state has changed.
- The remaining checkpoint is the grilling skill's explicit final confirmation
  of shared understanding. User-delegated packaging is no longer an open branch.

## Review evidence and cleanup

- All three agents performed read-only design work and reported no builds,
  tests, verification runs or queue mutations.
- Reviewed the current Rust source, Cargo topology, behavioral test inventory
  and ignored Native proof requirements. This is design validation, not a claim
  that proposed code compiles or verification passes.
- Documentation review: `git diff --check` passed; scratch documents have no
  trailing whitespace or unresolved TODOs. Cross-references point to the retained
  design, proposal, implementation plan and review ticket. HEAD remains `7702289`;
  the only tracked change is the glossary definition.
- No run artifacts, worktrees or disposable build output were created. Run
  cleanup is not applicable to this documentation work; bytes reclaimed: 0.
- `df -Pk /home/jim/sm-decomp` reported 144,470,992 KiB available (137.78 GiB)
  during the design review on 2026-10-15.
- The original architecture HTML report and its completed browser-check cleanup
  remain recorded in `../../architecture-scope-review-20261014/issues/01-scope-review.md`.

## Completion evidence

- The user approved the rendered architecture and instructed implementation:
  "i approve. go ahead" on 2026-10-15. The shared-design checkpoint is complete.
- Browser diagrams: `/tmp/offline-architecture-20261015T053033Z.html` with embedded
  SVG and no network dependency; PNG: `/tmp/offline-architecture-20261015T053033Z.png`.
- A headless Chromium render was visually inspected. The user-facing Chromium
  window titled `Offline verification architecture — sm-decomp` was observed by
  `xwininfo` at window `0x600003`; its session 75668 later exited normally.
- HTML SHA-256: `f794fdeb81be6bb147e3a52899895c38a67bfd919517bfd82a6901ac3cc8c293`.
  PNG SHA-256: `c2435812139c1b2a7de3c156753cbd17c05e6be125dbcf8dda9f9b2d0e104e98`.
- After browser writers stopped, removed only the identified check/viewer
  profiles and logs with `rm -r -- /tmp/offline-architecture-20261015T053033Z-check
  /tmp/offline-architecture-20261015T053033Z-check.log` and the corresponding
  `-viewer`/`-viewer.log` paths. Browser profiles are not pipeline run artifacts.
  Measured reclamation: 4,028,059 + 4,407,872 = 8,435,931 apparent bytes.
- Remaining measured space: 140,602,352 KiB. The HTML and PNG remain; no browser
  profile, verification run, build output or implementation worktree is needed
  to view them. Design work changed no Rust source or production state.
