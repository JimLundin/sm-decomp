# Entering the Ceres shaft during the escape

This note covers the [shaft entry](world-shaft-entry.md). The frozen movement
implementation at `c44e3bcef` holds room 2 through cut 221 and the continuous escape
through cut 1,075. It reports `Load(DoorRule(ToCeresElevatorShaft))` on cuts 222
and 1,076, respectively. The room 2 row is `chain-any-0010-DF8D`. The following
shaft row is `chain-any-0011-DF45`. Vectors name the room 2 row `rS2`.

## Sources

| Entry | Original source | What it publishes |
| --- | --- | --- |
| ShaftViewStarts | Incoming door from `$8F:DF8D` selects `$8F:E4E0..E512` | A room mode request, identity matrix, centre and enabled affine bit. It sets the first rotation-table index to 34 without the reverse bit and sets the hold to 60. |
| ShaftViewStarts | Room FX through `$89:AB82` selects `$88:D928..D94F` | A line program for the HUD and room modes. Its table uses 31 lines of the HUD mode, then 112 of the room mode. The second mode remains set after the program's last line. |
| ShaftDoorSeals | Version `$8F:DF71` selects setup `$8F:C96E`, which places `$84:BA48` at (15, 38). There is no population placement in `$8F:C8BB`. | The placement's setup `$84:B9F1..BA47` preserves the low 12 bits of each of the four cell words and replaces the collision kind. It does not write any mark. Its one script instruction at `$84:AAE3` deletes the placement without a tile redraw. |
| ShaftDoorSeals | The shaft setup falls through to `$8F:C976` and calls `$88:DDC7`. | The room places the red haze program `$88:DEEB` for the defeated-boss state. `$88:DE15` initializes its red component and zero intensity when its pre-instruction runs. The ordinary shaft version uses `C976` without the four-cell placement. |

The passage has selected the destination room header before this work. Inside the
contents fetch, `$82:E4A9` clears expiry-bound room objects and invokes
`$82:EB6C`. The room's ordinary placements come first, then the door rule and the
room setup. The room FX and the destination actors follow within the same work.
Once the slide has ended, `$82:E53C` runs block scripts once while the contents
fetch is still in progress. The door placement disappears then. Its collision
changes remain.

[Bank 8F](https://patrickjohnston.org/bank/8F) · [Bank 84](https://patrickjohnston.org/bank/84) · [Bank 82](https://patrickjohnston.org/bank/82) · [Bank 88](https://patrickjohnston.org/bank/88)

## Record comparison

| Quantity | Mapping |
| --- | --- |
| HUD and room mode requests | Low bytes at `$7E:0055` and `$7E:0056`. The room split uses the published mode bytes at `$7E:07EB` and `$7E:07EC`. |
| Affine state | Enabled at `$7E:0783`. Signed 8.8 matrix at `$7E:0078..007F`, centre at `$7E:0080..0083`. Actual PPU matrix and centre at `$211B..2120`, separate from `$2105`. Observe outside and flip settings through PPU `$211A`. |
| Rotation seed | `$7E:07E1` gives sample and direction, `$7E:07E3` the hold. The incoming hall uses the former word for debris delay. At cut 221 its value is 2. It is replaced at 222. The hold already reads 60 at 221, so a constructed case checks the overwrite from a different hold. |
| Room cells | `$7E:07A5` × `$7E:07A7` gives 16 × 48. Cell words at `$7F:0002` and marks at `$7F:6402`. Compare all 768 words and marks. |
| Door placement | `$7E:1C85` holds `BA48` on cuts 222–233 and clears at 234. Its location word at `$7E:1CD5` names the first sealed cell. |
| Haze and mode line program | Active slots at `$7E:18B4` and instruction positions at `$7E:18CC`. The programs are already present on cut 222. |

At cut 221 the destination grid already has the four open door cells. At interrupted
cut 222 they are solid, the placement and both line programs are present, and the
matrix and pivot have reached both their mirrors and the PPU. The PPU mode at that
cut still reads the HUD value `09h`. The room rule writes `07h` to the hardware as
it runs, but NMI's upload `$80:91FD..91FF` writes the HUD mode again. Independently,
`$80:92E3..9336` prepares the room mode value for the line program and uploads the
affine registers. The inactive affine hardware registers at 221 retain an earlier
transform even though the entry's software matrix is zero. Initialize both on 222.

At cut 234 the slide is complete and the placement retires. The fetch phase is
still visible, changing on 235. The PPU mode first reads `07h` on the interrupted
brightening cut 239. On 240 it again reads `09h` after the register upload.
The line program determines the mode during scanout. Its room mode request and
the PPU mode at a cut are distinct comparison fields. The R2 witness compares
the setup's view and affine registers. It observes the PPU mode separately and
does not infer a line schedule from one mode value. The setup leaves the slide's
scroll, the affine repeat and flip controls, and the room's current escape stage
where their owners put them.

[Bank 80](https://patrickjohnston.org/bank/80) · [Bank 89](https://patrickjohnston.org/bank/89)

## Production join

The door rule and room setup already have typed variants `ToCeresElevatorShaft` and
`CeresHazeDoorShut`. The loader can run this static setup from those variants. Its
current `PlmPopulation::placed_blocks` sees only population scroll and extension
entries, so filtering that population cannot find this separate room setup. The
setup variant identifies one fixed closing placement. Attach its anchor in the
typed setup content for the loader. Apply the four-cell write to the built room at the early contents hook. Preserve each
typed `data::level::Block`'s tile and enter its solid kind with the prior mark.
Rebuild movement geometry and omit these four cells from the usable doorways.
Carry the passage's already selected arrival through the slide. A loader that
closes the cells during `BuildRoom` changes the pre-setup cut.

Publish a typed affine view and shaft rotation seed to the screen and actor
consumers. Keep the mode request and the renderer's HUD mode separate.
`game::state::video::Mode7` and the runtime's affine renderer can encode the
matrix and pivot. The played loader still needs the handoff to the display
scheduler and record comparison. Placement retirement follows the block-script
turn. E4/T1 owns the scanout and the actual cut of publication. The red haze
placement is part of the room setup, and the haze owner runs its subsequent
display decisions. The static setup needs production and typed content wiring.
The static rules fit the existing cell and display types. Separate the early setup
result from the actor result in the contents join. The current loader rejects both
at the same early call, ahead of their independently visible cuts. An R2-only
proof must scope the setup fields. The full passage still needs actor clearance
and placement at their own opportunities.

## Separate actor boundary and elevator handoff

| Cut | Actor-side observation |
| --- | --- |
| Room 2 221–224 | Old hall actors are being removed. At 222 one hall vent and two hall door actors remain. At 223 and 224 their slots are empty. |
| Room 2 225 | The two destination door actors are initialized in slots 0 and 1. The first is the overlay and the second is the invisible wall. Eleven directed shaft vents occupy slots 2–12. The first actor's pre-explosion tile upload reaches VRAM here. |
| Room 2 239 | The overlay enters rumble with 48 turns left and the first platform tile selection. The beacon's palette animation waits for the palette fade. |
| Room 2 241 | The first rumble spawn publishes a particle and requests the explosion sound. The passage disables effects at this cut, so the sound queue stays unchanged. Earlier room 2 falling debris has its own creator and clears in the fetch. |
| Shaft 34 | The overlay hides and enables rotation. The room's first rotation turn counts the 60 hold down to 59. |
| Shaft 94, 96 | The room reads the first rotation-table sample at 94 and advances the sample cursor. The first nonidentity matrix is published at 96. |
| Shaft 164 | The elevator trigger enters the elevator scene and gives it the locked standing pose and its initial hold and fade state. Rotation remains live in that scene. |

The destination actor order is from the room's population. Loading and initialization
run at `$82:E4D4..E4DB`. The two door actors initialize through `$A6:F6C5`, and the
overlay's upload and target palette are actor output. The first run `$A6:F7BD` queues
the platform tile update before switching to rumble. Rumble spawns at `$A6:F7DC`,
then the room's rotation uses `$89:ACC3`. Vents, invisible wall, overlay, platform
tiles, beacon, dust, explosion and falling debris keep their actor or presentation
owners. The current loader's `TurningShaftVent` guard and the shaft lights' escape
guard remain meaningful.

The elevator trigger at `$89:ACC3..AD17` requires the rotating stage and ordinary
play, centre x greater than 112 and at most 144 px, centre y at least 75 and below
128 px, and zero vertical speed. It requests the standing pose in the previous
facing, resets the animation's new step and pending pose changes, locks Samus,
starts a 60-turn elevator hold and clears fade counter and delay. Shaft cut 164
shows (144, 75) px in whole coordinates, standing left, a hold of 60 and both
fade values zero. Pass these intents
to movement, appearance and the elevator scene. The display keeps receiving the
shaft's affine view as the new scene begins.

The first sound request is an actor intent. The source calls the library 2 queue entrance
at `$A6:F83B`. The sender's disable check `$80:90E7` applies while `$7E:05F5` is set.
The lane's head and tail at `$7E:0644` and `$7E:0647` hold at cut 241.

[Bank A6](https://patrickjohnston.org/bank/A6) · [Bank 90](https://patrickjohnston.org/bank/90)

## Evidence

The independent reference starts from cut 221's room and view. It takes the
setup and block-script opportunities as inputs. It matches all 57 cuts through
278 for the software setup, physical matrix and centre, full grid and live
placement. A frozen cut 221 matches zero. The grid and retired placement hold
across 165 more cuts through the elevator trigger. Delaying setup or omitting
closure or placement fails at 222. Two recorded cases pin the setup and retirement. The three constructed cases distinguish
a carried old hold, a changed HUD mode and a clobbered mark. These are observer
checks. Indexed production replay still needs the installer and the separate
actor, haze and display routes for the remaining comparisons.
