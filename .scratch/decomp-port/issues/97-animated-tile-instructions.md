# Animated tile instructions
Status: claimed
Blocked by: None
Triage: ready-for-agent

Translate native delete, absolute/relative goto, decrement branches and byte
timer assignment. Verify full-WRAM and returns against actual C. Extend to
audio, event/boss, statue/palette and frame closure in subsequent work; no
unsupported frame instruction should silently fall back or be ignored.

## Work evidence

Six control instructions translated. All 720 actual-C full-WRAM/return cases
pass: out/port/animated-tile-instructions/20261011T085437.495991Z/verdict.json.
Independent tests verify byte-only assignment/high-byte preservation, timer
underflow, relative signed wrap and decrement stop without ROM access.
Two animated tile tests and strict all-target Clippy pass. Cleanup --apply zero
bytes; dedup and post-cleanup authentication recorded below. No worktree.
Allocation live run still active via session 32896 at
out/port/animated-tiles-live/20261011T085347.505558Z. First C subject passed
40000 inputs with zero CPU fallback. Instruction/frame live closure still open.

Finished offline dedup reclaimed 243007488 bytes; all 720 cases authenticate
post-cleanup. Ticket remains claimed for remaining instructions/frame closure.

Audio extension: QueueMusic and all three QueueSfx instructions now call native
Rust queues with exact delayed-8/max-6 semantics. 144 actual-C full-WRAM/return
cases pass at out/port/animated-tile-audio/20261011T085551.828485Z/verdict.json.
Animated tile tests and strict Clippy pass. Event/boss/statue/palette and frame
closure still pending. Earlier control oracle is now historical module-bound
coverage because audio additions changed animated_tiles.rs.

Audio oracle cleanup ordinary zero bytes, dedup reclaimed 54394880 bytes;
all 144 cases authenticate afterward. No worktree created. Allocation live
session 32896 remains active; final cleanup pending until it terminates.

Boss/event extension: six native condition/set/wait instructions translated
through existing Rust system semantics. All 648 actual-C full-WRAM/return cases
pass at out/port/animated-tile-events/20261011T085720.360416Z/verdict.json.
Covers zero/set bits, areas 0/3/7, event indices 1/7/63 and all six slots.
Animated tile tests and strict Clippy pass. Statue/palette instructions and
frame closure remain pending; preceding oracles remain historical module seals.

Boss/event cleanup --apply zero bytes; dedup reclaimed 224133120 bytes;
all 648 cases authenticate afterward. No implementer worktree created.
Allocation live session 32896 remains active; first C process has finished.

Statue/palette extension: busy conditional, set/clear state, clear-three current
palette colors and write-eight target colors implemented with exact odd-offset
word alignment. All 360 actual-C full-WRAM/return cases pass:
out/port/animated-tile-palette/20261011T085828.013203Z/verdict.json.
Animated tile tests and strict Clippy pass. Remaining instructions spawn statue
projectiles or palette effects; frame closure remains pending. Allocation live
session 32896 remains active, second C subject passed 80000 inputs with zero
CPU fallback. No live ownership claim for the added instructions yet.

Palette oracle cleanup ordinary zero bytes; dedup reclaimed 136839168 bytes.
All 360 cases authenticate afterward. No implementer worktree created.

Palette-effect allocation dependency translated, including all four actual
initializer dispatch paths: no-op, intro preinstruction, Norfair suit-dependent
script, Brinstar boss-state deletion. Animated tile SpawnPalfxObj now calls this
Rust allocator. All 288 actual-C allocator full-WRAM cases pass:
out/port/palette-effect-allocation/20261011T090022.412533Z/verdict.json.
Tests and strict Clippy pass. Direct animated-tile spawn instruction oracle,
statue projectile spawn and frame closure remain pending. No live palette-effect
ownership claimed. Existing allocation live job session 32896 remains active.

Palette allocator cleanup ordinary zero bytes; dedup reclaimed 70254592 bytes.
All 288 cases authenticate afterward; approximately 255 GiB free. No worktree.

Statue eye/soul spawn instructions now call existing Rust room projectile
allocator; direct three-spawn instruction fixture uses explicit operand bytes
and pinned real headers. Synthetic operand ROM is retained at
out/port/animated-spawn-operands.sfc and SHA256 sealed in manifest.
First small-family oracle link failed due full projectile dispatcher dependencies
at out/port/animated-tile-spawns/20261011T090222.453480Z; cleaned ordinary/dedup
zero bytes. Full-C oracle rerun active via session 56770; final result and cleanup
remain required. Tests and strict Clippy pass. Allocation replay session 32896
still active at mixed-0 frame 74000, zero CPU fallback.

All 36 direct spawn instruction actual-C full-WRAM/return cases pass:
out/port/animated-tile-spawns/20261011T090242.780018Z/verdict.json.
Cleanup ordinary zero; dedup reclaimed 13500416 bytes. Evidence authenticates
post-cleanup. Frame interpreter and current-source combined verification remain.

Frame interpreter implemented with all 24 native opcode mappings, exact
predecrement, stop/wait behavior, timed source/list writes and descending
scheduler. Explicit malformed-slot/unsupported-opcode/cycle errors. Frame C
oracle running via session 75003 under out/port/animated-tile-frames. Existing
allocation replay session 32896 remains active; poll before restarting.
Tests and strict Clippy pass. Final frame oracle, combined current-source
instruction verification, live admission and cleanup remain required.

Initial frame oracle rejected malformed fixture using header as script pointer:
out/port/animated-tile-frames/20261011T090413.884959Z. Cleanup ordinary/dedup
zero bytes; diagnostic retained. Corrected fixtures to actual header script
pointers 8293/82B1/82CF/82ED. Rerun session 94757 active, awaiting final result
and cleanup. No implementation change was needed for this fixture correction.

Corrected frame oracle passes all 192 actual-C cases:
out/port/animated-tile-frames/20261011T090520.929394Z/verdict.json.
Cleanup --apply zero bytes; dedup reclaimed 46137344 bytes; all cases authenticate.
Combined current-source 21 non-spawning instruction oracle running session 59918.
Independent interpreter control-to-frame/wait test added. Live frame admission
still pending. Allocation replay session 32896 remains active in mixed-1.

Combined current-source 21 non-spawning instructions: all 1872 actual-C
full-WRAM/return cases pass at
out/port/animated-tile-combined/20261011T090607.371012Z/verdict.json.
Cleanup ordinary zero bytes; dedup reclaimed 658374656 bytes; current oracle
authenticates. Live-compatible frame oracle uses full C-bank dependencies and
is running session 47836. Allocation final mixed replay passed 93000 inputs
with zero CPU fallback; original session 32896 remains active.

Live-compatible frame oracle passes 192 cases:
out/port/animated-tile-frame-live/20261011T090726.920804Z/verdict.json.
Cleanup --apply zero bytes; dedup reclaimed 46137344 bytes; authenticates.
Fresh full-movie four-process frame replay started via session 25580 under
out/port/animated-tile-frame-replay. Poll existing job before restarting.
Final replay result/cleanup/authentication remain required. No worktree created.

Corrected byte operands to read exactly one byte for timer, relative branch,
audio and boss-bit byte instructions. Prior word reads unnecessarily rejected
FFFF one-byte operands. Independent bank-end test passes (four animated tile
tests total); strict Clippy passes. Combined oracle refresh active session 4637.
Existing full-movie frame replay session 25580 remains frozen to its earlier
binary; post-change current-source evidence must be authenticated separately.
No live claim is made for changed bank-end behavior until verified.

Refreshed combined oracle passes 1872 cases:
out/port/animated-tile-combined/20261011T091045.853791Z/verdict.json.
Post-run ordinary cleanup zero bytes; dedup/authentication output recorded below.
Frame first C replay passed 94000 inputs with zero CPU fallback; job active.

Refreshed combined dedup reclaimed 658374656 bytes; all 1872 cases authenticate
post-cleanup. No implementer worktree created. Remaining live/frame cleanup pending.

Current-source frame oracle refresh running via session 34860 after exact-byte
operand correction. Existing full-movie replay remains session 25580, first C
subject at 120000 inputs with zero CPU fallback. Do not restart existing job.

Current-source frame oracle passes 192 cases:
out/port/animated-tile-frame-live/20261011T091219.904361Z/verdict.json.
Cleanup ordinary zero bytes; dedup 46137344 bytes; post-cleanup authentication
passes. First movie C subject completed with 95385 handler calls and zero direct
ProcessAnimtilesObject calls. All eight retained handler entry samples have zero
active IDs; these samples do not establish internal interpreter exercise.
Live scope must distinguish handler ownership from active scripted execution.
Second C subject running, original session 25580 still active.

Added tools/port_animated_frame_fixtures.py: reproduces both corrected frame
manifests exactly against retained oracle manifests (192 cases each), preserving
historical case labels while deriving valid script pointers. No gameplay source
change. Existing replay session 25580 remains active in second C subject.

Full cargo test -p port --quiet passes on current tree. Frame replay second C
subject at 80000 inputs with zero CPU fallback. Run remains active; finished-run
cleanup awaits terminal status. No new verification artifact directories created
by the crate test beyond the project target directory.

Full frame replay finished successfully:
out/port/animated-tile-frame-replay/20261011T090833.758853Z/verdict.json.
Four fresh 129712-input processes match all 990 state/effect artifacts. Each
mixed process executes 95385 exclusive Rust AnimtilesHandler calls; direct
ProcessAnimtilesObject calls are zero. CPU fallback zero. Original-ROM parity
not tested. Retained entry samples contain no active animated tile IDs, so
active interpreter live execution remains unproved; ticket stays claimed.
The replay binary predates byte-operand corrections and newer palette additions;
current-source oracle seals remain separate from this frozen-binary evidence.

Finished replay cleanup --apply reclaimed zero bytes; --deduplicate --apply
reclaimed 927481150 bytes. Runner additionally shared 3506438144 bytes of ROM
fixtures and 1263285504 bytes of finished baseline observations. All 990 artifacts in all four
subjects plus retained binary authenticate after cleanup:
out/port/animated-tile-frame-replay/20261011T090833.758853Z/storage-authentication.json.
Free disk 271297265664 bytes at authentication. Retained source/ROM/predecessor
evidence dependencies remain in place; no implementer worktree created.
