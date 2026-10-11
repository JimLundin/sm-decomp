# Projectile fixed sound instructions
Status: resolved
Blocked by: None
Triage: ready-for-agent

Translate native fixed channel-two sound instructions EE8B/EE97/EEA3 and direct
wrappers. Preserve max-one audio semantics and unchanged instruction pointer.
Verify full-WRAM/return oracle, complete tests and scoped cleanup.

## Work evidence

Three opcodes and direct pointer wrappers translated. Full port tests and
strict Clippy pass. Initial 24-case oracle passes, cleaned zero ordinary and
8388608 dedup bytes and authenticated. Expanded 96-case oracle passes at
out/port/projectile-fixed-sounds/20261011T093135.247259Z/verdict.json;
cleanup ordinary zero, dedup 35127296 bytes. Its source seal became historical
when room.rs changed for ticket 102; authentication correctly rejected stale
dependency. Current-source refresh running session 98780; final cleanup required.
No live ownership claimed.

## Completion evidence

Current-source refresh passes all 96 actual-C full-WRAM/pointer-return cases:
out/port/projectile-fixed-sounds/20261011T093305.680449Z/verdict.json.
Covers both slots, bank-end pointer, disabled audio, max-one occupancy and
ring wrap. Full port tests and strict Clippy pass. Cleanup ordinary zero;
dedup reclaimed 35127296 bytes; all cases authenticate afterward. Free disk
269744803840 bytes. Retained sealed binary, pinned source and ROM dependencies;
no implementer worktree created. Offline coverage only. Integration branch
decomp-port-integration.
