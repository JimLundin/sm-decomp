# Room/door data loading and camera scroll state
Status: resolved
Blocked by: 02, 03
Triage: ready-for-agent

Own bank82 door/room/state header loading, bank8F known room-state selector bytecode, room music and bank80 parallax/background block/offset calculation plus automatic and triggered camera scroll constraints. Preserve original RAM overlays and byte/word truncation. ROM header/table spans, event/area arrays and camera scroll extents are executable admission guards.

Tilemap upload, door transition scrolling setup calling those uploads, IRQ/HDMA hardware scheduling and room coroutine lifecycle remain unowned dependencies; unknown operations fail explicitly. This ticket does not claim full door transition engine.

## Completion evidence
27 actual routines translated, complete non-hardware data/camera closure: door/room/state headers and destination CRE; known room state script interpreter and all six selectors; room music queues; layer2 parallax/manual modes; scroll register/block/offset state; four triggered camera directions and two automatic camera bounds. Original ROM pointers are guarded for correct bank/extents; tileset graphics0..28, event0..63, area0..7 and scroll-array indices0..255 admitted. Unknown selector or unowned hardware operation fails explicitly.

Canonical scroll fields deliberately preserve horizontal fractional word0x90f versus vertical0x913, signed block shifts, swapped room dimension boundary, closed scroll-cell pushback, 31-pixel vertical allowance and update ordering. State selectors preserve native system bitmask/index scratch writes. TileSet and RoomState 24-bit pointers copy three bytes into overlapping WRAM precisely.

Prior full actual compiled pinned C324-case evidence `/tmp/room-final/20261010T205534.557481Z/verdict.json` passed. Final stable328-case evidence `/tmp/room-stable-final/20261010T205556.344378Z/verdict.json` passed. Four extra cases cover both directions of ideal camera fractional reset. All source C82/80/8F and actual sm_rtl/sm_cpu_infra link closure compiled without fabricated stubs; full WRAM+return comparisons, no live claim and original-ROM parity not tested.

Five independent synthetic tests pass: fractional parallax/manual mode and signed blocks; closed-cell camera pushback; door negative distance fallback; morph+missile selector destination; horizontal fractional reset keeps vertical word. rustfmt --check passes. Worktree lacks ignored RustySNES dependencies, so tests compile actual port library and integration test with rustc; root Cargo integration required after public registration.

Admission rejected fixtures pointing at blank door entries and impossible negative-row scroll lookups before unchecked C; retained `/tmp/room-final/20261010T205343.683945Z/verdict.json`, `/tmp/room-final/20261010T205415.496051Z/verdict.json` and later guard rejections. Invalid fixture inputs were corrected to actual bounded ROM doors and camera row domains. Horizontal fractional offset corrected through source review and independent worked test.

Commit: this ticket's implementation commit in git history.
