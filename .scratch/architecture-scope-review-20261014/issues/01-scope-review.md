# Review crate and module purposes

Status: resolved
Blocked by: None
Triage: ready-for-agent
Type: research

## Question

Which existing crate and module seams concentrate domain knowledge and which
spread one responsibility across callers or expose implementation details?

## Scope

Read-only inspection of all workspace crates, relevant tests and architecture
documentation. Use Cargo metadata to validate dependency direction; trace actual
callers before recommending a change. Do not create speculative interfaces or
equate crate count or source length with architectural friction.

## Comments

- The HTML report will be written only to the OS temporary directory.
- Existing dirty source and external session artifacts remain owned by their
  original sessions. No gameplay claim, verification run or worktree is needed.

## Answer

All 14 workspace crates have an identifiable purpose. The strongest changes
concentrate ownership within the existing pipeline and gameplay modules:

1. Strong: Offline family verification owns its Fixture execution, adapter
   recipes and source/evidence contract. Currently every pipeline path selects
   all families, and main.rs is sealed into retained family admission
   (affected.rs:237/550, inputs.rs:376, family.rs:1321/1332).
2. Strong: complete each installed execution adapter's ownership of preparation,
   acceptance, receipt authentication and retention. Preserve the existing
   Supervisor and active Original-acceptance work (translation_execution.rs:838,
   875, 891, 948, 991, 1487).
3. Worth exploring: give fresh and retained Live replay observations common
   projection/admission ownership while preserving distinct claims
   (live.rs:1241, observations.rs:141/185/203/206).
4. Worth exploring: concentrate accepted Translation attempt publication and
   recovery, including the fenced source/assignment/ticket/progress sequence
   (assignment.rs:1384, translation_migration.rs:2889/3127/3542).
5. Worth exploring: finish the existing scheduled gameplay module's ownership
   of WRAM publication and host-effect finality; preserve actual-C Fixture
   adapters, aliases and native admission (game_host.rs:119/148,
   samus_projectile_frame.rs:198/200/413, room_scroll.rs:270/293).

The report contains before/after visuals, existing behavioral tests, deletion
tests, dependency categories, source references and the complete crate map.
Start with candidate 01; independently audit verifier dependencies before
narrowing Affected Offline family selection. Directory moves alone do not
establish that independence.

Retain the deep build cache, Fixture scheduler, evidence store, Supervisor,
instruction/hardware seams and independent bootstrap package. The architecture
guide lists 8 of 14 crates and README lists 9; update the written map when the
structure decision is made. No concrete new interfaces were designed.

## Completion evidence

- Report: `/tmp/architecture-review-20261014T210732Z.html`.
- Full-page visual check: `/tmp/architecture-review-20261014T210732Z-full.png`.
- Source branch: `main`; initial HEAD
  `108bee396b8afa2ad20f0981b25fff2091ccacc9`, later inspected HEAD
  `177778561b5f0d07c397db486ea173e8dd743a5b`, with concurrent uncommitted work.
  The later changes to inputs.rs preserved the source-identity issue; ongoing
  Native receipt/confinement changes remain owned by their existing session.
- `cargo metadata --locked --offline --no-deps --format-version 1` confirmed
  14 workspace crates, 19 ordinary project dependencies, two build dependencies
  and one redundant development dependency.
- Browser inspection rendered all four Mermaid graphs independently with node
  counts 5, 5, 4 and 14; no parse error or empty graph remained. The final
  full-page screenshot shows all five candidate cards and the complete crate
  map. All 42 distinct local source links resolve.
- `xdg-open` was absent and `gio open` had no registered HTML handler.
  The installed Chromium executable successfully opened the report instead;
  `xwininfo -root -tree` confirmed the report-titled window.
- Source and existing tests were inspected; no gameplay, performance or project
  verification gates were run. No source changes, queue mutations, verification
  batches, build output or implementation worktrees were created by this review.

### Cleanup

All headless browser checks exited before removal. Browser artifacts are not a
supported pipeline run format; only identified unsealed inspection outputs were
removed. The exact cleanup was:

```sh
rm -r -- /tmp/architecture-review-20261014T210732Z-browser \
  /tmp/architecture-review-20261014T210732Z-rendered.html \
  /tmp/architecture-review-20261014T210732Z-browser.log \
  /tmp/architecture-review-20261014T210732Z-map.png \
  /tmp/architecture-review-20261014T210732Z-desktop.png \
  /tmp/architecture-review-20261014T210732Z-map-output.log
```

Measured removal: 5,452,739 apparent bytes (`du -sb`).
Remaining space after cleanup: 162,714,337,280 bytes (`df -B1`).
The report and final full-page screenshot remain. The user-facing Chromium
window remains intentionally open (observed PID 587821) with its separate
`/tmp/architecture-review-20261014T210732Z-viewer` profile; it is excluded from
finished-check cleanup.

Retained artifact hashes:

```text
f068e29f9b201941e3e5b0b1b13069cc5f8a4534234e0fe5155f45818e4dda96  /tmp/architecture-review-20261014T210732Z.html
c7c5e4cf9dd15237d1503ad92f815f4275571d579d12a6cb022cc347296e2775  /tmp/architecture-review-20261014T210732Z-full.png
```

No implementation/integration commit was needed for this read-only research.
The requested report stage is complete; the skill's candidate-selection step
precedes interface design or implementation.
