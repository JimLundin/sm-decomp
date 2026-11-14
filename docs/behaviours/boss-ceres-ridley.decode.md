# Ceres Ridley and the escape, part 1, the Ceres doors: decode notes

For the observer, the adversary, the integrator and the domain harness's decoder only.
Implementers never see this file (MODEL §5). It holds every source the spec draws on: the
original's routines, its RAM and its codes. Addresses are the bank logs'
(https://patrickjohnston.org/bank/; local copies `$TEMP/collab/ceres-survey/ref/bankA6.txt`,
`$TEMP/collab/banklog/A0.txt`, `A9.txt`, and the RAM map `ram.asm` beside them).

## How the spec was made

Behaviour first. The eleven Ceres chain rows of CI's lean record cache (`chain-any-0001-DF45` to
`chain-any-0011-DF45`, `$TEMP/collab/ci-cache/rows`) were dumped per frame by a cdylib over
`verify::cache` and `oracle_record` (`$TEMP/collab/b8-spec/dump/dump.rs`, run through
`$TEMP/collab/wire-r7/run.py`; output `$TEMP/collab/b8-spec/d<row>.txt`, local only): every
slot whose species is `$E23F`, with its words, her position, the camera, the Ceres status, the
earthquake words, the random number, the doors' first target colour and the CPU's APU port
writes. They were read over time, then the bank log was read to name the steps and pin the
arithmetic (the reach's compare, the shake's chance).

The reference model, `$TEMP/collab/b8-spec/ref/model.py`, steps a door by the spec's rules: it
decodes the door at record N (the tables below), steps it when the enemy's frame counter `0FA4`
advanced by one from N to N+1 (the framework ran it; Header, Running), with her position at N,
and compares the phase, the wait, the figure, the hidden and intangible flags and the "new
figure" flag (`0F88` bit 15, the harness's, derived from the step) at N+1. The shaft's wall reads
`093F` non-zero, the shake `093F >= 2`. Over the 7,045 door-frames of parameters 0, 1 and 4 that
stay in one room (2,972 runs and 4,073 frames the door didn't run), it agrees on all 7,045; a
do-nothing model (the door's words never change) agrees on 4,326. The escape's shaking: over the
860 frames of the escape in which at least one door ran, the model's shake starts (each door in
slot order, reading the random number one step on from N's, below) agree on all 860 with
`1840`/`183E` at N+1: a frame with a start has its type and its timer (or the timer less the
frame's count-down); a frame without one keeps the type, and the timer counts down by one or
holds. Under that check a model with the chance on the low 8 bits agrees on 689, one that
ignores the frames left on 482, one that swaps the two shakes on 510. `ref/vecrun.py` runs every
vector of `scenarios/vectors/boss-ceres-ridley.toml` through the model, the placing vectors
through its placing: 56 of 56 agree.

## State (the RAM mapping)

A door is an enemy slot (`0F78 + 40h × slot`); the words below are the slot's, at offsets from
`0F78`.

| Spec | RAM | Notes |
| --- | --- | --- |
| kind, facing | `0FB4` (parameter 1): 0 `Door` facing right, 1 `Door` facing left, 4 `ShaftDoor` | 2, 3, 5 and 6 are later pieces (below) |
| position | `0F7A`, `0F7E` (pixels; the subpixels `0F7C`, `0F80` stay 0) | the population's |
| the box | `0F82`, `0F84` (radii 8, 20h), from the header `$A0:E23F` | |
| phase | the instruction list pointer `0F92`, by the table below | |
| wait | the instruction timer `0F94` | |
| figure | the spritemap pointer `0F8E`, by the table below | `$804D` while placed (the loader's; the init writes `$FAC7` first) |
| shown | `0F86` bit `0100h` clear | |
| solid | `0F86` bit `0400h` clear (tangible); the population sets `8000h` (solid to Samus) on every door, so tangible means solid | |
| (not in the spec) | `0F88` bit `8000h`, the "new figure" bit (set by `$A0:C2A6`, cleared at `$A0:C2B1` when the timer didn't run out) | the player can't see it: only the extended-spritemap drawing reads it (`$A0:952B`), which the door doesn't use. The harness's RAM translation derives it from the step: set on a run whose wait ran out, cleared on any other run, unchanged on a frame without a run (or C1 owns it as every enemy's) |
| the framework's run | `0FA4` +1 per run | C1's |
| `0FA8` | the AI function, `$F76B` for 0, 1 and 4 (`$F72B` table): constant | not the door's State |
| `0FAA`..`0FB2` | 0 for these kinds | the drawn-by-Ridley flag and the rumble's words, later pieces |
| palette | `0F96` = `0E00h` (sprite palette 7) for 0, 1, 4 | |

**Phase from the instruction pointer** (the pointer after the instruction that set the timer):

| Phase | Facing right (parameter 0) | Facing left (parameter 1, and 4 before the getaway) |
| --- | --- | --- |
| `Placed` | `$F56C` | `$F5BE` (`$F61A` for 4) |
| `Starting` | `$F574` | `$F5C6` |
| `Open` | `$F57C` | `$F5CE` |
| `Closing(1..4)` | `$F58C`, `$F590`, `$F594`, `$F598` | `$F5DE`, `$F5E2`, `$F5E6`, `$F5EA` |
| `Closed` | `$F5A0` | `$F5F2` |
| `Opening(1..4)` | `$F5AA`, `$F5AE`, `$F5B2`, `$F5B6` | `$F5FC`, `$F600`, `$F604`, `$F608` |
| `Wall` | - | `$F626` (parameter 4) |

**Figures from the spritemap pointer:** facing right `Closed` `$FA13`, `Ajar1` `$FA3D`, `Ajar2`
`$FA67`, `Ajar3` `$FA87`, `Open` `$FAA7`; facing left `Closed` `$F95F`, `Ajar1` `$F989`, `Ajar2`
`$F9B3`, `Ajar3` `$F9D3`, `Open` `$F9F3`. The importer (AF3) turns these spritemaps into our
figures; no vector carries them.

## Inputs

| Spec | RAM | Notes |
| --- | --- | --- |
| where she was | `0AF6`, `0AFA` as at the end of the last frame | the doors run in the main enemy routine before Samus's movement (`$90` new-state handling) |
| the stage | `093F`: 0 `Calm`, 1 `Getaway`, 2 `Escape`, `8000h` (the shaft rotates, set by `$A6:F67F`) also `Escape` | the init (`$A6:F6F7`) and `$A6:F678` test non-zero; the quake (`$A6:F773`) tests `>= 2` unsigned, so `8000h` shakes |
| the random number | `05E5` at the door's run | the main game loop steps it once a frame (`$82:894F`, `JSL $80:8111`, `x × 5 + 111h`, 16-bit) before it dispatches the game state's routine (main gameplay `$82:8B44`); over every recorded escape frame nothing else steps it before the doors run, so it is N's value stepped once; the harness reads it so |
| the shake's frames left | `1840`, as the earlier door left it this frame | |

## Outputs

| Spec | RAM / stream | Notes |
| --- | --- | --- |
| the opening sound | `$80:914D` with A = `2Ch`: sound library 3, at most 6 queued | the port write (`$2143` = `2Ch`) lands 0 to 5 frames after the run, as the queue sends it (U1): rows 1, 3, 4, 8, 9, 10 in the run's frame, 2, 5, 7 later |
| a shake | `1840` timer, `183E` type: small = timer 2, type `14h` (BG1, BG2 and enemies, 1 px, diagonal); strong = timer 4, type `1Ah` (3 px) | |
| the doors' colours | target palette `7EC3E2..7EC3FF` (sprite palette 7, colours 1 to Fh) by `$A9:D2F6` from `$A6:F4EE` (calm) or `$A6:F50E` (escape) | at the door's init; the door transition's fade brings `CGRAM` to them (R4) |
| shown, figure | the enemy drawing queue, then OAM | C1's drawing and the sprite encoder |
| solid | the interactive list (`17EC`) and `$A0:A8F0`'s solid-enemy collision | C1's. Main gameplay builds the list (`$82:8B47`, `$A0:8EB6` → `$A0:8F40`, intangible excluded) before her state handler, her projectiles' interaction (`$A0:9785`) and the main enemy routine (`$82:8B65`), so her motion, touch and shots meet the door as it was when the frame began (spec §4); the drawing (`$A0:884D`) runs after, with the door as it is after its run |

## Sources per entry

| Entry | Sources |
| --- | --- |
| Waiting | `$A0:C26A` (process enemy instructions: decrement, run to the next timer/spritemap pair, set `0F88` bit 15); the records |
| SheIsNear | `$A6:F63E` (go to if Samus is not within 30h pixels: per-axis `|Δ|` by negation, `CMP #$0030`, both axes; the log's "taxicab" comment is wrong, it is a box); `$A0:8FD4` order vs `$90` |
| RoomStart | `$A6:F6C5` (init), `$A6:F56C`, `$A6:F5BE`, `$A6:F61A`, `$A6:F678`; `$A0:8A1E`/`$A0:8CD7` (load) |
| Polling | `$A6:F59C`, `$A6:F578`, `$A6:F5EE`, `$A6:F5CA` |
| Opening | `$A6:F5A4`..`$A6:F5BC`, `$A6:F5F6`..`$A6:F60E`, `$A6:F6BD` (sound) |
| Closing | `$A6:F584`..`$A6:F596`, `$A6:F5D6`..`$A6:F5E8` |
| ShaftWall | `$A6:F61A`, `$A6:F622` |
| Solid | `$A0:8F40` (interactive list), `$A0:A9E1` (solid to Samus), `$A6:F68B`/`$A6:F695`, `$A6:F920` (touch and shot: RTL), `$A0:A1F7` (projectiles deleted unless plasma, `1000h` clear) |
| Shown | `$A6:F6A6`/`$A6:F6B3`, `$A0:9105` (drawn unless invisible) |
| EscapeShakes | `$A6:F765`, `$A6:F76B`, `$A6:F773`; `$80:8111` |
| TheDoorsColours | `$A6:F6F7`..`$A6:F72A`, `$A6:F4EC`, `$A9:D2F6` |

## Findings behind the spec's choices

- **The poll and the hold** are the instruction list's timer words (2 and 5); the spec names them
  as the door's rhythm, not as list steps.
- **Hidden figures.** The left door's open loop shows `$F95F` (its closed spritemap) and both
  start lists show `$FA13` (the right door's closed one); they are never drawn (invisible) but are
  compared in the slot's words, so the spec gives each phase its figure.
- **The "new figure" bit** (`0F88` bit 15) is set on every run that ran out the timer, loops
  included, and cleared on every other run; frames the door doesn't run leave it as it was (row 3,
  frames 305-344, the door passage's fade-in). The spec leaves it out: nothing the player sees
  reads it for a door (State table).
- **Order.** The door reads `0AF6` before her movement: at `chain-any-0003-DFD7` frame 7 the open
  door stayed open with her at 45 px at frame 6's end and 51 px at frame 7's; frame 9 closed it
  with 57 px at frame 8's end.
- **The shake's random number** is N's `05E5` stepped once (the main game loop's step, Inputs);
  with N's value unstepped 21 of the 860 frames disagree (among them `chain-any-0007-E06B` frames
  6 and 28).

## The harness (for the integrator)

- **Rows:** the eleven `chain-any-*` rows; the door slots with parameter 0, 1 or 4.
- **Compared per door, per frame it runs** (its `0FA4` advanced): `0F92` through the phase table,
  `0F94`, `0F8E` through the figure table, `0F86` bits `0100h` and `0400h`, `0F88` bit `8000h`.
  On a frame it doesn't run, all of them unchanged.
- **Compared per frame of the escape** in which a door ran: `1840` and `183E` against the shake
  each door's run starts (a shake not started leaves them to R3, which counts `1840` down;
  frames where R3 doesn't count are not the door's).
- **The sound:** one `2Ch` request per `Opening(1)`; matched to the APU port stream by U1's queue.
- **Skipped, and why:** parameters 2 (the shaft's cover), 3 (Ridley's room door), 5 and 6 (the
  getaway's walls) are later pieces of B8, so their slots stay `EnemyBody::Unmodelled`; `0FA4`
  itself and the drawing (C1). The frame a room's doors are placed is compared: the model places
  each door from the loader's slot, kind, facing and position.
- **Masks:** the shake on a door-passage frame with one running (the passage's room set-up clears
  its timer), and the doors' colours on the passage's other frames (the passage's palette loading
  writes the target colours). Each is masked under its writer's name; the free run carries through.

## The harness cases

Real content no vector may carry, loaded by the importer in the integrator's harness:

| Case | Row:frame | What | Checks |
| --- | --- | --- | --- |
| H1 | `chain-any-0002-DF8D`:198 | room `DFD7` placed while calm: target colours `7EC3E2..` take the calm set | TheDoorsColours |
| H2 | `chain-any-0008-E021`:180 | room `DFD7` placed in the escape: the escape set | TheDoorsColours |

**Recorded cases** behind the vectors (positions moved to invented ones, offsets kept):

| Case | Row:frame | Vectors |
| --- | --- | --- |
| R1 | `chain-any-0003-DFD7`:305 | D-start-first-run, D-start-left-first-run |
| R2 | `chain-any-0003-DFD7`:307 | D-start-wait |
| R3 | `chain-any-0003-DFD7`:308 | D-start-near-opens, D-start-far-closes |
| R4, R5 | `chain-any-0003-DFD7`:155, 156 | D-closed-far-stays, D-closed-wait |
| R6 | `chain-any-0003-DFD7`:157 | D-closed-near-opens |
| R7, R8, R9 | `chain-any-0003-DFD7`:162, 170, 177 | D-opening-step, D-opening-wait, D-opening-last-left |
| R10, R11, R12 | `chain-any-0003-DFD7`:7, 8, 9 | D-open-near-stays, D-open-wait, D-open-far-closes |
| R13, R14 | `chain-any-0003-DFD7`:14, 29 | D-closing-step, D-closing-last |
| R15, R16 | `chain-any-0011-DF45`:0, 1 | D-shaft-wall, D-shaft-wall-again |
| R17, R18 | `chain-any-0007-E06B`:6, 3 | D-shake-strong, D-shake-busy |

## What the enemy framework (C1) must supply

The smallest interface the door's model needs, each a piece C1 owns for every enemy:

1. **Run** (`$A0:8EB6` selection, `$A0:8FD4` main routine): each frame the enemies run, every
   door once, in slot order, processed off-screen (`0F86` bit `0800h`), its frame counter
   `0FA4` + 1, its main AI then its instructions. The door's model is a pure step: (door, her
   position at the frame's start, stage, random number, shake's frames left) → (door, sound,
   shake). Which frames run (gameplay, the door passage's fade-out, the fade-in's
   lag frames) is C1's and R1's.
2. **Draw** (`$A0:9423` queue, `$A0:944A` OAM): a shown enemy's figure at its position less the
   camera, palette 7, layer 2 (header), when on screen; the figures imported (AF3).
3. **Solid contact** (`$A0:A8F0`, `$A0:9758`, `$A0:A1F7`): the interactive list's solid boxes
   stop her; touch and shot reactions (none for a door); non-plasma shots deleted. The list is
   built from the doors' shown and solid as the frame begins, before their runs (Outputs, solid).
4. **Place** (`$A0:8A1E`, `$A0:8CD7`): slot, position, parameter, properties `A800h`, the
   instruction list by parameter, timer 1, the target colours.

**Ordering recommendation.** The door needs none of C1's clean-room model to be built: the old
engine's `game::enemy` already selects (building the list before the run, as the original does),
runs (frame counter, instruction timer), draws (`enemy::drawing`) and collides with solid enemies
(`collision::solid_enemy`) for slice 0's species. B8's door round can land first, wired as a new
`EnemyBody` the old runner steps and draws through its existing queue, with C1's first round
replacing the runner and drawing later, on four conditions:

1. **The body bypasses the old instruction processing.** `game::enemy::run` runs
   `instructions::process` for every enemy that processes instructions; the door's body skips
   it, and writes shown and solid back to `EnemyCommon.properties` after each step, so the next
   frame's `select` lists the door right (spec §4).
2. **The shakes and the sound stay harness-only until R3 and U1.** The engine has no screen-shake
   state and no sound queue, so `EscapeShakes` has no source for the frames left and no consumer,
   and the opening sound none until U1. The door round wires `Waiting` through `Shown` and
   `TheDoorsColours` for M0, and compares the shakes and the sound in the harness only, until R3
   owns the shake's timer as State.
3. **The random number's step.** The main game loop steps `05E5` once a frame before the game
   state's routine (`$82:894F`, Inputs), and in the cached escape frames nothing else steps it
   before the doors run. Before free-running the escape the integrator checks that the engine
   steps it there, and that no enemy earlier in slot order steps it in a room with doors.
4. **Touch and shots go in LEGACY when the door is wired.** The old runner treats them as
   `open` (`$A0:9758`); no M0 row has her touch or shoot a blocking door, so it is harmless for
   M0, but the difference is listed in `docs/LEGACY.md` in the wiring commit.

The remaining edge is the figure → sprite mapping, which goes in the bridge or AF3.

## Budget

Moved from the spec's header (MODEL §1 decision 10): the implementer is not told the score or
the budget. `tools spec-units` reads k and the areas here.

**Budget (MODEL §1 decision 7).** k = 37.5 weighted tokens per spec unit, frozen on
samus-movement. There is no old code: the engine carries the doors unrun
(`EnemyBody::Unmodelled`, LEGACY "slots it carries but doesn't run"), so no area has `old`, and
there is no old-code guideline to compare (MODEL §1 decision 19).

- Area doors: 1, Waiting, SheIsNear, RoomStart, Polling, Opening, Closing, ShaftWall, Solid, Shown
- Area station: EscapeShakes, TheDoorsColours

Budget: 37.5 × 30.7 units = 1,150 (doors 950, station 200)
