# Ceres Ridley and the escape, parts 1 to 3, the Ceres doors, the shaft as she arrives and Ridley's room as she enters: decode notes

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

## Part 3, Ridley's room as she enters

### How it was made

Behaviour first, from CI's lean record cache: `chain-any-0005-E06B` frames 170-320 (the passage
into Ridley's room `E0B5`, placed at 183-184) and `chain-any-0006-E0B5` frames 0-1785 (the room
to the passage out), read per frame by scratch readers of the lean tier
(`$TEMP/collab/ceres-748/scratch/`, local only: `dump.py`, `events.py`, `door.py`, `rid2.py`,
`cg.py`, `pal.py`): the enemy slots, her position and energy, `093F`, `7ED82E`, `183E`/`1840`,
`05E5`, `0797`, the baby's words, CGRAM and the palette buffers. The bank log A6 (`$F4EC..FB71`,
`$A0F5..A388`, `$A2DC..A353`, `$BE93..C04D`) then named the steps and pinned the numbers.

The reference model, `$TEMP/collab/ceres-748/scratch/model3.py`, decodes the door (parameter 3),
the walls (5, 6) and Ridley (`E13F`, slot 0) at record N and steps each when its `0FA4` advanced
by one from N to N+1, with her position, `05E5` stepped once (part 1's `rng`), `1840` and `0797`
at N, and `093F` and `7ED82E` bit 0 at N+1 (Ridley runs first, so the door reads what his run
left). It compares phase, wait, figure, shown, solid and drawn-with-Ridley (`0FAA`) at N+1:
- `chain-any-0005-E06B`: 15 runs of the door, 15 agree; do-nothing 0. 121 frames without a run,
  121 unchanged. The placing at 184 is `Placed`.
- `chain-any-0006-E0B5`: 2,564 runs of the door and the walls (to the passage's end at 1690),
  2,564 agree; do-nothing 934. 189 frames without a run, 189 unchanged. The walls' placing at
  1157 is `Placed`, stage `Calm`.
- The door's shakes in the escape (`chain-any-0006-E0B5` 1586-1609, game state 8): 24 of 24
  frames agree on `183E`/`1840` at N+1 (a start: type `1Dh` or `23h`, timer less the frame's
  count-down; no start: unchanged type, timer down by one).
- Ridley: `chain-any-0005-E06B` 15 runs, 15 agree; `chain-any-0006-E0B5` 0-512, 512 runs, 512
  agree (phase, wait, look, the baby's routine and runs); do-nothing 0 and 0. The baby's colours
  (`7EC162..7EC17F`, sprite palette 3 colours 1-Fh): over the 527 runs the row stays as it was
  on every run that names no set, and each named set shows the same colours each time it is
  named (one cry, at 145; its first set, `Glow1`, matches the row as it stood).
- `model3.py vectors` runs every part-3 vector: 29 of 29 door and wall vectors and 15 of 15
  Ridley vectors agree.

### State (the RAM mapping)

| Spec | RAM | Notes |
| --- | --- | --- |
| `RidleyDoor` | `E23F` with parameter 1 (`0FB4`) = 3 | `0FA8` `$F770` (the escape's shake type `1Dh`) |
| `GetawayWall` `Left`, `Right` | parameter 5, 6 | `0FA8` `$F7A5`; spawned by `$A6:AA20` at (8, 7Fh) and (F8h, 7Fh) with properties `2800h` (no `8000h`: never solid to her) |
| `Placed` | `0F92` `$F53A`, `$F62A`, `$F634`; `0F8E` `$804D` | |
| `Lingering` | `$F542` | intangible, invisible, `$FAA7` |
| `Slamming(1..4)` | `$F54A`, `$F54E`, `$F552`, `$F556` | `$FAA7`, `$FA87`, `$FA67`, `$FA3D` |
| `Slammed` | `$F55C` | `0FAA` = 1 (`$F69F`), `$FA13`, visible |
| `Locked` | `$F562` | invisible, `0FAA` = 1; `$F66A` reads `7ED828 + [079F]` bit 0 |
| released | `$F6B0` clears `0FAA`, then `$F598` → `$F5A0` (`Closed`); part 1's right-facing table from then on | |
| `Standing`, `Unseen` | `$F630`, `$F63A`, `0F86` bit `0100h` | `$F7A5` sets invisible, then visible when `093F` = 1 (`LSR`, carry), before the instructions |
| `Side` | `$FACE` (left), `$FB2F` (right) | |
| Ridley beaten | `7ED82E` bit 0 (area 6's boss) | set with `093F` = 2 at `chain-any-0006-E0B5`:1586 |
| the `Own` palette | sprite palette 2 (`0F96` = `0400h`), target `7EC342..7EC35F` | only when `093F` = 0 at the init (`$A6:F6FC`) |
| `RidleyPhase` | `0FA8`: `$A35B` `Holding`, `$A377` `Waiting`, `$A389` (and later) `Awake` | `$A35B` falls through into `$A377`, so the first run after the passage counts |
| his wait | `0FB2` | `DEC`/`BPL`: the wake at -1 |
| `Rest` | `0F8E` `$E983` (list `$E538`, facing left; then sleep, `0F92` `$E540`) | the instruction timer `0F94` wraps through 0 and is not compared |
| his position | `0F7A`, `0F7E` = `BAh`, `A9h` (`$A6:A1F7`); the population's `ABh` is overwritten | |
| `BabyRoutine` | `7E7806` (list pointer), `7E7808` (runs, counting up, `$A6:DBCB`) | `$BF31` `New`; `Fidget(0..7)` `$BF35`..`$BF41`, `$BF49`..`$BF55`; `Glow(j)` `$BF61 + 8j` |
| `BabyLook` | `$BFFD` `First`, `$C018` `Second`, `$C033` `Third` | |
| the baby's position | `7E8804`, `7E8808` (`$BE9C`) | |
| the baby's colours | `$E1F1` `Rest`, `$E20F` `Glow1`, `$E22D` `Glow2`, `$E24B` `Glow3`, to sprite palette 3 colours 1-Fh (`$A9:D2E4`, the current colours) | |

### Inputs and order

| Spec | RAM / code | Notes |
| --- | --- | --- |
| a passage under way | `0797` (door transition flag) as Ridley runs | set through the passage's last frame of work (`chain-any-0005-E06B`:320, game state 8, still set) |
| the random number | `05E5` as the baby reads it (`$A6:BFCF`, bit 0, not stepped) | N's stepped once, as part 1's doors |
| order | slot 0 Ridley, slot 1 the door, slots 2 and 3 the walls | the main AI `$A6:A288` runs his function, then (`093F` = 0, `7E7804` = 0) `$A2DC`: the baby's function, then falls into `$A2F2`, which draws the baby (`$BF1A`) and, with `0FEA` set, the door (`$A329`) |

### Outputs

| Spec | RAM / code |
| --- | --- |
| a boss encountered | `$90:A7E2` from the init (`05F7` = 1), `chain-any-0005-E06B`:184; interface-hud's `BossBlanksTheMap` (Ridley in Ceres: no screens) |
| the music's stop | `$80:8FC1` with 0 |
| the shake's stop | `183E`, `1840` = 0 |
| the stage | `093F` = 0 |
| his room's colours | target sprite palettes 2-3 from `$A6:E16F`; target sprite palette 7 colours 1-Fh = 0 |
| the baby's cry | `$80:914D` with A = `24h`, library 3, limit 6 |
| beaten at the placing | `$A6:A101`: `0F86` \|= `0700h` (intangible, invisible, deleted) and return |

### Sources per entry

| Entry | Sources |
| --- | --- |
| RidleyRoomStart, SlamShut, LockedIn, DrawnWithRidley | `$A6:F53A..F56B`, `$F66A`, `$F69F`, `$F6B0`, `$F6C5..F72A` |
| GetawayWalls | `$A6:F62A..F63D`, `$F7A5`, `$AA20` |
| EscapeShakes (Back) | `$A6:F770`, `$F773` |
| TheDoorsColours (Own) | `$A6:F6FC..F711` |
| RidleyPlaced | `$A6:A0F5..A287` |
| RidleyWaits | `$A6:A288`, `$A35B`, `$A377` |
| BabyHeld | `$A6:BE9C` |
| BabyFidgets, BabyCries | `$A6:BF31..BFF8`, `$DBCB`, `$BFC9`, `$BFE1` |

### Findings behind the spec's choices

- **The door alone unblocks nothing.** The passage's load (`chain-any-0005-E06B`:183-184) places
  Ridley in slot 0 before the door; the played run refuses either. So part 3 holds his placing
  and waiting too, up to his waking at `chain-any-0006-E0B5`:512 (his function `$A389`; the eye's
  colours, CGRAM `FCh..FEh`, change from 514).
- **The baby is drawn while he waits.** With `7E7804` = 0 the main AI jumps into `$A2F2` itself
  each run, so the baby and the shut door are drawn by his run, without the graphics hook.
- **The glow is the screen's colours.** `$A9:D2E4` writes the current palette, so the played run's
  screen colours see the glow (row 0006, 151-207); the fade's goal colours don't change.
- **Two writers of the `Own` palette.** His init writes target sprite palettes 2-3 (`$E16F`, 20h
  colours), then the door's init writes palette 2 colours 1-Fh with the calm set; both give the
  same colours in the records.
- **The walls are placed while calm** (`chain-any-0006-E0B5`:1157, `093F` 0; it becomes 1 at
  1158, in Ridley's run before theirs), so their placing writes the calm set to the doors' palette
  target; the screen's palette 7 then holds his getaway's colours, which a later piece owns.
- **The door's shake reads the stage after Ridley's run**: at 1586 his run sets `093F` = 2 and
  `7ED82E` bit 0, and the door, run after him, is let go and shakes on the same frame.
- **Rows past this piece**: the fight from 0006:512 (fade in, roar, five contact hits at 609,
  705, 801, 897 and 993, energy 99 to 24, nine hurt flashes of his from 651), the retreat (994), the
  getaway (1158, `093F` 1, mode 7), the self-destruct text (1270), the escape (1586) and the
  passage out (1610).

### The harness (for the integrator)

- **Rows:** `chain-any-0005-E06B` from 184 and `chain-any-0006-E0B5` to 512 for Ridley; to 1690
  for the door and the walls.
- **Compared per run** (`0FA4` advanced): the door and wall slots' `0F92` through the phase
  table, `0F94`, `0F8E` through the figures, `0F86` bits `0100h` and `0400h`, `0FAA`; Ridley's
  `0FA8` through `RidleyPhase`, `0FB2` while waiting, `0F8E`, the baby's `7E7806`, `7E7808`,
  `7E8804`, `7E8808`; the screen's sprite palette 3 colours 1-Fh on a frame the model names a set.
  On a frame without a run, all unchanged.
- **The escape's shakes** as part 1, type `1Dh`/`23h` for the door.
- **Sounds** (the music's stop, the baby's cry `24h`, the door's opening `2Ch`): produced, not
  compared (MODEL decision 37).
- **Skipped:** Ridley's instruction timer `0F94` (it wraps and nothing reads it while he waits);
  his `0F8C` (written 7FFFh each run); `7E2000..2FFF` (cleared at the init, his later pieces');
  the drawing.

### The harness cases

| Case | Row:frame | What | Checks |
| --- | --- | --- | --- |
| H5 | `chain-any-0005-E06B`:184 | Ridley's room placed while calm: target sprite palettes 2-3 take the room's sets, palette 2 colours 1-Fh the doors' calm set, palette 7 colours 1-Fh black | RidleyPlaced, TheDoorsColours |
| H6 | `chain-any-0006-E0B5`:145-207 | the baby's four colour sets | BabyCries |

**Recorded cases** behind the part-3 vectors (door at `0140,0090`, the walls at `0100,0200` and
`01F0,0200`, her offsets kept; Ridley at his place):

| Case | Row:frame | Vectors |
| --- | --- | --- |
| N1 | `chain-any-0005-E06B`:184 | R-placed-calm |
| N2, N3 | `chain-any-0005-E06B`:281, 283 | R-first-run, R-linger-wait |
| N4, N5, N6 | `chain-any-0005-E06B`:286, 292, 310 | R-slam-start, R-slam-step, R-slam-last |
| N7, N8, N9 | `chain-any-0005-E06B`:313, 316, 319 | R-locked, R-locked-wait, R-locked-again |
| N10, N11 | `chain-any-0006-E0B5`:1586, 1588 | R-released, R-released-opens |
| N12 | `chain-any-0006-E0B5`:1157 | W-placed |
| N13, N14 | `chain-any-0006-E0B5`:1158, 1159 | W-first-run, W-right, W-standing |
| N15 | `chain-any-0006-E0B5`:1586 | W-hides |
| N16 | `chain-any-0006-E0B5`:1610 | R-shake-strong-back |
| K1, K2 | `chain-any-0005-E06B`:184, 281 | K-placed, K-first-run |
| K3, K4, K5 | `chain-any-0006-E0B5`:0, 1, 512 | K-first-wait, K-wait-counts, K-wake |
| K6, K7 | `chain-any-0006-E0B5`:65, 145 | K-calm, K-cries |
| K8, K9, K10 | `chain-any-0006-E0B5`:151, 207, 5 | K-glow-next, K-glow-last, K-fidget-next |

## Part 4a, Ceres Ridley wakes

### How it was made

Behaviour first, from CI's lean record cache, row `chain-any-0006-E0B5` cuts 512-867 (his waking
to the run he turns to fly), read per frame by scratch readers of the lean tier
(`$TEMP/collab/fight-753/scratch/`, local only: `dump.py`, `ev.py`, `pal.py`, `cases.py`,
`runs.py`, `oam.py`): his slot 0 (`0FA8`, `0FB0`, `0FB2`, `0F86`, `0F8E`, `0F92`, `0F94`,
`0F7A`/`0F7E`), `7E7804`, `7E781E`, the baby's words, the current palette and CGRAM. His frame
count `0FA4` steps by one on every frame 505-870, so every frame is a run. The bank log A6
(`$A389..A4D5`, `$D467`, `$E269`, `$E2AA`, `$E30A`, `$E4BE`, `$E4CA`, `$E517`, `$E690`) then named
the steps and pinned the numbers.

The reference model, `$TEMP/collab/fight-753/scratch/model4a.py` (it borrows part 3's baby from
`$TEMP/collab/ceres-748/scratch/model3.py`), decodes him at cut N and steps him once to N+1 with
the game's content (read from the bank log into a local scratch module, never committed):
- `chain-any-0006-E0B5` 513-867: 355 runs; the phase with its count, runs, wait, figure and the
  figure's runs agree on 355 (at 867 only the phase `TakingOff`, the rest being part 4c's);
  do-nothing 0. Tangible (`0F86` bit `0400h` clear), moving (`7E7804` = 1), roaring (`7E781E`
  non-zero), his look (`0F8E`, but at 867) and his position: 355 of 355.
- His colours, 355 of 355 runs: on a run the model names an eye set, sprite palette 7 colours
  Ch..Eh hold that set's colours (exactly, against the bank log's data) in the current palette
  and in CGRAM; on a run it names a body set, sprite palettes 1 and 7 colours 1..Bh hold it; on
  every other run those colours are unchanged.
- The baby, part 3's rules: 355 of 355 (do-nothing 0); one cry, at 767.
- `model4a.py vectors` runs the 29 F- vectors: 29 of 29 agree; part 3's `model3.py vectors`
  still agrees on its 15 Ridley vectors.

### State (the RAM mapping)

| Spec | RAM | Notes |
| --- | --- | --- |
| `Awake` | `0FA8` = `$A389`, `0FB0` = 0 | part 3's `$A377` sets it at the wake, `chain-any-0006-E0B5`:512 |
| `EyeFading`, steps shown `n` | `0FA8` `$A389`, `0FB0` = `n` | `0FB2` counts to 1 and is cleared on the same run (the eye's pace, 1, is fixed); `0FB0` negative returns at once (never seen) |
| `BodyFading`, sets `s`, runs `t` | `0FA8` `$A3DF`, `0FB0` = 22 × `s` (bytes into `$E30A`), `0FB2` = `t` | the end is `0FB0` ≥ `160h` after the write: 16 sets of 11 colours |
| `ReadyToRoar`, wait | `0FA8` `$A455`, `0FB2` | `DEC`/`BPL`: roars at −1 |
| `Roaring`, wait | `0FA8` `$A478`, `0FB2` = `FCh` at the start | Ceres only: the Norfair branch (`079F` = 2) fades the room's background instead and is not this domain's |
| the roar's figure and runs | `0F92` = `$E698`, `$E69E`, `$E6A2`, `$E6A6`, `$E6AC` for figures 1 to 5; runs = length − `0F94` + 1 | list `$E690`, facing left (`7E7820` = 0, `$E517` falls through); figure 5 is `0001,E983` then sleep (`$812F`) at `$E6AC` |
| `TakingOff` | `0FA8` `$A354` | set with list `$E91D`, wings `7E7810`/`7E7812` = 8, tail neutral (`$D3D9`, `7E2000` = 1), `0FB0` = 0: part 4c's |
| `Rest`, `Open`, `Wide` | `0F8E` `$E983`, `$E9C7`, `$E9E9` | the facing-right figures (`$E9A5`, `$EA0B`, `$EA2D`, list `$E6AE`) are Norfair's |
| tangible | `0F86` bit `0400h` clear (`3C00h` to `3800h` at 609) | |
| moving | `7E7804` = 1 (`$A3D7`) | from then the main AI no longer draws the baby itself; the graphics hook does |
| roaring | `7E781E` = `59h` (`$E4BE`) to 0 (`$E4CA`) | read by `$A6:D2C1` (tail whip sound) and `$B674` (fireballs) |
| the eye's steps | `$A6:E269`, 64 bytes then `FFh` | each × 6 indexes `$E2AA` |
| the eye's colour sets | `$A6:E2AA`, 16 sets of 3 colours | to `7EC1F8..7EC1FD` |
| the body's colour sets | `$A6:E30A`, 16 sets of 11 colours | to `7EC122..7EC137` and `7EC1E2..7EC1F7` |
| his second palette | sprite palette 1 (`7EC120..7EC13F`) | the palette he is drawn in is sprite palette 7, the doors' (part 3) |

### Outputs

| Spec | RAM / code |
| --- | --- |
| the eye's colours | current sprite palette 7 colours Ch..Eh (`$A3B5..A3C9`); no target palette is written |
| the body's colours | current sprite palettes 1 and 7 colours 1..Bh (`$A3FD..A419`) |
| the fight's music | `$80:8FC1` with A = 5 (`$A44D`), `chain-any-0006-E0B5`:609 |
| the roar | `$80:90CB` with A = `59h`, library 2, limit 6 (`$E4BE`), `chain-any-0006-E0B5`:620 |
| tangible | `0F86` &= `FBFFh` (`$A432`) |

Sprite palette 1 is drawn by none of his sprites in 0006 (OAM at 620 and 700 uses palettes 0, 2, 3,
4 and 7); the screen's colours still hold it, so it is an output.

### Sources per entry

| Entry | Sources |
| --- | --- |
| EyeFadesIn | `$A6:A389..A3DE`, `$E269`, `$E2AA` |
| BodyFadesIn | `$A6:A3DF..A454`, `$E30A` |
| ReadyToRoar | `$A6:A455..A477`, `$D467` |
| Roars | `$A6:A478`, `$A4AC..A4D5`, `$E690`, `$E4BE`, `$E4CA`, `$E517` |

### Findings behind the spec's choices

- **The step table and the two colour lists are content.** The eye's 64 steps name 16 sets (the
  first 16 steps walk the sets from black to full, then 48 hold the last); the body's 16 sets run
  black to full. Their lengths come from the content, so the spec states the hand-overs as "every
  step shown" and "the last set", and the vectors use invented, shorter content.
- **The eye's first step changes nothing on screen** (0006:513): the game's first step names the
  black set. The colours change from 514 to 528 and hold from 529.
- **The body's first run shows no set** (578): the run that ends the eye's fade (577) starts the
  body's with its runs at 0, so the first set shows on 579 and the 16th on 609.
- **The roar's figure list is run by the framework's instruction step in the original**; the spec
  keeps the look in the domain, as part 3 does for `Rest`. The list is set with its timer at 1, so
  figure 1 shows on the run the roar starts (614).
- **The hand-over at 867** sets the flying list, the wings and the tail in the same run; the spec
  gives that run to part 4c, and the model compares only the phase there.
- **Rows past this part, by later part:** the contact hits at 609, 705, 801, 897 and 993 (part 4b,
  H1 `TailHit`; her knockback, samus-movement; her hurt colours, sprite palette 4, samus-appearance);
  his nine hurt flashes from her beam, 651-857 (drawing only, no palette write: C1 and a later part);
  his flight from 867 (part 4c).

### The harness (for the integrator)

- **Rows:** `chain-any-0006-E0B5` 512-866 for this part (867 for the phase `TakingOff` only).
- **Compared per run:** `0FA8` through the phases above, `0FB0` and `0FB2` as the phase's count,
  runs or wait, `0F92`/`0F94` as the roar's figure and runs while `Roaring`, `0F8E` through the
  looks, `0F86` bit `0400h`, `7E7804`, `7E781E` non-zero; the baby as part 3; the screen's sprite
  palette 7 colours 1..Eh and sprite palette 1 colours 1..Bh on every run.
- **Sounds** (the fight's music, track 5; the roar `59h`): produced, not compared (MODEL
  decision 37).
- **Skipped:** `0F94` outside the roar (it wraps, as part 3); the drawing; `7E2000..2FFF` and the
  wings' words (part 4b and 4c); her energy, position and colours from 609 (other domains).

**How the played frame runs it:**
1. Production (`crates/game/src/production/played/ridley.rs`, `run`) steps him through this
   part's phases. From the run that makes him tangible his tail can hit her (part 4b,
   below). Part 4a refused a run that started `TakingOff`; since part 4c-1a he takes off and
   flies through production, and a run that starts in an attack or as he gives up is refused
   (`Fault::RidleyAttacks`, `Fault::RidleyGivesUp`, later parts).
2. His colours go to the screen as the baby's do (`production/ridley.rs`, `show_eye` and
   `show_body`): the eye's set to sprite palette 7 colours 12-14, the body's set to sprite
   palettes 1 and 7 colours 1-11, at once.
3. The verify reader (`crates/verify/src/replay/ridley.rs`) decodes `$A389` with `0FB0`, `$A3DF`,
   `$A455`, `$A478` with the figure from `0F92`/`0F94`, and `$A354` as `TakingOff`.
4. The importer brings in the eye's step table (`$E269`, to its `FFh` before the eye's sets),
   the eye's 16 colour sets (`$E2AA`) and the body's 16 colour sets (`$E30A`) as content of
   Ceres Ridley; no ROM bytes in the vectors or the crate.
5. The vector harness, `crates/clean/boss_ceres_ridley/tests/vectors.rs`, parses the F- form
   (`content`, the new phases and `ridley_after` fields).

Played alone, room 6 (`chain-any-0006-E0B5`) holds through cut 538, the eye's first 26 of its 64
steps; it stops at cut 539 in `movement.pose`, her crouched turn ending (samus-movement's). The
body's fade, his readying, his roar and the two refusals hold by the vectors and the production
tests only. Past them the first contact hit, at 609, needs part 4b, S4 knockback and
samus-appearance's hurt colours.

### The harness cases

| Case | Row:frame | Vectors |
| --- | --- | --- |
| F1, F2, F3, F4 | `chain-any-0006-E0B5`:513, 514, 576, 577 | F-eye-first, F-eye-step, F-eye-last, F-eye-done |
| F5, F6, F7, F8 | `chain-any-0006-E0B5`:578, 579, 607, 609 | F-body-wait, F-body-first, F-body-set, F-body-last |
| F9, F10, F11 | `chain-any-0006-E0B5`:610, 613, 614 | F-ready-counts, F-ready-last, F-ready-roars |
| F12, F13, F14 | `chain-any-0006-E0B5`:615, 620, 628 | F-roar-counts, F-roar-opens, F-roar-wide |
| F15, F16, F17 | `chain-any-0006-E0B5`:724, 732, 733 | F-roar-shuts, F-roar-rests, F-roar-held |
| F18, F19, F20, F21 | `chain-any-0006-E0B5`:866, 867, 767, 700 | F-roar-last, F-roar-flies, F-baby-cries, F-roar-wide-counts |
| H7 | `chain-any-0006-E0B5`:513-609 | the eye's and the body's sets with the game's content (indexed replay) |

## Part 4b, his tail at rest and its hit

### How it was made

Behaviour first, from CI's lean record cache, row `chain-any-0006-E0B5` cuts 570-1000, read per
frame by scratch readers (`$TEMP/collab/tail-753/scratch/`, local only: `td.py`, `tc.py`,
`chg.py`, `hits.py`, `curl.py`, `ext.py`): the tail's block `7E2000..20AF` (its function index,
angle delta, curl, seven segments of 14h bytes), his position `0F7A`/`0F7E`, `0F86`, `7E7804`,
`7E7836`, `7E7838`, and her `0AF6`/`0AFA`, `0AFE`/`0B00`, `18A8`, `18AA`, `0A54`, `09C2`. The
bank log A6 (`$A2BD`, `$CAF5..CB20`, `$CEBA..CF53`, `$CF5A`, `$D2AA`, `$D2D6..D3B3`,
`$DF29..DF58`, `$DFD9..E01A`, the main AI `$A288..A2BC`) then pinned the arithmetic.

The reference model, `$TEMP/collab/tail-753/scratch/model4b.py` (it borrows part 4a's model and
part 3's baby), on 0006 577-866, the 290 runs that end in `BodyFading`, `ReadyToRoar` or
`Roaring`:

| Compared | Model | Do-nothing (last run's value) | No tail (none, no hit) |
| --- | --- | --- | --- |
| the tip (`20A4`, `20A6`) | 290 | 289 | 0 |
| all seven segments at the tip (`202C`/`202E` + 14h × i) | 290 | 289 | 0 |
| the curl (`201E`) | 290 | 290 | |
| a hit (`18A8` 0 to 5Fh) | 290 (3 hits: 609, 705, 801) | 287 | 287 |
| its side (`0A54`) | 290 | 287 | 287 |

Every hit's push time is 5 (`18AA` 4 at the hit's record) and costs 15 whole energy (`09C2`). The curl rule
also holds on every run from 513 to the escape, 1,073 of 1,073 (one change, at 945, in his
flight). Beyond this part the hit rule on the recorded tip holds 867-1585, 719 of 719 runs,
with the two flight hits (897 and 993, both `Left`): part 4c needs only the moving tip.
`model4b.py vectors` runs the 23 T- vectors: 23 of 23; `model4a.py vectors` still agrees on
its 29 F- vectors.

### State (the RAM mapping)

| Spec | RAM | Notes |
| --- | --- | --- |
| the tail's place, at rest | `7E202C`/`202E` + 14h × i, i = 0..6; the tip is segment 6, `7E20A4`/`20A6` | `$CEBA`: segment 0 at his position + (`$CF54` 20h + its x offset, its y offset + 10h), each next one at the last + its offsets; at rest every offset (`2030`/`2032` + 14h × i) is 0, since tail function 0 (`$CB20`, `RTS`) never updates the angles |
| at rest | `7E2000` = 0 | set to 1 (neutral) at the take-off (`$A354`): part 4c |
| the tail's curl | `7E201E`, the ideal angle between segments | 10h from `$D2D6` at his placing (`$A156`; 0005:184); `$A2BD` on every moving run |
| the tail's reach (`tail_reach`) | `#$000E` at `$DFD9` (both axes) | a rule key of its own: the doors' `reach` (48, `SheIsNear`) is another Parameter |
| the push time | `#$0005` to `18AA` at `$E003` | 4 at the hit's record: movement's step counts it down the same frame |

Untouched at rest: the segments' distances (`2026` + 14h × i: `$CF5A` shrinks only above 800h,
and the distances start at 200h, 800h ... 500h), the angle delta (`2014` = 1, so the whip's
sound, `$D2AA`, needs 8 or more), the whip targets.

### Inputs and order

- The tail runs inside his main AI after his function (`$A28E` executes `0FA8`, then `$A299`
  checks `7E7804`), so the run that turns him tangible (`$A432`, 609) is judged tangible.
- Her position and radius are read as the frame starts (the root's ruling, from game state 8's
  order, `$82:8B44`: see part 4c-1a's findings below): his run reports the hit, Health applies
  it, then her moves and her movement's reaction starts the push, all in the same frame. Each
  recorded case takes her place from the record before the hit's: T-hit-left (216, 192) from
  608, T-protected (216, 187) from 703, T-hit-right (220, 187) from 704, T-hit-jumping
  (234, 159) from 800. At 609 the hit's record shows y 187, after the push's 5 px pose fit. Her
  body is the one she started the frame in: the hit's record keeps the old body (16 at
  0006:609, 12 at 993) and the new one shows from the next frame; read from her movement's
  content for the pose she began the frame in.
- Her protection is read before her own step counts it down: at 704 the record before shows
  1 and no hit lands; at 705 the record before shows 0 and the hit lands.
- The random number is `05E5` as his run reads it; its value at the record agrees on all 1,073
  runs. The baby's routine reads the same word later in the run (part 3).
- `7E7836` (the hold) is 0 on every run of the Ceres rows (0006 0-1776). It is not "never set in
  Ceres": the fake retreat sets it at `$A6:BE45` (`$BE03`, his feet closing on the baby), reached
  from `$A6E8` to `$BD9A` once his hit counter `7E781A` reaches 100 (`$A6EC`); only `$BCA7`
  (Norfair) and `$9664` (init) clear it, so from then on the tail hits her no more. The any% row
  goes `A6E8` to `A971` at 994 instead. Part 4c (or the retreat's part) states it.

### Outputs

| Spec | RAM / code |
| --- | --- |
| the frame's hit | `$DFD9..E01A`: `18A8` = 60h, `18AA` = 5, `0A54` = 0 (left) or 1 (right); its worth, 15 whole energy: `7E7838` = 0Fh, set at his placing (`$A6:A216..A219`), taken through `$A0:A45E` and `$91:DF51` (samus-health's `TailHit`); 99 to 84 at 0006:609 |
| the tail's place | `7E202C..20A7` (drawing, `$DB2A`) |

### Sources per entry

| Entry | Sources |
| --- | --- |
| TailAtRest | `$A6:CAF5..CB0D`, `$CB20`, `$CEBA..CF53`, `$CF54`, `$CF5A` |
| TailCurls | `$A6:A2BD..A2D2`, `$D2FD` |
| TailHits | `$A6:CB0E..CB1D`, `$DFD9..E01A`, `$DF29..DF58`; the worth `$A6:A216..A219` (`7E7838`) |

### Findings behind the spec's choices

- **The reach is a box, not a radius.** `$DF29` compares `|dx|` with her x radius + 14 and
  `|dy|` with her y radius + 14, each by a borrow and then a strict compare: so the edges are
  `<` on both axes. The differences are 16-bit, made absolute from their sign (`BPL`/negate).
- **The side** is the sign of her x less the tip's x (`BMI`): 0 counts as right.
- **His tangibility, her protection and his hold** are one `ORA` test at `$CB0E`; the spec
  keeps the first two and leaves the hold to the part that owns the fake retreat (set at
  `$BE45`, after 100 hits, `$A6EC`); in this part it is always clear.
- **The tail is at rest on all of part 4a's moving runs.** Its block changes only at 577 (the
  segments' places set); the curl changes first at 945. The flight's tail (from 867) is an
  articulated chain of angles, distances and whips (`$CBC0`, `$CC39`, `$D09F`, `$D19D`): part
  4c, which this part's `TailHits` serves unchanged.
- **His wings** (`7E780A`/`780C`) animate from 577 and are drawing only; left to C1 (and part 4c
  for their speed in flight).
- **The tail's place before he moves** is the initial table (`$D398`/`$D3A6`, not his position):
  never read, as the tail doesn't run then; the spec reports none.

### The harness (for the integrator)

- **Rows:** `chain-any-0006-E0B5` 577-866.
- **Compared per run:** the tip `20A4`/`20A6` and the seven segments' places; `201E` as the curl;
  the frame's hit as `18A8` going from 0 to 5Fh, with `0A54` as its side and `18AA` + 1 as its
  push time (movement's step has spent one); her energy and the rest of the push are
  samus-health's and samus-movement's.
- **Skipped:** the tail's drawing (C1); everything else in `7E2000..20AF` (angles, distances,
  stagger, whip targets: unchanged at rest and part 4c's).

**What the integrator needs:**
1. Production: `run_ridley` produces the frame's hit before her moves, and routes it to
   samus-health (`TailImpact`, worth 15) and samus-movement (the frame's hit, for its reaction
   after her moves); her position, her body and Health's protection as the frame starts come in.
   Part 4b's production reads her position as her moves leave it (`played.rs`, `station.rs`);
   on every indexed hit (0006 609, 705, 801) the two readings give the same hit and side, so
   moving the read to the frame's start changes no played outcome. It is the integrator's.
2. The verify reader (`crates/verify/src/replay/ridley.rs`) decodes the tip, the segments and the
   curl, and the hit from `18A8`/`0A54`/`18AA` as above.
3. The vector harness, `crates/clean/boss_ceres_ridley/tests/vectors.rs`, parses the T- fields
   (`ridley.curl`, `her`, `ridley_after.curl`, `tail`, `hit`, `rules.tail_reach`); until it does, its
   `deny_unknown_fields` rejects the file.

With part 4a, S4's knockback and samus-appearance's hurt colours, the played run's stop moves
from 609 to 867, his take-off (part 4c).

### The harness cases

| Case | Row:frame | Vectors |
| --- | --- | --- |
| T1 | `chain-any-0005-E06B`:184 | T-placed |
| T2, T3, T4 | `chain-any-0006-E0B5`:577, 608, 609 | T-rest-first, T-rest-intangible, T-hit-left |
| T5, T6, T7 | `chain-any-0006-E0B5`:704, 705, 801 | T-protected, T-hit-right, T-hit-jumping |
| T8 | `chain-any-0006-E0B5`:945 (his flight; the same rule) | T-curl-recorded |

## Part 4c-1a, his take-off and flight

### How it was made

Behaviour first, from CI's lean record cache, row `chain-any-0006-E0B5` cuts 860-1000, read per
frame by scratch readers of the lean tier (`$TEMP/collab/ridley-4c/scratch/`, local only:
`rec4c1.py`, `cases.py`; the reader is `$TEMP/collab/escape-survey/scripts/lean.py`): his slot 0
(`0FA8`, `0F7A`/`0F7C`, `0F7E`/`0F80`, `0FAA`, `0FAC`, `0F92`/`0F94`, `0F8E`, `0F82`/`0F84`),
`7E7800`, `7E7802`, `7E781A`, `7E783E`, `7E8000..8007`, the baby's `7E8804`/`7E8808`, `09C2`,
`05E5`, `183E`/`1840`. His frame count `0FA4` steps by one on every frame 866-994, so every frame
is a run. The bank log A6 (`$A288..A2BC`, `$A354`, `$A6AF..A781`, `$D62F..D711`, `$D712`,
`$D86B..D954`, `$E517`, `$E51F`, `$E91D..E975`) and A9 (`$EF06`, local copy
`$TEMP/collab/ridley-4c-ref/`) then pinned the arithmetic.

The reference model, `$TEMP/collab/ridley-4c/scratch/model4c1.py`, decodes him at cut N and
steps him once to N+1, with her energy and his hits at N and the random number at N+1:

| Compared, 0006 867-994 (127 runs from `TakingOff` on, 994 the hand-over) | Model | Do-nothing (last run's value) |
| --- | --- | --- |
| the phase (`0FA8`) | 127 | 123 |
| his whole x, y (`0F7A`, `0F7E`) | 127, 127 | 113, 48 |
| his fractions (`0F7D`, `0F81`) | 127, 127 | 48, 14 |
| his speeds (`0FAA`, `0FAC`) | 127, 127 | 52, 11 |
| the take-off's figure and runs (`0F92`, `0F94`) | 127, 127 | 122, 97 |
| his look (`0F8E`) | 127 | 122 |
| his hover's runs (`7E7800`) | 127 | 48 |
| the bound he met (`7E783E`) | 127 (none met) | |
| every field at once | 127 | 0 |

The run 866-867 (part 4a's `Roars` hand-over) gives figure 1, its first run, speed 0, as
`TakesOff` says. The baby's place (`BabyHeld`) agrees on 126 of the 128 runs 867-994 at his
place as the record leaves it; the other two, 870 and 874, are the hops' runs, where it is at
his place before the hop: 128 of 128 with the hop rule. Tangible and moving (`0F86` bit 400h
clear, `7E7804` = 1) on all 127 records from `TakingOff` through `Hovering` (867-993). `vecrun.py`
runs the 48 L- vectors: 48 of 48 agree.

### State (the RAM mapping)

| Spec | RAM | Notes |
| --- | --- | --- |
| `TakingOff` (his take-off) | `0FA8` = `$A354` | zeroes `0FAA`/`0FAC` each run |
| `Rising`, `Slowing`, `Hovering` | `0FA8` = `$A6AF`, `$A6C8`, `$A6E8` | `$A6AF` falls through into `$A6C8` on the run it switches (`L-rise-slows`) |
| `Retreating`, `FakeRetreat` | `0FA8` = `$A971`, `$BD9A`, entered by `JMP` on the same run | 4c-2 and the fake retreat |
| `Fireballs`, `Lunge`, `Swoop` | `0FA8` = `$A782`, `$A83C`, `$A88D` (table `$A743`, random & Fh) | later parts; set, then the run goes on to his move |
| his position and fractions | `0F7A`.`0F7D`, `0F7E`.`0F81` | the low bytes `0F7C`/`0F80` stay 0 on every run (asserted by the reader) |
| his speed | `0FAA`, `0FAC`, 1/256 px | `$D86B` adds the low byte to the fraction byte and the sign-extended high byte, with the carry, to the whole px |
| the take-off's figure | `0F92` = `$E925`, `$E92F`, `$E939`, `$E93F`, `$E943` for figures 1 to 5 (after the step), runs = length − `0F94` + 1 | list `$E91D` (facing left, `7E7820` = 0); figure 5 is `0011,E983` then sleep: its 17 runs change nothing seen, so the spec holds it |
| `TakeOff1`, `TakeOff2` | `0F8E` `$EA4F`, `$EA71` | the facing-right list `$E945` is Norfair's |
| his hover's runs | `7E7800` | 0 from his placing; reset when an attack is chosen |
| the bound he met | `7E783E` 1, 2, 4, 8 (`Left`, `Right`, `Top`, `Bottom`) | y's store overwrites x's |
| the room's bounds | `7E8000` y min (`FFE0h`), `8002` y max (B0h), `8004` x min (28h), `8006` x max (E0h) | set at his placing; constant in 0006 |
| his point | (C0h, 64h) at `$A763`, inertia 10h (`$D712` with Y = 0) | |

### Inputs and order

- Main AI order (`$A288`): his function (`0FA8`), then, while moving (`7E7804`): hurt flash,
  `$D86B` (his move), wings, curl (`$A2BD`), tail (`$CAF5`), palette, ribs, drawing; then the
  baby (`$A2DC`). The framework's instruction step (`0F92`) runs after the main AI, so a hop
  (`$E51F`) and the lift-off (`$E969`, `0FAC` = `FEA0h`) land after his move and after the
  baby took his place.
- Her energy is `09C2` as his function reads it: the tail's hit lands later in his own run
  (`$DFD9`, through `$A0:A45E`), so the hit at 993 leaves 24 and the run at 994 gives up.
- His hits taken is `7E781A` (11 in 0006 from 857 on, counted by her shots against him), as it
  stands when he runs: her shots against enemies (`$A0:9785`) run before the main enemy routine
  (`$A0:8FD4`) in a frame, so a shot of the same frame is already counted.
  `$D4B5` changes his palette once it reaches 50 while `7E7802` is 1 (set at `$A6E0` as he
  hovers): a later part; 11 in this movie, so nothing.
- The random number is `05E5` as his run reads it, the one the curl and the baby read.
- `$A9:EF06` returns carry clear on a collision: |C0h − x| < 8 + `0F82` (8) + 1 and the same in y
  with `0F84` (8): within 16 px.

### Outputs

| Spec | RAM / code |
| --- | --- |
| the shake | `$D914`: max(|`0FAA`|, |`0FAC`|) ≥ 280h at the left bound → `183E` = 21h (BG2 and enemies, 3 px, horizontal), `1840` = Ch (Ceres; Norfair's 18h is not this domain's) |
| the bound he met | `7E783E` |
| his look | `0F8E` (drawing) |

### Sources per entry

| Entry | Sources |
| --- | --- |
| TakesOff | `$A6:A354`, `$E91D..E944`, `$E517`, `$E51F`, `$E969`, `$D467` |
| Rises | `$A6:A6AF..A6C7` |
| SlowsDown | `$A6:A6C8..A6E7` |
| Hovers | `$A6:A6E8..A762`, `$A763..A781`, `$A9:EF06` |
| PulledToHisPoint | `$A6:D62F..D711`, `$D712` |
| Flies | `$A6:D86B..D954` |

### Findings behind the spec's choices

- **The pull's carries.** `$D63E`/`$D6AA` leave the compare's carry in place for the first add
  or subtract; with both positions under 8000h it is exact, and the spec states it as the odd
  unit `c`, with the cases far apart (above the room's top). In the three-step branch the missing `CLC`/`SEC` carry the
  zero crossing of the first or second step into the next: the `e` term, seen once in 0006
  (963: −5 to 5 with pulls of 3).
- **The take-off's first figure** is set by part 4a's hand-over with its timer at 1 (`$D467`),
  so it shows on the run he takes off (867); the hops land on 870 and 874, the lift-off on 879.
- **The lift-off speed** is `FEA0h`, −352 (the bank log's comment says −60h).
- **Near his point** is judged before his move, on whole px; in 0006 he never comes near
  (he hovers 35 to 57 px above it), and his hover's runs reach 78 before he gives up.
- **Which of her positions an enemy's contact reads.** Game state 8 (`$82:8B44`) runs her
  current state handler (`$90:E695`: her radius `$EC22` from the pose she starts the frame in,
  then her pose input), then the main enemy routine (`$A0:8FD4`: Ridley's main AI, the tail's
  test `$A6:DFD9` reading `0AF6`/`0AFA`), then her new state handler (`$90:E725`: her movement
  `$E94B`, then the hit's reaction `$DDE9`). So the tail reads her position as the frame
  starts (the root's ruling for every hitter: her position, body and protection as the frame
  starts). Over 0006's five hits, her place at N−1 against the tip at N: 609 (216, 192),
  705 (220, 187), 801 (234, 159), 897 (223, 159), 993 (235, 77): each in reach, the side as
  recorded (left, right, right, left, left). Part 4b's vectors take these places (T-hit-right
  moved from 224 to 220, T-hit-jumping from y 161 to 159, T-protected from x 220 to 216); their
  outcomes are unchanged. Each hit lands on the first frame with her protection 0 at the
  frame's start, so no frame of the cached rows separates the two readings.
- **The pull's cap is a signed test of the new speed less 500h** (`CMP #$0500`/`BMI`, and
  `CMP #$FB00`/`BPL`), after the adds wrap at 16 bits: L-pull-wrap (32,767 + 1 wraps to
  −32,768, then capped at 1,280). With the cap stated exactly, every speed is an allowed state.
- **Rows past this part:** the retreat from 994 (4c-2: rises off the top to y −128, the hold,
  the walls); his tail in flight 867-993 (4c-1b).

### The harness (for the integrator)

- **Rows:** `chain-any-0006-E0B5` 867-993 for this part (994 for the phase `Retreating` only).
- **Compared per run:** `0FA8` through the phases above; `0F7A`, `0F7D`, `0F7E`, `0F81`;
  `0FAA`, `0FAC`; `0F92`/`0F94` as the take-off's figure and runs; `0F8E` through the looks
  (from 867, where part 4a's reader stops); `7E7800`; `7E783E`; the baby's place as part 3.
- **Skipped:** `0F7C`/`0F80` low bytes (always 0); the wings (`7E780A..7812`, drawing); the
  tail's block `7E2000..20AF` from 867 (4c-1b); `7E781A` (a later part's count: an input here);
  `7E7802` (follows from the phase).

**What the integrator needs:**
1. Production (`crates/game/src/production/played/ridley.rs`): run him from `TakingOff` through
   `Hovering`; refuse a run that starts `Retreating` or `FakeRetreat`, and one that starts in an
   attack, each as its part's fault; hand R3 the shake; report him tangible and moving, not
   roaring. Her energy comes in from samus-health as it stands when he runs. His hits taken
   has no producer yet (her shots against him are a later part's): production holds 0. Of the
   cached rows, two reach his flight: 0006 (11 from 857 on) and `samus-ceres-push` (6). Every
   indexed run stays under 50 there, so `Hovers` (100) and the later palette rule (50) give the
   same outcome with 0 as with the game's count.
2. The verify reader (`crates/verify/src/replay/ridley.rs`) decodes the new phases, the
   take-off's figure from `0F92`/`0F94`, the speeds, fractions, `7E7800` and `7E783E`.
3. The vector harness, `crates/clean/boss_ceres_ridley/tests/vectors.rs`, parses the L- form;
   until it does, its `deny_unknown_fields` rejects the file.

Played alone with this piece, the run would still lack his tail in flight: the first flight
hit lands at 897 on the moving tip, so without 4c-1b the run's first difference is her energy
and her push at 897. With 4c-1b (and her own domains holding 868-993) it stops at 994, the
retreat's first run (4c-2). The claim holds with one unproduced input, his hits taken (held at
0 where the game has 11; `7E781A` skipped), which changes no outcome while under 50.

### The harness cases

| Case | Row:frame | Vectors |
| --- | --- | --- |
| L0, L1, L2, L3 | `chain-any-0006-E0B5`:867, 868, 870, 874 | L-takeoff, L-figure-counts, L-hop-first, L-hop-second |
| L4, L5, L6, L7 | `chain-any-0006-E0B5`:879, 880, 896, 899 | L-lift-off, L-rise-counts, L-figure-rest, L-rise-slows |
| L8, L9, L10, L11 | `chain-any-0006-E0B5`:900, 915, 916, 963 | L-slow-counts, L-slow-hovers, L-hover-pull, L-pull-turns |
| L12, L13 | `chain-any-0006-E0B5`:993, 994 | L-pull-back, L-gives-up (and the edges built on them) |

## Part 5, the escape's steam

### How it was made

Behaviour first, from CI's lean record cache, rows `chain-any-0009-DFD7` (room 3) cuts 0-168
and `chain-any-0010-DF8D` (room 2) cuts 0-111, read per frame by scratch readers
(`$TEMP/collab/steam-ref/`, local only: `dump.py`, `rec.py`, `near.py`, `cases.py`): each
steam slot (species `E1FF` in `0F78` + 40h × i) with its `0F7A`/`0F7E`, `0F86`, `0F8E`,
`0F92`, `0F94`, `0FAE` and `0FB4`; the camera `0911`/`0915`; her `0AF6`/`0AFA`, `0AFE`/`0B00`,
`0A1C`, `09C2`, `18A8`, `18AA` and `0A54`. The bank logs then pinned the arithmetic: A6
`$EFB1..F4EB` (the steam), A0 `$8EB6` (the window), `$8FD4..9057` (the order of contact and the
run), `$9A5A..9B7E` (contact with a box per figure), `$A477`/`$A4A1..A596` (the touch), `$804F`
and `$8059` (the never-run figure and its box), and 82 `$8B44..8BAC` (the frame's order).

The reference model, `$TEMP/collab/steam-ref/model.py`, run per frame by `rec.py` from the
records' state the frame before (the camera, her position, body and protection as that record
leaves them):

| Row (play cuts) | Vent-runs compared | Model | Do-nothing (last frame's state) | Hits: model / record |
| --- | --- | --- | --- | --- |
| `chain-any-0009-DFD7` (0-168) | 840 | 840 | 341 | 137 Left / 137 Left |
| `chain-any-0010-DF8D` (0-111) | 1,221 | 1,221 | 473 | 66 Right / 66 Right |

A vent-run is one slot's phase and count on one frame. No other frame has a hit, in the model
or the records: 276 vent-runs in view have a reach out, with her 21 px away at the nearest
unprotected (0009:82), and touching but protected at 0010:9-12 and 67-74. Both hits keep her
energy (24 before and after) and leave `18A8` 95 and `18AA` 4 at the record (her steps that
frame spent one of each). `vecrun.py` runs the 50 ST- vectors: 50 of 50.

### State (the RAM mapping)

| Spec | RAM | Notes |
| --- | --- | --- |
| its facing | `0FB4` (parameter 1): 0 `Up`, 1 `Left`, 2 `Down`, 3 `Right`; 4 and 5 are `Left` and `Right` drawn where the turning shaft puts them (`$F019`, part 7) | rooms 3 and 2 (0009 0-168, 0010 0-111) hold 0 to 3 only; the 4s and 5s show from 0010:225, in the next room (part 7); the graphical offset (`7E7010`/`7012`) is drawing only |
| its position | `0F7A`, `0F7E` | never changed by the steam |
| `Placed(w)` | `0F8E` = `804F` (extended spritemap "nothing"), `0F92` the list's start, `0FAE` = `w`, `0F86` tangible and visible | from `$EFB1` |
| `Waiting(w)` | `0F92` = list + 6 (at `$F127`), `0F94` = 1, `0FAE` = `w`, hidden (`0F86` | 0500h) | |
| `Puffing(k, n)` | `0F8E` = figure `k`'s extended spritemap (`$F142` + 0Ah × (k − 1) for up, `$F188` left, `$F1CE` down, `$F214` right), `0F94` = `n`, `0F86` tangible and visible | |
| `Resting(n)` | `0F92` = list + 12h (at the show, `$F135`), `0F94` = `n`, hidden; `0F8E` figure 1's | the 0040h at `$F05B` and the others is the rest, `steam_rest` |

Each figure's instruction pair is `0003,<figure>`, and the puff's end (`$80ED`, go to) leads to
the hide (`$F11D`) and the rest (`0040`). The show (`$F135`) follows the wait's end too: `$F127`
falls through from `$F131` into `$F135`.

### Inputs and order

- The frame (82 `$8B44..8BAC`): which enemies run (`$A0:8EB6`, the window, on the camera as
  the last frame left it: the scrolling, `$90:94EC`, comes later); her current state handler
  (`$90:E695`: her radius from her pose, `$EC22`, and her pose input; no movement); the main
  enemy routine (`$A0:8FD4`); then her new state handler (`$90:E725`), where her movement
  (`$E94B`) moves her. So the vents meet her **before she moves this frame**, at her position
  as the last frame left it, with the body of the pose she started it in. At 0009:137 the
  record before shows x 101 (her box 96-105, figure 1's 104-119: touching) and the hit's
  record 98 (93-102: not touching); reading her after her moves would miss the movie's first
  touch. At 0010:66 both touch. This is the root's ruling for every hitter (spec-gate finding 2):
  Ridley's tail runs inside his main AI in the same enemy routine, so it reads her as the frame
  starts too (§14.3).
- The window (`$8EB6`, not processed off-screen, `0F86` bit 800h clear): `x + 0F82 − 0911`,
  `0911 + 100h + 0F82 − x`, `y + 8 − 0915` and `0915 + F8h − y`, each `BMI` to skip; `0F82` is
  8 from the species header (`$A0:E1FF`). The rows meet the left edge at 0 (0010:22) and −1
  (0010:32), the right at 0 (0010:28) and −1 (0010:52), the bottom at 0 (0009:104).
- Contact before the run: `$9021` skips an intangible slot entirely; `$903E` runs contact
  (`$9758` → `$9A5A`, extended spritemaps) before the main AI (`$908F`), whose instruction
  processing changes the figure. So contact reads the phase the frame started with.
- Her protection: `$9A95` returns while `18A8` is not 0, read before her own step counts it
  down (as §14's tail). Contact damage index `0A6E` is normal in Ceres (no speed booster or
  screw attack); the other branch is out of scope.
- The first touching vent wins: the touch sets `18A8` = 60h, so the next slot's `$9A95` returns.
- The random number: `$EFD8` draws one (`$80:8111`) per vent as the room places it, so
  `0FAE` = 1 + (new number & 1Fh). Where these draws fall among the room's other placing draws
  is R2's. The individual room seeds are after loading. Replay from room 4 into room 3
  compares the five new vent states and the resulting random number at the settled cut 219.

### Outputs

| Spec | RAM / code |
| --- | --- |
| the frame's hit | `$A562..A596`: damage from the species header (`$A0:E1FF` + 6: 0) through `$A0:A45E` and `$91:DF51` (0 taken), `18A8` = 60h, `18AA` = 5, `0A54` = 0 (left) or 1 (right) from `0AF6 − 0F7A` (`BMI`) |
| shown, its figure | `0F86` bits 0100h/0400h (`$F11D`, `$F135`); `0F8E` |

### Sources per entry

| Entry | Sources |
| --- | --- |
| VentPlaced | `$A6:EFB1..F00C`; `$80:8111` |
| VentInView | `$A0:8EB6..8F54`; `$A0:E1FF` (x radius) |
| VentWaits | `$A6:F04D..F059` (and the three other lists), `$F11D`, `$F127..F141` |
| VentPuffs | `$A6:F061..F07F` (and the others), `$80ED`; `$A0:8BA9` (the instruction runner) |
| VentRests | `$A6:F059..F05F` (and the others), `$F135` |
| PuffReach | `$A6:F142..F259`, `$F25A..F373`; `$A0:804F`, `$8059` |
| PuffTouches | `$A0:9A5A..9B7E`, `$A6:F03F`, `$A0:A477..A596`, `$A0:A45E`, `$91:DF51`; `$A0:9021..903E` |

### Findings behind the spec's choices

- **The reach is a box per figure, edges included on the far side.** `$9B0A..9B3B` tests
  `box left − her right` `BPL` (skip when ≥ 0), `box right − her left` `BMI` (skip when < 0),
  the same down: 16-bit differences, signed by bit 15. Figures 6 and 7 point at the empty
  hitbox list `$F25A`; a hidden slot is skipped before contact (`$9021`).
- **The never-run vent reaches one pixel.** Its spritemap is `$804F`, whose one entry's hitbox
  `$8059` is 0,0,0,0 with the ordinary touch `$8023`: the same hit. A vent placed in view has its
  first run on the same frame as its first contact, so this is reachable only on the frame a
  vent first comes into view; no row has her there.
- **The side is the vent's position**, not the puff's: `$A587` subtracts `0F7A`.
- **The touch keeps the steam alive**: `$F03F` sets its health to 7FFFh before the touch, so the
  touch's death check never fires; nothing of the vent changes on a touch.
- **No energy, all the rest**: the species' damage is 0 (`$A0:E1FF` + 6, the species header's
  damage word, read by `$A0:A45E`), so `$91:DF51` takes none; this is the source of
  `PuffTouches`' worth, 0 whole energy, which production hands to samus-health's `Impact` (the
  shared hit itself holds only the side and the push time). `18A8`, `18AA` and `0A54` are set as
  by any touch: the protection, the push and the flash all follow (samus-health,
  samus-movement, samus-appearance).
- **The shaft's vents** (parameters 4 and 5) run the same cycle; `$F019` computes where they
  are drawn in the turning room each run. Their reach is at their unturned position: part 7.
- **Shots**: the hitboxes' shot pointer is `$804C` (`RTL`); C1's.

### The harness (for the integrator)

- **Rows:** `chain-any-0009-DFD7` 0-168 and `chain-any-0010-DF8D` 0-111 (the passages after
  them don't run enemies: no vent changes).
- **Compared per frame and slot:** the phase and its count decoded as in the State table;
  shown from `0F86` 0400h clear; the frame's hit as `18A8` going from 0 to 5Fh with `0A54` as
  its side and `18AA` + 1 as its push time; her energy (`09C2`) unchanged by it.
- **Skipped:** the graphical offset `7E7010`/`7012` (part 7); `0F86`'s other bits; `0FAE` once
  the wait is over (left at 0).

**Production and replay.** Room loading places the vents in slot order, advancing the random
sequence for each. A room seed restores its current vent phases and counters. The station's
enemy step reads the frame-start camera and Samus contact state, then sends the first vent hit
to movement and health. The shared hit carries its side and push, and the health impact carries
zero whole energy.

The played comparison checks complete vent state at finished cuts and the hit's side and push
during play. It separately checks the resulting energy and protection. Room 3 plays through its
cut 137 hit and starts the passage, stopping at 200 when drawing updates jump history. Room 2
plays through its cut 66 hit, stopping at 87 when Samus releases jump as her backward leap lands.

### The harness cases

| Case | Row:frame (vent slot) | Vectors |
| --- | --- | --- |
| ST1, ST2, ST3 | `chain-any-0009-DFD7`:46 (2), 43 (3), 51 (2) | ST-first-run, ST-wait, ST-wait-ends |
| ST4, ST5, ST6 | `chain-any-0009-DFD7`:2 (5), 1 (5), 7 (6) | ST-puff-counts, ST-puff-next, ST-puff-last |
| ST7, ST8 | `chain-any-0009-DFD7`:8 (6), 81 (4) | ST-rest-first, ST-rest-ends |
| ST9 | `chain-any-0009-DFD7`:137 (2) | ST-hit-left |
| ST10 | `chain-any-0010-DF8D`:66 (0) | ST-hit-right |
| ST11, ST12 | `chain-any-0010-DF8D`:9 (4), 67 (0) | ST-protected-room-2, ST-protected-after-hit |
| ST13 | `chain-any-0009-DFD7`:82 (4) | ST-near-miss |
| ST14, ST15, ST16, ST17 | `chain-any-0010-DF8D`:32 (0), 22 (6), 28 (4), 52 (3) | ST-out-left, ST-view-left-edge, ST-view-right-edge, ST-out-right |
| ST18 | `chain-any-0009-DFD7`:104 (4) | ST-view-bottom-edge |

Each case's frame is the record the run ends at; the vector's `vents` and `her` are the record
before it, moved 1,000 px right and 600 px down with the camera.
## Part 4c-2, Ridley leaves the room

### Cuts and source mapping

Both `chain-any-0006-E0B5` and `samus-ceres-push` show him still hovering at cut 993.
The tail hit has taken her from 39 energy to 24, pushed her left, and left 95 frames of
protection at the cut. At 994 he already has a new speed and both fractions change.
His flight's tangible and moving outputs still hold. The first unsupported cut after flight
and its tail is 994 (`Fault::RidleyGivesUp`). Carry the body and tail produced by 993 into
the low-energy retreat. The other handoff at 100 hits, the fake retreat, remains separate.

| Entry | Sources |
| --- | --- |
| FliesOutOfReach | `$A6:A6E8..A708`, `$A6:A971..A9A4`, `$A6:D62F..D713`, `$A6:D86B..D954`, `$A6:A294..A2BA`, `$A6:CAF5..CB20` |
| GetawayBackdrop | `$A6:A9A0..AA10`, `$A6:AA20..AA4E`, `$A6:CAF5..CB32`, `$A6:CEBA..CF59`, `$A6:CF5A..D09E`, `$A6:D2AA..D2D5`, `$A6:DFD9..E01A`, `$A0:9275`, `$A9:D2E4..D2F5` |

`A988` uses acceleration factor 1, which selects divisor 15. The compare at `A98E`
tests the incoming whole y, after the speed adjustment but before the position advances.
The switch to `A9A0` falls through that function on the same run. Its countdown therefore
first reads 63 in a cut, then reaches 0 a full run before the walls. In the airborne
main AI, `093F` is still clear when the walls are placed. The tail function's switch to
0 skips the neutral swing only. The common swish, linking, stretch and hit code still runs.

| Physical quantity | Source-side check |
| --- | --- |
| body, bounds and phase | `0FA8` goes from `A6E8` to `A971`, `A9A0`, then `AA11`. §15's positions, fractions, speeds, look, hover count and `7E783E` remain comparable. The top limit at `7E8000` becomes −192. Compare `0FB2` while coasting. |
| held baby, tail, eligibility | `7E8804/8808`; all §17 segment, mark, pace, swish and curl observations; `7E2000` switches from 1 to 0 on the wall placement. `0F86`'s intangible bit stays clear and `7E7804` remains set. |
| walls | Cut 1157 places species `E23F` as slots 2/3 with parameter 5/6 and content positions (8,127)/(248,127). Their own frame counters still read 0. Part 3's N12 covers `Placed`, N13 their first run in the getaway. |
| active colours | The 15 words at A6:`A9E3` go to current BG palette 5, entries 1–15. The eight words at A6:`AA01` go to current BG palette 2 and sprite palette 7, entries 1–8. Compare the three spans at `7EC0A2`, `7EC042`, `7EC1E2` and their CGRAM counterparts, offsets `A2`, `42`, `1E2`. The middle span already has the selected colours before 1157. Placing a wall separately writes the doors' **target** colours (§11). |

### Reference and integration

`$TEMP/collab/ridley-retreat-evidence/reference.py` seeds the boss and full tail once at cut
993. It advances them to 1157 from predicted state, takes her body and protection from the
frame start and advances game-random's shared sequence once per run from its initial
number. His hit count and her energy come from their respective owners. His phase, speeds,
fractions, bound, held baby, segments and the wall placement come from the prediction.
The two rows are distinct player paths through the same boss sequence. No later record
supplies the boss a phase, a wait, a bound, a place, a tail tip or a reason to stop.

| Both rows separately, cuts 994–1157 | Reference | Do nothing |
| --- | --- | --- |
| complete body state | 164/164 | 0/164 |
| complete tail state (tip alone) | 164/164 (164/164) | 1/164 (9/164) |
| held baby's position | 164/164 | 50/164 |
| hit, and placement of the pair | 164/164 each | 164/164 no hit, 163/164 no placement |

The reference checks both walls' end-of-frame slots and all three current and physical CGRAM
spans against the imported assets on 1157. It compares the body's bound on every cut,
including the upper bound at 1115. Holding the boss on the handoff fails first on phase at
994. Substituting the hover divisor fails first on the position fraction at 1004. Each
fault was applied in the reference, on both rows. Production fault planting remains part
of the integration. No recorded retreat hit occurs. The constructed contact cases make
the shared hit positive and test its side, push and worth.

In RT-near-her the last base is at x 227, her x is 228 and the chance-to-whip draw
would accept mark 16,385 while hovering. The retreat keeps pace 2 and no mark. In
RT-walls-and-tip the base is held on its existing mark, the next segment grows, the third
shrinks, and the tip stays within its 5 px cap. Retaining the chain's offsets still puts
the tip in reach of her body while a swish falls on that same last run.

The RT vectors carry RAM quantities and symbolic outputs. The recorded cases are the
transition from the previous cut to the named cut in either row's boss sequence. Their
player inputs come from `chain-any-0006-E0B5`.

| Cut | Vector | Witness |
| --- | --- | --- |
| 994 | RT-first-pull | the handoff already moves both axes |
| 1005 | RT-pull-divisor | differing pull with the hover divisor, also from a restored body |
| 1092 | RT-crosses | the move takes him above the target height |
| 1093 | RT-coasts | the last pull and first counted coast |
| 1094 | RT-momentum | both speeds carry over |
| 1115 | RT-ceiling | y stops at −192, x still moves |
| 1156 | RT-last-coast | horizontal motion on the run with one coast left |
| 1157 | RT-walls | final fractions and tail tip held, new wall actors present |

The integration can decode these quantities at a checkpoint. In continuous play the
camera, progress, health, time and random owners supply their normal inputs. Compare both
players' resulting health, protection and movement alongside the boss. Use the drawing
and room adapters' wall content and imported palette colours for the terminal placements
and current-colour outputs. The baby has its own animation and colours, and the wings,
ribs and poses have their usual drawing owners. Compare this part's named spans, without
masking one palette behind the other. The next original cut after the proposed scope is
1158. It needs the getaway picture, its sound request and the station stage advance.

### From the getaway to the running timer

`$TEMP/collab/ridley-retreat-evidence/timeline.py` checks these joins against each row.
For the any% row, cut 0 is the seed at movie frame 10,162 and cut 1 is its first
successor at 10,163. Add the local cut to the seed frame for the movie labels below.
This table identifies the next owners' outputs. It supplies no elapsed-time trigger to the retreat model.

| Cut (movie frame for any%) | Source and effect |
| --- | --- |
| 1158 (11,320) | A6:`AA11..AA1F` sets `093F` to Getaway and enters the plane setup. The wall actors run in Getaway for the first time. Room work runs A6:`AABD` on the same frame. Its first picture requests the getaway sound in effects channel 2, cap six. The channel's write cursor advances and its entry records the sound. |
| 1262 (11,424) | A6:`AAD4..AAE8` requests the directional push and starts the diagonal 3 px Back shake for 64 frames. R3 leaves 63 at the cut. Her health stays 24 and her protection stays zero. Movement's push setup (`90:E119`) begins at 1262, her knockback pose at 1263. The left push in the movie releases at 1264. The other row pushes her right until 1269. |
| 1270 (11,432) | The getaway path ends (`A6:AB2E..AB5E`), restores the room presentation and selects self-destruct. The station stays Getaway. The text colours load at 1271. The sprite tile batches finish at 1277 and the background batches at 1284. |
| 1284 (11,446) | Finishing the background tiles draws EMERGENCY, arms its 128-run hold and queues the escape track (`A6:C09F..C0BA`). The music queue's write cursor `0639` advances, its new track is 7 with delay 8. `07F5`, the current track, changes on **1293** (11,455). These are two different audio observations. |
| 1412 (11,574) to 1585 (11,747) | The hold ends, arming the English typewriter (`A6:C0BB..C0F4`). Its completion selects the escape start on 1585 (`C104..C116`), while the station and countdown still read Getaway and dormant. |
| 1586 (11,748) | A6:`C117..C135` emits the Ceres countdown start, changes `093F` to Escape and sets the area boss bit in `7ED82E`. H2 consumes the start on this same maintained frame. Its end-of-frame status `0943` is `8003` with 1:00.00 in `0945..0947`. The door clears its lock and starts its shake. Both walls hide in the new stage. |
| 1602–1611 (11,764–11,773) | H2 finishes the initial hold at 1602. The first changed digits are 59.98 on 1603. The movie triggers the door on 1610, still ticking to 59.87. Its passage holds that time from 1611. The other player stays in the room and continues ticking. |

The last health hit in both rows is 993. Neither the offscreen flight nor the getaway
push starts another protection interval. Use the boss's placement event to create walls,
the getaway's push to drive movement, the self-destruct completion to drive progress,
and its one-minute start as the input to H2. Reading the recorded wall slots, stage,
boss bit or timer status as new inputs each frame would hide a missing producer.

## Part 4c-3, Ridley's flyby and its push

Both `chain-any-0006-E0B5` and `samus-ceres-push` start here at the cut after
the two walls have been placed. Cut 1157 has the getaway colours but still has
a calm station. The first change at 1158 is Ridley's advance to Getaway. His
ordinary body, tail and carried-baby animation stop running. The door and
both walls see the new stage before the room plays the first flyby picture.
The effective scroll is already (−127, 26), around (64, 64), at zoom 2,048 and
turn 49. The flyby sound is already in effects queue 2. Neither the setup
scroll nor an empty staging frame survives to the cut.

| Entry | Sources |
| --- | --- |
| GetawayPass | `$A6:A288..A2BC`, `$A6:AA11..AA1F`, `$A6:AA54..AAAE`, `$8F:E571..E57B`, `$A6:AAAF..AB5E`, `$A6:AE4D..B0EE`, `$90:E119..E12D` |
| GetawayPicture | `$A6:AA54..AAAE`, `$A6:AAE9..AB5E`, `$A6:AB5F..ACBB`, `$A0:B443`, `$A6:ACBC..AE4C`, `$A6:B0EF..B226`, `$80:8B4F`, `$A9:D2E4` |

| Quantity | How to decode it and what to compare |
| --- | --- |
| Boss, stage, appearance | `0FA8` goes from `AA11` to `AA4F` to `C04E`. `093F` goes from 0 to 1 at 1158 and stays there after the flyby. Compare the actual walls' lists at `1012/1052`, figures at `100E/104E` and visibility at `1006/1046`; the room door's lock is `0FEA`. The escape bit is `7ED82E` bit 0 and the countdown status is `0943`. |
| Restorable picture state | During the flyby, `7E8026 / 2` counts pictures shown. `7E8028/802A` hold signed scroll. `7E7814` holds the full wrapping turn. `7E802C/802E` hold baby and wing beats. The first picture starts with both beats at 0 in these rows. Decode only these two beats while Ready. The incoming path cursor, turn and position are inactive then. Poisoning them at 1157 preserves the first picture. The inactive room does not read the spent path cursor or its position. |
| Content for the data importer | Pair the scale at `A6:AE4D` with signed changes in scroll. Use the negative of the horizontal value at `A6:B00F` and the vertical value at `A6:AF2F`. The scale terminator ends the path after 112 pictures. The pass is fixed at the 105th room run, including the first picture. Its test precedes the path end, so it also occurs on the return for an invented 104-picture path. It is independent of the value of its scale and of Samus's place. |
| Effective picture | Compare WRAM shadows `56` (7 flyby, 9 room), `5F` (outside transparent during the flyby, zero at the return), `78/7A/7C/7E` (matrix), `80/82` (pivot), `B1/B3` (scroll) and `7E8020` (zoom while flying). The room's `0783` remains set on the return. Its ordinary camera writes the following frame. The renderer reads the actual selected plane. Architectural BGMODE at the NMI cuts is 9 even on the flyby. The active physical matrix and pivot match the requested picture. On the return at 1270 the inactive physical matrix still reads (27,15,−15,27) and the pivot (64,64), though their effective shadows are zero. The overflow selection and ten-bit scroll clear. The physical display-memory checks cover the art; the composed picture is outside this proof. |
| Rotation | The same sine content at `A0:B443` as the cinematic plane. Here `A6:AB5F..AC0D` multiplies magnitudes and restores each sign after division. In a restored negative quadrant a nonintegral product truncates toward zero. The cinematic plane's signed floor would give (−205,−262) in place of (−204,−261) for G-negative-turn. Quantizing only after multiplication gives (−204,−262). Both agree throughout the recorded arc. |
| Body, baby and wings | Compare each active body set against both current colours `7EC0A2..C0BF` and CGRAM `A2..BF`. Its source is `A6:B107 + 32 × (zoom / 256)`. `A6:ACDA` maps the four baby beats to three art sets. `A6:AD45` maps the two wing beats. NMI puts these changes into the low bytes of the plane's tilemap in VRAM. In word coordinates the two baby spans start at `0504/0584`; the six wing spans at `000B/0080/0100/0181/0201/0280`. Compare their full extents through the room return. |
| Sound | On picture one the queue with byte cursors `0647/064A` accepts effect `4E` with cap six. Check the write cursor and the newly written entry in `0666`. The later escape track is a distinct music request. |
| Push and quake | The room calls `90:E119` on pass. At this cut `0A58/0A5A` show the push armed, before its pose is selected. The earthquake request is type `23` for 64 frames at `183E/1840`. The ordinary quake update spends one immediately. The pass ignores protection, health and existing shake. Its push has no side until movement compares Samus with the view centre. |

### Observable order and row differences

* **1158.** The first enemies' run sets the stage, both walls run, Samus makes her
  ordinary move, then room work plays the first picture and offers its sound.
  The animation clock uses `05B6` at the incoming cut, so both looks change.
* **1262.** The two scenes still agree on every flyby and quake output. Samus
  has already moved this frame. Her position is (30,139) on the movie and
  (216,187) on the other row. Each has 24 energy and zero protection.
  The quake has 63 frames left.
* **1263.** Movement at `90:E12E..E1C7` fits the recoil pose, sets downward
  speed to 5 px per move, and chooses left when her x relative to the front
  layer is below 128 px, right at or above 128. With `0911` zero, she takes
  the left push at (30,137) on the movie, the right push at (216,187) on the
  other row. Facing determines her pose independently of that push direction.
* **1264 through 1269.** `90:E1C8..E23A` sweeps horizontally first. Solid
  horizontal contact releases the push and clears its speed before a vertical
  move. Otherwise the downward move also runs, and contact with the floor
  alone leaves the push active. The movie releases at (29,137) on 1264.
  On the other row she stays at y 187 and reaches x 218, 221, 225, 230 and
  235 on 1264 through 1268. The next move releases her at (235,187) on 1269. The movie meets the still-locked room door at x 16, half-width 8. The other row meets the solid room blocks at x 240. Neither of the getaway walls obstructs her.
  A fixed duration for her push cannot reproduce both rows.
* **1269 and 1270.** Picture 112 is still on screen at 1269. Its scroll is
  (−268,1019), its zoom 32 and its turn 5,377. On the following run the
  depleted path selects the ordinary room view with zero scroll, pivot and
  matrix. Glowing text, escape music, the boss bit and the countdown all have
  later producers. The movie's ordinary motion has resumed before that return.

### Reference and acceptance

The observer reference in `$TEMP/collab/ridley-getaway-motion-evidence/flyby.py`
starts with cut 1157's owned state and incoming clock. It consumes the path
and art from the player's ROM, carries the complete flyby forward and
compares every requested NMI cut from 1158 through 1270 for both rows.
Her move, quake, door, transport and H2 are consumer checks. None supplies the
flyby with its phase, path position, first sound, pass cue, next stage or end.

| Each row, separately | Reference | Previous cut or no request |
| --- | --- | --- |
| Complete flyby state and effective geometry | 113/113 each | 0/113 each |
| Live and physical body colours while the picture runs | 112/112 | 103/112, physical colours |
| Physical baby and wing tiles | 113/113 | 85/113 |
| Getaway stage, its sound and the pass | 113/113 each | 112/113 each |

Delaying the pass by one picture fails at 1262 on the armed push in each row.
Freezing the turn fails at 1158 on the full turn. A wrong quake fails on its reach, direction, strength or starting duration at 1262 and on G-pass. Production can exercise the
same faults through its played runner. For resume, restore its actual phase,
completed pictures, scroll, full turn and both beats plus the owning clock.
Use the same checked room content after resume. Do not re-read the expected
record, the displayed coefficients or the actors' responses as inputs.
G-room uses the observed state at 1269 on either row and the result at
1270. The remaining G cases use invented path samples and restored states.
The clean flyby scope ends on the return to the room. Its first unmet successor
is the warning's selection of text colours on 1271.

## Part 4c-1b, his tail in flight

### How it was made

The lean tier rows `chain-any-0006-E0B5` and `samus-ceres-push` show a moving chain
from cut 867, a whip from 915 and contact at 897 and 993. Both rows use the same 127-run
window, cuts 867 through 993 inclusive. Cut 866 seeds the transition from rest. The
bank log sources below resolve what the records alone leave open, including short lags,
turning at a mark, long deltas in the bearing calculation and the sound queue.

The reference check in `$TEMP/collab/ridley-4c1b/scratch/reference_proof.py` advances
`tail.py` continuously from take-off. It uses the model's previous tail state and
computes the curl from the run's random number. It reads his position after his move
and before the take-off hop, recovered from the baby's place, and reads her position,
body and protection from the previous cut. It reads the run's phase eligibility and
tangibility from the current cut. Its do-nothing comparison retains the previous cut's
tail state and predicts no hit.

| Row | State and tip | Hit and side | Do-nothing state | Do-nothing tip | Do-nothing hit |
| --- | --- | --- | --- | --- | --- |
| `chain-any-0006-E0B5` | 127/127 | 127/127 | 0/127 | 19/127 | 125/127 |
| `samus-ceres-push` | 127/127 | 127/127 | 0/127 | 19/127 | 125/127 |

On each row the hit at 897 and the hit at 993 push left and each costs 15 energy.
`genw.py` checks the 21 recorded cases against their cuts and checks the constructed
boundaries with explicit expected values. `vecrun.py` reads the repository TOML and
recomputes all 49 FT cases, 21 recorded and 28 constructed.

### State (the RAM mapping)

All addresses here refer to WRAM. Segment words start at `7E:2020 + 14h * i` for
`i = 0..6`, base first. The two-byte coordinates in each pair are x then y.

| Spec | Address | Interpretation |
| --- | --- | --- |
| curl | `7E:201E` | §14's count |
| pace | `7E:2014` | angle units per run |
| forward mark | `7E:201A` | negative word means absent |
| swish count | `7E:781C` | runs at fast pace, modulo the swish period |
| swinging | segment + `00h` | sign bit, set or clear |
| lag | segment + `02h` | count, with `FFFFh` meaning passed |
| way | segment + `04h` | sign bit set for ahead, clear for back |
| length | segment + `06h` | 1/256 px |
| stretching | segment + `08h` | zero or the stretch threshold |
| angle | segment + `0Ah` | count of 256ths of a turn |
| place | segment + `0Ch`, `0Eh` | whole unsigned px |
| offset | segment + `10h`, `12h` | whole signed px |

A present forward mark must be one of the angles made by the Ceres aim. The sign
test at `$D128..D12B` treats high-bit values as absent. Aiming can prefer a missile
at `$D1AF/$D242`. Ordinary Ceres equipment has zero missile capacity, as described
in the [intro decode](screens-intro.decode.md), so this branch is outside the scope.

The backward mark at `7E:201C` stays absent on both rows. Ceres take-off and recovery
clear both marks. The Ceres branches of the aim source write only the forward mark.
A backward mark's effect on the opposite swing is outside this round's state.
The base's stretching state can be set, like the other segments', and kept. Its length
is never updated by the stretch rule.

### Inputs and order

- `$A6:A288` runs `$D86B` for his movement, then curl `$A2BD`, then his tail
  `$CAF5`. His take-off instruction hop follows the tail and the baby. `7E:8804`
  plus 16 and `7E:8808` less 22 reconstruct his position at the tail's run, including
  the two hops.
- The tail starts with the swish counter from the pace at entry. The whip aims
  with the base coordinates `7E:202C/202E` left by the preceding run. The Ceres flag
  that enables whip aiming is `7E:7802`, set by his first hover and kept through his
  attacks. At 915 the phase changes before the tail runs, so he can aim on that run.
- Her position, body and protection are read as in parts 4b and 4c-1a. His random
  number is the same `05E5` the baby and curl use. `$DFD9` tests contact after the
  chain's coordinates are made. Her protection `18A8` changes from zero to 95 at
  each hit, and `0A54` records the side. The tail's worth is `7E:7838`.
- The air-tail run begins at 867. On the run at 994 he is still moving and tangible,
  but `$A6:A971` has already made the first retreat move. That run is part 4c-2,
  including its own tail inputs. The attack selection can leave the attack as his
  phase before the tail sees it. Its first run still uses this tail, with later
  runs of the attack left to that part.

### Sources per entry

Bank A6 uses the [Ridley bank log](https://patrickjohnston.org/bank/A6.html).
The bearing and offset math use the
[enemy math bank log](https://patrickjohnston.org/bank/A0.html) and the
[projection bank log](https://patrickjohnston.org/bank/A9.html).

| Entry | Sources |
| --- | --- |
| TailComesLoose | `$A6:A4C5`, `$D2D6..D452` |
| TailFollows | `$A6:CC39..CC7C`, `$D09F..D19C` |
| TailSwings | `$A6:D09F..D19C`, `$D19D` |
| TailWhips | `$A6:CC39`, `$CC7D..CEB9`, `$D19D`, `$A0:C0AE` |
| TailRecovers | `$A6:CC39..CC7C`, `$D09F` |
| TailStretches | `$A6:CF5A..D09E`, `$D09F..D19C` |
| TailBends | `$A6:CEBA..CF59`, `$D09F..D19C`, `$A9:C46C` |
| TailSwishes | `$A6:CAF5`, `$D2AA..D2D5`, `$80:914D` |
| TailCurls, TailHits in flight | `$A6:A2BD..A2D2`, `$CAF5..CB20`, `$DFD9` |

### Findings behind the spec's choices

- The bearing divides the low bytes of the magnitudes after choosing the larger
  whole magnitude as divisor. It therefore retains the angle approximation's far
  deltas, including a divisor with low byte zero. The projection's negative
  products truncate downward after multiplying by the whole length.
- The forward test uses the signed subtraction after an extra unit, so meeting
  the limit exactly can turn or settle a segment. Both waiting and settling keep
  the last offset for this run, even though the chain can move with his body.
- The closing directions differ by segment, and a length can finish below its
  resting threshold after a shrink. On the run a segment stops stretching at the
  12 px threshold it still gains its stretch increment, subject to its cap.
- The swish source requests sound `21h` on queue 3 with a limit of 6. His roaring
  flag can suppress it in the original and is clear in the flight window. The
  lean comparison covers the swish counter, which resets at cut 931. Library 3's
  write index advances over the swish cue at that cut in both routes. The played
  comparison checks when the tail returns its swish request. The source identifies
  the channel and queue limit. The FT-swish vector checks the request itself.

### The harness (for the integrator)

- Compare the fields in the state table in the order of the chain for cuts 867
  through 993. Admit flight checkpoints by decoding each field. Compare the
  drawing positions here as outputs, independent of the drawing implementation.
  The prior `tail_segments` reader covers the same positions at rest. Compare the
  swish request against the newly appended cue in the library's ring, using the
  write indices at the two cuts.
- At cut 902 the spent push keeps Samus recoiling, and its recorded vertical
  direction and zero speed leave footing implicit. The previous cut holds a rise
  and a spent push counter. The final push has no solid contact (`7E:0DD0`), so the
  expected Movement phase is Falling. A grounded push retains GroundFalling.
  In the alternate row the grounded push first chooses the leap pose at 612
  and 706. At 612 the preceding Down push was already grounded. At 706 the
  last Down move hits the floor, sets the solid-contact flag and clears her
  horizontal speed before the leap is chosen. Either way, the unset direction
  on the first cut is Ground, and the next cut's first leap move begins Moonfall.
- Compare the hit's same-frame energy and movement reaction in the played row.
  Use the shared hit and the hitter's worth 15. Keep his hit eligibility and
  pre-move body read identical to the at-rest test. The model's vector adapter
  uses the new FT form. At-rest vectors and L scenarios without a tail assertion
  keep their existing assertions.
- Skip the routine selector `7E:2000` and constants `7E:2002/2004/2012/2016/2018`
  and `7E:7820`; these are the Ceres mode, activation, facing and parameters
  supplied by placement and flight scope. Skip the unused backward mark after
  validating it is absent. Mask the swinging and way words to their sign
  bits, written by `$A6:D2D6`, the take-off routine `$A4C5` and the swing/recovery
  updates `$CC39..CC7C`, `$D09F..D19C`. Decode the forward mark's sign as absent
  (writers `$D2D6`, `$CC39..CC7C`, `$CC7D..CEB9`). The phase owns all other
  eligibility. Retain part 4c-1a's drawing and unsupported-input skips.
- Stop the played row at 994 with the retreat fault. Checking only the resulting
  phase is sufficient for the scoped flight hand-over. Comparing a partial
  retreat as a full frame would report a difference in his position. All of
  the effects from that first retreat run belong to 4c-2.

### Recorded cases

All 21 cases cite `chain-any-0006-E0B5`. Equivalent low-byte draws are used for the
FT-hit-rising and the paired FT-reach-reference and FT-reach-adjusted examples. The
substitution preserves the curl and the rising tail's branch.

| Case | Cut | Vector |
| --- | --- | --- |
| W1 | 867 | FT-takeoff |
| W2 | 868 | FT-swing-ahead |
| W3 | 870 | FT-hop |
| W4 | 882 | FT-turn-ahead |
| W5 | 885 | FT-chance-rising |
| W6 | 897 | FT-hit-rising |
| W7 | 898 | FT-protected |
| W8 | 915 | FT-aim |
| W9 | 916 | FT-whip-stretch |
| W10 | 920 | FT-stretch-past |
| W11 | 921 | FT-shrink |
| W12 | 927 | FT-reaim |
| W13 | 929 | FT-whip-settles |
| W14 | 931 | FT-swish |
| W15 | 941 | FT-restart |
| W16 | 942 | FT-lag-counts |
| W17 | 945 | FT-curl-flight |
| W18 | 950 | FT-passes-on |
| W19 | 959 | FT-shrink-stops |
| W20 | 982 | FT-turn-back |
| W21 | 993 | FT-hit-hovering |

## Part 4c-4, the self-destruct warning

The existing played path at `74f10d6c8` stops at Ridley's retreat. There is no
existing warning owner or competing implementation. This round starts from the
flyby's `WarningReady` and uses the existing B8 doors, P1 boss progress, H2
countdown, U1 transport, and R1 passage. The prior flyby remains its own round.

### Sources and mapping

The original sources are in the [A6 bank log](https://patrickjohnston.org/bank/A6).
The warning's successor is established by `$A6:AB2E..AB5E`. Its ordinary
runs are selected by `$A6:C04E..C061`. The apparent `$0FB0` hold of 32
after English is armed has no reader on the English path.

| Entry | Source | State and observed fields |
| --- | --- | --- |
| WarningArt | `$A6:C062..C0BA`, `C136..C175`, `C26E..C2A6`, `C4CB..C537` | `0FA8=C04E`, `0FB2` values 0, 2, 4 map to ready, sprite progress and background progress. `0FB0` advances through the two art lists. At the overlap, the first background batch has already gone out. `C002/C006` and `C022/C026` supply the colour pairs and `C0C2/C0C6` and `C0A2/C0A6` receive them. |
| WarningHeading | `$A6:C0BB..C103`, `C383..C44F` | `0FB2` 6 is the heading, 8 the Japanese wait. `0FB0` is the remaining hold. `09E2` is the file's subtitle setting. |
| WarningTyping | `$A6:C104..C116`, `C2A7..C382`, `C450..C49B` | `0FB2=10`. `8036` and `8038` give the current cell and its destination. `803A` is wait and `803E` is whether the next printed letter makes a request. The first cell has zero wait because the reset value is read before the leading timing entry supplies 2. After the last wait reaches zero, the delimiter gets its own run. `803C` follows content and is implied by the cell progress. When the English text ends before the subtitle, the delimiter reloads a live two-run wait. In the usual completion it reloads scratch whose reader has finished. |
| WarningGlow | `$A6:C19C..C23E`, `AA50..AA53` | `8030` is the look. Incoming `05B6`, `7EC400` and `0797` supply the cadence, fade and passage. Live `7EC0C2..C0C7` and physical CGRAM entries 97 through 99 hold the heading colours. |
| WarningEnds | `$A6:C117..C135`, `AA50..AA53`, `$80:81A6`, `$80:9DE7..A07A` | `0FB2=12` readies the start. Activation clears it to 0 with `0FA8=AA50`, and zeroes Ridley's remaining velocities. `093F=2`, Ceres MainBoss bit 0 of `7ED82E`, and the start consumed into H2's `0943..094B` prove the effect. `0FEA` is the door lock, `1006/1046` bit 8 the two walls' hidden states. |

The current heading and English palette colours, prior baked room plane,
instantiated doors and wall visibility belong to their existing owners.
The inactive flyby matrix can retain its last physical value after the return.
This round compares the room layer and warning footprint with that transform
inactive. It does not mask a warning tile or colour written in the qualified
range. At the exit the door scheduler is the writer of palette fades, room
setup and when H2 is drawn. The inactive `0FB0`, `8038`, `803A`, `803C`,
and `803E` scratch values outside their live phases are excluded by owner
and phase. The rendered text and all previously updated art still carry.

### Asset import

Decode the art batches in `$A6:C4CB..C537` at the ROM edge. Verify the seven
sprite batches and eight foreground/door batches against the actual target tile
memory mapping. Those eight batches are five foreground text batches followed
by three door batches. The overlap puts two writes on run seven and the last
door batch and heading on run fourteen. Convert to a checked
schedule of semantic sheet patches with their foreground-versus-sprite placement.
The heading and subtitle retain each tile's relative position, palette and
priority. Decode positioned English cells from `C450` once and check that
the first rest is zero and subsequent rests are two. Ceres uses foreground
tile base `$6000` words and map base `$5000` words. Verify compatible bank
placement for the warning text tiles, partly reached by the tile-pattern
address wrap. Resolve blank slots to an absent glyph. Select the stroke on
the audible beat as sound 45h on library 2 with cap 3, and escape track 7
with delay 8. The track request on 1284 and the reply on 1293 are separate
observations. Keep a music queue at the audio boundary. No warning phase may
infer completion from a track reply.

The Japanese subtitle appears on the 32nd ordinary run after the heading hold.
At `C0F8` the nonzero branch goes directly to `C104`. On the last run
`C100` places four subtitle rows and returns to `C104`. Both routes run the
glow and English typing, so `8036`, `8038`, `803A`, and `803E` are live
during the subtitle wait. The first English glyph is still on projected cut
1413. On 1444 the subtitle rows and an English glyph are written. With the
original text, completion and activation remain on 1585 and 1586. A shortened
invented text can finish before the subtitle appears. In that case the first
finish moves from phase 8 to 10. The reset at `C2BD` leaves two wait runs
before the finished text advances phase 10 to 12. The cached records use
the English setting. Japanese played admission needs an indexed
original-side capture and a qualified importer.

### Recorded cuts and acceptance

WX1 through WX22 are observation groups for `chain-any-0006-E0B5` and
`samus-ceres-push`. The English warning before the door is common to both.
Read each vector's recorded cut as the result cut. Use the corresponding
prior cut for its input.

| IDs | Cuts | Observation |
| --- | --- | --- |
| WX1–WX6 | 1270–1276 | Room return and colour copies, then first six sprite transfers |
| WX7 | 1277 | Last sprite transfer and first background transfer |
| WX8–WX14 | 1278–1284 | Remaining background and door art, then heading and music request |
| WX15 | 1285–1411 | Heading rest and glow, track acknowledgement on 1293 |
| WX16 | 1412 | English typing ready, no first glyph yet |
| WX17 | 1413–1582 | Cells, gaps, glow and alternating letter sounds |
| WX18 | 1583–1584 | Final wait reduces to zero |
| WX19 | 1585 | Escape ready, still locked, no clock |
| WX20 | 1586 | MainBoss progress, stage Escape, door release, walls hide, quake, clock at 1:00.00 |
| WX21 | 1587–1609 | Glow, countdown hold reaches age 16 at 1602, first ticks at 1603 and 59.98 |
| WX22 | 1610–1612 | Movie touches the door with 59.87, keeps time through the first step without clock sprites, then draws the frozen time. The second row continues play and reaches 59.85 at 1611. |

The reference reads the original cache and ROM once for content, restores the
warning and picture at 1270, and evolves its own state through each row.
Ordinary versus passage opportunity, the current count of finished frames, and
the fade and passage flags are borrowed from their owners only in the scoped
reference. A played witness must derive them. Never use the result cut as a
phase trigger or supply the result's clock, actor, palette or progress state
back to the warning as fresh input. Time expiry belongs to the later escape.

The checked fields are the warning phase and progress, palette selections,
live and physical colours, the cumulative warning/door art and all its
placed text cells, actual foreground layer and text priorities, both audio
offers, stage, boss defeat, lock and walls, clock state, and the clock sprite
buffer and physical OAM. The previous cut is the null baseline. Each of 70
transfer events changes its physical target in both rows. The figure and
room composition checks use tile/palette/layer and sprite bytes in the lean
cache. Rendered picture dots are not available in this tier.

| Comparison | Movie | Movie null | Second row | Second row null |
| --- | ---: | ---: | ---: | ---: |
| Warning phase | 342/342 | 336/342 | 341/341 | 335/341 |
| Heading glow look | 342/342 | 260/342 | 341/341 | 259/341 |
| Warning art and text footprint | 342/342 | 274/342 | 341/341 | 273/341 |
| Transfer events | 70/70 | 0/70 | 70/70 | 0/70 |
| Begun clock state and display | 27/27 | 2/27 state only | 26/26 | 0/26 state only |

`$TEMP/collab/ridley-escape-warning-evidence/` holds the scripts and receipts.
`reference.py` runs the 30 strict warning vectors. `records.py --restores`
checks both rows and independently decodes and continues eight live states
on each route, plus the first passage step on the movie. These checkpoints
cover middle art, heading, typing start, partial text, completion wait,
EscapeReady, the initial clock hold ending, and the clock/door handoff.
The existing `ridley-getaway-motion-evidence/warning.py` adds random and
physical quake through 1611. The original-side mutations pin the art overlap,
heading duration, glow, blank-cell beat, one-off start and passage clock.

Production acceptance should use the played paths and the ordinary room and
door entry points. Run the movie from 1270 through the first passage handoff
and the other row to 1611 with the same owners. Qualify checkpoints using
production carry and compare their next cuts against the uninterrupted run
and oracle. Plant a production fault at the overlap, a displaced glow,
an early or repeated escape start, and a countdown step during the passage.
Report those as production evidence only once those checks run.

## The shaft overlay during the escape

### Sources and join

| Entry | Source and mapping |
| --- | --- |
| OverlayAppears | The room's enemy load and initialization at `$82:E4D4..E4DB` creates the overlay in slot 0. `$A6:F6C5..F72A` seeds its figure and existing escape door target colours. `$F739..F764` queues a contiguous 1,024-byte overlay sprite strip at `$B0:C400` for `VRAM $7000` in word units. The record's region first equals the imported strip on interrupted room-2 cut 225. Target sprite palette 7 colours 1–15 also match at 225. |
| OverlayRumbles | `$A6:F7BD..F7DB` requests lights on its first escape turn before it arms rumble. `$F7DC..F83F` holds the 48-turn rumble. It uses one shared draw for each due burst. `$F840` supplies the four offsets. The first turn after the count reaches zero hides the overlay, requests rotation through `$F67F`, and leaves the next turn to `$F850`. |
| BeaconCycles and PlatformTurns | `$A6:F850` sees the fade numerator before the fade's own step. `$A6:F871` supplies the beacon's sets. `$F8F1` queues one of two four-byte looks from `$F918` and `$F91C` for the low byte of each word `VRAM $060E..0611`. The four high bytes carry through. Use the gameplay frame count, not the HUD counter. |
| Figure and particles | The overlay becomes intangible and holds its one figure through `$A6:F610`. `$86:E509` receives the burst with its whole room position and smoke or small-explosion clip. It stays in place, uses the common effect sprites, and can retire on the same turn if outside the effect player's 256 × 256 px view. Its graphics and clip belong to the effect player. |

Room-2 cut 239 interrupts its first brightening work after the actor turn. The
overlay figure and the new rumble count are present. The beacon's six colours
at `$7EC052..C05D` match the selected asset, with the fade numerator at zero.
The platform chooses its second look on that turn. The display publishes its
four low bytes on the completed cut 240. Its four high bytes agree across the
transfer. The first particle at 241 is smoke at (234,643) px. The sender still
suppresses effects, so the actor's sound request leaves the queue unchanged.
The source calls the library 2 queue entrance with limit six at `$A6:F83B`.

At shaft cut 34 the overlay hides and requests rotation. The shaft's hold
decrements on that turn after it receives the request. The overlay's next turn
is shaft cut 35 and again chooses lights. There is no retile request at the
moment of hiding. The invisible wall is the existing shaft wall actor and has
no role in the overlay's presentation. This piece leaves the rotation track,
elevator trigger, effect player and turning-vent projection to their own rules.

[Bank 82](https://patrickjohnston.org/bank/82) · [Bank A6](https://patrickjohnston.org/bank/A6) · [Bank 86](https://patrickjohnston.org/bank/86)

### Placement and vent dependency

The destination slots first become visible together on interrupted cut 225.
They include the overlay, the invisible wall and eleven vents in population
order. The turning variants initialize as ordinary rightward or leftward
vents according to their mount. The waits match eleven successive ordinary
draws beginning with the admitted random number at cut 224. The last draw is
the carried random number at 225. Calling the old fixed-vent initializer is
enough for that part of placement once the mount metadata is retained. Later
projection belongs to drawing and reads the room matrix and pivot. It leaves a
vent's unrotated anchor and collision reach in place. The earliest integrated
actor check needs the overlay tile strip and target palette, both door states,
the eleven waits and the outgoing random number.

### Harness

Read the overlay function and visibility from `$7E:0FA8` and `$7E:0F86`, and the
rumble timer, burst timer and point from `$7E:0FAE`, `$7E:0FB0` and `$7E:0FB2`.
The point counts backwards in the record. Normalize it to the spec's offset
cycle. The actor stays in slot 0. Check particle slots independently of the
overlay state, using the centre and the first position in the imported clip.
The one spawn absent at shaft cut 31 is outside the effect player's view.
Its RNG draw is still observable. The placement, first actor turn, first
particle and disappearance have different observable cuts.

The reference carries the overlay through 219 cuts from room-2 cut 225 to
the shaft's cut 164, with its actor opportunities supplied by passage and
platform timing. It agrees on the 180 actor turns and all retained cuts.
Holding the initial overlay matches 14/219. Nine spawns agree in a live
particle slot, and all ten agree on the resulting shared RNG. The platform and
beacon check matches all 130 hidden-overlay turns after the rumble. The seven
constructed vectors exercise the initial tile intent, first lights, waits,
both sides of the smoke threshold, final turn and hidden lights. The frozen
state first differs at 239. Advancing on an interrupted continuation rather
than an actor opportunity first differs at 240.

## Shaft rotation and elevator reach

### Sources

| Entry | Source and mapping |
| --- | --- |
| ShaftTilts | `$89:ACC3..AD5D` runs late in the shaft's ordinary room work. The overlay enables rotation through `$A6:F67F` on its last turn. The rotation timer and next sample come from the initial room setup. `$89:AD5F` has 69 triples in hold, cross term, diagonal order. Import the hold as whole turns and the two terms as signed 8.8. The first track update reads the centre sample, zero-based index 34. |
| ElevatorReached | `$89:ACCD..AD17` reads Samus's post-motion whole coordinates and both vertical speed words. It calls `$90:F125..F14F` for the standing pose and hold, then resets the fade delay and counter. The lock at `$90:F109..F116` cancels pending pose changes. The timer and rotation phase already ran or can still run on this turn in their own order. |

In ordinary play, the actors and projectiles run before the room's late update
at `$82:8BA8`. A vent turn sees the matrix carried in from the previous cut.
The room accepts the elevator reach and then still takes its rotation step.
The scene's hold and blackout call the same ordinary room work at `$82:8373`
and `$82:8394`. The final room work is already present on interrupted cut 239.
No new room turn runs when the scene change appears at cut 240.

[Bank 82](https://patrickjohnston.org/bank/82) · [Bank 89](https://patrickjohnston.org/bank/89) · [Bank 90](https://patrickjohnston.org/bank/90)

### Record evidence and integration

| Quantity | Observation |
| --- | --- |
| Enable, cursor, hold | Read `$7E:093F`, `$7E:07E1` and `$7E:07E3`. The rotation bit first appears at shaft cut 34 and that turn reduces the hold from 60 to 59. Cut 93 reaches zero. Cut 94 reads the identity sample and advances the cursor. Cut 96 first publishes a tilted sample. A cursor already at the high endpoint moves back after displaying it. The low endpoint displays once and the cursor moves forward. |
| Affine output | Read the first matrix coefficient at `$7E:0078` and the next three words. Compare the physical PPU matrix and centre too. The view keeps the room setup's pivot (128,1008) px. |
| Elevator trigger | The shaft row `chain-any-0011-DF45` is `rS1` in vectors. At cut 163 she is at x = 147 px, too far right. At 164 she reaches (144,75) px with zero vertical speed. It sets standing left, input lock, timer 60 and both fade fields zero. The reach checks the whole pixel coordinates at `$7E:0AF6` and `$7E:0AFA` after motion, vertical speed at `$7E:0B2C` and `$7E:0B2E`, and ordinary play at `$7E:0998`. |
| Standing animation | The call to `$91:FB08` compares the destination pose at `$7E:0A1C` against the previous pose at `$7E:0A20`. Equality skips the writes to `$7E:0A96` and `$7E:0A94`. At witness cut 164 the previous pose is `0Ah` and the destination is `02h`. The call writes frame zero and count ten. |
| Handoff | Movement takes the standing pose and lock. Appearance retains its clip frame and remaining count if the previous pose equals the destination. Appearance starts the destination clip when the poses differ. The elevator scene starts its hold at 60 with both fade fields zero. Play's timer work already took its regular turn. The scene draws the held timer. The room keeps its mode, matrix and rotation state through lock and blackout. |

The reference takes the overlay's enable event and the post-motion centre and
speed as inputs. It carries its model State from the already prepared shaft at the
end of room 2. State and native PPU matrix and pivot agree at 240/240 shaft cuts
through the final room turn, with 27 track reads. A held seed matches 34/240.
The scoped elevator event matches at cut 164 and never repeats during blackout.
The constructed tracks distinguish both endpoint reversals and the step from hold
one to zero without changing the matrix. The reach cases cover both facing
directions, the strict aperture edges, the fractional speed and both mode guards.

The scope checks room rotation through blackout. It does not claim that the
existing production branch accepts this passage or plays the following scene
without its own handoff.

## Drawing the shaft's mounted vents

### Sources

| Entry | Source and mapping |
| --- | --- |
| VentPlaced | `$A6:EFB1..EFF4` uses the same wait and clip setup for the fixed and mounted vents. Mount subtype 4 shares the left clip, subtype 5 the right clip. `CeresVentFacing::ShaftLeft` and `ShaftRight` map to `VentFacing::Left` and `Right`, with the mounted marker carried separately. The eleven first waits and their outgoing RNG agree when drawn once each from the pre-placement number, in slot order at room-2 cut 225. |
| VentDrawFollowsShaft | `$A6:F019..F03E` takes the unrotated anchor and publishes the two signed differences from that anchor. `$8B:8B66..8BE8` uses the pivot and the two diagonal and signed cross coefficients. It takes each signed product's high part before summation. The matrix this shaft produces has equal diagonal terms and opposite cross terms. |
| Drawing and contact | `$A0:9461` and `$A0:9472` add these differences for drawing. The ordinary puff hit test still uses the unrotated coordinates. The existing puff figures, holds, reach, eligibility and contact scale apply. |

The vent runs at the enemy opportunity, before the shaft's late room turn.
Use the carried matrix even if the shaft will publish a new one later in that
same turn. Account for an interrupted first actor turn separately from later
cuts inside the same work. A missed opportunity holds the last offset. Setup
clears the offsets at placement.

[Bank 82](https://patrickjohnston.org/bank/82) · [Bank 8B](https://patrickjohnston.org/bank/8B) · [Bank A0](https://patrickjohnston.org/bank/A0) · [Bank A6](https://patrickjohnston.org/bank/A6)

### Harness and runtime join

Check anchor from each enemy slot, and the two signed draw offsets at
`$7E:7010` and `$7E:7012` plus the slot stride of 64 bytes. Use the slot clock
`$7E:0FA4` to check the eligibility signal. The existing `VentInView` model and
the platform's supplied actor opportunity select the same turns in the witness.
Read the matrix and pivot before the actor turn. Store the projected offset for
each mounted actor. Its draw intent uses the ordinary puff figure and the
anchor plus the offset. The effect's movement contact continues from the typed
unrotated vent. Figure tiles and palette encoding remain the display's inputs.

The 11 slots agree at 3,234/3,234 cuts from placement in room 2 through the
last shaft turn. This includes 1,077/1,077 turns with a new offset decision.
Zero offset matches 2,394/3,234. Reading the shaft's new matrix on the current
turn first differs at shaft cut 96, slot 4. Truncating each signed product
toward zero first differs at 97, slot 4. Combining products before rounding
fails `C-vent-offset`. The three constructed vectors also cover identity and
a retained offset.

The existing vent phase model matches 3,223/3,223 carried slot comparisons,
versus 2,146 held. On 165 ordinary shaft turns it agrees that the vents give
no hit. This shaft row reaches no positive contact. The source shares the
ordinary hit path. The scoped check covers the offset, phase and no-contact
observations and leaves native OAM comparisons to the sprite join.

## Ordinary fight picture

### Source and observations

| Entry | Native source and comparison |
| --- | --- |
| WingsBeat | `$A6:A279..A27F` seeds the beat and `$A6:A294..A2A5` qualifies the update after movement. `$A6:D97D..D9EC` uses the pace values at `$A6:D9ED`. Compare beat `$7E:780E`, pace `$7E:7810`, and remaining count `$7E:7812`. The signed speeds are at `$7E:0FAA` and `$7E:0FAC`. Their absolute sum is tested with 16-bit wrapping. Both signed minima together therefore select zero pace. |
| FightParts | `$A6:DB2A..DBC8` draws the visible tail from tip back to base. Link positions begin at `$7E:202C` and `$7E:202E`, with 20 bytes between links. Angles at `$7E:208E` and `$7E:20A2` supply the tip. At rest their low bytes are 80 and 96 in decimal. `$A6:DAD8..DB29` selects ordinary wings; `$7E:7820` distinguishes left, front and right. |
| FightParts colours | `$A6:DF92..DFA6` refreshes the hurt wait when Ridley accepts a shot. The hit count at `$7E:781A` qualifies that input. `$A6:D4DA..D4F8` reads `$7E:0F9C` before the common timer decrement and reads the cycle as `$7E:0E44 + 1`. Compare the part colours at `$7E:7818`. `$A0:9128` spends the common timer. `$A0:9149` advances the cycle. The layer draw at `$A0:9497..94C0` reads the advanced cycle and post-decrement timer for the extended body, and also owns frozen colours. |
| FightPictureOrder | `$A6:A2DC..A2FA` selects the ordinary baby and attached door's opportunity. Read animation at `$7E:7804`, station status at `$7E:093F`, attached door at `$7E:0FEA`, and the installed late hook at `$7E:178C`. `$A0:884D..88CB` sequences room layers and the late hook. `$A6:BF1A..BF2E` takes the current baby look and anchor at `$7E:8804` and `$7E:8808`. The three baby maps are `$A6:BFFD`, `$A6:C018` and `$A6:C033`. |

The indexed source row is `chain-any-0006-E0B5`, the row also used by
`ridley/flight-tail`. The ordinary reference starts before animation at
fight cut 576. At c867 `$A6:A4C1` sets the count to 8 before the ordinary
wing step. The stationary step leaves beat 5, count 8. At c879
`$A6:E969` changes vertical speed to −352 after the ordinary wing draw.
That cut still shows beat 5, count 8 and zero pace. Cut 880 moves with
vertical speed −368 and spends 32, which advances to beat 6 with 32 left.
At c881 beat 7 draws three wing pieces. Cuts 879 and 880 draw six and five.
Take-off hops at c870 and c874 move the body later than the wing, tail and
baby draw anchors.

The source-side reference carries the beat, count and hurt wait through the
row. It borrows the accepted-hit count delta, animation qualification and
flash cycle, the existing body phase and speed, tail geometry, and camera.
The phase transition at c879 selects the stationary sample. Record end speed
supplies the movement sample elsewhere in this row. The reference uses the
baby anchor plus (+16, −22) px to recover the wing anchor at the draw.
Integrated production supplies the causal pre-hop movement result directly.
The reference compares the strict part entries at the start of prepared WRAM
and physical OAM separately, including each occupied slot's high pair.
It carries 581/581 beat and countdown pairs and 581/581 paces. A held
initial clock matches 290/581 pairs. The carried hurt wait matches 1,157/1,157
pre-getaway cuts. The colour decision matches 581/581 active cuts, compared
with 518/581 if held normal. Both part-image comparisons match 581/581
active cuts. Opposite facing and hidden tail are constructed cases.

| Counterfactual | First difference |
| --- | --- |
| Omit the take-off count refresh | c867, beat and count |
| Always use outgoing speed | c879, pace and beat and count |
| Draw with normal part colours throughout | c651, part colours |
| Keep vertically clipped pieces | c1027, prepared tail and wings |
| Hold the tip pointing down | c577, prepared tail and wings |

[Bank A0](https://patrickjohnston.org/bank/A0) · [Bank A6](https://patrickjohnston.org/bank/A6)

### Renderer join and retained bytes

The room draw appends sprite objects, bomb or explosion sprites and high
enemy projectiles before its actor layers. Samus and her projectiles
appear immediately before layer 3, low enemy projectiles before layer 6,
and Ridley's extended body occupies layer 5. Samus's draw reads her
appearance before the late pose lock. The weapons owner supplies any
active weapon pieces. The trace at c675 exercises a weapon append before
the body. Both c682 and c702 have no weapon append before the door.

| Fight row cut | Tail and wings | Samus | Extended body | Baby | First attached door slot |
| --- | --- | --- | --- | --- | --- |
| 7 | 0 | after door | after door if drawn | 5 | 5 |
| 682 | 13 | 12 | 29 | 5 | 59 |
| 702 | 13 | 11 | 29 | 5 | 58 |

The art at `$A6:DB02`, `$A6:DC90..DE79` and `$A6:E983` belongs
in AF3 imported content. A full wing look has 6, 5, 3, 3, 5, 6, 5, 3,
3 or 5 pieces by beat for either side. The body uses ordered leg, hand,
torso and head maps. Its fully projected left poses have 26 rest pieces,
27 with the half-open mouth, 29 with the wide mouth and 25 in each take-off
pose. AF3 resolves semantic looks to maps. The C1 body draw supplies camera,
graphical shake and palette selection. It culls each extended submap by
its screen origin in `[-128,384)` on both axes. Its bank 81 encoders
clamp part Y and allocate the entry. The strict tail, wing and baby maps
instead omit individual pieces whose projected top Y is outside `[0,224)`
and use a wrapping cursor. The attached door uses `$81:879F`, which
parks a clipped piece while consuming its slot and caps at 128 slots.
Sample its quake before the room decrements the timer. The attached door
shifts −1 and −4 px on c1263 and c1264. It keeps its late opportunity
from c1158, without the ordinary baby. At c1586 the door's own draw can
write through the same exact writer after its attachment has cleared.

At shaft c240 (`chain-any-0011-DF45`), slots 64 and 65 keep the low X,
tile and attribute bytes last written by the attached door at fight c702.
Slot 66 keeps those bytes from c682. Their Y bytes are parked and the
resulting high-table pairs follow the later room-wide writer. The bytes
are a property of the carried buffer, including entries hidden at the
save. Slots 24 through 63 have separate shaft writers. A finished fight
frame must build the cursor from actual appends. Case c702 can only be
explained by the changed Samus count, not by a different Ridley look.
Carry the complete OAM images through real room transitions and final
save restore. Keep prepared WRAM, at `$7E:0370..058F`, and physical OAM
as separate 544-byte comparisons.

The independent writer trace reaches the shaft directly by playing the
movie from power-on. Its complete prepared and physical images agree
with the lsnes records on all 1,591 selected cuts. These are the picture
cuts through c1586, the two publication controls and shaft c239–240. Fight c1613 is a publication-only held control. Its prepared
image changes while physical OAM holds. Fight c1614 is requested and
selects the prepared image, which matches the carried physical image on
that cut. The later passage cut c1625 lies outside this picture scope.
Its native trace and pinned row differ in the prepared image and agree
in physical OAM. Suppressing requested publication first
differs at c1. Publishing on a held cut first differs at c1613. The
published OAM and prepared shadow both hold at shaft c239–240.
A separate door-operator witness qualifies all eight entries at six
fight cuts with the independently measured cursor and quake inputs.
Its shifted-position fault fails the prepared comparison at c7. A
suppressed upload fails the physical comparison at c7.

T1 supplies the qualified work and upload opportunities. The source
reference qualifies B8 state and the attributed draws locally. The
connected replay must carry the real body, appearance, weapons,
door, room renderer and PPU output through the transitions. Its scope
compares the entire prepared and physical images on each admitted cut,
the placement and high pairs of the attached door, and the restored
continuation after the final shaft save. A future production fault that
changes the leading sprite count must move or change the attached door
at c682 or c702 and change the retained bytes at shaft c240. A separate
fault must distinguish requested publication from a held cut. Keep
existing replay outcomes and unsupported boundaries until the
connected runner earns the new comparison.

[Bank 81](https://patrickjohnston.org/bank/81) · [Bank 90](https://patrickjohnston.org/bank/90) · [Bank A0](https://patrickjohnston.org/bank/A0) · [Bank A6](https://patrickjohnston.org/bank/A6)

## Budget

Moved from the spec's header (MODEL §1 decision 10): the implementer is not told the score or
the budget. `tools spec-units` reads k and the areas here.

**Budget (MODEL §1 decision 7).** k = 37.5 weighted tokens per spec unit, frozen on
samus-movement. There is no old code: the engine carries the doors unrun
(`EnemyBody::Unmodelled`, LEGACY "slots it carries but doesn't run") and has nothing for the pad,
the cover, the beacon or the platform, nor for the door of Ridley's room, the walls or Ridley
(`LoadFault::LaterCeresActor`), so no area has `old`, and there is no old-code guideline to
compare (MODEL §1 decision 19). Part 4a has none either: no other code runs his waking. Part 4b has none:
the engine has no tail. Part 4c-1a has none: the engine refuses his flight.
Part 5 has no old model.
At 63a431866 the played path refuses the real retreat, with no retreat implementation to replace.
Part 4c-1b adds the moving tail, for which base `a6a2a0604` has no old implementation.

- Area doors: 1, Waiting, SheIsNear, RoomStart, Polling, Opening, Closing, ShaftWall, Solid, Shown
- Area station: EscapeShakes, TheDoorsColours
- Area shaft: PadPlaced, PadWaits, PadCarries, PadFlickers, CoverHides
- Area lights: BeaconCycles, PlatformTurns
- Area ridley-door: RidleyRoomStart, SlamShut, LockedIn, DrawnWithRidley, GetawayWalls
- Area ridley: RidleyPlaced, RidleyWaits, BabyHeld, BabyFidgets, BabyCries
- Area ridley-wakes: EyeFadesIn, BodyFadesIn, ReadyToRoar, Roars
- Area ridley-tail: TailAtRest, TailCurls, TailHits
- Area ridley-flight: TakesOff, Rises, SlowsDown, Hovers, PulledToHisPoint, Flies
- Area shaft overlay: OverlayAppears, OverlayRumbles, 18
- Area shaft rotation: ShaftTilts, ElevatorReached, 19
- Area steam: 20, VentDrawFollowsShaft, VentPlaced, VentInView, VentWaits, VentPuffs, VentRests, PuffReach, PuffTouches
- Area ridley-tail-flight: TailComesLoose, TailFollows, TailSwings, TailWhips, TailRecovers, TailStretches, TailBends, TailSwishes

- Area ridley-retreat: FliesOutOfReach, GetawayBackdrop

- Area ridley-getaway: GetawayPass, GetawayPicture
- Area ridley-warning: WarningArt, WarningHeading, WarningTyping, WarningGlow, WarningEnds
- Area ridley-fight-picture: WingsBeat, FightParts, FightPictureOrder

Budget: 37.5 × 276.0 units = 10,347 (doors 1,062, station 237, shaft 462, lights 200, ridley-door 425, ridley 700, ridley-wakes 625, ridley-tail 350, ridley-flight 1,212, shaft overlay 350, shaft rotation 212, steam 1,375, ridley-tail-flight 1,350, ridley-retreat 350, ridley-getaway 362, ridley-warning 675, ridley-fight-picture 400)
