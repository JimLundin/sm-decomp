# Super Metroid state ownership and interaction contracts

Research date: 2026-10-22. Baseline: `f2198988`.
This note follows [PLAN.md](PLAN.md#validation-strategy-2026-10-21).
The findings below come from current Rust source. Proposed owners and interfaces
are design recommendations, not a completed replacement of the shared RAM model.
This is a traced set of important interactions, not an exhaustive write inventory.
Source symbols identify the implementation if later edits move the linked lines.

## Finding

The useful unit of ownership is a gameplay responsibility **plus the lifetime of
its state**. Giving Samus, creatures and rooms separate structs is insufficient:
some values belong to the preceding movement update, some change during a
callback, and some survive a suspended gameplay update. The ordinary gameplay
dispatcher and projectile executors expose these distinctions directly.[^frame]
[^flight][^eproj]

The strongest initial experiments are a named gameplay continuation owner and
the flight/collision selector contract. Both have a small interface and observable
ordering rules. They can make behavior explicit while retaining compatible RAM;
neither requires a speculative whole-engine event system.

## What owns memory today

`Game` owns one 128 KiB `g_ram` array, ROM access, host/backend references and
oracle state. The allocation must remain fixed while hardware holds pointers
into RAM. This is allocation ownership, not a decomposition of gameplay.[^game]

The `word!` interface names byte locations; it does not assign authority to a
gameplay module. It intentionally supports odd addresses, overlapping fields,
signed offsets and ordered updates. Its raw operations avoid introducing a
whole-RAM borrow while retained enemy pointers remain live.[^word]

Checked byte-backed access already exists in `enemy_ram`, `EprojState` and
individual `Game` methods. These can eliminate raw accesses in complete paths,
but a named getter alone does not establish a narrow gameplay interface.[^enemyview]
[^eproj][^game]

| State class | Observed examples | Proposed authority |
| --- | --- | --- |
| Persistent progression | Collected equipment, event/boss bits, opened doors, explored map | Progression rules; persistence encodes/decodes them |
| Room instance | Selected room state, block data, active enemies and mechanisms | Room lifecycle coordinates creation and retirement |
| Actor state | Samus position/pose/health; creature position/AI/status | The actor's behavior module, with explicit contact operations |
| Ordered communication | External Samus displacement, launch inheritance, requested pose | Named producer/consumer contracts with specified update phase |
| Execution state | Current projectile/enemy/PLM selector; continuation bytes | The executor that defines each capture and reread |
| Shared scratch | Bitmask, collision index, block-reaction context | One operation at a time; preserve exposed byte effects during migration |
| Mode-dependent storage | Effect slots reused by cinematics/options | Session mode grants a temporary interpretation of the bytes |

The classifications are proposals grounded in the concrete traces below.
They are deliberately not one Rust struct per RAM interval.

## The gameplay update is a resumable protocol

On entry, the original `GameState_8_MainGameplay` (now `gameplay::advance`)
reads the continuation byte once. The
ordinary path has these phases, in this order:[^frame]

1. Select active/interactive enemies; advance palette effects; prepare Samus;
   resolve Samus/projectile interaction; run enemies.
2. Display a queued message, potentially suspending the update.
3. Complete Samus movement and transitions; run enemy projectiles/effects.
4. Run room mechanisms, potentially suspending inside a message.
5. Advance animated tiles; resolve effect contacts and Power Bomb interactions;
   scroll; draw; update HUD; run room logic, health/game-time handling, shaking
   and timers.

Continuation `1` resumes at the queued message; `2` resumes room mechanisms.
Other nonzero values above `2` currently skip the phases and reset to zero.
Callbacks can mutate the byte, but branches use the captured entry value until
the dispatcher writes its yield/completion value. Preserve this distinction
when replacing the retained raw pointer with a named continuation.[^frame]

Door fade-in deliberately uses another sequence: enemy selection, enemy update,
effects and drawing can execute without normal Samus preparation/completion.
Room loading clears and recreates several families in a prescribed order.
A generic `update_everything()` cannot replace these schedules by assumption.
[^door]

## Traced interaction contracts

### 1. Room contact and platforms contribute to Samus movement

**Producers:** `BlockInsideDetection` clears four external-displacement words,
then invokes block reactions. Sand/quicksand setup writes displacement and can
clear momentum. Platform/shutter behavior subsequently writes or adds carry/push
displacement while enemies run.[^inside][^sand][^platform]

**Consumer:** Samus horizontal and vertical movement incorporates those words
in the later completion phase. **Lifetime:** reset during preparation, updated
by ordered contacts, consumed later in the same ordinary update. A suspended
message must preserve the contributions until completion resumes.[^movement]
[^frame]

**Constraint:** writes are not all additive; some replace a value or clear only
one fractional word. An unordered collection of `PushSamus` events would need to
preserve those exact rules. A proposed `ExternalMotion` interface must distinguish
replacement, addition and clearing, or hide the complete ordered algorithm.

### 2. Samus movement supplies the next launch's inherited speed

**Producer:** movement routines write directional
`projectile_init_speed_samus_moved_*` integer/fraction words.
**Consumer:** projectile initialization combines their overlapping word reads
with launch speed. During ordinary preparation, HUD/firing/projectile processing
runs before `Samus_Func10` clears the inheritance fields; later movement writes
them again.[^inheritance][^launch][^prepare][^hud]

**Lifetime:** these are a handoff across phases and ordinary gameplay updates,
not a fresh snapshot of Samus velocity taken at firing time. Preserve the actual
byte packing and up/down asymmetry. A `LaunchMotion` value is plausible only if
captured where the present consumer reads it, before the producer's next reset.

### 3. Enemy contact changes Samus before her movement completes

**Producer:** `EnemyMain` runs collision handling before selected AI and instruction
work. `SamusProjectileInteractionHandler` can set damage, invincibility,
knockback or bomb-jump direction. **Consumer:** the later Samus completion handler
performs movement, hit interruption and pose transitions.[^enemies][^contact]
[^complete]

**Ordering:** ordinary Samus completion first clears `samus_contact_damage_index`,
then movement can set it again. Enemy contact earlier in the update and effect
contact later in the update therefore see different phases of the field.
The effect/Samus collision pass also consults invincibility and this contact
mode. A single end-of-update damage application would change that timing.
[^complete][^effectcontact]

### 4. Projectile execution has several different selector lifetimes

**Producer/consumer:** `projectile_flight::update` receives `k`, decrements that
slot's trail timer, calls `SpawnProjectileTrail`, then reloads `projectile_index`
before advancing. `advance` uses its argument for direction/acceleration, but
`collide` independently captures the live global selector.[^flight]

**Pinned identity:** diagonal `collide` uses its captured `k` for both horizontal
and vertical calls, even if the horizontal call changes the live selector.
Horizontal hit returns early; beams use bit zero of the result, while missiles
use any nonzero result. Super-missile extension collision still runs before that
early return. **Live identity:** offscreen cleanup subsequently rereads the global
selector. Extension collision temporarily selects its linked slot and restores
the saved selector after calling collision.[^flight]

These facts rule out one cached slot for the entire update. They also rule out
blindly rereading after every callback. A proposed `ShotCursor`/outcome interface
must name which identity is retained and which is reloaded. The actual block
collision routines perform movement, reactions and possible projectile death;
they are not read-only geometry queries.[^shotcollision]

### 5. Enemy-projectile scripts have a different capture protocol

`EprojRunAll` descends from byte index 34. `EprojRunOne` calls the selected
pre-instruction, then reloads `eproj_index`; the timer and instruction-list lookup
use that new selection. It captures that value as `v1` for instruction callbacks
and final timer/spritemap/list writes. On return, the outer loop reloads the
selector again before subtracting two.[^eproj]

**Lifetime:** the pre-instruction can affect which script is interpreted, while
instruction callbacks receive a retained script identity. The next loop position
is live again. Scoped byte views fit this protocol if they end before callbacks;
a retained mutable slot borrow or entry snapshot does not express it.

Constructed callback mutations can verify this contract. This source trace alone
does not establish that a particular movie exercises a selector-changing callback.
Likewise, a reread after `SpawnProjectileTrail` is present in source; a natural
trail path that changes the selector has not been established here.[^trail]

### 6. Picking up an item commits progression before the message finishes

**Producer:** PLM pickup instructions update collected/equipped items, capacities
or health; beam pickup applies two ordered exclusion masks and refreshes visuals.
They then queue a message. **Consumer:** `PlmHandler_Async` stores the next script
position before displaying it and restores that position after display completes.
[^pickup][^plm]

**Lifetime:** the item change is already committed while the gameplay update is
suspended. PLM continuation, gameplay continuation and message animation each
have separate state bytes. A queue entry is one overwriteable message word, not
an arbitrary-length event queue. Display may also change session state when
closing message 20. Replaying the pickup on resume or deferring the item award
until message dismissal would change behavior.[^plm][^message]

### 7. Progression selects room instances and survives room retirement

**Producers:** pickup/door instructions and event/boss setters mutate persistent
bits. **Consumers:** room-state selection checks event bits, area boss bits,
collected Morph Ball and ammo capacity, then selects the room-state record.
`LoadRoomHeader` establishes the area before selection; `LoadStateHeader` derives
the room's enemies, graphics, effects, music and main logic from that selection.
[^bits][^pickup][^doorbits][^roomselect][^roomload]

**Lifetime:** room loading retires/recreates effects, mechanisms and enemies,
while save/load serializes progression and map/player data. The persistent facts
and the selected room instance are related but should not have the same lifetime.
**Scratch alias:** `CheckEventHappened`, despite its query-like name, writes
`bitmask`; boss checks write only its low byte. Preserve those side effects in
the compatibility implementation before proposing a pure progression query.
[^door][^save][^bits]

### 8. Pose, animation and gameplay are one coordinated state machine

**Producer:** pose refresh updates direction and movement-type bytes with a word
store. `record_pose_history` copies previous to last-different before current to
previous; each record is four bytes: pose word, facing byte, movement byte.
**Consumers:** pose transition logic compares history; drawing derives tiles
from current pose and animation frame.[^pose][^posemap][^posetransition][^tiles]

**Reverse interaction:** animation instructions can request transitional poses,
change the movement handler and install the autojump input handler. Animation
cannot yet be treated as a passive renderer. The proposed Samus owner should
include gameplay animation decisions, leaving sprite/tile submission to
presentation. Preserve the packed records and ordered history copy.[^animation]

### 9. Enemy identity is not always the current enemy selector

`EnemyMain` captures a pointer to an enemy slot, calls collision/AI/instruction
handlers, then reads that same slot again for status and drawing decisions.
Other operations consult live `cur_enemy_index`. Replacing the pointer with an
offset is promising; replacing subsequent reads with a copied `EnemyData` is a
behavior change. Scoped `enemy_state` and `enemy_coordinates` views already show
the offset-and-reread direction.[^enemies][^enemyview]

An enemy handle must initially preserve slot identity and reuse behavior.
Adding generation checks or rejecting a callback's selector change would require
separate behavioral evidence, even if attractive for a later engine design.

### 10. Effect slots belong to a session mode as well as an effect

The enemy-projectile executor also spawns pickups. Its RAM aliases cinematic
variables and options-menu arrays. Spawning initializes selected fields, calls
the initializer immediately and returns; clearing effects clears IDs rather
than zeroing every slot byte. Gameplay, room transition and menu/cinematic use
therefore cannot be modeled as independent permanent owners of these bytes.
[^eproj][^effectmap][^drops]

Use a session-mode lease as a design concept: the active mode defines the storage
interpretation and reset rules. Do not infer that unused slot data can be erased,
or that deferred spawning has equivalent same-update execution order.

## Comparing interface choices

| Choice | What it makes explicit | Main limitation here | Recommendation |
| --- | --- | --- | --- |
| Scoped live byte views | Field meaning, widths, short borrow lifetime, checked access | A view retained across callbacks conflicts with live rereads; whole-RAM borrowing can invalidate existing raw pointers | Use after migrating the complete retained-pointer path; store offsets between calls |
| Snapshots and deferred events | Clear inputs/outputs; independent calculation | Loses same-update visibility, overwrite order, selector changes and committed state during suspension | Restrict snapshots to proven stable inputs; apply ordered outcomes at the original execution point |
| Explicit outcomes/continuations | Completion vs suspension, committed effects, next operation | A result can still hide uncontrolled mutation; invented event lists add a second execution model | Pilot a named gameplay resume state and narrowly scoped collision outcomes |
| One monolithic frame update | Existing order remains in one place | The update can suspend; doors use different schedules; ownership remains implicit | Keep an orchestration owner, but move complete behavior behind gameplay interfaces |

A deep module should hide an algorithm and its ordering rules, not expose a
large bag of RAM setters. Deduplication is useful when two callers share the
same transition/interaction contract; similar memory layouts do not prove that.
Keep authored room/equipment/creature definitions separate from live execution
state without requiring the compatibility bytes to disappear first.

## Selected experiment questions

1. **Gameplay continuation pilot:** compare an explicit gameplay owner against the
   actual dispatcher using full RAM, return values and ordered callback traces.
   Cover fresh execution, both suspension points, repeated resume, queue clearing,
   debug/interaction gates, entry continuation mutation and invalid entry bytes.
   Then select the smallest available oracle segment that executes the changed
   owner and its downstream effects. Unvisited continuation branches need direct
   actual-code checks; a green normal frame is insufficient.[^validation]
2. **Projectile contract experiment:** hold actual `update`/`collide` code fixed
   while controlled calls change selector/slot bytes. Compare proposed retained
   identity and live-view interpretations. Cover trail expiry/nonexpiry, pinned
   diagonal identity, horizontal early return, beam/missile result encoding,
   extension save/restore and live offscreen cleanup. For enemy effects, contrast
   pre-instruction redirection with pinned instruction identity and outer-loop
   rereads. These constructed mutations test executor contracts, not movie reachability.

## Experiment results

### Explicit gameplay continuation

The implementation now lives in [gameplay.rs](../sm/src/game/gameplay.rs).
`ResumePoint::{Start, EnemyMessage, RoomObjects, Skip}` names the captured entry
continuation. The scheduler uses existing checked continuation accessors instead
of retaining a raw pointer across callbacks. All ordinary callers and the game
state table call this owner directly; the old bank function is removed.
Queue and interaction-policy reads remain live at their original positions.
This is the first ownership implementation, not a complete safe gameplay module:
its existing unsafe callees and remaining word accesses are explicitly retained.

Actual original and replacement scheduler bodies pass **18,400 full-RAM,
return-value and ordered-callback comparisons**, plus **18 persistent update
steps**, with **97,349 matching callback observations**. The cases cover all
256 continuation bytes, both suspension points, yield bytes including 128/255,
message queue changes, interaction-policy changes, callbacks changing the
continuation byte, and repeated message/PLM resumption. They run against the
real compiled `Game` allocation and accessors, whose embedded source is checked.

The external gameplay callees are controlled recording/mutating substitutes.
The harness expands the two relevant `word!` fields into checked little-endian
accesses at their source-declared locations; it does not establish raw-pointer
aliasing safety or execute those callees' real behavior. Full RAM means the
entire 128 KiB array after each actual scheduler call, including unchanged bytes.
This is an actual-code scheduler experiment, not a whole-game component replay.

A deliberately wrong version captures queue and interaction-policy values at
entry. **Four targeted counterexamples** reject it: an enemy queues a message,
Samus preparation disables interactions, PLM execution changes the later
interaction gate, or an enemy clears an already queued message. This establishes
why those reads must stay live; it does not establish movie reachability of the
constructed mutations.

### Explicit projectile outcome and identity lifetimes

**2,560 cases** execute the actual current flight-update/collision bodies with
controlled trail/collision/extension callees. An experimental `TrailOutcome`
returns the post-trail resume slot explicitly and matches the original update's
RAM and call sequence. It is a tested design example, not adopted projectile
code. The current projectile implementation remains unchanged.

The same cases check captured diagonal identity, horizontal early return,
beam-bit-zero versus missile-nonzero hit encoding, and extension call order.
**104 counterexamples** reject keeping the pre-trail slot; **78 counterexamples**
reject rereading the live selector for diagonal vertical collision. A separate
**48-case** check executes actual offscreen cleanup around both clipping edges
and confirms it uses the live slot. Actual extension save/restore internals,
enemy-projectile script execution and naturally reachable selector-changing
callbacks remain source findings or future experiments, not established runtime
coverage from this harness.

The durable experiment is
`Local/sm-decomp-experiments/20261022/ram-ownership/` under the user's local app
data directory. It contains frozen original Rust, compiled component inputs,
source hashes and the isolated replay source. It is not maintained implementation.

### Oracle and observed phase mutations

The instrumented and clean isolated-source runs each pass **8,359 frames with
zero failing frames**, with normal oracle, rendering and audio checks. Inputs
0..8323 are the setup prefix; the selected mutation sample is **8324..8358**.
Four required probes execute 35 times each: Samus preparation, Samus completion,
room objects and actor drawing. The sample spans Ceres fade-in and six frames in
gameplay state 8. It supplies no normal player-control, suspension, later-room,
boss or credits coverage. Suspension and unusual resume bytes have the direct
scheduler coverage above.

There are **735 callback observations across 21 executed phases** in that
35-frame sample. Selected net-change counts illustrate why ownership must be
defined by responsibility and lifetime:

| Phase | Distinct bytes changed on return, across the sample |
| --- | ---: |
| Actor drawing | 94 |
| Enemy processing | 48 |
| Samus completion | 22 |
| Active-enemy selection | 16 |
| HUD | 9 |
| Room objects | 8 |
| Enemy projectiles/effects | 7 |
| Samus preparation | 0 |

Seven byte locations change in more than one phase: the first words of
`active_enemy_indexes` and `interactive_enemy_indexes`, `cur_enemy_index`, and
the low byte of `enemy_processing_stage`. The last is changed by enemy
processing and four interaction passes. These are observed coordination fields,
not evidence that one module can exclusively own the whole surrounding RAM range.

Callback-boundary snapshots measure net changed bytes, not every read or write:
writing the same value or changing a byte and restoring it is invisible to them.
Zero net changes also do not imply a callback is pure. The counters can confirm
observed producers and ordering, but cannot prove the absence of other writers.
No raw gameplay snapshots or extracted ROM content are committed with this note.

The replay snapshot includes the concurrent checked-platform work and precedes
the independent owned-ROM change. The adopted gameplay file and its caller
changes match that snapshot. Library/frontend/verifier release compilation also
passed on the composed pre-ROM source with all 97 embedded inputs checked.
Independent changes retain their own validation; the replay is not claimed as
validation of a later ROM ownership implementation.

## Next ownership experiments

1. Codify `ExternalMotion` as ordered reset, replacement and addition operations;
   trace real sand/platform contacts through message suspension and movement.
2. Codify launch inheritance at its actual cross-update handoff; compare against
   the tempting but different rule of sampling Samus's velocity at firing time.
3. Replace retained enemy pointers with slot offsets while preserving live
   rereads, then test reaction-induced deletion/reuse and instruction selection.
4. Model PLM item-award/message continuation together, preserving already-committed
   progression during suspension. Keep its scheduler distinct from enemy scripts.

These are identified next experiments, not completed migrations. An exhaustive
writer inventory, full gameplay reachability of selector mutations, safe ROM and
RAM access throughout the engine, and boot-to-credits coverage remain open.

## Primary sources

[^frame]: [gameplay::advance and ResumePoint](../sm/src/game/gameplay.rs); the original dispatcher is in `f2198988:sm/src/game/sm_82.rs`.
[^game]: [Game and checked accessors](../sm/src/game_state.rs#L8).
[^word]: [word! evaluation and aliasing contract](../sm/src/ram_map.rs#L85).
[^enemyview]: [EnemyState and EnemyCoordinates](../sm/src/game/enemy_ram.rs#L5).
[^flight]: [projectile_flight::update, advance and collide](../sm/src/game/projectile_flight.rs#L26); [extension selection/restoration](../sm/src/game/projectile_flight.rs#L233).
[^eproj]: [SpawnEprojInner, EprojRunOne and EprojRunAll](../sm/src/game/sm_86.rs#L234).
[^door]: [DoorTransitionFunction_LoadMoreThings_Async and fade-in](../sm/src/game/sm_82.rs#L4012).
[^inside]: [BlockInsideDetection](../sm/src/game/sm_94.rs#L1197).
[^sand]: [PlmSetup_QuicksandSurface and sandfalls](../sm/src/game/sm_84.rs#L1736).
[^platform]: [moving_platforms::travel and react](../sm/src/game/moving_platforms.rs).
[^movement]: [Samus_MoveY_WithSpeedCalc](../sm/src/game/sm_90.rs#L448); [Samus_CalcDisplacementMoveLeft/Right](../sm/src/game/sm_90.rs#L3932).
[^inheritance]: [Samus movement inheritance writes](../sm/src/game/sm_90.rs#L287); [Samus_Func10](../sm/src/game/sm_90.rs#L4579).
[^launch]: [projectile_flight::initialize_speed](../sm/src/game/projectile_flight.rs#L119).
[^prepare]: [Samus_FrameHandlerAlfa_Func11/12](../sm/src/game/sm_90.rs#L4009).
[^hud]: [Samus_HandleHudSpecificBehaviorAndProjs](../sm/src/game/sm_90.rs#L3477).
[^enemies]: [EnemyMain](../sm/src/game/sm_a0.rs#L879).
[^contact]: [SamusProjectileInteractionHandler](../sm/src/game/sm_a0.rs#L1259).
[^complete]: [Samus_FrameHandlerBeta_Func17](../sm/src/game/sm_90.rs#L4058).
[^effectcontact]: [EprojSamusCollDetect](../sm/src/game/sm_a0.rs#L1332).
[^shotcollision]: [beam_block_collision and missile_block_collision](../sm/src/game/sm_94.rs#L1648).
[^trail]: [SpawnProjectileTrail](../sm/src/game/sm_90.rs#L2076); [ProjectileTrail_Func5](../sm/src/game/sm_9b.rs#L61).
[^pickup]: [PLM item pickup instructions](../sm/src/game/sm_84.rs#L2695).
[^plm]: [PlmHandler_Async](../sm/src/game/sm_84.rs#L292).
[^message]: [message_boxes::display, poll and queue](../sm/src/game/message_boxes.rs#L397).
[^bits]: [PrepareBitAccess and event/boss operations](../sm/src/game/sm_80.rs#L111).
[^doorbits]: [PlmInstr_IncrementDoorHitCounterAndJGE](../sm/src/game/sm_84.rs#L2796).
[^roomselect]: [CallRoomDefStateSelect and HandleRoomDefStateSelect](../sm/src/game/sm_8f.rs#L125).
[^roomload]: [LoadRoomHeader and LoadStateHeader](../sm/src/game/sm_82.rs#L3711).
[^save]: [SaveToSram and LoadFromSram](../sm/src/game/sm_81.rs#L71).
[^pose]: [samus_pose](../sm/src/game/samus_pose.rs#L1).
[^posemap]: [pose/history metadata](../sm/src/ram_map.rs); search `samus_pose`, `samus_prev_pose`, `samus_last_different_pose`.
[^posetransition]: [SamusFunc_F404 and SamusFunc_F433](../sm/src/game/sm_91.rs#L2496).
[^tiles]: [SetSamusTilesDefsForCurAnim](../sm/src/game/samus_animation.rs#L863).
[^animation]: [Samus_AnimDelayFunc_7/8/9/13](../sm/src/game/samus_animation.rs#L231).
[^effectmap]: [effect/cinematic/options aliases](../sm/src/ram_map.rs); search `eproj_index`, `eproj_id`, `optionsmenu_cur_data`.
[^drops]: [SpawnEnemyDrops](../sm/src/game/sm_a0.rs#L1012).
[^validation]: [Validation strategy](PLAN.md#validation-strategy-2026-10-21).
