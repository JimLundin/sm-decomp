# The station explosion, decode and replay evidence

Companion to the [behaviour spec](screens-explosion.md). Observer and integrator only.

## Sources

The row is `chain-any-0011-DF45` in the pinned lean cache. Add 13,072 to a row cut to
get its movie frame. The escape route is game state 22h. This round ends with the
handoff to the flight to Zebes and leaves the timer-expiry route
(game state 25h) outside this boundary.

| Behaviour | Original sources and interpretation |
| --- | --- |
| StationPictureArrives | Bank 82 `$82:83A7..840F` owns the elevator blackout, save, destination and initial stop and silences. The blacked-out cut at 240 precedes its final writes. Cut 241 has the request and completed elevator exit. In bank 8B, `$8B:C11B..C263` waits through nine blank intervals, clears the cinematic storage and installs the drawing. `$8B:819B` establishes transparent affine overflow and the sprite sizes. `$8B:C267..C2AF` clears prior effects, builds the sky and starts the lights. `$8B:C2B8..C2E3` sends the scene's own stop, data and song, separate from the elevator's stop. Bank 80 `$80:8FC1..901F` sets the delays. Enabling NMI in the setup services a waiting interval. The first settled cut at 283 publishes two cinematic ticks and increments the finished-frame count twice. |
| StationAwaitsMusic | Bank 8B `$8B:C2E4..C2F0` uses the music queue status from bank 80 `$80:8EF4..8F0B`. Audio service `$80:8F0C..8F9F` runs before the cinematic sample. The last timer is 1 at cut 347 and zero at 348, when the scene enters reveal. The data transfer interrupts the wait between cuts 305 and 333. |
| StationBrightens | Bank 8B `$8B:C2F1..C344` updates the plane before the fade from bank 80 `$80:894D`. Bank 8B `$8B:8532..85CF` computes the shared matrix and integer scroll after scene updates. |
| SpaceDrifts | Bank 8B `$8B:BF22..BF4B`, `$8B:BF76..BFD9`, `$8B:CC3F`, `$8B:CC4F` and `$8B:CC57`. The vortex uses the stationary branch at `$8B:BFA5`. Its pictures alternate every frame. The large and small asteroid fields are placed at 15 and 1, the vortex at 0. |
| StationBursts | Bank 8B `$8B:CE35..CE4A` defines the opening, interval and end of the placement schedule. `$8B:C404..C588` places the groups and chooses their offsets and start delays. `$8B:CCDB..CD38` and `$8B:CE1B..CE34` give the four finite drawing sequences. `$8B:C582..C5A8` drifts all bursts, including those hidden by a delay or blank sample. `$8B:938A..93D8` assigns the first free place, clears fractional position and sets the first timer. `$8B:93EF..943F` runs in descending slot order. Spawner slot 14 is used for the final burst after the last group has been released. |
| StationBreaks | Bank 8B `$8B:C345..C403` tests the old zoom and uses the post-drift scroll for the effect's center. It selects the gunship front and blank lower half and queues the two map transfers. Bank 88 `$88:8AA4..8B7F` creates and runs the window flash, with its imported shape and additive colours. Use a presentation request for this shared effect. Cleanup's gameplay sound path in bank 90 `$90:F4A2..F4CF` exits immediately in the cinematic. No sound request is sent when the large flash starts or ends. |
| ShipDeparts | Bank 8B `$8B:C5CA..C60F` turns and shifts even on the cut that enters the hold. The plane has zero zoom for one cut before the hold starts. |
| SpaceHolds | Bank 8B `$8B:C610..C626` arms the fade after 192 finished frames. |
| SceneBlacksOut | Bank 8B `$8B:C627..C64D` calls bank 80 `$80:8924` and publishes the Zebes setup at blank. The common runner `$8B:A35B..A38E` still updates and draws the sky and advances palette effects on that cut. Flight setup starts on the next scheduled frame. |

## Content and effects

The player ROM supplies bank 95 `$A82F` for the shared affine graphics, bank 96 `$FE69` for
the gunship and station maps, bank 96 `$D10A` for space sprite tiles, and bank 9A `$D200` for
the common sprite tiles installed over them. Bank 8B `$C154..C21E` defines the graphics
destinations. After decompression the input map is split at `$7F:4000` for the gunship
front, `$4300` for its back, `$4600` for the station and `$4C00` for blank. The working
map has the station in the first 1,536 entries, empty space outside it and transparent
overflow. The flash replaces two 768-entry regions. The shared graphics, back map and
current colours carry across to the next scene. Sprite picture references are in bank 8B
`$CC3F..CC5F`, `$CCDB..CD38` and `$CE1B..CE34`, with their drawing data in bank 8C as selected
by bank 8B `$9746..9798`. Use typed picture lists with positive
sample holds. Draws have palettes 4 for the sky and 5 for bursts.

The initial palette is at `$8C:E5E9..E7E8`. Decode the repeating navigation effect from bank
8D `$E1B8` via `$C906` to `$C892`, starting at palette byte offset `$DA`. R4 owns its samples.
Bank 88 `$8B14..8B4D` shows the flash's four presentation phases. It uses the flash's
pre-scaled boundaries at `$9F06` and `$9246`, with the growing window and colour stages
driven by the respective bank 88 pre-instructions. The owner starts at the recorded
station center. At 733 the two effect slots are active and colour math mirrors are set.
At 735 the main screen also has BG2 enabled. Both effect slots are gone by 828. The
layer configuration remains through the handoff at 1,004, with the display blanked.
Derive the window lines, palette and compositing through the shared effect implementation.

## Mapping and replay

| Evidence | Fields and comparison |
| --- | --- |
| Flow and setup | `0998` is game flow. `1F51` progresses through `C11B` pending, `C2E4` music wait, `C2F1` reveal, `C345` station, `C5CA` departure, `C610` hold, `C627` fade and `C699` Zebes pending. `7E:D914` carries the landing destination. |
| Cadence | `05B6` identifies advanced finished frames after setup. `1A51` gives scene age, with value 2 at cut 283. Cuts 241–250 are blank setup waits with advancing `05B6`. Cuts 251–282 are partial setup, with the first settled snapshot at 283. Cuts 306–332 are borrowed audio-transfer lag, with the scene age fixed. |
| Light and pose | `0051` bit 7 is forced blank and its low nibble is brightness. `0725` gives a live fade wait. `198D`, `198F`, `1993.1991`, `1997.1995` are angle, zoom and scroll in whole and fractional parts. `0078..007E` are the four matrix entries, `00B1/00B3` integer scroll, `0080/0082` pivot. `1A49` is hold only in Holding. During station bursts the same location belongs to the middle-group clock. |
| Sprites | `1A5D` plus twice the index gives the picture, zero for hidden. Position uses `1A7D.1ADD` and `1A9D.1AFD` with the same offset. `1B1D` and `1B5D` identify the current sequence and its sample time. Read 14 through 2 in descending draw order and retain abstract group and launch occurrence. Before age 273, slot 14 schedules bursts. Omit it from the draws. Slots 15, 1 and 0 are large asteroids, small asteroids and vortex. |
| Music | `0619..0628` and `0629..0638` contain queued requests and delay timers. `0639/063B` give queue write/read positions. Busy is any live queue timer at the post-service sample. At cut 283 the three appended requests are music stop (0), Ceres music data (FF2D), and the Zebes flight track (7), with delays 8, 8 and 14. The remaining stop is the elevator's. `07F3/07F5` give the delivered data and track identities. |
| Flash and drawing | `0CE2/0CE4` are the flash center. `18B4..` show its active effect channels. `0069` gives the main layers, and `006F/0072` give the two colour math mirrors. `1E7D..1E8B` identifies the current R4 effect. Check the station and gunship map and flash setup through E2 and the window effect service respectively. |

Start the indexed `cache-cut` case at cut 241 with already-completed blackout, save,
initial stop and silences. Use cut 240 for the incoming transition witness in integration.
Mask partial setup writes at cuts 251–282, including page transfers, stage internals and
PPU mirrors, whose writers are T1, E2, E4 and R4. Compare carry and blanking throughout.
At 283 compare the complete stage, installed drawing references, palette replacement,
and ordered music intents. During audio stall cuts 306–332 hold F4 state and compare its
plane and sprite draws normally. The transfer and the receiver's progress belong to U1
and T1. Sample the music busy reply as seen by F4 separately from the produced requests.

Through 1,004 compare stage, brightness, active fade/hold, plane origin, pose, matrix,
integer scroll, sky position and pictures, and every live burst's place, position and
current picture. At 733 compare source and destination of the map swaps, the flash request, style and screen center. Confirm complete event presence at every cut,
including the absence of repeated requests while E3 still has active channels. At 1,004 compare the published Zebes setup event and the retained
music, palette, health and destination. At 1,005 the incoming flight starts replacing
the drawing and clearing explosion pictures. Scope full encoded OAM to E1, raw
VRAM transfers and their lag to E2 and T1, palette and scanline-window bytes to their
presentation services. The observer reference checks the scene side of those outputs
and the E4 mirrors. The reference burst model steps drawings by group and
occurrence and matches their position, signed age, sample and presented order. The production witness compares the complete palette, the visible OAM entries and
settled transfer bytes. The flash window and composed pixels require the line-window owner.

| Cut | Vector |
| --- | --- |
| 284 / 348 | R-music-waits / R-music-ready |
| 349 / 350 / 378 | R-reveal-wait / R-first-light / R-full-light |
| 285 | R-sky |
| 437 / 452 / 518 / 530 / 581 / 582 / 763 | R-first-group / R-first-picture / R-middle-start / R-middle-next / R-last-group / R-drift / R-burst-end |
| 379 / 732 / 733 | R-station-drifts / R-zoom-limit / R-flash |
| 734 / 781 / 782 | R-turn-wrap / R-zero-zoom / R-hold-start |
| 783 / 974 / 975 / 976 / 1,004 | R-hold-ticks / R-fade-start / R-fade-wait / R-fade-step / R-handoff |

The local reference in `$TEMP/collab/explosion-evidence` matches 24 recorded and
10 constructed vectors, including an adjusted two-frame hold. Using the reference hold
instead makes that vector fail. The reference matches 722 of 722 settled cuts
(695 advancing, 27 borrowed) on the scene fields, pose, shared matrix and scroll, sky,
burst draws and the flash center and outgoing flow edges. Holding the first settled
picture matches 1 of 722 using the same projection. The preceding 42 cuts hold forced
blank and explosion flow. Independent witnesses compare the navigation light and flash activity
on all 722 settled cuts and the retained palette at Zebes setup. Both map regions change
at 733. The setup changes the affine map, affine graphics and sprite graphics. The queued
requests and their delays match at 283.
