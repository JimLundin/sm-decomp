# Torizo sonic boom spawning and flight
Status: resolved
Blocked by: 46
Triage: ready-for-agent

Translate AE15 initializer, AE6C flight and AF36 position reset. Preserve RNG,
facing scripts, signed acceleration, impact E/F snapshots and deletion range.
Verify pinned C allocation/instruction/frame seams and clean runs.

## Completion evidence

Implemented both sonic-boom definitions with RNG height/facing scripts, signed
acceleration/deletion, collision E/F snapshots and AF36 reset instruction.
196 actual pinned C full-WRAM/raw-return cases pass (64 allocation, 64
preinstruction, 64 frame and four reset) in
`out/port/torizo-sonic-boom/20261011T035953.583199Z/verdict.json`.
Independent Rust test verifies saved impact coordinates without acceleration.
Full port suite, fmt and strict Clippy pass. Offline fixture equivalence only;
live ownership remains open.

Cleanup: scoped `port_cleanup.py --apply` reclaimed zero bytes;
`--deduplicate --apply` reclaimed 61,341,696 snapshot bytes. No implementer
worktree created. Retain script ROM `out/port/torizo-sonic-fixtures/sonic.sfc`
and immutable source/run evidence; final verdict authenticated after cleanup.
35 GiB free.
