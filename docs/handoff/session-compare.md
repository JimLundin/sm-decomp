# Comparing in Session space (end state step 1)

Jim's end state (REVIEW, 2026-11-04), step 1: verification decodes the oracle
emulator's RAM into a Session and compares it with the engine's Session (D6:
RAM translation belongs to verification). After that, the engine no longer
has to export its Session as RAM for comparison. This note maps what is
compared today, gives the design, and records the first slice (lane
`collab/session-compare`, on `0b19d8a24`) and the second (lane
`collab/session-compare-2`, on `8f90bd4bf`: next slice 1's findings and fix,
and slice 2's coverage report).

## Today's comparison

Each paired frame (`run_one_frame_of_game_both`, `sm/src/cpu_infra.rs`) runs
the oracle frame, snapshots it (`g_snapshot_theirs`), restores the frame's
start, runs the engine's Tick, and snapshots that (`g_snapshot_mine`).

| Check | Compares | Decides |
| --- | --- | --- |
| Lockstep RAM (`verify_snapshots_eq`) | all 128 KiB of WRAM byte by byte after `normalize_snapshot_ram`; SRAM; VRAM; OAM and high OAM; CGRAM; about 20 PPU register groups (INIDISP, windows, scroll, Mode 7, ports, latches, sprite state, counters, open bus); the brightness caches; the rendered 256x240 frame when `VERIFY_ACTIVE`; the physical APU state when the oracle's APU runs | the frame's pass or fail (`g_fail`) |
| APU queue (after `verify_snapshots_eq`) | the four queued APU ports, with one probed startup-reset exception; the physical queue when the oracle's APU runs | the frame |
| Observation (`observation::observe`, `sm_tools::lockstep`) | typed domains (Samus, Camera, Room, Progress, Actors, Effects): the engine's from its Session (`sm_tools::engine::project` of the retained Session or the Tick's last lease), the oracle's from its RAM through `ram_map` (`observation/oracle.rs`), by hash, with a field diff at the first divergence | nothing in lockstep (report and RAM agreement counts); the free run's verdict |
| Lock digests (`check-lock`, `observation/machine.rs`) | one engine-only run against the faithful oracle's recorded lock: a running digest per machine stream (`wram` with the normalized ranges and `$00C2-$00C3` zeroed, `sram`, `vram`, `cgram`, `oam`, `ppu`, `video`, `apu`) and per typed domain, a combined digest of the machine streams, and a mark every 1,000 Ticks; `compare_lock` compares the streams both entries have | the gate's lock rows; a mismatch reruns its first span in faithful lockstep to localise it |
| Video and native lines | `SM_NATIVE_LINES` (nightly room jobs): each natively drawn line against the emulated PPU's; `SM_CROSS_RENDER=30`: sampled lines against LakeSnes's per-pixel selection; `SM_PPU_MODEL_CHECK`, `SM_HDMA_TABLE`, `SM_IRQ_TABLE`: the native PPU, HDMA and IRQ models against what the beam ran | the room job (any logged line fails it) |

**Where the RAM image comes from.** Ours: `make_snapshot_with_player_owner`
copies the live `WorkRam` (`ram.copy_out`), which reading settles. It holds the
Session as the leases exported it (`encode_spans` at each release, or a frame
lease's deferred export), plus whatever `Game`-side code stored after the last
lease: the NMI, sm/ legacy routines, `foreign`, `export_native_spawn`. The
oracle's: the emulated 65816 running the ROM (patched or faithful,
`SM_ORACLE`) in the same `WorkRam`, snapshotted before the engine's Tick
restores it.

**Normalization.** `NORMALIZED_RAM` (31 ranges, `cpu_infra.rs`; table and
reasons in `docs/LEGACY.md`, `LEGACY(verifier)`) copies one side's bytes over
the other's before the byte compare, and the same ranges are copied in the
persistent Session payloads (`normalize_from`). They are three kinds: RAM the
engine does not model (direct-page scratch `$00-$50`, the stack
`$1F5B-$1FFF`, multiply products, Samus tile temporaries, door VRAM update
words: "theirs"); temporaries the engine keeps but leaves at other values
(collision, enemy loop, scroll and drawing scratch: "mine"); and snesrev's
own bookkeeping in game RAM (`$0611-$0616` coroutines, `$077E-$0782`
counters: "mine"). The lock's `wram` stream zeroes the same ranges and
`$00C2-$00C3` (snesrev's patch selector, `ORACLE_RAM`).

## Design

**Decoder.** Verification owns the oracle RAM to Session decoder. It starts as
the engine's `session_encoding::decode` (with `fx_encoding`, `actors::codec`
and `menu_window_encoding`'s decode halves), called from
`sm/src/verification/observation/session.rs`. While leases still import RAM
the engine needs that code too, so the slice calls it in place. The decode
half then moves into sm/ verification (copy, prove equal on every compared
frame, then the engine stops calling it), and the encode half is deleted with
the export (step 2).

**Comparison.** `sm_tools::session_compare` compares the two Sessions field by
field. It destructures `Session` exhaustively, so a new field must be listed
or excluded. At the first divergence it names the path
(`room.terrain.cells[812].level`, read from the two debug texts: field names
and indexes only, never values) and counts divergent Ticks per top-level
field, plus Session/RAM agreement as the observation compare does.
`world` is excluded: it is the content binding, not RAM state. Field paths
localise better than byte addresses, and a field is the owner a fix goes to.

**RAM the Session does not own.** It stops being compared. Scratch the
original leaves (direct page, collision and enemy temporaries, multiply
products), the stack and snesrev's bookkeeping are not game state: no later
behaviour depends on them except through state the Session owns or through
the outputs (video, audio ports, SRAM), and all of those stay compared. A
wrong scratch value that matters shows up in an owned field or an output
one or more Ticks later, so the check against the real game is not weakened;
localisation moves to that Tick. That is already true for the 31 normalized
ranges today, and each is a listed blind spot. The verifier keeps, as its own
table, the RAM ranges no Session field decodes from, with the reason for
each. It replaces `NORMALIZED_RAM` and its `LEGACY(verifier)` rows, and the
decoded ranges plus that table must cover all 128 KiB, so no byte goes
unaccounted for. Oracle-only facts (probes, expectations, required hits) keep
reading oracle RAM through `ram_map`, as facts about the real game, not as an
engine comparison. Differences from hardware stay fixed or listed in
`docs/LEGACY.md`.

**Locks.** Lock entries gain Session streams beside the machine ones: one
digest per top-level Session field group, over a canonical byte form of the
decoded Session (the checkpoint codec's, not RAM). `compare_lock` already
compares only the streams both entries have, and the combined digest folds
only machine streams, so new streams invalidate nothing. The rotating faithful
oracle sample (and one deliberate full record) re-records entries with them.
Once every entry carries them, one change bumps `trace::FORMAT`, drops `wram`
from the combined digest and the stream list, and re-records the corpus once
(the oracle alone runs at about 150-220 Ticks/s). The check-lock localisation
already prints the Session line (below), so both are seen side by side first.

**Exit.** The engine stops exporting RAM when (1) Session compare plus the
non-RAM machine streams decide lockstep and the locks, and the WRAM byte
compare is gone; (2) the engine's Session is compared directly, without the
slice's encode round trip, using a declared list of fields RAM cannot carry;
(3) the decoder and the ~20 verification observers that call
`encode`/`decode` use the verification copy; and (4) the runtime owns the
tick (step 2), so no lease imports from or exports to RAM. Then
`session_encoding`, `fx_encoding`, `actors::codec`, `menu_window_encoding`
and `Field::address`/`Decode` leave sm_game and `engine_ram_export_lines`
reaches 0.

## First slice

Lockstep now runs the Session compare beside the RAM compare on every paired
frame, report-only. It never sets `g_fail` and never changes a frame's
result; it only adds console lines.

- `Leases` keeps a clone of each Tick's last lease Session
  (`keep_sessions`, `last_session`); the retained Session is used when it is
  current, as the observation compare does.
- `observation::observe_session`, called just after `observe` and before RAM
  normalization: the oracle's Session is `decode` of `g_snapshot_theirs.ram`.
  The engine's is its Session encoded onto its own frame result (which supplies
  the bytes the Session does not own), with every `NORMALIZED_RAM` range taken
  from the oracle, then decoded. Both sides therefore carry exactly what the
  codec carries, with the RAM compare's blind spots. A decode the codec rejects
  (a panic, caught with the hook silenced) counts as undecodable.
- Console: `@<tick>: Session divergence in <path>; also differs: <fields>` once
  per replay, then at the end `Session compare: ...`,
  `Session divergent ticks by field: ...` and `Session/RAM agreement: ...`.
  `SM_VERIFY_SESSION_COMPARE=0` turns it off.
- `verify_dll.py`'s localisation (`LOCALIZED`) and `replay_job.py`'s audio
  filter pass the Session line, as they pass the observation line.

**Cost.** Per paired frame: one Session encode (about 1.5 ms on Kraid, Xeon
8573C), two decodes (about 0.25 ms each), one Session clone per lease release
and a `PartialEq` walk, so about 2-4 ms against a 65-90 ms lockstep Tick
(3-6%). The pretty debug text is built only at the first divergence. The
summary line reports the measured seconds, so CI shows the real cost.

**Where CI shows it.** Every lockstep row: Verify scope `gate`'s lockstep
rows (those the lock cannot hold) and its check-lock localisation reruns,
scope `full` (the whole corpus and both route jobs) and dispatched scope
`rows`, with either oracle. Engine-only check-lock and free-run rows have no
oracle RAM in process and do not run it.

**Measure.** `engine_ram_export_lines` in `tools/ram_ratchet.py`: 4,685 on
`0b19d8a24` (session_encoding 1,997, fx_encoding 2,028, actors/codec 477,
menu_window_encoding 79, projection.rs's `Field::address` and `Decode` 104).
Advisory until it first falls, then strict. `ram.rs` (the name list the codec
addresses, about 810 lines) and the 16 other `fn address` helpers are not
counted; they go with the codec.

## Slice 1: what the first report showed

The first report (#221's lockstep rows, 52 replays) had 0 RAM-only failures
and Session-only failures on about 30-100% of each replay's Ticks, all in two
field groups: `control[0].input.ports[0]` (`.sampled` first on 28 replays,
`.repeat_counter` 14, `.pressed` 9) and `queued_tilemaps` (`.tail` 7,
`.queue[7]` 3), with `room` (2 replays) and `presentation` (6) on 1-4 Ticks.

**Cause: timing, every class.** RAM passed on every one of those Ticks, and
the engine side was its Session encoded onto its own passing RAM, so the
Sessions could differ only where the encoded Session differed from the
engine's own frame-end RAM. They did, because the Session was the Tick's
*last lease* and Game-side code finishes the frame after it
(`gameplay_frame::frame`, on `Game`, not on a lease):

- `display_refresh::run` is the NMI. Its `flush_tilemaps`
  (`sm/src/game/display_transfers.rs`) stores the queue's end word at the
  byte tail (`queue[tail..tail + 2] = 0`: `queue[7]` after one 7-byte entry)
  and clears `tail`. The lease Session holds the pre-flush queue.
- `controller_input::sample_latched` then latches the joypad: it writes
  `sampled`, `pressed`, `repeated`, `previous` and `repeat_counter`.
  `sampled` differs whenever the buttons change between frames,
  `repeat_counter` while one is held, `pressed` on an edge.
- `room` and `presentation` on a few Ticks: the same mechanism for
  Game-side room-entry and display writes after the lease (expected to show on
  the new "written after the last lease" line, below, and not as divergence).

None is a codec asymmetry, a field RAM does not carry, or a real divergence:
the oracle holds the same values, one frame-end later in the engine's
order. No `docs/LEGACY.md` row is needed.

**Fix (comparison only).** The engine's frame-end Session is the Session its
next lease takes up, in the lease's own order (`Runtime::new`): the
retained Session while it is current with RAM (`observed_session`); else the
kept frame Session with the RAM ranges written since its export, which
`reuse_frame_session` takes from RAM (`kept_session_writes` peeks the write
log without closing it; `logged_writes`); else a decode of the engine's RAM
(an import). So the comparison reads the engine at the oracle's snapshot
point. The last lease Session is still compared, against the frame-end
Session, and its differing fields go on their own summary line:

```
Session written after the last lease: <n> ticks; first write at tick <t> in <path>; by field: control <n>, queued_tilemaps <n>
```

That line is the list of Game-side frame-end writers that the runtime's
Tick (step 2) must take over before slice 4 can compare the engine's
Session directly. The `Session compare:` line also counts the engine
Session's source (`retained`, `kept`, `imported`); `without an engine
Session` is gone (a Tick with no lease is an import).

No stale-slot normalization was added: `queue[7]` is the NMI's end word, not
a stale slot, and the bytes past the tail are real RAM the engine reproduces
(the RAM compare passes them). Whether a direct compare (slice 4) should
compare the queue only up to its tail is decided there, when the queue is no
longer read from RAM.

**What remains possible.** With the aligned engine image, a Session-only
failure can now only be an encoded byte that differs from the engine's own
RAM outside the write log: an export the codec does not reproduce (a codec
asymmetry), or a Game-side write the log missed. Both are worth a row.

## Slice 2: coverage (started)

Each frame's encode of the engine's lease Session marks the bytes its spans
write; at the replay's end `session_compare::Coverage` classes all 128 KiB
against the unowned table, which today is `NORMALIZED_RAM`:

```
Session RAM coverage: <n> bytes decoded, <n> unowned, <n> both, <n> neither (in <n> ranges)
Session RAM neither decoded nor unowned: $xxxxx-$xxxxx, ... (first 12)
Session RAM decoded and unowned: $xxxxx-$xxxxx, ... (first 12)
```

"Decoded" is the encoder's spans (the decode reads the same layout);
"both" are Session bytes the RAM compare normalizes, so neither compare
sees them today; "neither" are bytes with no field and no listed reason,
which must reach 0. Report-only. The union over a replay is the measure,
because spans depend on state (room size, actor slots).

## Next slices

1. Done (above). Confirm on dispatched rows: Session-only failures 0 and
   the fields on the "written after the last lease" line.
2. Read the coverage lines over the corpus and both routes. Give each
   "neither" range a field or a row in a new unowned table (with its
   reason), and decide each "both" range: owned (drop it from the table) or
   unowned (the encoder stops writing it, or the comparison ignores it). The
   unowned table then replaces `NORMALIZED_RAM` in the coverage check.
3. Copy the decode half into `sm/src/verification`, compare it with the
   engine's decode on every frame, then use it for the oracle side.
4. Compare the engine's Session directly, with a declared list of fields RAM
   cannot carry, and drop the encode round trip. It needs the frame-end
   Session as an object: until the runtime owns the NMI flush and the
   controller latch (step 2), the frame-end Session is the next lease's take
   on RAM, so the direct compare either waits for step 2 or applies the
   logged writes to the kept Session itself (`take_written_bytes`).
5. Add the Session streams to the lock; re-record through the oracle sample.
   The streams digest the frame-end Session (as above), not the last lease.
6. Switch lockstep and locks to Session space: drop the WRAM byte compare and
   `wram` (one `FORMAT` bump, one re-record), replace `NORMALIZED_RAM` with the
   unowned table, and retire its `LEGACY(verifier)` rows. Needs slice 2's
   "neither" at 0 and the "both" ranges decided.
7. With step 2 (the runtime owns the tick), delete the export and import, and
   move the observers' `encode`/`decode` calls to verification:
   `engine_ram_export_lines` reaches 0. The "written after the last lease"
   line reaches 0 ticks when the NMI and controller latch run inside the
   Tick, which is step 2's own check.
