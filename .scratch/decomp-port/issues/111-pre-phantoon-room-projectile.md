# Pre-Phantoon room projectile
Status: resolved
Blocked by: None
Triage: ready-for-agent

Translate RoomCode_8FC8C8 and its native A3A3 projectile initializer/preinstruction.
Preserve room allocation and BG2 scroll clearing. Verify actual-C full WRAM,
tests, cleanup and authentication. Live ownership remains separate.

## Completion evidence

Room callback and A3A3 initializer/preinstruction now Rust. Validation recognizes
the initializer as independent of PLM context. All 57 actual-C full-WRAM room
allocation cases pass across all 18 free slots/full table and scroll values:
out/port/pre-phantoon-room-projectile/20261011T100054.468745Z/verdict.json.
Independent test confirms BG2 scroll clearing and preserved projectile position.
Full port suite and strict Clippy pass. Cleanup ordinary zero; dedup reclaimed
12189696 bytes; all 57 cases authenticate afterward. Retained binary/source/
pinned ROM dependencies; no implementer worktree created. Offline allocation
scope only; direct preinstruction and live ownership remain separate.
Integration branch decomp-port-integration.

Free disk after cleanup: 268641054720 bytes.
