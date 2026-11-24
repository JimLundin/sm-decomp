# Ceres Ridley and the escape, parts 1 and 2, the Ceres doors and the shaft as she arrives: decode notes

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
| the random number | `05E5` at the door's run | the main game loop steps it once a frame (`$82:894F`, `JSL $80:8111`, [carry-aware advance](game-random.decode.md#sources-and-ownership)) before it dispatches the game state's routine (main gameplay `$82:8B44`); over every recorded escape frame nothing else steps it before the doors run, so it is N's value stepped once; the harness reads it so |
| the shake's frames left | `1840`, as the earlier door left it this frame | |

## Outputs

| Spec | RAM / stream | Notes |
| --- | --- | --- |
| the opening sound | `$80:914D` with A = `2Ch`: sound library 3, at most 6 queued | the port write (`$2143` = `2Ch`) lands 0 to 5 frames after the run, as the queue sends it (U1): rows 1, 3, 4, 8, 9, 10 in the run's frame, 2, 5, 7 later |
| a shake | `1840` timer, `183E` type: small = timer 2, type `14h` (BG1, BG2 and enemies, 1 px, diagonal); strong = timer 4, type `1Ah` (3 px) | |
| the doors' colours | target palette `7EC3E2..7EC3FF` (sprite palette 7, colours 1 to Fh) by `$A9:D2F6` from `$A6:F4EE` (calm) or `$A6:F50E` (escape) | at the door's init; the door transition's fade brings `CGRAM` to them (world-doors' `TheFades`; Ceres has no R4 palette FX) |
| shown, figure | the enemy drawing queue, then OAM | C1's drawing and the sprite encoder |
| solid | the interactive list (`17EC`) and `$A0:A8F0`'s solid-enemy collision | C1's. Main gameplay builds the list (`$82:8B47`, `$A0:8EB6` → `$A0:8F40`, intangible excluded) before her state handler, her projectiles' interaction (`$A0:9785`) and the main enemy routine (`$82:8B65`), so her solid motion uses the selected beginning-of-frame membership (spec §4); shots instead use current per-processing-actor pre-AI contact properties (see [enemy-interactions](enemy-interactions.decode.md)); the drawing (`$A0:884D`) runs after, with the door as it is after its run |

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
- **Skipped, and why:** parameter 2 (the shaft's overlay, the pre-explosion door overlay, whose
  beacon and platform are part 2's), 3 (Ridley's room door), 5 and 6 (the getaway's walls) are
  later pieces of B8, so their slots stay `EnemyBody::Unmodelled`; `0FA4`
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
3. **Solid contact** (`$A0:A8F0`): the interactive list's solid boxes stop her; membership is
   selected before the runs. **Shot contact** (`$A0:9758`, `$A0:A1F7`): current processing
   membership and each actor's current pre-AI properties, independently of the solid list;
   ordinary power shots are marked consumed, and the door response is no-op. See
   [enemy-interactions](enemy-interactions.decode.md) for the narrow Ceres contract.
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
   `open` (`$A0:9758`); the ordinary power shot in M0 row 3 is consumed at frame160; the missing production contact
   callback must be supplied by [enemy-interactions](enemy-interactions.decode.md), and the difference is listed in `docs/LEGACY.md` in the wiring commit.

The remaining edge is the figure → sprite mapping, which goes in the bridge or AF3.

## Part 2, the shaft as she arrives

### How it was made

Behaviour first, from CI's lean record cache (`$TEMP/collab/ci-cache/rows`): `any-ceres-arrival`
frames 185-222 and `chain-any-0001-DF45` cuts 0-300, read per frame by a scratch reader of the
lean tier (`$TEMP/collab/ceres-739/lean.py`, local only): her handlers, position and pose, the
projectile slots, the enemy slots, the frame counter, the palette buffer, CGRAM and VRAM. The
bank logs (86, 90, 82, A6) were then read to name the actors and pin the arithmetic: the
floor's signed compare, the wait's count, the beacon's table index.

The reference model, `$TEMP/collab/ceres-739/model.py`, steps the pad and the cover by the
spec's rules from record N−1 to N (her height at N−1: her own frame doesn't move her while she
is held), on frames of game state 7 or 8, and compares the pad (present, wait, x, y, look), the
cover (present, x, y), her `0AFA` and whether she is held, at N:
- `any-ceres-arrival` 193-222: 30 of 30 frames agree; a do-nothing model agrees on 0.
- `chain-any-0001-DF45` 1-101: 101 of 101; do-nothing 0.
- The beacon and the platform (gate fixes, `$TEMP/collab/ceres-739/fixtools/lights.py`), with
  the overlay's runs (a slot with id `E23F`, parameter `0FB4` = 2, whose `0FA4` advanced) and the
  fade's steps taken (`7EC400` at N when `099C` at N−1 is `$E2DB`, the fade out, else at N−1) as
  inputs, `093F` under 2. The beacon against the palette buffer `7EC052..5D` and the table
  `$A6:F871` row directly; the platform against the VRAM words first seen for each look:
  - `any-ceres-arrival`: 30 runs, 30 steps match; platform 30 of 30. 192 frames without the
    overlay or a fade: 189 unchanged; frames 3, 25 (loading) and 192 (the arrival) are other
    writers'.
  - `chain-any-0001-DF45`: 266 runs: 252 steps match, 14 runs with the fade's steps 1 to 14 and
    none of them writes a table row; platform 266 of 266. 80 frames without the overlay or a fade
    (the passage's idle frames and room 2): 79 unchanged; cut 322 is room 2's loading (VRAM).
  - `chain-any-0002-DF8D`: no overlay; 202 frames without a fade, 202 unchanged.
  - The do-nothing model (keep the last colours and picture) agrees with 26 and 225 of the
    beacon's steps and 15 and 125 of the platform's looks.
- `model.py vectors` runs every part-2 vector: 30 of 30 agree.

### State (the RAM mapping)

| Spec | RAM | Notes |
| --- | --- | --- |
| the pad | enemy projectile slot 22h: id `19B9` = `$86:A387` (Ceres elevator pad); 0 when gone | spawned last, so the highest slot |
| the pad's wait | the projectile's timer `1B21` (`1AFF + 22h`) | 3Ch at the spawn (`$86:A2FB`) |
| the pad's position | `1A6D`, `1AB5` (`1A4B`, `1A93` + 22h) | x from `0AF6` at the spawn |
| the pad's look | spritemap `1B8D`: `8000h` none (spawned), `$B1BA` `First`, `$B1D0` `Second`; the list pointer `1B69` `$A291` / `$A295` with them | the list `$86:A28D`: two spritemaps, one frame each, looping |
| the cover | slot 20h: id `19B7` = `$86:A395` (the pad's level data concealer); position `1A6B`, `1AB3` (61h, `$86:A31E`); spritemap `1B8B` = `$846D` | |
| held | `0A42` = `$E8CD` (RTL) and `0A44` = `$E8DC` (locked: contact damage index normal, mini-map) | set by `$90:F1E9`; at the pad's end `$90:F2CA` sets `$E695` and `$E8AA` (Ceres: normal, then the escape's time-up check) |
| her height | `0AFA` | `0AFC` (subpixel) stays 0 |
| the beacon's step | the palette buffer `7EC052..7EC05D` (BG palette 2, colours 9h-Eh), CGRAM 29h-2Eh | from the table `$A6:F871`, 8 rows of 8 colours (10h bytes, the step × 10h), the first 6 written to colours 29h-2Eh; rows 0 and 7, 1 and 6, 2 and 5, 3 and 4 are equal |
| the platform's look | VRAM `060Eh..0611h` (the mode 7 tilemap) | `$A6:F900`: `$F904` `First`, `$F90E` `Second` |

### Inputs and order

| Spec | RAM / code | Notes |
| --- | --- | --- |
| the arrival | Samus command 8 (`$90:F1E9`), at `any-ceres-arrival` frame 192 (game state `1Fh` to 7) | sets `0A1C` = 0 (facing forward), `0AFA` = 0, spawns `$A387` then `$A395`, enables the mini-map (`05F7`), plays the room's music: the pose, the music and the mini-map are §5's |
| her height | `0AFA` when the projectile handler runs | main gameplay `$82:8B44`: Samus current state handler (`8B58`), enemies (`8B65`), new state handler (`8B69`), then the projectile handler (`8B6D`); the scrolling (`8B8A`) and the HUD after |
| when the lights are stepped | the Ceres door (`E23F`) with parameter 2, slot 0 in `DF45`, runs (`$A6:F765` → `$A6:F7BD`) | the function `$A6:F72B` + 2 × parameter (`$A6:F6E5`); the other Ceres doors run `$F76B`/`$F770`, so no other room steps the beacon. In the passage the enemies run in `$82:E29E`, `$E2DB` and `$E737` only: row 0001 cuts 254, 257, ... 291 |
| the count of finished frames | `05B6` at record N−1 | incremented in NMI (`$80:95F4`) only when the frame's work was done; never reset. The HUD's frame count is `05B5` (8-bit, `$80:95F1`), which the door passage resets; the beacon at cut 8 reads `1EA8` (record 7) |
| the fade's steps taken | `7EC400` (palette change numerator, world-doors' `t`) non-zero | `$A6:F854`. Fade out `$82:E2DB` advances the colours (`$82:D961` → `$DA02`, which increments the numerator) before the enemies, fade in `$82:E737` after them; the reset to 0 at the end (`$DA0F`). The arrival's fade in (game state 7) is brightness only and leaves it 0 (arrival 193-221) |
| the stage | `093F` 0 or 1; 2 or more (and `8000h`) `Escape` | at 2 or more the overlay's function turns to the rumble (`$A6:F7C1`), a later piece, which owns every escape frame of the shaft, its first run's step included |

Projectile order: the handler runs the slots from 22h down, so the pad before the cover: at row
0001 cut 100 the pad set her at 48h and the cover saw it the same frame.

### Sources per entry

| Entry | Sources |
| --- | --- |
| PadPlaced | `$90:F1E9`; `$86:A2EE`, `$86:A301`, `$86:A31B`; the headers `$86:A387`, `$86:A395` |
| PadWaits | `$86:A328`..`$86:A332` (decrement if non-zero; return while positive) |
| PadCarries | `$86:A334`..`$86:A363` (`CMP #$0049`, `BMI`: the signed compare on `y + 1 − 49h`, i.e. `y − 48h`); `$90:F2CA` |
| PadFlickers | `$86:A28D` |
| CoverHides | `$86:A364` (`CMP #$0048`, equal), `$86:A299` |
| BeaconCycles | `$A6:F7BD` → `$A6:F850` (`05B6 & 38h`, `$A9:D2E4`), the table `$A6:F871`; the door slot of parameter 2 (`0FB4` = 2, slot 0 in `DF45`) runs it; `$A6:F854` (the numerator) |
| the overlay's figure (§5) | instruction list `$A6:F610`: intangible (`$F68B`), spritemap `$A6:F921`, looping; tiles `$A6:F739` at the init |
| PlatformTurns | `$A6:F8F1`, `$A6:F900`, `$80:8B4F` |

### Findings behind the spec's choices

- **The stop the diagnosis didn't predict.** From row 0001's seed with her lock modelled, the
  first stop is the beacon at cut 8 (CGRAM 29h-2Eh), not the pad at cut 29; the beacon is also
  the `doors.colours.screen` stop at cut 104 from seed 101. So the beacon is in this piece.
- **Held** is the pad's presence: she is locked by the arrival on the frame it places the pad and
  unlocked by the pad's last run, and nothing else locks or unlocks her in these frames.
- **Placed, not run.** The pad's timer is 3Ch at frame 192 and 3Bh at 193: the spawn comes after
  the projectile handler's run that frame.
- **The wait at 0.** A timer of 0 skips the decrement and carries; a timer of 1 decrements to 0
  and carries in the same run (cut 29). Both are `PadWaits`' second arm.
- **The wait's range.** `DEC`/`BPL` is signed: a wait of 32,769 or more would carry on its first
  run and keep a non-zero wait, so the rule `w` is bounded at 32,768.
- **The cover's figure.** Its spritemap is `8000h` at frame 192 (in forced blank) and `$846D`
  from its first run, so the placing reports no figure for it.
- **No frame without a run follows the arrival.** `any-ceres-arrival` 193 to 222: game state 7
  (the fade-in) from 192, then 8 at 222; `05B6` rises by one a frame (no lag frame), and the
  pad's timer falls 59 to 30, one a frame, with its spritemap turning each frame. The fade-in's
  routine runs the projectile handler on each of its frames. So the cover's undrawn frame is the
  placing frame only (E-placed-drawn, E-first-drawn); the pieces' split needs no rest input.
- **When the lights are stepped.** On a frame the overlay doesn't run the caller doesn't step
  them: `chain-any-0001-DF45`:253 (the passage's `$E2DB` frame, count 8093) leaves the buffer
  and the tilemap unchanged (B2, formerly the vector B-not-run, retired with the input).
- **The escape (open question 4).** A room start in the escape still sets `$F7BD`: its first run
  steps the beacon and the platform, then switches to the rumble `$F7C1` → `$F7DC`; the rumble
  ends at `$F850` with no stage check (`$A6:F7EA`), and `$F67F` writes the stage `8000h`.
  Recorded at `chain-any-0010-DF8D` frame 239 (the fade in's first frame of work, `7EC400` 0):
  the buffer takes the table row for step 0 and the function becomes `$F7DC`.

### The harness (for the integrator)

- **Compared per frame** the projectiles run (game state 7 or 8): the pad's and the cover's slot
  words above, `0AFA`, `0A42`/`0A44` as held or not; per frame the enemies run (slot 0's `0FA4`
  advanced, slot 0 of `DF45` with parameter 2), with `093F` under 2: CGRAM 29h-2Eh against the
  step's colour set, unless the fade's steps taken are non-zero (world-doors' fade is the writer
  then), and VRAM `060Eh..0611h` against the look; on other frames both unchanged, but for the
  room's loading and world-doors' fade, each masked under its writer. The shaft's frames with
  `093F` 2 or more (`chain-any-0010-DF8D` from 239) are outside this piece's claim (B-escape).
- **Harness cases** with the game's content, loaded by the importer: H3, the beacon's eight
  colour sets (`chain-any-0001-DF45` cuts 1-250); H4, the platform's two looks (the same cuts).
- **Skipped:** the pad's and the cover's drawing (C1); her pose and her frame while held (§5).

**Recorded cases** behind the part-2 vectors (x moved to 0140, heights kept):

| Case | Row:frame | Vectors |
| --- | --- | --- |
| A1, A2 | `any-ceres-arrival`:192, 193 | E-placed, E-placed-drawn; E-first-run, E-first-drawn |
| A3, A4, A5 | `chain-any-0001-DF45`:1, 28, 29 | E-wait-counts, E-wait-two, E-wait-last |
| A6, A7, A8 | `chain-any-0001-DF45`:30, 100, 101 | E-carry, E-carry-to-floor, E-floor |
| B1 | `chain-any-0001-DF45`:8 | B-step |
| B2 | `chain-any-0001-DF45`:253 (no run in the passage) | none: the caller doesn't step the lights |
| B3 | `any-ceres-arrival`:193 (the brightness fade in) | B-arrival |
| B4 | `chain-any-0001-DF45`:350 (`05B6` 8159, `05B5` 1) | B-count-not-hud (its counts) |
| P1, P2 | `chain-any-0001-DF45`:1, 2 | P-first, P-second |

## Budget

Moved from the spec's header (MODEL §1 decision 10): the implementer is not told the score or
the budget. `tools spec-units` reads k and the areas here.

**Budget (MODEL §1 decision 7).** k = 37.5 weighted tokens per spec unit, frozen on
samus-movement. There is no old code: the engine carries the doors unrun
(`EnemyBody::Unmodelled`, LEGACY "slots it carries but doesn't run") and has nothing for the pad,
the cover, the beacon or the platform, so no area has `old`, and there is no old-code guideline
to compare (MODEL §1 decision 19).

- Area doors: 1, Waiting, SheIsNear, RoomStart, Polling, Opening, Closing, ShaftWall, Solid, Shown
- Area station: EscapeShakes, TheDoorsColours
- Area shaft: PadPlaced, PadWaits, PadCarries, PadFlickers, CoverHides
- Area lights: BeaconCycles, PlatformTurns

Budget: 37.5 × 50.3 units = 1,887 (doors 950, station 200, shaft 462, lights 275)
