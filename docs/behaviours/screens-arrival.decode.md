# Her arrival at Ceres: decode and harness

Companion to the [arrival's behaviour spec](screens-arrival.md). Keep this note outside
confined exports.

## How it was made

Behaviour first, from CI's lean record cache (`$TEMP/collab/ci-cache/rows`): `any-ceres-arrival`
cuts 0-222 and `chain-any-0001-DF45` to `chain-any-0005-E06B`, read per frame by a scratch
reader of the lean tier (`$TEMP/collab/ceres-739/lean.py`, local only): the game state, the
room, the brightness mirror, the screen fade's delay and counter, her handlers, pose and
position, the enemy and projectile slots, the palette buffers, the music queue, the random
number and the count of finished frames. The bank logs (82, 80, 90, 91) were then read to pin
the fade's arithmetic and the arrival's writes.

The reference model, `$TEMP/collab/intro-756/scratch/model.py` (local only), steps the spec's
rules from record N−1 to N, with the setup done when game state turns from `1Fh` to 7 between
them and her x the start's, and compares the stage (game state `1Fh` Preparing, 7 FadingIn, 8 to
`0Bh` Playing), the brightness (`0051`: bit 7 Blanked, the low nibble the level) and the fade's
count (`0725`, not compared in Preparing, below):
- `model.py vectors`: 16 of 16 agree.
- `any-ceres-arrival` cuts 1-222: 222 of 222 agree; a do-nothing model (keep the last record)
  agrees on 191 (the setup's unchanging frames).
- The fade in alone, cuts 193-222: 30 of 30; do-nothing 0 of 30.
- On through `chain-any-0001-DF45` to `chain-any-0005-E06B` (1,846 cuts, door passages
  included): 1,846 of 1,846; do-nothing 1,815.
- The arrival's colours, cut 191 to 192: the goal buffer unchanged; the 239 colours outside
  192-207 and 223 take their goal (239 of 239); colour 223 is 0; a do-nothing model (screen kept)
  agrees on 77 of 256.

## Sources

| Entry | Sources |
| --- | --- |
| StartsInTheDark | game state `1Fh`, `$82:8000` (set up new game): one call from the main loop that spans cuts 1-192, with its own waits for NMI (`$80:8338`) and NMI off 2-85 and 154-163; forced blank throughout (`0051` = 80h) |
| ArrivesAtTheTop | `$82:8106`..`$82:813B`: game state = 7; palettes = target palettes (`7EC200` → `7EC000`, 200h bytes); `7EC1BE` (sprite palette 5 colour Fh, index 223) = 0; Samus command 8 (`$90:F084` → `$90:F1E9`): handlers `$E8CD`/`$E8DC` (locked), pose 0 (facing forward, power suit), `$91:DEBA` (load suit palette by equipped suit, 20h bytes to `7EC180`), `0AFA` = 0, spawn `$86:A387` and `$86:A395`, `05F7` = 0 (mini-map on), `$82:E118` with A = 20h (the room's music: §5's, U1) |
| the count `d` at the arrival | `$82:8083`..`$82:8089`: screen fade delay `0723` = 1 and counter `0725` = 1, written inside the setup (cut 184), before the arrival |
| PictureBrightens | `$82:8B20` (game state 7: main gameplay `$82:8B44`, then `$80:894D`); `$80:894D`: counter − 1 unless negative; else counter = delay and `(0051 + 1) & 0Fh`, stored unless 0 (so level 15 stays, blank included; a store clears bit 7) |
| PlayBegins | `$82:8B2A`..`$82:8B3A`: `0051` = 0Fh (byte compare: blank clear and level 15) → delay 0, counter 0, game state 8 |

## RAM mapping

| Spec | RAM | Notes |
| --- | --- | --- |
| the stage | `0998`: `1Fh` Preparing, 7 FadingIn, 8 and the door passage's 9 to `0Bh` Playing | the arrival's own; play's states are other domains' |
| the brightness | `0051` (INIDISP mirror): bit 7 Blanked, bits 0-3 the level | 80h through the setup and 192-193, then 01h at 194 rising by one every 2 cuts to 0Fh at 222 |
| the fade's count | `0725` | 1 at 192, alternating 0 and 1 from 193, 0 from 222 |
| the rhythm `d` | `0723` | 1 from cut 184 to 221, 0 from 222; a rule in the spec, not State |
| her place at the arrival | `0AF6`, `0AFA` | x 80h from the load station (written at cut 3 with y 48h); y 0 at 192 |
| the screen's colours | `7EC000..7EC1FF`; goal `7EC200..7EC3FF` | world-doors' palette, as the played runner compares them |
| the room's music (spec §5, U1's) | music queue `0619`/`0629`: at 192, entries 1 and 2 hold a music stop (track 0) with timer 20h, then the room's track 6 with timer 8; the music timer is 32 at 193 | `$82:E118` queues the stop with A's delay (`$80:8FF7`), then the track (`$80:8FC1`, the queue's own wait). A = 20h is a leftover register value (the index of the second projectile spawned), not a chosen wait. The played runner compares no music request (it borrows the queue only as `music_ready`), so the request stays U1's and outside this spec |

## Harness (for the integrator)

The row is `any-ceres-arrival` (seed `any-opening:6`, movie frame 8,315, 223 cuts). The played
runner refuses it at cut 0 today. What it needs, as one admission of this row:

1. **Admission from the start, not from ordinary play.** The seed is the intro's last record
   (game state `1Fh`, room 0000, her place and handlers the intro's, an unset actor roster, which
   `seed_doors`/`movement::roster_slots` read as a DataFailure). Build State from the start
   instead: the arrival's stage Preparing with the seed's brightness (blanked, level 0) and count;
   the start's room (`DF45`, the shaft) loaded by `room_loading` as a passage loads a room, with
   its two Ceres doors (slots 0 and 1, parameters 2 and 1, `RoomStart`) and the shaft's overlay;
   a fresh Samus (standing at rest, pose facing forward, no speed: every field `$91:E00D`
   clears); a fresh HUD; the new file's progress (energy 99, no items or ammunition, play time 0,
   stage Calm), which the seed already holds.
2. **Cuts 1-191 (the setup): a borrowed span that counts as played.** "The start ready" is a
   declared borrowed input, owned by R2/C1's loading with T1's lag; the setup's fixed waits
   (the seven passes of enemy setup with a wait for NMI each, cuts 185-191) are part of that
   length. The input is read from the record's game state (`0998` turning `1Fh` to 7 between
   N−1 and N, so cut 192), the same field the stage maps to: the stage's change earns no
   evidence there; the evidence at 192 is her place, the pad and cover, the colours and the HUD.
   Compare at every cut of the span: the stage, the brightness, and the random number from
   cut 2 (one draw at cut 1 → 2, the main loop's draw for the frame of work that runs the
   setup, 588Eh from AB19h; then none until 193). The other owners' fields are the setup's own
   working values, with nothing in the loop that produces them yet: her place (`0AF6`/`0AFA`
   written at 3), the colours (the setup's working palette), the HUD (its buffer written at 184),
   the enemy and projectile slots (doors placed at 191), the fade's count (written at 184). The
   finished-frames count rises on the setup's own NMI waits (cuts 1, 86-105, 107-122, 150-153,
   164-189, 191): borrowed, as the lag is.
3. **Cut 192 (the arrival).** Compare from here every field the runner compares in play: her
   place `(0080, 0000)` and pose; the doors; the pad and cover as `PadPlaced` leaves them; the
   colours (screen = goal but 192-207 her suit's and 223 black; the goal 0-191 and 208-255 as
   the room's loading leaves them, 192-207 the start's goal colours for her block, harness case
   H3); the HUD; the random number as cut 2 left it (no draw on the arrival's frame); the fade's
   count, compared from this cut on (1 at 192, alternately 0 and 1 from 193, 0 from 222; the
   rhythm `0723` is a rule and stays uncompared).
4. **Cuts 193-222 (the fade in).** Each a frame of play (her frame held, the doors, the pad and
   cover, the beacon and platform, the HUD, one random draw, play time), then `PictureBrightens` and
   `PlayBegins`. Admit game state 7 beside 8. Add two observations, the screen's brightness
   (blanked or shown, and the level), compared at every cut of the row, and the fade's count,
   compared from 192; it stays 0Fh through
   rooms 1 to 5 (the model above). No cut in 193-222 is a lag cut.
5. **The join.** From cut 222 the run goes on into `chain-any-0001-DF45` cut 0 (finished frames
   7,840 then 7,841; movie frames 8,537 then 8,538). `Chain::following` follows seeds of one
   capture only (`any-opening:6` has `:7` next, after the flight to Zebes), so the link to
   `any-boundary:1` must be made by the seeds' movie frames: 8,315 + 223 = 8,538.

**Harness cases** with the game's content (importer): H1, her suit's colours at cut 192 are the
power suit's sixteen (samus-appearance's content); H2, the start's x and room (cut 192: `0080`,
`DF45`); H3, the goal's 192-207 at cut 192: the start's goal colours for her block. The
setup copies them into the goal at cut 184 from its working palette (`$82:806B`..`$82:8082`,
"target Samus palette = Samus palette"), not from her suit: 5 of the 16 (block places 1, 2, 10,
11, 12) differ from the screen's suit colours at 192, 222 and `chain-any-0001-DF45` 0-1. The
room's loading doesn't write them, so a harness that builds the goal as a passage does must take
them from the start's content.

**Recorded cases** behind the vectors (her x moved to 0140, brightness and counts kept):

| Case | Row:cut | Vectors |
| --- | --- | --- |
| S1 | `any-ceres-arrival`:185 → 186 | R-setup |
| S2 | `any-ceres-arrival`:191 → 192 | R-arrival |
| S3 | `any-ceres-arrival`:191 → 192, the palette buffers | C-arrival-colours (invented colours in the recorded pattern), harness case H1 |
| F1 | `any-ceres-arrival`:192 → 193 | R-first-frame |
| F2 | `any-ceres-arrival`:193 → 194 | R-first-step |
| F3 | `any-ceres-arrival`:207 → 208 | R-step-mid |
| F4 | `any-ceres-arrival`:221 → 222 | R-last-step |

## Findings behind the spec's choices

- **The arrival's frame plays nothing.** Game state turns to 7 at the end of the setup's call,
  so the first main gameplay (and the pad's first run, the first random draw after cut 2, the
  first play-time frame) is cut 193: play time is 0 at 192 and 1 at 193.
- **Blanked level 15.** The fade's store is skipped at level 15 whatever bit 7 holds, and
  PlayBegins compares the whole byte, so 8Fh would hold the fade in forever (C-full-blanked).
- **The count's range.** `LDA`, `DEC A`, `BMI` on 16 bits tests count − 1: a count of 32,768
  decrements to 32,767 and counts down, while counts of 32,769 to 65,535 step at once. So the
  count and the rhythm run from 0 to 32,768, the bound boss-ceres-ridley's `PadWaits` uses for
  the same pattern.
- **Ownership.** The arrival is not the station's (boss-ceres-ridley part 2 takes the pad's
  placing from it as an input, §5 there, and names F4 and P2 as the arrival's owners), so it has
  its own spec and file; the brightness fade is shared with every scene that fades in (the
  title, the intro's pages, the flights), which F2 and F4's later pieces may read from here.

## Budget

Moved from the spec's header (MODEL §1 decision 10): the implementer is not told the score or
the budget. `tools spec-units` reads k and the areas here.

**Budget (MODEL §1 decision 7).** k = 37.5 weighted tokens per spec unit. There is no old code:
the engine has no arrival (the played runner refuses a seed outside ordinary play), so no area
has `old`.

- Area arrival: StartsInTheDark, ArrivesAtTheTop
- Area fade: PictureBrightens, PlayBegins

The spec-units counter reports four entries, nine arms, no cells and two operators.
Budget: 37.5 × 9.7 units = 362 (arrival 112, fade 250)
