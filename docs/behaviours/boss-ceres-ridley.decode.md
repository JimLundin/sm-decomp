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
  figure's runs agree on 355 (at 867 only the phase `Flying`, the rest being part 4c's);
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
| `Flying` | `0FA8` `$A354` | set with list `$E91D`, wings `7E7810`/`7E7812` = 8, tail neutral (`$D3D9`, `7E2000` = 1), `0FB0` = 0: part 4c's |
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

- **Rows:** `chain-any-0006-E0B5` 512-866 for this part (867 for the phase `Flying` only).
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
   below). Part 4a refused a run that started `Flying`; since part 4c-1a he takes off and
   flies through production, and a run that starts in an attack or as he gives up is refused
   (`Fault::RidleyAttacks`, `Fault::RidleyGivesUp`, later parts).
2. His colours go to the screen as the baby's do (`production/ridley.rs`, `show_eye` and
   `show_body`): the eye's set to sprite palette 7 colours 12-14, the body's set to sprite
   palettes 1 and 7 colours 1-11, at once.
3. The verify reader (`crates/verify/src/replay/ridley.rs`) decodes `$A389` with `0FB0`, `$A3DF`,
   `$A455`, `$A478` with the figure from `0F92`/`0F94`, and `$A354` as `Flying`.
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

| Compared, 0006 867-994 (127 runs from `Flying` on, 994 the hand-over) | Model | Do-nothing (last run's value) |
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
clear, `7E7804` = 1) on all 127 records from `Flying` through `Hovering` (867-993). `vecrun.py`
runs the 48 L- vectors: 48 of 48 agree.

### State (the RAM mapping)

| Spec | RAM | Notes |
| --- | --- | --- |
| `Flying` (his take-off) | `0FA8` = `$A354` | zeroes `0FAA`/`0FAC` each run |
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
1. Production (`crates/game/src/production/played/ridley.rs`): run him from `Flying` through
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

- Area doors: 1, Waiting, SheIsNear, RoomStart, Polling, Opening, Closing, ShaftWall, Solid, Shown
- Area station: EscapeShakes, TheDoorsColours
- Area shaft: PadPlaced, PadWaits, PadCarries, PadFlickers, CoverHides
- Area lights: BeaconCycles, PlatformTurns
- Area ridley-door: RidleyRoomStart, SlamShut, LockedIn, DrawnWithRidley, GetawayWalls
- Area ridley: RidleyPlaced, RidleyWaits, BabyHeld, BabyFidgets, BabyCries
- Area ridley-wakes: EyeFadesIn, BodyFadesIn, ReadyToRoar, Roars
- Area ridley-tail: TailAtRest, TailCurls, TailHits
- Area ridley-flight: TakesOff, Rises, SlowsDown, Hovers, PulledToHisPoint, Flies
- Area steam: VentPlaced, VentInView, VentWaits, VentPuffs, VentRests, PuffReach, PuffTouches

Budget: 37.5 × 172.0 units = 6,448 (doors 1,062, station 237, shaft 462, lights 275, ridley-door 425, ridley 700, ridley-wakes 625, ridley-tail 350, ridley-flight 1,212, steam 1,100)
