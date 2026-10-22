# Super Metroid state ownership and interaction contracts

Research date: 2026-10-22. Initial baseline: `f2198988`; follow-up baselines are
identified with their findings and experiments.
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

**Standard combat ownership preflight (source findings, not new runtime checks):**
`NormalEnemyShotAiSkipDeathAnim` samples initial projectile damage/type and the
vulnerability record, then sets flash/AI state before a hurt-sound callback. After
that callback it rereads the enemy selector for health damage and independently
rereads the projectile selector/type. A single captured enemy/projectile pair
would therefore change this interaction. The original health target becomes
captured only after the sound call. The beam-only `last_enemy_power` write also
matters: non-beam lethal checks can observe its earlier value.

Freezing through vulnerability byte 255 captures the enemy before its optional
sound, then reads the area and writes freeze state after sound returns. Lethal
ice damage writes freeze state before playing sound and leaves health unchanged.
Normal thawing has another deliberate quirk: clearing the frozen AI bit copies
the resulting AI word into the frozen timer. Contact damage uses a signed wrapped
subtraction test; shot and Power Bomb health subtraction use unsigned saturation.
These rules cannot be replaced with one generic health update.

`NormalEnemyShotAi` retains the enemy slot across the hit-sprite callback for its
death test and enemy-definition lookup, while cause-of-death storage and death
animation select the live current enemy. `EnemyDeathAnimation` in turn captures
its cleanup slot before spawning the explosion, reads that slot's preservation
bit after spawning, clears its 64 bytes, and then increments the live kill count.
An index handle can replace the retained raw pointer; recapturing the selector
at cleanup would target a different slot after callback mutation. Contact mode 4
similarly retains its original enemy/definition while calling Samus code, then
reads the retained definition's live vulnerability bytes.

The complete isolated model/adapter draft costs **+203 Rust lines** on 461097ea,
including its retained bank facade and registrations. It has only a formatted
price, with no compiler or behavioral validation, and is not adopted. It separates
decisions from RAM but retains persistent state and the existing call topology;
the price supplies no large reduction budget. These findings narrow the contract
for a broader enemy lifecycle owner rather than validating this draft. Local
source: `Local/sm-decomp-experiments/20261022/enemy-combat`.

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

**Enemy-hit protocol (source audit on `04859525`, not new execution coverage):**
the ordinary and multibox enemy collision searches select the first eligible hit.
They request shaking for a super missile, then mark the projectile's direction
with `0x10` unless plasma penetration is allowed, publish the collision selector,
and call the selected shot reaction. They return after that reaction; the enemy
executor continues. The mark is visible to the reaction. It is not slot deletion.
[^enemyshotsearch]

Successful Reflec and reflecting Space Pirate reactions replace the direction
word before calling `ProjectileReflection`, thereby removing that mark. Reflec
also clears projectile type bit `0x8000`; the pirate retains it. Samus/projectile
contact rejects that bit, while the enemy shot searches do not. Thus reflection
can enable contact with Samus without moving the projectile to the hostile-shot
pool or disabling enemy contact. Represent target eligibility separately from
the state owner; an exclusive player/enemy faction swap would invent a rule.
Other contact gates still apply, and the selected Samus-damage branch contains a
native assertion. This source trace establishes eligibility, not a successfully
played reflected-hit route.[^reflection][^contact]

The caller-specific exceptional paths matter too. A reflecting pirate can return
before changing direction for an unactivated super missile. Reflec's `0x8000`
table result instead retains the mark and performs its unusual damage path:
the table lookup byte offset is also used as an enemy byte offset, while the damage
callee uses the live enemy selector. Do not substitute the initially contacted
enemy for that retained offset. This is a source-observed quirk, not proof of
natural reachability or a reason to normalize it.[^reflection]

The ownership interface therefore needs two ordered operations: prepare the
contact, then execute/apply its reaction before the next observer. A single
`Hit -> Delete` outcome loses the reflected continuation. The existing 120
reflection steps below exercise `ProjectileReflection` with controlled callees;
they do not execute either reflector's full shot reaction or these collision
searches. Those integrations remain a distinct experiment.

**Retirement is also several transitions.** For beams and missiles,
`KillProjectile` starts an impact animation and installs the inactive flight
handler; the impact retains a nonzero damage word and an allocated slot.
Higher projectile classes can instead clear immediately. The animation's delete
instruction later calls `ClearProjectile`. The following differences must survive
an owned lifecycle; no single `active` flag or record default captures them.
[^shotretirement][^shotimpact]

| Operation | State effect that callers can distinguish |
| --- | --- |
| Mark enemy contact | Sets a direction flag; a reaction can replace it before flight sees it |
| Begin beam/missile impact | Changes type to impact, sets damage to 8 and animation timer to 1; keeps the slot and count |
| Clear one projectile | Clears fractions and most slot fields, installs inactive flight, decrements the applicable count; retains trail timer and auxiliary word |
| Reset projectile system | Clears trail timers but retains position fractions and auxiliary words; zeroes counts/cooldown/Power Bomb flag and performs selected HUD, speed-echo and palette work |

Allocation/reservation has its own observed rules too: beam reservation increments
the count before placement, searches damage words, and reuses the last beam slot
when the search finds none. The count is not interchangeable with the number of
nonempty records. These are named weapon rules to preserve behind the interface,
not a mandate to expose counters or RAM fields to engine callers.[^beamlaunch]

**Room-contact protocol (same source audit):** beam/missile collision commits
position and subposition before its bounds rejection. Returning zero can still
mean the shot moved. A normal nonwave beam samples its span and stops when all
sampled cells block, subject to explicit counter overrides. A missile samples
one center cell and uses its reaction result directly. The raw room-size
precheck returns blocking without decrementing the beam's hit counter; it is
another reason that these are different policies, not one generic hit test.
[^roomshotscan]

Each block reaction runs before the scan continues. Shootable/bombable air and
solid blocks return their own collision classification regardless of whether
`SpawnPLM` found a free mechanism slot. The special `B974` header changes scan
counters before attempting allocation. Wave beams still perform these reactions
and obey scan termination even though they do not enter impact here.
[^roomshotreaction]

`SpawnPLM` initializes the selected slot and calls setup synchronously. Setup
can immediately wake another mechanism or alter live level geometry. Gate lookup
searches block associations from the last slot down without requiring an active
header, so an old association can win. Strange Door uses the same kind of lookup
and samples projectile type through a signed global selector shift. These are
source-confirmed setup contracts; this audit has not enumerated ROM tables to
prove every setup reachable from every shot family.[^roomshotsetup]

Extension blocks redirect the shared block cursor and retry without another
room-size check. The outer scan then advances its original sample offset;
horizontal scans reread room width for that advance. Slope reactions and gate
setups also read the global projectile selector, which can differ from the
collision function's explicit argument. No whole-scan geometry/shot snapshot is
proven interchangeable with these reads.[^roomshotcursor]

The proposed interface separates blocking, scan continuation and mechanism
allocation results, and applies each selected mechanism response before the
next sample. Its decisive comparisons should include air/solid versus solid/solid
spans, rejected bounds after movement, full mechanism capacity with `B974`, stale
gate associations, and a later sample observing an earlier setup's geometry
change. These are proposed experiments, not results of the existing launch or
controlled flight tests. Local reading notes:
`Local/sm-decomp-experiments/20261022/projectile-room-contact/RESEARCH.md`.

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

**Adopted model:** `item_pickups::Inventory` owns the nine award transitions and
returns named visual/message effects. Its checked RAM adapter commits only the
affected inventory group before effects run; it retains no inventory snapshot
across those calls. `MessageHandoff` names instruction execution and suspended
display. The queued message, current PLM selector and saved script position stay
live at their original observation points. Both new modules forbid unsafe code.
The compatibility bridge retains raw script operands and unsafe visual/music
calls; script traversal, drawing and persistent RAM ownership remain transitional.

Beam masks execute in the original order inside the model, followed by one
equipped-beam store. The original intermediate stores had no callback, volatile
access or fallible operation between them. Likewise, storing an unchanged nonzero
reserve mode adds no observable effect under serialized Game access. Ammunition
adds to current and maximum separately; only an energy tank refills current to
the new maximum. Beam/equipment messages are read after visual/music effects.

Final-source checks pass **144 actual linked pickup comparisons**, **1,814
controlled instruction comparisons** and **1,008 actual executor-body steps**,
including 7,794 ordered callback observations and 434 matching injected boundary
failures. Full RAM, return values and failure flags match. Seventeen discriminating
checks reject early message reads and repeated awards. Cases include wrapping
amounts, both beam exclusions, reserve modes, repeated yields, invalid continuation,
disabled/inactive slots, null instruction returns with stale messages, and selector,
queue and inventory mutation in callbacks. The controlled executor uses recording
visual/music/display/drawing/preinstruction boundaries; its scenarios do not
establish gameplay-route or actual message animation coverage. No movie ran.
Odd script cases compare observed optimized host behavior, not defined alignment
of the old raw dereferences. Retained bank code and all numeric routes match.

Normal release library/frontend/verifier, all **111 embedded library inputs**
and installed native source checks pass. Complete ownership cost is **+154
maintained Rust lines**, including
model, adapter, bridge, callers and registrations; no new macros. The explicit
legacy bridge adds one counted unsafe declaration while retiring raw inventory
operations and the retained continuation pointer. This is a safety/ownership
step, not volume savings or completion of PLM safety. Local reproduction is
`Local/sm-decomp-experiments/20261022/plm-pickups`; its synthetic inputs remain
outside the repository.

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

**Adopted population lifecycle:** `enemy_population` owns the spawn search and
multipart sequence plus the different room/dynamic creation rules. It forbids
unsafe code and needs no Game, RAM or pointers. `enemy_population_ram` captures
owned ROM records, uses checked byte access, commits initialized fields and
spawn records before calling AI, and retains the original slot for post-callback
spritemap changes. Room creation leaves the callback's current selector live;
dynamic creation restores the caller's saved selector after the final part.
All five old entry points and the raw population layout/helper are removed from
bank A0; callers in 82/A0/A2/A6/A9 use the new module directly. Initializer calls
and native invalid-ROM diagnostics remain explicit unsafe interfaces. Existing
enemy execution, collision and persistent RAM state are not migrated by this step.

The original search does **not** restart its remaining-empty-probe count after
a failed candidate window. It can revisit already-counted empty slots and select
a region containing an occupied slot. Creation has a separate stopping rule:
requests for two parts create one, while requests for three normally create two.
These quirks are preserved rather than replaced with a conventional contiguous
allocator. Room initialization preserves script variables, always calls the
initializer and resets/selects its empty spritemap; dynamic initialization clears
the six script variables, gates the callback on bit 15 and otherwise retains its
spritemap. The definition's adjacent bank/hurt-time bytes still populate the
enemy's bank/field_2F word. Spawn-name copying deliberately skips source word five
and preserves the final six destination bytes.

Final-source verification passes **152 actual production comparisons** and
**404 controlled comparisons**, with full RAM, return and failure equality,
840 ordered initializer observations, 57 matching injected callback failures
and 16 matching invalid-address diagnostic cases. Cases include empty/full and
fragmented pools, signed part-count boundaries, name/no-name records, nested
spawning, callback deletion/selector changes, live ROM patches, wrapped population
addresses, connected deletion and nonzero/wrapping graphics indices. Seven checks
discriminate against conventional allocation/counting, premature ROM reads or
incorrect selector restoration. Six production/body comparisons separately match
the real no-op initializer route to its controlled counterpart.

Controlled checks use the original function bodies/macros and the new model and
adapter, with recording/mutating AI callbacks. The new adapter's private
`Game.loaded_rom` acquisition is substituted with the same initialized cartridge
image in this harness; linked production checks use the actual Game owner. No
cartridge replacement is claimed in those controlled callbacks. Odd-address
cases observe the optimized old raw accesses, without establishing defined
alignment for its explicit name-word loads. No movie ran, and these constructed
cases establish no room-route, boss or enemy-specific AI coverage.

Normal release library/frontend/verifier, all **115 embedded library inputs**
and installed native source checks pass. Complete cost is **+310 Rust lines**:
133 model, 379 adapter and 14 caller/registration lines, offset by 216 bank lines.
One additional counted unsafe helper names the shared initializer handoff; raw
RAM operations are removed from the migrated paths rather than hidden in that
helper. This is ownership/safety work, not volume savings. Local reproduction:
`Local/sm-decomp-experiments/20261022/enemy-population`.

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

### 11. Samus weapon control crosses projectiles, grapple and encounters

Follow-up source audit at `0f98f559`, checked unchanged through `4c979503` for the
cited gameplay and RAM metadata. This identifies ownership interactions, not
complete writer/alias closure. The aim, flare, feedback, cancellation, drain/X-ray
and protection experiments below have bounded execution coverage; other findings
are source evidence.

**Pending aim is a one-use handoff.** Normal-jump pose handling captures the
current pose's direction with flag `0x8000`; moonwalk-turn handling captures the
previous pose's direction with `0x0100`. A nonzero pending value forces beam input
onto the release branch, even while shoot remains held. Placement consumes the
pending value, uses its low byte and bypasses normal pose-direction rejection.
Origin height still comes from the live current pose. Without pending aim,
direction `0x10` falls back to the last-different pose's direction, again retaining
the current pose's origin height. Invalid default/fallback directions decrement
the reservation with wrapping arithmetic.[^aimproducers][^aimplacement][^aiminput]

Consumption, successful placement and phase expiry are different transitions.
Pending direction `0x17` is accepted, with origin indexing by its low nibble;
do not reject it just because projectile direction later contains lifecycle flags.
An invalid origin index can fail after clearing pending aim and storing direction,
before storing X/Y. Reservation rejection leaves pending aim available;
`Samus_Func10` later clears it. Captured aim, live origin and launch history thus
have three different observation contracts.[^aimplacement][^aiminput][^aimclear]

Placement's rejection result has different callers. Beam and missile firing
stop on rejection. Super-missile extension creation first copies its parent's
position/direction, ignores placement rejection, then initializes and links the
extension. Rejection decrements the count before extension creation increments
it, leaving the original count; copied origin/direction remain on this path.
Origin-table failure instead stops before that initialization. The typed follow-up
below exercises this distinction through the actual extension caller with a
controlled initializer.[^extensionplacement][^beamlaunch][^missilelaunch]

| State | Proposed owner | Observed interaction |
| --- | --- | --- |
| Charge/control (`flare_counter`) | Samus weapon control | Beam and grapple use one counter; other consumers include contact attacks, bomb spread, PLM cancellation and encounter transitions. |
| Flare frames/timers | Samus weapon appearance | Beam and grapple have different advancement rules; drawing currently advances them and can clear hyper's active charge marker. |
| Shot glow | Samus appearance | Firing starts it; palette handling advances it separately and can skip advancement under special palette routing. |
| Cooldown | Samus weapon control | Reservation and impact change it; encounter restrictions and X-ray also observe its exact value. |
| Projectile protection | Samus contact protection | Firing grants a global timer, Samus/projectile contact rejects while nonzero, and the completed gameplay tail decrements it. |
| Resume-charge sound request | Samus weapon sound continuity | Hurt/pose transitions publish it; draw-tail processing consumes, defers or republishes it. |

Grapple start sets charge to 1 and clears the resume-sound request. Its draw path
advances flare first, draws Samus/cannon, updates grapple tiles and then increments
charge toward 120 where applicable. Ordinary beam flare has transitions at
1/15/16/30; hyper uses `0x8000` as an active marker, cleared when its fast-spark
frame expires. Charge visuals and glow cannot become one generic per-frame timer.
The future visual consumer must receive results of these scheduled gameplay
transitions, even when backend drawing is omitted.[^weaponvisual][^grapplecharge]
[^weaponglow]

**Cooldown also encodes an encounter restriction.** Shitroid drain writes
cooldown 8, bomb count 5 and speed divisor 2. Ordinary weapon preparation advances
cooldown before the selected HUD handler; `Xray_Initialize` rejects the exact tuple
7/5/2. Mother Brain phases also write cooldown 8/0. A named restriction must
preserve this predicate and timing, including paths that skip preparation, until
all producers and consumers migrate. Treating cooldown only as a next-fire time
would lose this communication.[^weaponphase][^drainrestriction]

Projectile protection (`$18AC`) is independent of ordinary Samus invincibility
and of each projectile's lifetime. Firing sets it to 10/20 after placement;
ordinary pool release and `ResetProjectileData` do not directly clear it. Accepted
broad release indices can alias this field, and callbacks may change it. Samus initialization's
`$0A02..$0E0B` workspace clear excludes it too. A suspended gameplay pass does not
reach its normal countdown until the pass completes.[^beamlaunch][^missilelaunch]
[^contact][^weapontimers][^resetslots][^samusinit]

An ordinary resume-sound request can play sound and clear; subsequent echo/pose
logic in the same call can republish 1. A high-bit request instead skips that work
and becomes 1 for a later call. Message/unpause completion directly restores
movement sound through a different path. PLM cancel-charge clears only the counter;
mode/grapple transitions clear selected bundles. Projectile reset retains charge,
flare, glow, resume sound and protection. A blanket weapon reset or an immediate
Boolean sound event would change these rules.[^weaponsound][^weaponcancel]
[^grapplecharge][^resetslots]

The full projectile replacement must migrate these interactions with Samus weapon
control. Pool ownership does not imply ownership of ammunition, global protection
or an independent beam charge clock. The source audit's local notes are at
`Local/sm-decomp-experiments/20261022/player-weapon-control/RESEARCH.md`.

## Target: coordinated gameplay state machines

The proposed engine has explicit state and transitions for Samus, weapon flight,
creatures/encounters, room mechanisms, room/session lifecycle and progression.
Each state machine owns its rules and emits named outcomes consumed at specified
points by another owner. Movement can produce launch history and contact effects;
pickups can commit progression and request a message; a message can suspend a
room script. These are proposals derived from the contracts above, not completed
interfaces. The existing order, lifetime and byte-alias rules constrain each move.

The gameplay coordinator advances these state machines in the established order
and preserves continuations across suspension. Typed outcomes should capture
decisions such as movement replaced, projectile reflected, item acquired or room
script suspended. Apply them at their present observation points: a frame-end
event flush would lose the contacts, reflection timing and pickup/message
ordering traced above. Each owner can contain several internal state machines;
there is no requirement to make every instruction a public interface.

Definitions describe authored room, creature, equipment and attack data. Live
state describes what those definitions are doing now. Rendering/audio consume
presentation requests, while gameplay animation transitions remain with their
behavior owner. A compatibility adapter continues to encode byte-compatible RAM
until the migrated state is directly owned by the engine; its names and offsets
do not define the long-term behavior interface.

The concrete launch migration below is the first value model in this note with
explicit record, observation and clear transitions. The existing named gameplay
continuation owner preserves scheduling; it does not yet make the complete game
a collection of independent state machines. Contact, pose/animation, reflection,
room scripts, progression and mode-dependent effect storage remain migration
work. Their traced contracts provide the next interfaces to design and test.

## Launch-motion handoff: closed source trace

Follow-up source study on `dc20e538`, before adopting a typed launch model.
The complete directional records extend beyond the initially interesting
`$0DA8..$0DB7` range: the down fraction ends at `$0DB9`. Each row below is a
little-endian word; the names are established by `ram_map`'s `$0DA0` stride.
[^launchmap]

| Address | Meaning | Direct gameplay writer |
| --- | --- | --- |
| `$0DA8` | Absolute previous-movement Y fraction | `Samus_CalcDistanceMoved_Y`; initialization |
| `$0DAA`, `$0DAC` | Left integer, fraction | Horizontal collision result; `Samus_MoveLeft_NoColl` |
| `$0DAE`, `$0DB0` | Right integer, fraction | Horizontal collision result; `Samus_MoveRight_NoColl` |
| `$0DB2`, `$0DB4` | Up integer, fraction | `Samus_MoveUp`; `Samus_MoveUp_NoColl` |
| `$0DB6`, `$0DB8` | Down integer, fraction | `Samus_MoveDown`; `Samus_MoveDown_NoColl` |

Those are the named directional write sites: the horizontal helper has separate
left/right branches, followed by two vertical collision branches and four
no-collision functions. Each replaces the selected record with the actual
movement result; it does not accumulate movement or clear the opposite direction.
Left/up no-collision callers negate their supplied movement amount; preserve the
resulting 32 bits. `SetHiLo` writes fraction before integer, as does the explicit
downward pair.[^launchwriters][^splitwords]

`Samus_Func10` clears the eight directional words in left/right/up/down order,
integer then fraction, after firing and projectile handling. It leaves `$0DA8`
intact. `Samus_Initialize` clears the encompassing `$0A02..$0E0B` workspace and
initializes absolute Y distance to integer 1, fraction 0. Reset also bulk-clears
the region. `ResetProjectileData` clears projectile slots but does not clear these
movement records.[^inheritance][^samusinit][^resetlaunch][^resetslots]

The nonlocal coupling is in the launch reads. Define
`pack(fraction, integer) = (fraction >> 8) | ((integer & 255) << 8)`.
`initialize_speed` reads these four odd-address words:[^launch]

| Read | Exact composed value |
| --- | --- |
| `$0DA9` for leftward fire | `pack(absolute_y_fraction, left_integer)` |
| `$0DAD` for rightward fire | `pack(left_fraction, right_integer)` |
| `$0DB1` for upward fire | `pack(right_fraction, up_integer)` |
| `$0DB5` for downward fire | `pack(up_fraction, down_integer)` |

These are cross-record reads, not each direction's own 16.16-to-8.8 conversion.
Upward inheritance is additionally disabled when the low byte of `up_integer`
is zero; otherwise it becomes `(word($0DB1) >> 2) | 0xC000`. Launch adds/subtracts
the selected base speed with 16-bit wrapping. Down fraction is currently written
and reset but has no direct launch read in this traced implementation.[^launch]

`$0DA8` has a separate lifecycle. Normal scrolling computes absolute current-minus-
previous 16.16 Y displacement plus `1 << 16`, choosing the subtraction direction
by signed integer-position comparison. It runs after movement and contact, before
refreshing previous position. Slow grapple scrolling skips this calculation and
retains the previous fraction. Thus the launch input is sampled from a persisted
movement observation, not necessarily this update's measured vertical velocity.
[^distance]

There are three launch timings. Ordinary beam firing reads history during Samus
preparation. New missiles first initialize with base speed zero; their first
projectile update can initialize again in the same preparation. The delayed
acceleration branch tests a zero high byte, adds `$0100`, and initializes from
the new value; that branch always crosses into a nonzero high byte on its first
execution. It is not a multi-update countdown.[^beamlaunch][^missilelaunch]

Reflection adds another consumer: enemy shot reactions call `ProjectileReflection`
after preparation has cleared directional history. Beams immediately resample
that state and the independently retained absolute Y fraction. Missiles/supers
instead set their variable to 240 and reinitialize on their next projectile
update, after the intervening movement/scrolling opportunity. A launch snapshot
taken only when the player presses fire misses this interaction.[^reflection]

Useful counterexamples for experiments, derived from these source rules:

- Clear every directional record, leave absolute Y fraction `$8000`, fire left
  with base `$0400`: horizontal speed is `$FC80`, not `$FC00`.
- Leave left fraction `$8000` and right integer 1: rightward inheritance is
  `$0180` regardless of the right fraction. Combining only each direction's own
  integer/fraction changes it.
- Record two rightward movements: the second replaces the first. Summing them,
  or deriving launch history from net position change, is a different model.
- Reflect a beam after preparation, then a missile: immediate beam launch and
  next-update missile launch must read at their different execution points.

These are constructed/source-composed scenarios, not claimed movie observations.
The implementation now encodes `LaunchMotion` with four signed 16.16 records,
`record_movement`, `finish_projectile_phase` and `launch` transitions/observation.
`launch` receives typed aim, base speed and the separately sampled absolute-Y
fraction, and returns signed X/Y launch velocity. Its pure calculation preserves
the cross-record rules above. `launch_motion_ram` supplies the existing state at
the actual execution point and serializes the transition's writes.[^launchmodel]

All eight direct directional-history writes, their ordinary phase clear and
`initialize_speed` now use this model/adapter. Both new modules declare
`forbid(unsafe_code)`; the flight caller retains the legacy invalid-direction
callback and its unsafe contract. Global workspace initialization/reset remain
outside the directional transition owner. RAM is still the persistent storage:
Samus does not yet retain a `LaunchMotion` field directly. This is an implemented
focused migration, not a completed engine rebuild; validation results belong in
the experiment section, not in this source study.[^launchmodel][^samusinit]

The adapter also needs precise access order. `initialize_speed` clears X/Y
subpositions, reads direction, computes and writes Y speed, then reads horizontal
inheritance and writes X speed. Ordinary projectile slots are disjoint from
launch history, but arbitrary in-bounds indices can alias it: `k=$021C` clears
`$0DA8`; `k=$01BC` writes Y speed at `$0DAC`, affecting the subsequent `$0DAD`
read. Preserve that sequence or establish the accepted slot domain explicitly.
These cases are constructed aliases, not established reachable gameplay.
[^launch][^launchmap]

The selected movement writers keep no local RAM pointer across their collision
calls, but reflected launch runs below `EnemyMain`'s retained enemy pointer.
Its later reads rule out introducing a whole-RAM borrow in the adapter without
migrating that caller path. Copy values at the specified point, keep direct
byte accesses scoped, and preserve the remaining unsafe caller contract.
[^enemies][^reflection][^word]

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

### Persistent authority transfer before the full projectile replacement

An isolated experiment on `15589b48` keeps `LaunchMotion` alive across typed
movement, repeated firing and phase clear. Its `LaunchOwner` holds
`Option<LaunchMotion>`: publishing consumes that value before legacy execution;
import reconstructs it before typed execution resumes. Typed access while the
runtime owns the bytes fails. The bridge and model forbid unsafe code. This is
an unadopted transfer experiment, not a second field added beside live Game RAM.

**60 comparisons pass:** 50 compare the entire 128 KiB RAM image, eight compare
produced projectile fields during uninterrupted typed execution, one compares
complete frontend snapshot bytes and one compares an actual bus read. Fourteen
discriminating checks reject missing publication, a stale value after an external
write/restore, or typed access during runtime ownership. The linked library's
115 embedded inputs and all seven experiment source inputs match their frozen
bytes. The release component DLL compiles; no movie ran.

The experiment calls the real banked/mirrored/CPU/WRAM-port writers, replay codec
and decoder, `snes_saveload`, and hard/soft `snes_reset`. It also exercises the
exact oracle capture/restore bodies, including their audio methods, using local
structurally equivalent snapshot types because those implementations are private.
The restore-before-Rust and adopted-result schedules use controlled execution,
not actual CPU/Rust frames. The checkpoint check uses the exact RAM archive stage,
including partial writes before truncation; the complete checkpoint file and
hardware restore are outside this experiment. Nested entry, suspension and a
failing callback are controlled operations. Legacy off-slot launch is explicitly
given runtime authority, preserving its Y-store/history/X-read alias sequence.

This supports publish/take/import at the selected execution points. It does not
establish that every production entry uses those points, prevent arbitrary native
RAM writes while a future owner is active, or implement the whole projectile
lifecycle. The next draft must close those access paths and include all retained
legacy execution, caller and runtime costs. No maintained code changes or savings
are claimed from this prototype. Reproduction remains outside the repository at
`Local/sm-decomp-experiments/20261022/projectile-authority` (`prepare.py`,
`build.py`, `check.py`); it uses the frozen population library and synthetic fixture.

### Whole-projectile draft

The latest reviewed stage of this unadopted candidate passes **3,427 full-RAM
comparisons**, **3,366 ordered observations** and **483 matching failures**.
It includes beam input/firing, typed pending aim and placement, initialization,
launch, flight, acceleration, extensions, release, animation, trail-frame sampling,
drawing decisions, beam/hyper/grapple flare, palette/glow, sound continuity and
weapon selection/cancellation. Beam firing and extension creation call owned
placement, initialization and launch directly. Their former legacy
effects and publish/import round trips are removed. Broad-index placement and
launch use checked adapters and the same pure models. Launch preserves Y writes
before aliased history rereads and the diagnostic order; other out-of-pool paths
retain reference bodies. Firing owns one `Flare` charge/frame/timer bundle, with
the complete `Firing` value moving through each remaining effect. Feedback reads
that live charge, and cancellation acts on it directly; there is no separate
`Control` or duplicated charge input. Shared Samus palette/echo writers outside
this draft still prevent a claim of complete authority.

State crosses remaining foreign operations by value. Returned inputs are sampled
again; invalid-table handling publishes partial state and diverges. Cases include
charge/reservation rules, pending/default/fallback aim, ignored extension rejection,
live camera changes, odd selectors, callback redirection, partial failure,
definition edits/replacement, damage-write ordering, script deletion/goto and
Ceres coordinate quirks. Linked cases exercise firing-to-flare, grapple drawing
before tiles/charge increment, and nested firing during a draw callback. Persistent
sequences retain typed state without intermediate RAM decoding. The latest
composition spans PLM charge clear, inactive grapple, unavailable missile and
accepted beam selection, 24 firing updates, flare/palette, and grapple selection.
It retains the full owner between steps; remaining legacy effects still transfer
state. The sound routine's original debug/time-up tail follows its owned prefix.

The coordinator independently checked all **115 embedded library inputs** against
`15589b48` and **30 compiled draft inputs/hashes**, reviewed the integrated
feedback/cancellation source and inspected the result without repeating its run. The
preceding launch stage separately passed 1,945 comparisons, 1,638 observations
and 227 matching failures; its twenty compiled draft inputs were also checked.
The two reused launch-motion files also match current-main bytes in this stage.
Reproduction: `Local/sm-decomp-experiments/20261022/player-projectiles/cancellation-stage`.
The older `beam-stage/` (202), `flight-stage/` (425), `animation-stage/` (1,295),
`aim-stage/` (1,634), `launch-stage/` (1,945), `flare-stage/` (2,330) and
`feedback-stage/` (2,772) remain preserved. Counts are not additive or
unchanged-suite comparisons: controlled initializer, placement and
launch cases changed as those calls became owned.

Collision, drawing/trail consumers, special attacks, palette output and native
diagnostics remain controlled in this stage. Definitions are synthetic; no movie
ran. The harness imports at entry and publishes on return;
global writer/alias closure, production runtime binding, remaining modes/trails/
reset, named definition/motion interfaces and raw-address retirement remain open.
Neither production promotion nor a complete price or saving is established.

The subsequent **reset-stage** passes **89 targeted full-RAM comparisons**, with
142 ordered observations and 22 matching failures. Its canonical pool reset
retains fractions and auxiliary words while clearing other shot state and the
counts/cooldown. Charge, flare, glow and protection are not directly cleared. A
checked environment adapter resets Power Bomb, HUD and echo state, then handles
suit-target palette and a live reread of hyper state. Tests include callback
changes, recursive reset/failure, both reset/release orders, firing-reset-firing
and palette-reset-palette with one `Firing` value. The coordinator checked all
115 library and 34 local compiled inputs and reviewed the result without rerunning
it. Twenty-eight prior compiled files are unchanged; engine changes add three
effect variants, with separate harness wiring. **The previous 3,427 suite was not
rerun**, so the counts are separate. The preserved source is `player-projectiles/reset-stage`
under the same experiment root. Palette execution and the remaining environment
handoff are controlled; this is not runtime integration or complete owner closure.

The subsequent **encounter-stage** passes **686 targeted full-RAM comparisons**,
1,921 ordered observations and 11 matching failures. It applies drain refresh and
release to the existing `Firing` cooldown and bomb reservations, with movement drag
supplied live by Samus. The actual HUD selector and beam-input consumer now execute
around the same drain/X-ray cases, plus eight switching/charged-release compositions.
The coordinator independently checked all 115 library and 41 draft compiled inputs
and reviewed the source/result without rerunning the suite. Neither the earlier
3,427-case suite nor reset's 89-case suite was rerun.

The final composed sequence retains `Firing` between refresh, preparation, query,
release and reset; `Legacy::perform` still publishes/imports at drag and environment
effects. It is not a sequence without RAM decoding. The actor/X-ray coordinator
remains legacy code; movement-specific HUD routing, projectile processing, HDMA,
palette and other effects remain controlled. There is no duplicate `Restrictions`
owner, production binding, movie coverage or complete replacement price. The
preserved stage is `player-projectiles/encounter-stage` under the same local root.

The reviewed **missile-stage** adds **334 targeted full-RAM comparisons**, with
253 ordered observations and 60 matching failures. Normal and super missiles use
the existing reservation, placement, launch and animation owner; the shared
reservation rule is also exercised through beam firing. It keeps the count's
wrapping comparison and low-byte cooldown gate. Empty ammunition or an occupied
missile pool reverses the count increment but retains cooldown 1. Beams instead
reuse their last slot after successful reservation. These are distinct caller
policies around the same mechanic.[^beamlaunch][^missilelaunch]

The candidate represents ammunition expenditure and HUD cancellation as ordered
Samus effects. Later selection, kind and remaining-ammunition queries are sampled
after intervening effects. The actual source and fixtures cover sound callbacks
that change selection/type or reenter firing, invalid-definition failures and
automatic selection cancellation. Eight compositions explicitly require success
into flight/animation and super-missile extensions, or missile-reset-beam firing
with the later protection grant overwriting 20 with 10.

The coordinator independently checked all **115 library inputs** and **44 compiled
draft inputs/hashes**, both actual missile reference functions and tables, and the
reviewed shared reservation/codec changes. Thirty-eight prior compiled files are
unchanged; `engine.rs`, `codec.rs` and harness wiring change, with three new missile
files. The reported result was reviewed without rerunning it or the prior suites.
Named missile motion variants replace their numeric encoding in the model.
Trail/collision output, sound and other native effects are controlled. No movie,
production binding or full replacement price is established. Preserved source:
`player-projectiles/missile-stage` under the same local experiment root.

### Integrated animation and trail definition owner

The reviewed **definition-stage** puts `Animation<Cursor, Sprite>` directly inside
each shot. Generic playback and trail sampling use scoped access through `Owner`;
definition operations take and return that complete owner by value. The encoded
provider alone interprets playback opcodes and cursor strides. An animation copy
does not survive beside a live `Firing` through a diagnostic.

**1,992 selected full-RAM/return comparisons pass**, with **1,569 ordered
observations** and **287 matching failures**: 348 playback cases, 351 trail cases,
72 complete-owner callback cases, 798 animation consumers, 89 reset cases and
334 missile/reservation cases. Diagnostics mutate charge, counts, protection,
feedback and another projectile, or recursively reset the owner. Those changes
survive the resumed animation operation. Canonical playback release invokes the
existing owned release; the reference invokes actual `ClearProjectile`. Out-of-pool
execution still uses retained reference bodies.

The selected scopes deliberately differ from the standalone playback's 363 cases:
release is now actual behavior rather than that experiment's controlled release
effect. The 3,427-case cancellation suite and 686-case encounter suite were not
rerun. These are selected consumer checks, not an additive full-projectile total.

Three native-definition sequences pass, including one using the **same
`Firing<Label, Sprite>` data structure**. It retains other-shot damage, shared
charge, deferred sound state and protection while sampling and advancing named
frames with no byte image or RAM/ROM decoding. Its terminal result is `Release`;
the native sequence does not execute the full release operation. At this stage,
firing, flight, initialization, reset and drawing remain bound to the encoded
variant. Initialization is extended below; the generic data structure alone does
not establish a completely native weapon lifecycle.

The coordinator checked **115 embedded library inputs** against `15589b48` and
**50 compiled draft inputs/hashes**, inspected the changed field bundle, codec,
providers and harness, and reviewed the result without rerunning it. Forty of the
prior 44 compiled files are unchanged; only `engine.rs`, `codec.rs`, `animation.rs`
and harness wiring change, with six new definition/provider/check files. Models
and checked providers forbid unsafe code. Sound, collision, drawing, trail output
and native diagnostics remain controlled; definitions are synthetic. There is no
movie or production integration result.

The changed model/provider cohort is **989 -> 1,194 formatted lines (+205)**:
engine, codec and animation versus those files plus playback, trail selection
and the encoded provider. Native example/assertion sources are another 78 lines.
Retained reference paths and runtime transport are not priced by that comparison;
it is an architecture cost, not a complete saving. Initialization, remaining
definition consumers and global writer/alias/runtime closure stay open.
Preserved source: `Local/sm-decomp-experiments/20261022/player-projectiles/definition-stage`.

### Named weapon initialization

The reviewed **initialization-stage** adds `Firing::initialize_from` with named
weapon definitions. It shares the actual owner used by playback and trail
sampling. The generic algorithm has no RAM, `Game`, encoded address or opcode;
the provider maps a weapon to a captured launch definition and an entry frame.
The five initialization policies preserve the existing ordering:[^projectiledefinitions]

| Policy | Damage and entry behavior |
| --- | --- |
| Ordinary projectile | Capture direction; diagnose invalid damage before writing it, then reread the retained definition. Write the entry before resolving radii; set duration last. |
| Fixed super missile, bomb or special attack | Write damage before its diagnostic, then select the fixed entry. Preserve existing radii. |
| Echo | Write damage before its diagnostic, then use captured direction to select the entry. Preserve existing radii. |

Definitions receive and return the complete owner. A damage diagnostic can mutate
another shot, charge, feedback or protection, or reset the owner. The retained
launch view observes edits to its original allocation; a later radius lookup
opens the then-current allocation. Replacing the ROM does not redirect the
already captured launch view. Eagerly copying all fields would lose this rule.

**786 selected full-RAM comparisons pass**, with **491 ordered observations** and
**178 matching failures**: 159 diagnostic/owner cases, 293 initialization and
firing/extension cases, and 334 missile/shared-reservation cases. Five additional
native-definition sequences initialize the **same `Firing<Label, Sprite>`**, then
sample trails and play animation to a `Release` result without RAM/ROM decoding.
They retain surrounding weapon state; they do not perform full native release,
firing selection, movement or rendering. These are synthetic definitions.

The coordinator reviewed the result, actual initializer bodies, provider and
native sequences without rerunning the suite. All **115 embedded library inputs**
match `15589b48`; **53 compiled draft inputs** match local source/hashes, with
46 prior files unchanged. Optimized compilation and formatting passed. The
formatted model/provider cohort was independently recounted at **1,194 -> 1,265
lines (+71)**; the native example/assertions add 70 separate lines. This is not a
whole-replacement price or production saving. Diagnostics remain controlled,
broad-index fallbacks remain, and no movie or production integration is claimed.
Preserved source: `Local/sm-decomp-experiments/20261022/player-projectiles/initialization-stage`.

### Named projectile animation definitions

A standalone experiment separates projectile playback from instruction encoding.
`Animation<Cursor, Sprite>` holds duration, selected sprite, contact radii and the
next cursor. `advance` interprets semantic `Show`, `Jump` and `Release` commands
through a definition provider. It contains no `Game`, RAM/ROM address, opcode or
hardware call. The compatibility provider contains the encoded cursors and
instruction decoding; a second provider uses ordinary Rust labels, sprites and
nodes. Playback and both providers forbid unsafe code.

**363 actual old/new full-RAM comparisons pass:** 224 direct playback cases, 135
callback cases and four linked steps, with **204 ordered observations** and
**58 matching failures**. The reference executes the frozen production `advance`,
`get_ProjectileInstr` and instruction-dispatch macro with its original macro
support. Cases cover normal, odd and broad valid selectors; countdown boundaries;
show/jump/release/unknown commands; low-bank diagnostics; wrapped jump operands;
definition replacement/mutation; selector/timer/cursor changes; and partial
failure.[^projectileplayback]

The compatibility provider publishes partial animation state before a diagnostic,
imports the returned state for the captured slot, then captures the current
definition allocation. It preserves frame-field read order, a jump to zero ending
the step, wrapped cursor arithmetic and unknown-command recovery. A changed live
projectile selector does not redirect the captured slot. Numeric encodings stay
in this provider, including the unusual failure behavior; eager immutable decoding
of every existing definition is not established by these cases.

One additional persistent sequence runs the **same playback algorithm from Rust
definitions without any byte image or RAM/ROM decoding**. Named `Start`, `Redirect`,
`Burst` and `Done` nodes demonstrate duration waiting, jump, changed sprite/radii
and release. This is a synthetic authored example, not extracted game content or
additional old/new comparisons.

The optimized DLL and source-checked run pass. All **115 embedded library inputs**
match `15589b48`; all **seven compiled experiment inputs** match exact local hashes.
Release and diagnostic callees are controlled: actual `ClearProjectile` internals
are not executed. No ROM or movie is loaded. Initialization, trail readers, attack
definition conversion, complete malformed-data coverage and production integration
remain open. Integrating this seam requires moving the **complete `Firing`** through
diagnostics; retaining its copied `Animation` while another owner can mutate that
shot would violate the ownership contract. The later reviewed definition stage
implements that complete-owner transfer for playback and sampling. No full
replacement cost or savings are established. Reproduction:
`Local/sm-decomp-experiments/20261022/named-projectile-animation`
(`playback.rs`, `compatibility.rs`, `native.rs`, `harness.rs`, `prepare.py`,
`build.py`, `check.py`, `result.json`).

### Named trail sampling

The follow-up reuses the same generic playback and definition interface, adding
`TrailDefinitions::sample_trail` and a pure `Upcoming`/`Active` selection. With one
tick remaining, an upcoming timed frame supplies the trail marker even before
playback selects its sprite. An upcoming command, or any other remaining duration,
uses the active sample. The compatibility provider implements that active sample
as the record eight bytes before the captured cursor. It is not an unconditional
cache of the last displayed frame; imported cursors and callbacks can differ.

**351 actual old/new full-RAM and return-value comparisons pass:** 224 trail cases,
120 callback cases and seven linked sample/playback steps. There are **263 ordered
observations** and **73 matching failures**. The reference executes the actual
frozen `trail_frame` and linked `advance`. Normal, odd and broad valid selectors,
timer boundaries, timed frames and commands, wrapping to the preceding record,
selector/cursor/timer mutation, definition mutation/replacement and partial
diagnostic failures are exercised.[^projectiletrailframe]

The query captures its cursor and timer decision before diagnostics. It resolves
the selected definition again before reading the trail marker, even when the
first check sampled that same record. A second diagnostic can replace the
definition allocation; retaining the first frame view would change the result.
The model preserves that observation point through its provider interface.
Both this model and the checked compatibility provider forbid unsafe code.

One separate persistent sequence uses named Rust definitions with no byte image
or RAM/ROM decoding. It explicitly samples a `Burst` trail while the visible sprite
is still `Charge`, then advances playback, and checks command fallback. This is
synthetic authored content, not an additional differential count.

The optimized build and source-checked run pass against `15589b48`: all **115
library inputs** and **nine compiled local inputs** match. Native diagnostics
are controlled. The sequence does not execute trail allocation, Mode 7 position,
offset tables, left/right trail scripts or drawing. Full-`Firing` diagnostic
transport is addressed by the later definition stage; production integration,
complete malformed-data coverage and the full replacement cost remain open.
No ROM or movie ran. The earlier 363-case playback
suite was not rerun. Reproduction:
`Local/sm-decomp-experiments/20261022/named-projectile-trail-selection`
(`trail.rs`, `compatibility.rs`, `native_trail.rs`, `harness.rs`, `prepare.py`,
`build.py`, `check.py`, `result.json`).

### Owned trail emission and placement

The standalone `owned-trail-emission` experiment gives 18 trail pairs typed left
and right state: duration, script cursor, tile and position. Generic `emit` and
`place` use a definition/context provider; neither contains `Game`, RAM/ROM
addresses, opcodes or unsafe code. Reservation searches pairs from last to first
using **only the left duration**. A live right side does not prevent reuse, and
its old tile remains until playback changes it.[^trail]

Emission chooses the style from the argument projectile, commits both durations
to one, then installs the left and right scripts in order. It next captures the
live selected projectile for placement. Placement captures its trail marker,
samples origin or performs Mode 7 projection, and then reads that captured
projectile's live type/direction for offsets. It writes left Y, left X, right Y,
right X with signed wrapping offsets and the existing minus-four adjustment.

The entire trail owner moves through script lookup, sampling, projection and
offset-definition diagnostics. In particular, **a failed first script read keeps
both durations; a failed second read also keeps the installed left script**.
Allocation is an ordered transition with partial results, not an atomic operation.
Callbacks can change the pool or recursively emit another trail; the resumed
operation uses the returned owner. Offset views retain their allocation while
later diagnostics can select a replacement for the next lookup.

**466 actual old/new full-RAM/completion comparisons pass**: 250 allocation cases,
54 placement cases, 160 callback cases and two checked script-read failures.
There are **461 matching ordered observations** and **108 matching failures**.
The reference executes the extracted production `SpawnProjectileTrail`,
`ProjectileTrail_Func5`, `trail_frame` and `get_ProjectileInstr`; all four bodies
and their three current source files match the frozen reference. Cases cover
distinct argument/selected projectiles, odd/broad source selectors, full/free
pools, all three offset families, live camera/type/definition changes, nested
emission and failure after partial mutation.

One additional persistent native sequence uses named scripts and ordinary signed
offset definitions without a byte image. It checks reuse with an active right
side, retained tile, coordinate wrapping, subsequent allocation and full rejection.
The formatted optimized build and source-checked run pass: **115 library inputs**
match `15589b48`, and **10 experiment inputs** match compiled bytes/hashes. Five
sampling/support inputs are unchanged from the earlier trail experiment.

Mode 7 projection and diagnostics are controlled. The destination is limited to
canonical pairs; broad destination indices, side playback/drawing, reset and
complete-Firing transport are not established here. The standalone owner is the
whole trail pool. Integration must move the complete projectile owner through
these effects, rather than retaining a detached pool while callbacks change it.
The new formatted model/provider cost is **271 lines** (91 model, 180 adapter),
plus 87 native-example lines; reused support, harnesses and production retirement
are outside that partial count. No complete price, adoption, ROM or movie result
is claimed. Reproduction: `Local/sm-decomp-experiments/20261022/owned-trail-emission`
(`emission.rs`, `environment.rs`, `native.rs`, `harness.rs`, `prepare.py`, `build.py`,
`check.py`, `result.json`).

The subsequent full-owner `player-projectiles/emission-stage` now records **603
selected comparisons** through emission, actual flight and missile callers,
reset and checked table failures. It puts the trail pool inside `Firing` and uses
that owner for style and projectile selection as well as callback transfers.
Its scope and partial cost are recorded in PLAN; this extends the standalone
owner described above. Side playback/output is a separate integration step.
The coordinator independently checked all **115+62 compiled inputs**, the direct
flight-to-emission call, complete-owner style/selector reads and checked-table
transfers. Forty-seven prior files are unchanged. The selected suite contains
484 emission/placement, 24 flight, four missile, 89 reset and two initializer
table-failure cases, with 718 ordered observations and 143 matching failures.
The latter failures retain the sound-resume clear performed after its callback.
The formatted model/provider/codec cost was independently recounted at
**1,257 -> 1,618 (+361)**; 54 authored native-example lines are separate. The
review did not rerun the suite. The native sequence retains surrounding state in
the same `Firing<Label, Sprite, Trail>`; its trail sprites still use the encoded
tile representation and it does not perform side playback or rendering.

### Owned trail playback and drawing

The `owned-trail-playback` experiment adds a generic side state machine to the
emission model. Its input includes the pass's captured freeze decision. Named
`Frame`, `Move` and `Unknown` instructions operate on generic cursors and sprite
identities. `Move` names its target side explicitly; it can change the opposite
side of the pair. The model processes pairs last to first, advancing and drawing
left before right. The complete owner moves through definition lookup and
presentation; no detached side survives an effect.[^trailpipeline]

Duration expiry captures a local cursor, follows movement commands and commits
the next duration. A zero duration ends the side while retaining its old tile and
stored cursor. A nonzero duration resolves the definition again for the sprite,
then installs the next cursor. The output adapter preserves live camera reads,
capacity, clipping and ordered byte writes. It imports the pool after output,
because accepted broad output indices can alias trail state. The generic model
and checked adapters forbid unsafe code.

**537 actual old/new full-RAM/completion comparisons pass**, with **2,046 ordered
probe/callback events** and **201 matching failures**. The selected cases are
96 timing, 36 commands, 96 output, 288 callback, eight outer drawing callers,
six explicit contract checks and seven linked emission/reset/playback steps.
The reference contains the actual trail dispatcher, side macros and
`HandleProjectileTrails`, plus the `draw_projectiles` caller. Probe events record
order; diagnostic/caller events also capture incoming RAM and the failure flag.
No game ROM is loaded; definitions are synthetic.

The six explicit checks establish the following distinctions:

| Contract | Observed consequence |
| --- | --- |
| Cross-side commands and draw order | The right script can change left state after the left sprite was emitted. |
| One freeze decision | A diagnostic setting the freeze flag does not freeze the remaining sides of that pass. |
| Fresh sprite lookup | A later definition edit changes the sprite without changing the duration already committed. |
| Failure after duration commit | A failed second lookup retains the new duration and old stored cursor/tile. |
| Y clipping after X output | The output bytes can change while the output cursor remains unchanged. |
| Aliased output | An X write into the current trail's Y field changes the following Y read. |

The seven linked steps retain the typed pool between operations. They emit,
advance, run the actual projectile reset and continue playback. Reset's direct
stores preserve the live trail-pair durations while clearing the shots' emission
countdowns. During that legacy reset interval the typed owner is consumed,
published and reconstructed. This sequence uses the existing reset body; it does
not replace reset or prove complete-Firing playback integration.[^resetslots]

One additional native sequence uses **named script and sprite enums** with no
byte image, RAM/ROM lookup or output-byte adapter. The same emission and playback
algorithms exercise cross-side commands, left/right presentation order, a frozen
pass, duration waiting, expiry and subsequent pair reuse.

The final formatted optimized build and source-checked run pass. **115 embedded
library inputs** match `15589b48`; **15 experiment inputs** match compiled bytes
and hashes. The extracted dispatcher/macros/caller and reset correspond to the
frozen source, and three relevant gameplay files match current main. The earlier
reset experiment's transfer support is reused unchanged. New model/adapter code
is **221 formatted lines** (106 playback, 115 adapter); generic sprite support
adds two lines to the 91-line emission model. The native example adds 142 lines.
This **223-line partial cost** excludes reused adapters/support, harnesses and
production retirement; it establishes no whole-replacement price or saving.

The drawing reference retains its existing potentially unaligned raw stores.
These comparisons establish the observed Windows native output, not defined
portable Rust behavior or general aliasing safety of the old code. The candidate
uses checked byte writes. Diagnostics, ordinary projectile/echo drawing before
trails, and reset palette calls are controlled. Actual sprite hardware consumption,
complete-Firing output, runtime integration and malformed-data closure remain open.
No movie or production adoption is claimed. Reproduction:
`Local/sm-decomp-experiments/20261022/owned-trail-playback`
(`side_playback.rs`, `side_adapter.rs`, `emission.rs`, `native.rs`, `harness.rs`,
`prepare.py`, `build.py`, `check.py`, `result.json`).

### Definition consumers still to migrate

This source trace identifies the contracts a named attack/animation definition
must support. The initialization and emission experiments above now exercise
their stated subsets; this table adds no execution coverage:

| Consumer | Required behavior |
| --- | --- |
| Ordinary projectile initialization | Capture direction and definition selection; check damage before publishing it, then reread damage from the retained definition after any diagnostic. Publish the directional entry before resolving initial radii; set the timer last. |
| Fixed-entry and echo initialization | Publish damage before the invalid-damage diagnostic. Fixed-entry setup then takes one entry; echo selects by captured direction. Neither directly initializes radii. |
| Trail allocation | Select trail scripts from the argument projectile's type, search only left-side timers for a free pair, and publish both timers and scripts before positioning. Positioning receives the live projectile selector. |
| Trail positioning | Capture the trail marker first. Mode 7 projection precedes the captured projectile's live type/direction lookup. Apply signed offsets with wrapping; keep left-Y, left-X, right-Y, right-X write order. |
| Trail side playback | Zero timer means inactive; a zero-duration frame ends that side without updating its tile or stored cursor. Commands choose their target side explicitly, which need not be the executing side. Frozen drawing skips progression but can still draw an active side. |
| Trail drawing | Capture the freeze decision once, visit pairs last to first, and advance/draw left before right. Preserve the live camera and output-capacity checks between sides. |
| Projectile reset | Clear each shot's emission countdown (`projectile_timers`); the routine's direct stores do not clear the visual trail pairs. Keep subsequent palette callbacks as separate effects. |

The initializer retains a definition allocation across the damage diagnostic but
observes its live contents. An eagerly copied immutable record would change
mutation/replacement cases unless the compatibility provider preserves those
reads. Native authored definitions can be immutable; that does not make legacy
definition capture eager.[^projectiledefinitions]

Trail emission runs during projectile flight before the later animation advance.
Its separate side scripts have different zero-timer and command rules from
projectile playback. Sharing named definition access is supported; one generic
script executor has not been established. Trail script choice, position sampling,
pair reservation and side playback therefore remain distinct operations. The
remaining closure includes side playback/drawing, complete-owner emission and
actual producer/consumer integration.[^flight][^trailpipeline]

The side interpreter is called by `draw_projectiles`, after ordinary projectile
drawing and shinespark echoes. It captures the freeze flag once for the whole
pass. On a duration expiry, it follows a local script cursor through movement
commands; callbacks can change the pool, but do not replace that captured cursor.
The next duration commits before the tile is looked up through a fresh definition
resolution. A zero duration preserves the stored cursor and tile. The three known
commands name left-down, right-down and left-up; either side's script can dispatch
any of them. Restricting command access to the executing side would narrow the
current contract.[^trailpipeline]

Drawing also has a partial-write contract in the compatibility renderer. After
capacity and X clipping succeed, it writes X as a word before checking Y. A Y-clipped
sprite can therefore change the output bytes without advancing the output cursor.
This belongs in the output adapter while the domain requests a trail sprite; an
atomic visible-sprite append would not reproduce the current output image.
The playback experiment above now exercises these source findings through its
stated native-output scope: cross-side commands, repeated definition resolution,
frozen passes, left/right output ordering, clipping with partial output, and
emission followed by projectile reset and side playback. Complete-owner/runtime
integration remains a separate requirement.[^trailpipeline][^resetslots]

### Aim capture, placement and expiry

An isolated experiment against frozen `15589b48` executes the production bodies
of `InitProjectilePositionDirection`, `SamusFunc_F468_NormalJump`,
`SamusFunc_F468_TurningAroundOnGround`, `HudSelectionHandler_TurningAround`,
`Samus_Func10` and beam input/firing. It uses the real compiled `Game` and RAM
macros with synthetic pose definitions; no game ROM is loaded.

**68 full-RAM fixture comparisons pass**, including **14 expected partial
failures**. These compare the real placement/cancellation behavior with explicit
expected state, not an old/new typed replacement. Indices include 0/1, 18/19,
`0x1bc`, `0xfffe` and `0xffff`, all within the actual RAM allocation. Cases cover
pending/default/fallback directions, wrapped reservation decrement, low-byte
consumption, retained current-pose origin and partial writes on origin-table failure.

**Eight linked sequences** additionally inspect selected state and callback
observations: two placements, pose producer to placement, moonwalk turn, reservation
rejection then phase clear, and held-shoot beam release. These are not full-RAM
old/new sequence comparisons. **Four counterexamples** reject a Boolean aim flag,
premature clearing, ignoring fallback and replacing captured aim with live aim.

Other firing callbacks, shinespark and the launch-history clear callee are
controlled. The check validates all **115 embedded library inputs** against the
frozen Git source and **five local compiled inputs** against exact hashes. The
release experiment compiles and passes; no movie ran. This establishes the selected
handoff behavior, not complete pose/weapon writer closure or gameplay reachability
of unusual indices. Reproduction: `Local/sm-decomp-experiments/20261022/aim-handoff`
(`prepare.py`, `build.py`, `check.py`, `result.json`). No production change or
saving is claimed.

### Typed aim and placement follow-up

The next isolated prototype implements `PendingAim` capture, consumption and
expiry, plus `PlacementAim` default/fallback selection and live-origin placement.
Its domain model contains no RAM/ROM addresses or `Game`; the separate checked
adapter projects results in their original order. The model and adapter both
forbid unsafe code. It preserves imported capture-marker residue for compatibility
without collapsing the captured direction into a Boolean flag.

**383 actual old/new full-RAM comparisons pass:** 322 direct cases, 37 linked
steps and 24 extension-caller compositions. They include **149 ordered full-RAM
callback observations** and **44 matching failures**. The actual jump/turn/phase
bodies replace only pending-aim capture/expiry stores; placement is replaced in
full. Real beam firing and extension creation compose with both placements, with
their other callees controlled. All ten origin directions, both posture tables,
odd/broad selectors, coordinate wrap, invalid directions, pending/default/fallback
selection and writes before bounds failure are covered.

The extension cases exercise ignored rejection, pending consumption, invalid
origin failure and selector redirection by the controlled initializer. An explicit
check confirms that rejected placement still initializes the copied extension
and balances its count decrement/increment. Separately, 1,024 marker/direction
codec cases round-trip and **one typed-owner sequence** captures/replaces/consumes/
expires pending aim without intermediate RAM decoding. Those are distinct from
the 383 full-RAM comparisons.

All **115 embedded library inputs** match `15589b48`, and all **ten compiled local
inputs** match their exact hashes. The optimized experiment DLL builds and passes
with synthetic pose definitions and the frozen library's preserved native-music
ABI. No game ROM or movie is used. Broad-index writes remain compatibility-adapter
operations; the surrounding producer bodies and controlled effects are not an
unsafe-elimination result. This code is supplied for the full projectile draft,
not adopted as a persistent production owner. Global writer/alias and runtime
closure, full integration cost and gameplay-route coverage remain open.
Reproduction: `Local/sm-decomp-experiments/20261022/owned-aim` (`aim.rs`,
`adapter.rs`, `prepare.py`, `build.py`, `check.py`, `result.json`).

### Scheduled beam and grapple flare transitions

A pure `Flare`/`Pass` model now advances ordinary beam flare, hyper flare, grapple
flare and the separate grapple tile-then-charge phase. One charge/frame/timer
bundle crosses sound, drawing and definition diagnostics by value. Its definition
interface names flare parts and duration sequences; bank addresses live only in
the adapter. Both model and checked adapter forbid unsafe code. This is supplied
for integration with the same weapon-control state, not a second persistent charge
copy alongside the projectile draft.

**324 actual old/new full-RAM and failure-flag comparisons pass:** 214 beam/hyper
cases, 102 grapple cases and eight linked steps. There are **529 ordered full-RAM
callback observations** and **66 matching failures**. Checks execute the frozen
bodies of `HandleChargingBeamGfxAudio`, `HandleGrappleBeamFlare` and
`UpdateGrappleBeamTilesAndIncrFlameCtr` against the typed replacement. They cover
charge/timer thresholds and wrap, restart/loop durations, live frame changes and
definition replacement during diagnostics, post-draw charge changes, clipping,
mode changes during sound and failures before subsequent transitions.

Normal beam drawing rereads charge after each part. Its mode is selected on entry;
a later callback changing the hyper flag does not switch the active branch.
Hyper advances fast spark, slow spark and main flare even when fast-spark expiry
clears charge. Grapple observes charge before drawing and increments it in the
later tile phase, after that phase's callback returns. One separate typed sequence
exercises these phases without RAM decoding and checks hyper's three draw outcomes.

All **115 embedded library inputs** match `15589b48`, and all **seven compiled
local inputs** match exact hashes. The optimized DLL builds and passes. Duration
and sprite definitions are synthetic; native diagnostics, actual sprite geometry/
output, sound and tile updates are controlled. Palette/glow advancement, sound
resumption, grapple start/release, cancellation and encounter restrictions are
outside this experiment. No movie, production adoption, full weapon ownership or
saving is claimed. Reproduction: `Local/sm-decomp-experiments/20261022/weapon-flare`
(`flare.rs`, `adapter.rs`, `prepare.py`, `build.py`, `check.py`, `result.json`).

### Weapon glow and sound continuity

A pure feedback model makes glow progression and sound continuity explicit with
`ResumeRequest::{Idle, Ready, Deferred}` and named palette/sound effects. Imported
request payloads survive until the original transition consumes them. Charge is
a live input from the shared flare owner. The model and checked adapter forbid
unsafe code; the model has no `Game` or RAM/ROM addresses. The prototype's grouping
does not establish exclusive ownership: the palette cursor serves other Samus
palette modes, and movement also writes the echo-sound flag.

**390 actual old/new full-RAM comparisons pass:** 177 palette cases, 160 sound
cases, 48 direct restoration cases and five linked steps. They contain **535
ordered observations** and **96 matching failures**. Frozen source bodies include
`HandleBeamChargePalettes`, the outer `Samus_HandlePalette` dispatcher,
`HandleMiscSamusPalette`, `Samus_ShootCheck` and `QueueSamusMovementSfx`.
The replacement changes only the sound prefix of `Samus_ShootCheck`; its exact
debug/time-up tail stays in the coordinator. A deferred sound request becomes
ready and skips the ordinary sound/echo/spin work, while that tail still executes.

Ordinary glow decrements before highlight/restoration selection. Hyper palette
application precedes decrement and its live reread. Charge-palette progression
likewise rereads the phase after the palette callback. Special-palette routing can
skip charge handling entirely. A ready sound request can clear and then republish
in the same call; echo clearing precedes its cue, and end-spin sound precedes the
live charge/held-input check. Direct restoration has different queue limits and
boost selection and does not consume the deferred request. A linked sequence
executes the actual hurt-flash producer at 39-to-40, consumption, direct restore,
deferral and later consumption. One separate typed sequence advances sound/glow
without RAM decoding.[^weaponglow][^weaponsound]

All **115 embedded library inputs** match `15589b48`, and **eight compiled local
inputs** match their exact hashes. The optimized DLL builds and passes using the
preserved native-music ABI. The original native/value word-transfer macro performs
the highlight write; only its unused private-ROM helper is removed from local
support. Special-palette handlers, visor processing, palette copies, sound queues,
movement callbacks, diagnostics and event queries are controlled. The outer
dispatch table preserves the tested missing-entry/bounds behavior but does not
execute those special handlers. Definitions are synthetic; no movie or played
message/unpause route ran. This is supplied for integration with firing/flare,
not production promotion or complete writer/alias/runtime closure. Cancellation,
encounter restrictions and complete palette ownership remain open. Reproduction:
`Local/sm-decomp-experiments/20261022/weapon-feedback` (`feedback.rs`, `adapter.rs`,
`prepare.py`, `build.py`, `check.py`, `result.json`).

### Weapon selection and cancellation

The cancellation prototype distinguishes `cancel_charge`, `clear_animation` and
the combined operation, using the same `Flare` value shape as firing. Named weapon
choices return `Accepted` or `Unavailable`. The model and checked adapter forbid
unsafe code; the model contains no `Game`, RAM or definition addresses. It is
supplied for integration into the complete firing owner, not a second live control
value. Grapple state changes remain an explicit effect on the grapple owner.

**654 actual old/new full-RAM comparisons pass:** 360 direct weapon-selection
cases, 184 actual HUD-selector cases, 105 clear/inactive-grapple cases and five
linked/discriminating steps. They contain **497 ordered observations** and **85
matching failures**. Frozen implementations comprise all six `SwitchToHudHandler`
functions, `HandleSwitchingHudSelection`, `clear_animation`, the PLM clear-charge
instruction and `GrappleBeamFunc_Inactive`. The callback bodies use their original
signatures/return expressions; the same actual HUD selector dispatches either the
old routines or the candidate. The PLM comparison includes its unchanged script
pointer return. That pointer is a synthetic local byte array, not a ROM asset.

Unavailable ammunition/equipment causes no charge or animation change. Accepted
beam/missile/super-missile/Power Bomb/X-ray selection clears both before restoring
the suit palette. Inactive grapple selection restores the palette first, then
clears both and forces grapple inactive, including after callback changes. Active
grapple selection succeeds without either effect. A palette failure therefore
leaves different partial state for those callers. Nested selection during the
palette callback also matches. The selector preserves cancellation priority,
skipping unavailable choices, live HUD changes after callbacks, auto-cancel state,
change-marker wrapping and invalid-index failures.[^weaponcancel]

PLM cancellation leaves flare frames/timers intact. A linked call to inactive
grapple with the resulting zero charge also retains them; selecting the ordinary
beam then clears them. Conversely, shoot input takes grapple's start path before
the cancellation check. A separate uninterrupted typed sequence checks charge-only
cancellation, zero-charge grapple, unavailable missile and accepted beam selection
without RAM decoding. These differences prevent a blanket weapon-state reset.

All **115 embedded library inputs** match `15589b48`, and **seven compiled local
inputs** match exact hashes. The optimized DLL builds and passes with preserved
native-music linkage. Suit palette and grapple-start execution are controlled;
their full-RAM observations, mutations and failures are compared, not their real
implementations. The unchanged flare module is reused for its state type; this
does not add new flare-animation coverage. No ROM or movie is used. Full grapple
release, suit/drained/shinespark transitions, initialization, writer/alias closure
and production ownership remain open. No adoption or complete savings are claimed.
Reproduction: `Local/sm-decomp-experiments/20261022/weapon-cancellation`
(`cancellation.rs`, `adapter.rs`, `prepare.py`, `build.py`, `check.py`, `result.json`).

### Drain restriction and X-ray observation

A bounded prototype names `DrainRefresh`, `DrainRelease` and `WeaponPreparation`
transitions and a live X-ray restriction query. Its pure model forbids unsafe code
and contains no RAM addresses or `Game`. Ordered outcomes update the same firing
delay, bomb reservations and movement drag used by their ordinary owners. It is
not an independent persistent restriction record or a replacement for all encounter
behavior. The surrounding drain and X-ray routines retain their legacy operations.

**678 actual old/new full-RAM comparisons pass:** 280 drain cases, 39 cooldown
cases, 330 X-ray cases, 15 callback cases, five linked steps and nine additional
coordinator cases. There are **1,883 ordered RAM observations**, **11 matching
failures** and **nine discriminating checks**. Actual extracted implementations
include `Shitroid_Func_16`, `Samus_DamageDueToShitroid`, `Samus_HandleCooldown`,
`Samus_HandleHudSpecificBehaviorAndProjs`, `HudSelectionHandler_Xray`,
`XrayRunHandler` and `Xray_Initialize`. The candidate substitutes only the restriction
writes, preparation countdown and exact query; the real surrounding bodies execute.

Drain refresh publishes 8/5/2 before movement and damage. Preparation changes the
delay to 7 before the selected X-ray handler; that combination rejects activation.
A direct query before preparation sees 8 and does not match. Another preparation
without refresh reaches 6 and also ceases to match. These constructed schedules
establish the phase dependency, not natural gameplay reachability. Low-health drain
release clears drag before bomb reservations and retains delay, including across
its callbacks and partial failures. Changing the live enemy selector does not
redirect the already captured enemy record.[^drainrestriction][^weaponphase]

The real HUD coordinator queries after switching. Callback changes to any member
of the tuple alter that later result, which rejects a cached frame-start query.
Rejected X-ray leaves time unfrozen and projectile processing runs; activation
normally freezes time and suppresses it. A later callback thaw is observed before
that decision. Forward-facing poses bypass cooldown/HUD processing while still
updating projectiles, and an already frozen entry uses delay 32. An unconditional
encounter freeze or permanent disabled bit would not preserve those behaviors.

Successful X-ray publishes interrupted pose, freeze state, transition graphics
and its 256-word HDMA table before disabling other systems. A failure at a disable
call retains those writes. The later X-ray angle rereads facing, so a callback can
change the angle without changing the earlier interrupted pose. These are controlled
observations, not evidence of naturally mutating disable callbacks.

All **115 embedded library inputs** match `15589b48`, and **ten compiled local
inputs** match exact hashes. The coordinator independently checked those inputs,
the result and the three current gameplay source files without repeating the run.
The optimized DLL builds and passes. HUD dispatch is controlled to select the real
X-ray handler; it does not exercise all movement-specific HUD choices. Instruction
selection, encounter script, sound, system-disable, projectile-update and HDMA-spawn
effects are controlled. A synthetic allocation supplies the pointer passed to the
spawn substitute; no game ROM or movie is used. Existing unsafe enemy access and
X-ray setup remain. Complete writer/alias closure, runtime binding, other encounter
phases and a full owner remain open. Reproduction:
`Local/sm-decomp-experiments/20261022/encounter-restrictions`
(`model.rs`, `prepare.py`, `build.py`, `check.py`, `result.json`, `RESEARCH.md`).

### Samus projectile protection and suspension

The pure `Protection` model names beam/missile grants, contact suppression and
completion of a gameplay pass. It forbids unsafe code and needs no `Game` or RAM
addresses. Grants replace the existing value: a beam's 10 can shorten a missile's
20. Reservation, ammunition or placement rejection does not grant protection;
failure after the grant preserves that already committed change. Protection is
owned by Samus, not by the lifetime of an individual projectile.

**345 full-RAM comparisons pass:** 87 contact cases, eight timer cases, 101 grant
cases, 69 retention cases and 80 scheduler cases. The scheduler cases include
**ten linked steps**. There are **931 ordered observations**, **61 matching
failures** and **13 discriminating checks**. Actual beam/missile firing, contact,
timer and gameplay-coordinator bodies execute; the candidate changes only the two
grants, contact predicate and tail decrement. Actual release/reset/Samus
initialization bodies remain unchanged on both sides.

Contact first records its processing stage, then its protection gate prevents
the projectile/bomb scan. It does not consume protection. Ordinary Samus damage
invincibility is a separate value and does not replace this gate. Protection also
suppresses bomb-jump contact. The unprotected damage path marks contact and applies
damage/timers before the existing native assertion; the experiment replaces that
abort with a controlled failure, so it supplies no playable reflected-damage route.
[^contact][^beamlaunch][^missilelaunch]

The real gameplay coordinator grants during the selected preparation callback,
observes contact and can then suspend for an enemy message or PLM. Repeated yields
do not regrant, repeat contact or tick protection. Resumption through completion
ticks once at the existing tail. A value of one protects the earlier contact pass,
then becomes zero; the next pass can initiate a bomb jump. Late callback writes
are reread at the tick. The timer routine itself has no freeze gate; executing
this scheduler with frozen state is a constructed case, not coverage of every
frozen game mode.[^frame][^weapontimers]

Ordinary pool release, projectile reset and Samus initialization retain protection
in their own stores. Their callbacks may still overwrite it. **Five valid broad
selector cases contradict unconditional release retention:** for example, `0x0D48`
aliases its X-position clear to protection, while `0x0C44` aliases its preinstruction
write there and stores `0xB169`. Odd variants normalize to the same addresses;
another timer-clear alias is also checked. A typed ten-slot pool alone cannot
close these legacy writers. They require the explicit authority transfer already
defined for broad operations.[^resetslots][^samusinit]

All **115 embedded library inputs** match `15589b48`, and **ten compiled local
inputs** match exact hashes; the coordinator independently reviewed those inputs,
the source substitutions and the result. The optimized DLL builds and passes.
Placement, initialization, damage calculation, other firing/gameplay callbacks,
message/PLM execution and palette output are controlled. Message rendering and
the nested message/PLM implementations are not exercised by the scheduler test.
Synthetic memory supplies the unused ROM pointer; no game ROM or movie is used.
The small model still samples shared state through its adapter at each operation;
persistent ownership, generic aliases, restoration and whole-game coverage remain
open. Reproduction: `Local/sm-decomp-experiments/20261022/projectile-protection`
(`protection.rs`, `prepare.py`, `build.py`, `check.py`, `result.json`, `RESEARCH.md`).

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

### Launch state and reflection experiments

The adopted launch model/adapter and the migrated source routines pass **13,958
old/new comparisons and persistent steps**, checking full Game RAM, completion
and ordered callback inputs. Selection targeted the new contract and the source
hazards above; no movie ran.

| Scenario | Comparisons / steps |
| --- | ---: |
| Launch direction, flags, speed wrapping, asymmetric histories, normal/odd slots and invalid directions | 12,288 |
| In-bounds off-table slot aliases, including subpixel clear and Y-before-X dependencies | 890 |
| Actual movement writers with solid/block collision outcomes and callback mutations | 384 |
| Actual phase clear, retaining absolute-Y fraction and unrelated fields | 16 |
| Persistent firing, clear, post-clear launch, movement and distance-accounting sequences | 60 |
| Linked production movement and phase-clear functions from the final release library | 40 |
| Actual beam/missile/super reflection, followed by movement and missile reinitialization | 120 |
| Initial missile launch followed by its first acceleration/initialization | 160 |

All **704 ordered callback observations** agree. The harness compiles original
and adopted Rust function bodies with the actual RAM macros, split-word helpers,
launch model and adapter against the real compiled `Game`. Movement collision,
pose reactions, animation initialization, clearing and extension spawning are
recording/mutating substitutes. The selected missile branch never requests ROM
acceleration data; an unexpected request fails the harness. Invalid-direction
diagnostics are controlled callbacks; the real flight caller retains its original
`Unreachable` call. This does not validate downstream collision, animation,
extension, native assertion, rendering or hardware implementations, nor prove
that unusual slot aliases occur in gameplay.

Five discriminating checks exercise cross-direction fraction dependence,
replacement rather than accumulation, repeated non-consuming launch, the upward
low-byte gate, retained absolute-Y fraction, and Y-commit-before-X-sample ordering
(the replacement check also checks repeated launch). The executed reflection
sequences produce **32 counterexamples** to sampling reflected missile velocity
at reflection time instead of at its subsequent activation. These are constructed
actual-code scenarios, not recorded player encounters. Global initialization and
reset are retained source findings, not newly executed paths here.

Normal release library/frontend/verifier builds and all **108 embedded library
source comparisons** pass. Both new modules enforce `forbid(unsafe_code)`.
Complete source cost is **+113 Rust lines**: 84 model, 67 adapter and 10 registration
lines, offset by 48 removed caller/calculation lines. No unsafe declarations were
hidden; the surrounding unsafe functions remain counted. Durable local reproduction
is `Local/sm-decomp-experiments/20261022/launch-motion/experiment.py`, with frozen
original Rust, build logs and generated comparison DLL. No ROM or game snapshots
are committed. Further cleanup of reflection or collision internals still needs
its own exercised checks.

## Next ownership experiments

Continue the same whole-projectile draft through Samus weapon control. Typed aim,
launch, flare, feedback and cancellation are integrated in the reviewed draft.
Drain outcomes now use existing weapon state in the reviewed encounter stage;
movement and environment effects still transfer through the legacy adapter.
Exercise remaining cancellation producers. Integrate Samus protection with its grant, contact and
completed-pass consumers, preserving broad alias transfers and reset distinctions.
Close remaining shared palette/sound writers and runtime transfers.
Complete-owner projectile playback, trail sampling and initialization are reviewed
in the draft. The emission integration now has its selected 603-case result.
Integrate the completed standalone trail side-playback/drawing model into that
owner, then close remaining attack-definition consumers and runtime transfers.
These are new coverage questions,
not additional pass counts for the existing firing or launch experiments.

1. Codify `ExternalMotion` as ordered reset, replacement and addition operations;
   trace real sand/platform contacts through message suspension and movement.
2. Move the now-explicit launch state into owned Samus state when its remaining
   initialization/reset and camera-accounting dependencies can migrate together.
   The calculation, eight movement writers and ordinary phase clear are adopted;
   RAM persistence remains transitional.
3. Replace retained enemy pointers with slot offsets while preserving live
   rereads, then test reaction-induced deletion/reuse and instruction selection.
4. Extend the adopted PLM award/message model through safe script traversal and
   owned progression persistence when their remaining writers and consumers can
   migrate together. Award timing and message continuation are adopted above;
   keep the PLM scheduler distinct from enemy scripts.

The remaining items are identified next experiments, not completed migrations. An exhaustive
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
[^launch]: Original `initialize_speed` source: `dc20e538:sm/src/game/projectile_flight.rs`; current [flight caller](../sm/src/game/projectile_flight.rs#L110), [launch calculation](../sm/src/game/launch_motion.rs), and [ordered byte adapter](../sm/src/game/launch_motion_ram.rs).
[^prepare]: [Samus_FrameHandlerAlfa_Func11/12](../sm/src/game/sm_90.rs#L4009).
[^hud]: [Samus_HandleHudSpecificBehaviorAndProjs](../sm/src/game/sm_90.rs#L3477).
[^enemies]: [EnemyMain](../sm/src/game/sm_a0.rs#L879).
[^contact]: [SamusProjectileInteractionHandler](../sm/src/game/sm_a0.rs#L1043).
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
[^launchmap]: [Directional movement and projectile-array RAM metadata](../sm/src/ram_map.rs#L554); the movement record starts at the `$0DA0` stride declaration.
[^launchwriters]: [Horizontal collision results](../sm/src/game/sm_90.rs#L277), [vertical collision results](../sm/src/game/sm_90.rs#L569), [four no-collision writes](../sm/src/game/sm_90.rs#L862).
[^splitwords]: [words! ordering](../sm/src/rtl.rs#L2); [SetHiLo](../sm/src/rtl.rs#L787).
[^samusinit]: [Samus_Initialize workspace clear and distance initialization](../sm/src/game/sm_91.rs#L1260).
[^resetlaunch]: [Vector_RESET_Async bulk clears](../sm/src/game/sm_80.rs#L188).
[^resetslots]: [ResetProjectileData](../sm/src/game/sm_90.rs#L1468).
[^distance]: [MainScrollingRoutine](../sm/src/game/sm_90.rs#L611); [Samus_CalcDistanceMoved_Y](../sm/src/game/sm_90.rs#L755).
[^beamlaunch]: [beam_firing launch and collision order](../sm/src/game/beam_firing.rs#L77).
[^missilelaunch]: [HudSelectionHandler_MissilesOrSuperMissiles](../sm/src/game/sm_90.rs#L2507); [accelerate_missile](../sm/src/game/projectile_flight.rs#L180).
[^reflection]: [ProjectileReflection](../sm/src/game/sm_90.rs#L2460); [Reflec_Shot](../sm/src/game/sm_a3.rs#L1923); [Space Pirate reflection](../sm/src/game/sm_b2.rs#L97).
[^launchmodel]: [LaunchMotion and typed inputs/output](../sm/src/game/launch_motion.rs); [transition serialization and initialization](../sm/src/game/launch_motion_ram.rs); migrated callers in [sm_90](../sm/src/game/sm_90.rs), symbols `record_movement` and `finish_projectile_phase`.
[^enemyshotsearch]: [EprojCollHandler_Multibox](../sm/src/game/sm_a0.rs#L1486); [EprojCollHandler](../sm/src/game/sm_a0.rs#L1714); [EnemyCollisionHandler](../sm/src/game/sm_a0.rs#L1032).
[^shotretirement]: [ResetProjectileData](../sm/src/game/sm_90.rs#L1468); [ClearProjectile](../sm/src/game/sm_90.rs#L1504); [KillProjectile](../sm/src/game/sm_90.rs#L1566).
[^shotimpact]: [start_impact](../sm/src/game/projectile_animation.rs#L102); [instruction_delete and advance](../sm/src/game/projectile_animation.rs#L185).
[^projectileplayback]: [get_ProjectileInstr](../sm/src/game/projectile_animation.rs#L33); [advance and instruction dispatch](../sm/src/game/projectile_animation.rs#L188).
[^projectiletrailframe]: [trail_frame](../sm/src/game/projectile_animation.rs#L170).
[^projectiledefinitions]: [Fixed and directional initialization](../sm/src/game/projectile_animation.rs#L36); [echo initialization](../sm/src/game/projectile_animation.rs#L148).
[^trailpipeline]: [HandleProjectile and SpawnProjectileTrail](../sm/src/game/sm_90.rs#L2036); [trail side playback and drawing](../sm/src/game/sm_90.rs#L2100); [ProjectileTrail_Func5](../sm/src/game/sm_9b.rs#L61).
[^roomshotscan]: [react_to_shot_block, beam_block_collision and missile_block_collision](../sm/src/game/sm_94.rs#L1538).
[^roomshotreaction]: [Shootable/bombable reactions and scan overrides](../sm/src/game/sm_94.rs#L1359); [PlmSetup_B974](../sm/src/game/sm_84.rs#L941).
[^roomshotsetup]: [SpawnPLM and SpawnPLMWithCollision](../sm/src/game/sm_84.rs#L249); [find_plm_at_block](../sm/src/game/sm_84.rs#L766); [sub_84C64C](../sm/src/game/sm_84.rs#L956); [Strange Door and respawning shot block](../sm/src/game/sm_84.rs#L1882).
[^roomshotcursor]: [Extension reactions](../sm/src/game/sm_94.rs#L800); [ShotAxis::advance and slope reactions](../sm/src/game/sm_94.rs#L1570); [Game::react_to_block](../sm/src/game_state.rs); [gate triggers](../sm/src/game/sm_84.rs#L1370).
[^aimproducers]: [SamusFunc_F468_NormalJump](../sm/src/game/sm_91.rs#L2587); [SamusFunc_F468_TurningAroundOnGround](../sm/src/game/sm_91.rs#L2802).
[^aimplacement]: [InitProjectilePositionDirection](../sm/src/game/sm_90.rs#L2226).
[^aiminput]: [beam_firing::update_input and reservation](../sm/src/game/beam_firing.rs#L8); [HudSelectionHandler_TurningAround](../sm/src/game/sm_90.rs#L3490).
[^aimclear]: [Samus_Func10](../sm/src/game/sm_90.rs#L4559).
[^weaponvisual]: [HandleChargingBeamGfxAudio](../sm/src/game/sm_90.rs#L2279); [DrawSamusAndProjectiles and draw handlers](../sm/src/game/sm_90.rs#L4565).
[^grapplecharge]: [UpdateGrappleBeamTilesAndIncrFlameCtr and HandleGrappleBeamFlare](../sm/src/game/sm_9b.rs#L519); [grapple release](../sm/src/game/sm_9b.rs#L687); [grapple start](../sm/src/game/sm_9b.rs#L971).
[^weaponglow]: [Samus_HandlePalette and HandleBeamChargePalettes](../sm/src/game/sm_91.rs#L678).
[^weaponphase]: [Samus_HandleCooldown](../sm/src/game/sm_90.rs#L1402); [Samus_HandleHudSpecificBehaviorAndProjs](../sm/src/game/sm_90.rs#L3457).
[^drainrestriction]: [Shitroid_Func_16](../sm/src/game/sm_a9.rs#L4526); [Xray_Initialize](../sm/src/game/sm_91.rs#L1337); [Mother Brain rainbow phases](../sm/src/game/sm_a9.rs#L720).
[^weapontimers]: [DecrementSamusTimers](../sm/src/game/sm_a0.rs#L888); [gameplay completion](../sm/src/game/gameplay.rs).
[^weaponsound]: [Samus_ShootCheck](../sm/src/game/sm_90.rs#L5133); [hurt-flash producer](../sm/src/game/sm_91.rs#L795); [QueueSamusMovementSfx](../sm/src/game/sm_82.rs#L3301); [message completion](../sm/src/game/message_boxes.rs#L468); [unpause completion](../sm/src/game/sm_82.rs#L1483).
[^weaponcancel]: [PLM charge cancellation](../sm/src/game/sm_84.rs#L3100); [HUD switching](../sm/src/game/sm_90.rs#L2747); [mode transitions](../sm/src/game/sm_90.rs#L4985).
[^extensionplacement]: [spawn_extension](../sm/src/game/projectile_flight.rs#L269).
