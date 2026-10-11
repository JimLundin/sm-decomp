# Lava seahorse fireball allocation and arc closure
Status: resolved
Blocked by: 42
Triage: ready-for-agent

Translate B4EF spawn and verify existing B535 arc through allocation and frames.
Preserve facing aliases, fractional motion, apex scripts and camera deletion.
Verify actual pinned C and finish scoped cleanup.

## Completion evidence

Implemented facing-dependent B4EF spawn and connected it to existing B535
full fixed-point arc, apex scripts and camera-relative deletion. 241 actual pinned
C full-WRAM/raw-return cases pass (48 allocation, 96 preinstruction, 96 frame,
one sleep control) in
`out/port/lava-seahorse-arc/20261011T040654.552808Z/verdict.json`.
Full port suite, fmt and strict Clippy pass. Offline fixture equivalence only;
live execution remains open.

Cleanup: scoped `port_cleanup.py --apply` reclaimed zero bytes;
`--deduplicate --apply` reclaimed 73,531,392 snapshot bytes. No implementer
worktree created. Immutable ROM/source/run evidence retained; final verdict
authenticated after cleanup. 35 GiB free.
