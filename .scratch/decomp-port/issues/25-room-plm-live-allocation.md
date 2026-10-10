# Native room and hardcoded PLM allocation in live gameplay
Status: resolved
Blocked by: 13, 24
Triage: ready-for-agent

Implement the user's selected next frontier: wire SpawnRoomPLM and
SpawnHardcodedPlm into Rust allocation/setup and exercise them in live replay.
Preserve highest-free-slot search, distinct preinstructions, byte coordinates,
room arguments, bank8F ROM entries and the special WRAM entry at 0x12.
Unsupported setup remains explicitly measured C ownership; supported native
errors fail without hidden retry or partial writes. Do not claim whole PLM
frame-engine/rendering ownership.

Use the approved full-WRAM actual-C allocator and independent public Rust seams
with TDD. Verify full-pool behavior, WRAM/ROM entries and setup effects offline;
require native room/hardcoded setup hits in four independent C/mixed live
processes with matching state, returns and ordered effects. Capture live entry
inventory to select additional setup closure only if needed. Preserve evidence,
source/dependency seals and fault controls. Complete docs/agents/run-cleanup.md
before resolution.

## Completion evidence

Implemented native `spawn_room` and `spawn_hardcoded`, bank8F/WRAM0x12 entry
reading, coordinate byte multiplication/truncation, distinct preinstructions,
room arguments, transactional setup and highest-free-slot behavior. Native
sound-enable setup0xb7c3 closes the actually observed boot header0xb7eb.
Original C allocator definitions are renamed in sealed temporary source copies
so same-bank calls reach the measured adapter; unsupported headers remain
explicit C ownership. Errors never trigger C retry.

Actual pinned full-tree C evidence:
`out/port/room-plm-final/20261010T223633.137790Z/verdict.json`,1,969cases:
673terrain,648room,648hardcoded; full WRAM/return/displacement comparisons,
fiveABI rejection controls and zero CPU fallback. Eight independent Rust PLM
examples pass; complete port tests, formatting and Clippy pass. Existing PLM
wrong-amount/tamper tests pass; seven live ABI/tooling tests pass (one unrelated
optional expensive live fault control skipped), five batch tests pass.

Final current-tooling live evidence:
`out/port/room-plm-current-live/20261010T224032.887967Z/verdict.json`:
four independent9,000-frame C/mixed processes match full state, returns and
ordered hardware/allocation effects. Each mixed process executes one native
hardcoded allocation/setup, zero C hardcoded calls,23 Rust decompressions and
8,783 Rust RNG calls, with zero measured CPU fallback. Earlier18,000-frame
comparison `out/port/room-plm-live/20261010T223747.065046Z/verdict.json` also
matches,37 Rust decompressions and17,594 RNG calls. Final short run corrects
a verdict-counter variable collision and verifies extracted host generator.

Room and terrain allocation have zero calls in both prefixes and remain offline;
this is live hardcoded allocation admission, not whole-room/PLM-engine ownership
or original-ROM parity. No state repair or hidden C retry is used.

Rejected evidence retained under `out/port/room-plm-offline` and
`out/port/room-plm-live`: malformed displaced scroll partner fixture, collision
quicksand without displacement context, missing host argument declaration,
zero-call native gate and stale shared dependency seal. Corrected input bounds
and actual observed setup closure precede final acceptance.

Cleanup completed for room-plm-offline/final/system/live/current-live using
`tools/port_cleanup.py --root <scope> --apply` and `--deduplicate --apply`.
Measured reclamation5,895,655,698bytes (~5.49GiB); live target/archive cleanup
also ran automatically. Required artifacts and diagnostic rejects are retained.
No implementer worktrees were created; all final evidence uses canonical source
paths. Remaining disk space approximately40GB. Final allocator seal is
reauthenticated after cleanup before resolving this ticket.
