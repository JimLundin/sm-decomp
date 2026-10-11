# Golden Torizo egg lifecycle
Status: resolved
Blocked by: 46
Triage: ready-for-agent

Translate B001 spawn, B043/B0B9/B0DD motion and B13E branch. Preserve fixed
timer quirk, RNG order, facing reflection, underflow and staged impact scripts.
Verify pinned actual C allocation/instruction/frame seams and clean runs.

## Completion evidence

Implemented egg table/RNG initialization with fixed F=66, timed bouncing/facing
reflection, timer-underflow launch, staged wall/floor transitions and B13E branch.
436 actual pinned C full-WRAM/raw-return cases pass (48 allocation, 192
preinstruction, 192 frame, four branch) in
`out/port/torizo-eggs/20261011T040150.996458Z/verdict.json`.
Independent Rust test proves timer-underflow launch skips movement. Full port
suite, fmt and strict Clippy pass. Offline fixture equivalence only; live open.

Cleanup: scoped `port_cleanup.py --apply` reclaimed zero bytes;
`--deduplicate --apply` reclaimed 137,494,528 snapshot bytes. No implementer
worktree created. Retain ROM `out/port/torizo-egg-fixtures/egg.sfc` and immutable
source/run evidence; final verdict authenticated after cleanup. 35 GiB free.
