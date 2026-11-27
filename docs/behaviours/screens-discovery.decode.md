# The baby's discovery, decode and replay evidence

Companion to the [behaviour spec](screens-discovery.md). Observer and integrator only.

## Sources

Use the pinned lean records in `any-intro-2219` and `any-intro-4323`. The first row's
comparison starts between cuts 1297 and 1298. The second row's cut 0 follows the first
row's cut 2103 at the same completed-frame count. [Bank 8B](https://patrickjohnston.org/bank/8B)
contains the cinematic code and authored motion. [Bank 8C](https://patrickjohnston.org/bank/8C)
contains the drawings.

| Behaviour | Sources and interpretation |
| --- | --- |
| EggHatches | `$8B:AF6C..B017` sets up room, egg, baby and initial draw order. `$A8D5..A902` gives the egg anchor and tests Samus's post-step x against 169. `$BA4B..BA5D` gives the baby anchor. `$CB33..CB9B` gives the egg drawings, holds and markers. `$93AC..93D8` explains why an unfinished setup cut can contain active placements with no picture. |
| ShellScatters | `$8B:CB3B..CB79` and `$A918..A993` select the six shell fragments, their sound and anchors. `$A9EA..AA99` gives horizontal drifts and 38 vertical displacements through the last drawable frame. `$A994..A9E9` advances on the birth frame and removes fragments where whole y reaches 168. `$CD39..CD68` identifies fragment drawings. |
| BabyEmerges | `$8B:BA5E..BB23` reads the marker reached by the egg's `$CB3B` sequence. On that frame the baby changes to rising. Actual movement starts on the following frame. The vertical comparison at `$BAA6` uses Samus's centre offset by 32 px. A nonnegative new velocity starts the hold. `$A35B..A38B` puts the movement after Samus's turn. |
| SlimeFalls | `$8B:BA73..BAA5` gives the rise y test, sound and four droplet releases. `$AA9A..AB34` gives birth at the whole-pixel anchor, the first step on that frame and the floor transition. `$AB35..AD54` gives the four drifts and two vertical paths. Decode the split signed whole and unsigned fraction words as signed fixed point. `$CD69..CD82` gives the air and splash pictures and holds. |
| BabyPulses | `$8B:CE79..CE7E` locates the baby's animation. `$CC2B..CC3E` supplies four ten-frame holds. `$93EF..9437` explains the first picture on the first completed actor turn. |
| BabyFollows | `$8B:BB0D..BC07` gives the hold release, both velocity limits and the cry cadence. The displayed order follows the new horizontal velocity. The inline comments on `$BB91..BBA0` disagree with the stores. `$8E2D..8E51` and the resulting OAM order disambiguate them. |
| PageThreeApproaches | `$8B:CB8D..CB9E` holds the spent shell for 320 frames before `$B33E` sets up the page-three cue. `$B370` starts page three on the next finished frame. `$B3F4..B457` decrements the return count before the room turn. `$8E0D..8E2C` uses the incoming visibility. `$A903..A917` and `$BB24..BB38` remove the egg and baby during the sprite pass when the updated count is zero. `$8E2D..8E51` draws from the resulting order. |

The art references come from `$8C:8D6F..8FD9`. Use the existing E1 sprite art import to
decode offsets, dimensions, tile and intra-picture order. The scene uses sprite palette 7.
The intro graphics are the tiles at `$95:E4C2`, installed by the intro setup at
`$8B:A43A` and `$A522`. The intro authoring source for their colours is `$8C:E3E9`.
The level data at `$8C:C083..C382` and its collision import belong to the demo room.
Discovery's art, placed picture references, holds and trajectories should have their own
typed asset record. The importer resolves the shipped shell's closed and spent pictures,
unrolls the finite egg animation, and places both scatter and release markers at age 252.
The sum of the finite holds is 632 finished frames. The recurring spent-shell hold does not
change its picture. The shell sound maps to sound library 2 offer 11, and the cry maps to
library 3 offer 35. In both cases the source sets the maximum queue size to six.

## Record mapping

| Evidence | Decode |
| --- | --- |
| Timing and frame inputs | `05B6` counts completed frames. Whole Samus centre is `0AF6,0AFA`, sampled after her room turn. Discovery placement starts on the partial cut with stage `1F51 = B250`; the late cue shows `B370`; page three's return dissolve stage is `B3F4`. Use `1A4B` for the updated return count. |
| Egg and baby | Native shell slot 14, baby slot 13. At each doubled slot offset, `1B1D` gives presence and next animation position, `1A5D` the current drawing, `1B5D` the picture hold, `1B3D` the motion stage. A zero animation position means absent; a zero current drawing in an active placement means its first picture has not run. Hatch age is elapsed finished frames since the trigger. Baby pulse age can be reconstructed from its picture occurrence and remaining hold. |
| Positions and baby motion | Use the same doubled slot offset at `1A7D.1ADD` and `1A9D.1AFD` for coordinates. Whole words are signed, fractions unsigned. `1A4D` is the hover wait during hovering and horizontal velocity in 256ths of a pixel per frame while following. `1A4F` is vertical velocity in the same units. `1B97` carries the cry age. |
| Fragments and drops | Identify fragments by their drawings in `$8C:8F7E..8FA1` and droplets and splashes in `$8C:8FA8..8FC4`. The per-slot `1B7D` word splits into flight age in the high byte and cohort identity in the low byte. Resolve the splash age from its drawing and `1B5D`. At impact the native writer leaves the pre-impact flight age frozen. Mask that byte during splash because motion has stopped and no behaviour reads it. The clean splash carries the stopped anchor and splash age. |
| Order and draw | `1A57` values 1, 65535 and 0 decode to scene first, Samus first and hidden. Subtract `0911/0915` for screen anchors. The OAM low and high tables in `0370..058F` show discovery's palette-7 entries and their precedence relative to Samus and projectiles. The caret uses palette 6. |
| Sounds | Track queue-end bytes `0647` and `0648` for libraries 2 and 3 and the added entries in `0666..0675` and `0676..0685`. Qualify the requests with the corresponding owner outputs. Page-three typing writes its own library-3 clicks, code 13. |

## Replay

Start a scoped witness at `any-intro-2219:1297` with the discovery owner absent. Use the
intro's real setup, run each actor turn from the room's output and continue across the
`any-intro-4323:0` seam through cut 129. If using the latter as a separate checkpoint,
restore the egg in spent with the active baby's observed phase, pulse, place and speed.
Do not repeat the page cue on the reseed cut. The scoped observer run borrows the
signals, actual finished-frame opportunities, Samus's centre and scroll from their existing
writers. Integrated replay must produce these in the intro, demo and T1. In particular,
do not read the trigger centre from the preceding cut or suppress the demo's last turn
with the same cut's hide output.

| Cut | Check |
| --- | --- |
| `any-intro-2219:1298` | Setup placed egg and baby, with neither picture shown. Frame count matches cut 1297. The partial stage setup belongs to the intro, T1 and E2. |
| `any-intro-2219:1299` | First finished turn with egg and baby pictures. Initial order is cinematic before Samus. |
| `any-intro-2219:1471` | Samus moves below x 169 and age zero begins on the same frame. |
| `any-intro-2219:1723`, `1724`, `1736` | The shell breaks and its six fragments already moved at 1723. The baby is released at 1723 and first moves at 1724. At 1736 the baby spawns droplets at its incoming whole-pixel anchor. The droplets already moved and the cry is offered there. |
| `any-intro-2219:1761..1763` | The last two fragments leave the drawable range at 1761. Native stale records persist for two cuts. Exclude their offscreen motion words, whose writer is `$8B:A994` reading beyond the 38 authored displacements. `$8B:9746` already omits the resulting out-of-range anchors from OAM. Do not import the subsequent code bytes as movement data. |
| `any-intro-2219:1767`, `1895`, `1896` | The baby stops and begins the 128-frame hover at 1767. The hold expires at 1895 and following first moves it at 1896. |
| `any-intro-2219:1959`, `2023`, `2065` | Both follow cries. The order goes from scene first to Samus first at zero horizontal speed at 2065. |
| `any-intro-2219:2103` | Cue after the room's turn, while both still draw. The spent shell and active baby carry into the return. |
| `any-intro-4323:0`, `1` | Cut 0 has the page setup, with the encounter unchanged and no completed actor turn. On cut 1 the encounter and page advance. |
| `any-intro-4323:127..129` | Cut 127 still shows the egg, baby and Samus. At 128 the return count is zero and the demo's final turn publishes neutral controls and completes its in-flight work before hide. On 129 neither the demo nor discovery takes a room turn. |

Compare encounter presence, stage, native pictures resolved to content identities, actor positions
and phase-specific timers at every scoped cut. Compare ordered sound offers, the handoff and the
OAM bytes of discovery entries (including size and high X). E1 writes the rest of OAM. Ignore
the unused OAM tail hidden by E1, the OAM shadow retained from the preceding finished turn
during T1's partial placement, and positions or timers after deletion by the cinematic
sprite pass. Mask the native spent shell's hidden repeating hold because the picture is fixed.
Leave introductory text, caret pixels, colours, room transfers and Samus's pose and projectile
art to their owners. The reference OAM check uses the order of palette-7 entries against
non-caret entries to check relative precedence. It checks their position and tile attributes
separately from the movement comparisons.

The observation vectors name the cut that ends each case. Use the preceding cut as the
starting observation. For `R-return-start` the preceding cut is `any-intro-2219:2103`.
The native rise checks the whole-pixel release height on each rising frame. The shipped
movie hits it once. A restored rise can remain at that height across multiple turns. Discovery
uses a single release over the encounter as its total fallback for those out-of-movie inputs,
so later qualifying turns only advance already placed droplets. Seed the release status for a
rising checkpoint from whether its earlier frames have emitted the release; on this movie the
first such cut is 1736.

The constructed vectors provide their own data for animation windows and drawing IDs. They
do not assume the cartridge's drawing numbers or script layout.
