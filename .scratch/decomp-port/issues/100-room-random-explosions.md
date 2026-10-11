# Room random explosions
Status: resolved
Blocked by: None
Triage: ready-for-agent

Translate RoomCode_GenRandomExplodes, Nonblank, 4th and ExplodeShakes through
existing Rust RNG, audio and sprite allocation. Preserve freeze/frame gates,
random-byte coordinates, native selector tables and earthquake flag.
Require actual-C full-WRAM equivalence and cleanup. Report movie ownership
separately; ticket 94's allocator live requirement remains independent.

## Completion evidence

All four native callers are Rust with exact selector/audio tables, byte-sized
multiply, promoted level-data indexing, freeze/cadence gates and shake behavior.
1024 actual-C full-WRAM cases pass on current source:
out/port/room-random-explosions/20261011T092841.944454Z/verdict.json.
Six room tests and strict Clippy pass. Independent test verifies frozen shake,
blank tile's one RNG draw and fourth-frame gate without requiring ROM access.
tools/port_room_explosion_fixtures.py reproduces oracle fixtures.

Cleanup ordinary zero bytes; dedup reclaimed 284688384 bytes; all 1024 cases
authenticate afterward. Free disk 269038538752 bytes. Retained sealed binaries,
pinned source and ROM dependencies; no implementer worktree created. Offline
coverage only; no movie caller or allocator ownership claim. Integration branch
decomp-port-integration.

After statue setup addition, refreshed current-source oracle passes all 1024
cases: out/port/room-random-explosions/20261011T093404.235705Z/verdict.json.
Cleanup ordinary zero; dedup 284688384 bytes; authentication passes afterward.
