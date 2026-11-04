# Comparing in Session space (end state step 1)

Jim's end state (REVIEW, 2026-11-04), step 1: verification decodes the oracle
emulator's RAM into a Session and compares it with the engine's Session (D6:
RAM translation belongs to verification). After that, the engine no longer
has to export its Session as RAM for comparison. This note maps what is
compared today, gives the design, and records the first slice (lane
`collab/session-compare`, on `0b19d8a24`) and the second (lane
`collab/session-compare-2`, on `8f90bd4bf`: next slice 1's findings and fix,
and slice 2's coverage report), and the third (lane
`collab/session-compare-3`, on `ec858a96f`: slice 2's classification),
and the fourth (lane `collab/session-compare-3` again, on `5bdb96711`: the
last "neither" owners and the "both" decisions as data), and the fifth (lane
`collab/session-compare-4`, on `4c97f65ec`: the held bytes in the Session,
the "both" evidence line and the lock's Session streams).

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

**The NMI and the latch moved into the Tick (lane `tick-s10`, 2026-11-04;
not yet replayed).** Every frame now opens one lease and runs its state, the
NMI (`display_refresh::run`, with `flush_tilemaps`) and `sample_latched` on it
(session-authoritative-tick.md, stage 10). The two writers above are inside the
last lease, so the line should fall to `0 ticks` on every row whose only
post-lease writers were they: the chain rooms, pause and unpause, the `item-*`
boxes, `credits`, the SFX-loss rows and the power-on rows (the logo's frames
now end on a lease too). Where it would still show: settlement, which stays
after the lease. Its IRQ stores are frame display words, held outside the
Session in Legacy, so they should not show as fields. The IRQ door resume's
own `Runtime::new` opens after the tick and is that Tick's last lease (on
`door-transition`, `elevator-up` and `door-loading-lag-nmi`). Any field on the
line there names a settlement writer.

No stale-slot normalization was added: `queue[7]` is the NMI's end word, not
a stale slot, and the bytes past the tail are real RAM the engine reproduces
(the RAM compare passes them). Whether a direct compare (slice 4) should
compare the queue only up to its tail is decided there, when the queue is no
longer read from RAM.

**What remains possible.** With the aligned engine image, a Session-only
failure can now only be an encoded byte that differs from the engine's own
RAM outside the write log: an export the codec does not reproduce (a codec
asymmetry), or a Game-side write the log missed. Both are worth a row.

## Slice 2: coverage

Each frame's encode of the engine's lease Session marks the bytes its spans
write; at the replay's end `session_compare::Coverage` classes all 128 KiB
against the unowned ranges (`NORMALIZED_RAM` and the verifier's
`UNOWNED_RAM`) and the ranges held outside the Session:

```
Session RAM coverage: <n> bytes decoded, <n> unowned, <n> both, <n> held outside the Session, <n> neither (in <n> ranges)
Session RAM held outside the Session: frame display <n>, scroll <n>
Session RAM neither decoded nor unowned: $xxxxx-$xxxxx, ... (first 12)
Session RAM decoded and unowned: $xxxxx-$xxxxx, ... (first 12)
Session RAM unowned beside the normalized ranges: unused <n>, debug <n>, never read <n>, handshake <n>
```

"Decoded" is the encoder's spans (the decode reads the same layout);
"both" are Session bytes the RAM compare normalizes, so neither compare
sees them today; "held" are bytes an engine owner outside the Session keeps;
"neither" are bytes with no field, no holder and no listed reason, which must
reach 0. Report-only. The union over a replay is the measure, because spans
could depend on state; on #223's dispatched rows every replay printed the
same counts, and a decode of zeroed RAM (`session_encoding::decode`, then
`encode`, in a verify-only DLL loaded from Python) reproduces them exactly,
so today's spans are fixed.

### Classification (lane `collab/session-compare-3`)

Before: 120,959 decoded, 357 unowned, 107 both, 9,649 neither in 56 ranges
(#223's rows). After, by the same zero-RAM measure: 120,963 decoded, 3,926
unowned, 107 both, 258 held, 5,818 neither in 6 ranges. The 9,649:

| Class | Bytes | What |
| --- | --- | --- |
| (a) held outside the Session | 258 | the frame display (`display::preparation::State`: the register mirrors $51..86, the interrupt commands $A7/$AB, the NMI counters $05B4..BB, the HUD palette bits $077C, auto-cancel $09EC, previous HUD values $0A06..0F, the non-minimap HUD tilemap at $7E:C608..C6C7), 242; the BG scroll words $B1..C0 (`display_scroll`), 16. The host keeps both as typed domains (`SessionSlot::frame_display`, `scroll`) over RAM while the presentation is Legacy; an owned Display carries them in the Session (`display_bytes!`, `display_words!`), but the corpus runs Legacy. The coverage map counts them as held, by owner, from `game_state::frame_display_ranges` |
| (c) unowned | 3,569 | `observation::coverage::UNOWNED_RAM`, 45 ranges with the RAM map's reason each: unused 3,403 (e.g. $9B..A2, $C1..CF, $0B4E..55, $1E9D..AC, $7E:F438..FFFF), debug 4 ($0DF4..F7), never read 160 (the debug enemy set, $7E:D552..D5F1), handshake 2 ($0641, the APU upload's poll budget) |
| (b) now Session owners | 4 | `Player::health_warning` ($0A6A) and `Room::level_data_size` ($7F:0000), scalar words with layout lanes; SMPLAYER 9, SMROOMS 13 |
| (b) owned in the fourth slice | 5,818 | below |

The fourth slice gave the last 5,818 bytes Session owners (SMAUXGS 19, then
20). Each is a byte buffer of the projection, as the original packs it, with
byte layout tests (`queued_transfers_round_trip_every_byte`,
`queued_transfer_tails_round_trip`, `tilemap_buffers_round_trip_every_byte`):

| Range | Bytes | What | Owner |
| --- | --- | --- | --- |
| $02D0..032F, $0334 | 98 | the Mode 7 transfer queue and its tail | `QueuedTilemaps::mode7`, `mode7_tail` |
| $0340..035F, $0360 | 34 | the VRAM read table and its tail | `QueuedTilemaps::readbacks`, `readback_tail` |
| $0956..098D | 56 | the BG1/BG2 column and row update requests, which the scrolling builds and the NMI consumes | `QueuedTilemaps::requests` |
| $7E:DF5C..EF5B | 4,096 | the pause menu's backup of the BG2 tilemap: pausing reads it back from VRAM, unpausing uploads it | `Session::pause_background` |
| $7F:FA02..FFFF | 1,534 | the ending's blank tilemap past the custom background ($7F:F800..FFFF: the ending's setup decompresses it, the cross-fade to the space view uploads it); snesrev's bug-fix counter copy ($7F:FF00) lies inside, and the Session owns it as the original's tilemap byte | `Session::ending_tilemap` |

The three queues' writers and readers already went through the lease's
projection (`Runtime::byte`/`set_byte`), and a readback's DMA writes return
to the Session through `start_dma`. The pause backup's writer is a VRAM
readback DMA on Game and the ending tilemap's a decompression on the lease
(`decompress_to_ram` takes its bytes back); RAM written off the lease reaches
the next lease through its write log or an import. So the RAM image is
unchanged. An owned Display still marks itself failed on a store to the
three queues (`Presentation::requires_semantic_counterpart`), and its own
`mode7_tail`/`readback_tail` words are shadowed by the Session's in the
projection, as its `upload_tail` already is by `QueuedTilemaps::tail`.

### The 107 "both" bytes

Every "both" byte is a Session field whose range a `LEGACY(verifier)` row
normalizes (found by decoding zeroed RAM with each byte set and naming the
field that changes). None can be dropped from `NORMALIZED_RAM` yet: each row
records frames where the two sides differ. Decision per range:

| Range | Session field | Decision |
| --- | --- | --- |
| $0000..0003 | `pointer_scratch` | unowned: the comparison ignores the field until the $0000-$0050 row retires (theirs: the engine does not leave the original's value) |
| $0012..0017 | `spawn_scratch` | as above |
| $060B | `enemy_projectiles.spawn.angle` | unowned until the $060B-$0610 row retires |
| $0612..0613 | `control[0].primary`, `.secondary` | unowned, permanent: snesrev's coroutine state, which a faithful oracle never writes; a field RAM cannot carry (slice 4's declared list) |
| $077E..0781 | `messages.delay`, `room.unpause_queue_tail` | unowned, permanent: the engine keeps them in snesrev's counter RAM, which the original leaves unused ($077E..82); fields RAM cannot carry |
| $0D82..0D8F | `game_loop.saved_bindings` | unowned until the $0D82-$0D8F row retires (the grapple temporaries share it) |
| $0D9C..0D9D | `wavy_samus.enabled` | unowned until the $0D9C-$0D9D row retires (the grapple swing's angular velocity shares it) |
| $0E20..0E45, $0E4A..0E4B, $0E54..0E59 | `compatibility_enemy_scratch`, `actors.compatibility.*`, `actors.combat.last_power`, `actors.controls.*` | unowned until the enemy temporary rows retire |
| $1784..178B, $17A8..17AB, $1834..183B, $1874..1875 | `actors.compatibility.pre_hook`, `actors.schedule.*`, `actors.compatibility.right_border` | as above |
| $18A6..18A7 | `room.collision_detection_index` | unowned until its row retires |
| $7E:EF74..EF77 | `actors.graphics.compatibility_*` | as above (the enemy graphics load frame) |

So 101 bytes become owned when their rows retire (drop the range from
`NORMALIZED_RAM` and its row in the same change, as `docs/LEGACY.md` says),
and 6 stay unowned for good. Slice 6 turns the table into the comparison's
ignore list for those fields.

The decisions are data since the fourth slice:
`observation::coverage::SHARED_RAM` lists the 16 ranges with their field and
`Shared` decision (`UntilRowRetires`, `Permanent`), and the coverage report
adds one line:

```
Session RAM decoded and unowned by decision: until its row retires 101, permanent 6 (control.primary, control.secondary; messages.delay, room.unpause_queue_tail), undecided 0
```

"undecided" counts "both" bytes no entry covers and names the first, so a
field that newly lands in a normalized range shows there. "Both" itself
reaches 0 only as the rows retire (101 bytes, behaviour fixes each) and the
6 permanent bytes move to slice 4's list of fields RAM cannot carry.

### Rows to confirm

`death` and `save-death-reload` (the low-health warning sets and clears),
`door-transition` and `tas-ceres-four-doors` (room loads store the level
data's size): lockstep must stay at 0 failing frames and 0 Session
divergences. For the fourth slice, the rows that drive the new owners:
the pause and unpause rows (the BG2 backup), Kraid and the X-ray
rows (the VRAM read table), the Ceres and ending rows (`credits`, the Mode 7
queue and the ending tilemap), and any scrolling room (the requests). On
every row the coverage line must print:

```
Session RAM coverage: 126781 bytes decoded, 3926 unowned, 107 both, 258 held outside the Session, 0 neither (in 0 ranges)
```

with `undecided 0` on the decision line. Detached checkpoints written before
SMAUXGS 20 are rejected as an unsupported schema.

## Fifth slice (lane `collab/session-compare-4`)

**The 258 held bytes are Session fields.** `LegacyPresentation` holds the
frame display as a `display::preparation::State` and the eight scroll words,
as the owned Display does in its state: decode loads them at every lease
import, encode publishes them at every export, and `projected_byte` and
`write_projected_byte` serve them from whichever presentation owns them.
The address map moved into sm_game (`preparation::Field::address`,
`Word::address`, `lane`, `display_scroll::Word::address`). The Runtime's
scroll access reads and stores the Session (`Presentation::scroll_word`),
and the RAW reads that bypass the owned Display's projection (the physical
scroll bytes, `$51` in the cinematic fade, the HUD's frame counter byte)
read the Legacy presentation's byte, which is the byte RAM held before; the
scroll stores' input refresh takes the frame display's bytes (the NMI
counters and the Mode 7 words) from it too. No checkpoint holds the Legacy
presentation (an owned Display's checkpoint already carries the bytes), so
there is no format change. Two of the frame display's 260 bytes keep their
other owners under Legacy (`session_owners::kept_elsewhere`), as the host's
domain left them: `$07EC`, the committed fake mode, is the high byte of
`room.ceres_hdma_table` ($07EB, which Ceres Ridley's room setup $8F:C97B
and the haze's HDMA instruction store), and `$0782`, the NMI acknowledgment,
stays RAM's (snesrev's counter RAM in `NORMALIZED_RAM`). They are not in the
Legacy frame display's RAM image at all: its decode
(`LegacyPresentation::decode_frame`) reads neither, and its projection,
store and encode skip both, so their fields stay zero. A first attempt
(`collab/sc4-held`) claimed `$07EC` twice, so the frame display's encode
overwrote the Ceres word's high byte (`chain-any-0006-E0B5`); its fix
skipped the two in the projection and encode but still decoded them, which
broke `session_round_trips_random_ram` and the verify build's
`assert_projects_ram` (`samus-scripted-intro`, `demo-attract-setup`). Layout test:
`legacy_frame_display_round_trips_every_byte`. Edits in `session_runtime.rs`
(the other lane's file): the scroll special cases in `byte`, `set_byte`,
`observed_inputs` and `refresh_scroll_inputs`.

The coverage line must now print, on every row:

```
Session RAM coverage: 127039 bytes decoded, 3926 unowned, 107 both, 0 held outside the Session, 0 neither (in 0 ranges)
```

and `Session RAM held outside the Session: none`. Rows that drive the moved
owners: every row (the NMI counters and HUD), and especially the scroll
writers (`boss-kraid*`, `boss-crocomire`, Draygon, Phantoon, Mother Brain's
body, Ceres Ridley), the cinematic fades (`samus-scripted-intro`, the Ceres
and ending rows, `credits`), the pause rows, and the rows the first
attempt failed: Ceres Ridley's room (`chain-any-0006-E0B5`, the `$07EC`
word), `samus-scripted-intro` and `demo-attract-setup` (the retained
Session against a fresh decode).

**The "both" evidence.** The 101 bytes behind `UntilRowRetires` wait on 14
`LEGACY(verifier)` rows, and none could be retired here: each row's
divergence has to be found on oracle replays, which this machine cannot run.
The lockstep summary now counts, per `SHARED_RAM` entry, the Ticks whose two
results differ before normalization:

```
Session RAM both bytes differing before normalization, of <n> ticks: $0000-$0003 pointer_scratch <n> from @<t>; ...
```

An entry at 0 on the corpus and both routes leaves its normalized range in
one change (split the row; the field is then compared and "both" falls by
its bytes). One that differs names the first Tick to localise. Expected
causes, from the rows: `pointer_scratch`, `spawn_scratch` (the $0000-$0050
row: other routines' direct-page scratch shares the range, so only a split
can retire these 10 bytes); `enemy_projectiles.spawn.angle` (the
$060B-$0610 row: $060B is also other routines' parameter, and the hitbox
counters at $060D the engine's loops do not leave); `game_loop.saved_bindings`
and `wavy_samus.enabled` (the grapple temporaries share $0D82 and $0D9C,
and the engine's grapple keeps them in its own fields); the enemy temporary
rows (`$0E20-$0E45`, `$0E4A`, `$0E54-$0E59`, `$1784-$178B`, `$17A8-$17AB`,
`$1834-$183B`, `$1874-$1875`: the indices and pointers the enemy loops
leave); `room.collision_detection_index` (the index the projectile loops
leave); `actors.graphics.compatibility` (the enemy graphics load, one frame
per room load).

**Session streams in the lock.** `session_compare::Group` puts the
top-level fields in eight groups (`session_loop`, `session_actors`,
`session_player`, `session_room`, `session_display`, `session_menus`,
`session_save`, `session_scratch`). Each Tick's group digest folds its
fields' digests, a field's digest being that of its debug text, recomputed
only on the Ticks the field changes. Until slice 4 both sides decode their
frame result with the normalized RAM masked, as `wram` masks it. Trace
records end with the digests (old traces read back); the lock writes them
only when every Tick had them; the combined digest and `FORMAT` (1) are
unchanged. The oracle always takes the streams; the engine takes them with
`--session-streams`, which check-lock passes only for entries that have
them, so the gate costs nothing until entries carry them.

How they reach the lock: the rotating faithful oracle sample
(`lock_gate.py record-oracle`, the scheduled Verify's `oracle_sample`, or a
dispatch with scope `full` and `oracle_sample`) now accepts recorded streams
the committed entry lacks and prints `STREAM <id> <stream> <digest>` for
each; the root runs `python tools/scenario.py lock --add-streams <job log>`
and commits the lock. Each scheduled run covers its sample of at least 30
entries; once every entry carries them, slice 6's one `FORMAT` bump drops
`wram`.

## Next slices

1. Done (above). Confirm on dispatched rows: Session-only failures 0 and
   the fields on the "written after the last lease" line.
2. Done (above): every one of the 9,649 "neither" bytes is unowned (with
   reasons), held outside the Session, or a Session owner, so "neither" is
   0; the "both" bytes' decisions are a typed table with 0 undecided; the
   frame display and scroll (258 held bytes) are in the Session (fifth
   slice). Left: the 101 "both" bytes that wait on their `LEGACY(verifier)`
   rows, before slice 6 drops the WRAM compare; the evidence line names
   which differ. Confirm the counts on the corpus and both routes.
3. Copy the decode half into `sm/src/verification`, compare it with the
   engine's decode on every frame, then use it for the oracle side.
4. Compare the engine's Session directly, with a declared list of fields RAM
   cannot carry, and drop the encode round trip. It needs the frame-end
   Session as an object: until the runtime owns the NMI flush and the
   controller latch (step 2), the frame-end Session is the next lease's take
   on RAM, so the direct compare either waits for step 2 or applies the
   logged writes to the kept Session itself (`take_written_bytes`).
5. Done (fifth slice): the lock has Session streams, populated through the
   oracle sample and `scenario.py lock --add-streams`. Left: the engine's
   streams digest its frame-end Session directly once slice 4 lands, not a
   decode of its RAM.
6. Switch lockstep and locks to Session space: drop the WRAM byte compare and
   `wram` (one `FORMAT` bump, one re-record), replace `NORMALIZED_RAM` with the
   unowned table, and retire its `LEGACY(verifier)` rows. Needs slice 2's
   "neither" and "held" at 0 (both are, from the fifth slice) and the
   "both" ranges' fields on the comparison's ignore list.
7. With step 2 (the runtime owns the tick), delete the export and import, and
   move the observers' `encode`/`decode` calls to verification:
   `engine_ram_export_lines` reaches 0. The "written after the last lease"
   line reaches 0 ticks when the NMI and controller latch run inside the
   Tick, which is step 2's own check (moved in `tick-s10`; confirm on the
   rows).
