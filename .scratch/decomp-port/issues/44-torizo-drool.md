# Bomb Torizo low-health drool allocation and random branch
Status: resolved
Blocked by: 36
Triage: ready-for-agent

Translate A5D3/A65D initializers and A456 probability branch. Preserve RNG call
order, enemy facing/flags, sine velocities and ROM script selection. Verify actual
C allocation/instruction/frame seams and finish run cleanup.

## Completion evidence

Implemented two drool initializers with two RNG calls each, facing/4000 flags,
ROM-selected script and sine velocities. A456 branch preserves one RNG call and
bank86 continuation through instruction/full-frame dispatch.
384 actual pinned C full-WRAM/raw-return cases pass (128 allocation, 128 branch,
128 frame) in `out/port/torizo-drool/20261011T035534.702582Z/verdict.json`.
Independent Rust test verifies exact one-call RNG state for both branch outcomes.
Full port suite, fmt and strict Clippy pass. Offline fixture equivalence only;
further Torizo behavior and live ownership remain open.

Cleanup: scoped `port_cleanup.py --apply` reclaimed zero bytes;
`--deduplicate --apply` reclaimed 58,720,256 snapshot bytes. No implementer
worktree created. Retain script ROM `out/port/torizo-drool-fixtures/drool.sfc`
and immutable source/run evidence; final verdict authenticated after cleanup.
35 GiB free.
