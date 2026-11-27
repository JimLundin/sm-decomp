# Ceres elevator blackout, decode and replay evidence

Companion to the [blackout contract](screens-blackout.md). Observer and integrator only.

## Coordinates

Use `chain-any-0011-DF45` with movie coordinate equal to cut plus 13,072. The cached
emulator video counter is one ahead. The numbers in the older escape survey's movie column
use that video convention for this interval.

| Row cut | Movie | What the cut proves |
| --- | --- | --- |
| 163 | 13,235 | Prior ordinary gameplay. Use this as the scoped start, with the existing shaft state, save, Health and display. |
| 164 | 13,236 | Reaches the elevator with 60 full waiting passes left. |
| 223 | 13,295 | Wait has one pass left. |
| 224 | 13,296 | Wait ends, fade state begins, full brightness is still published. |
| 225–238 | 13,297–13,310 | Fourteen published steps, brightness 14 through 1. |
| 239 | 13,311 | First overrun. Work has run once and the brightness mirror is forced blank. Actual display holds level 1. Waiting on vblank. |
| 240 | 13,312 | Second overrun. Sprite-only setup and destination selected, five changed battery payload bytes and old checks. Still inside save. |
| 241 | 13,313 | The requested commit finishes. Save checks now match the payload, timer and shaft flags clear, sound output and pending explosion are present. The display publishes forced blank. |
| 242 | 13,314 | First following scheduled explosion opportunity. |

## Owners and source

| Entry | Original sources and producing owners |
| --- | --- |
| ElevatorStopsSamus | [Bank 89](https://patrickjohnston.org/bank/89/) $89:ACC3..AD17 is late room work under $82:8BA8. A rotating shaft and normal play are prerequisites. With signed whole px in the room, x is strictly above 112 and at most 144, y is at least 75 and strictly below 128, and both y velocity words are zero. Cut 163 has x147 and cut 164 has x144. B8 alone emits the reach from the after-motion position. $90:F125..F14F chooses a standing pose respecting facing, initializes its animation and starts the wait. $90:F109..F116 sets the locked pre and post handlers and cancels pending pose changes. Ordinary timer work already ran on the reach cut. |
| PlatformWait | [Bank 82](https://patrickjohnston.org/bank/82/) $82:8367..8387 runs the wait and $8388..83A6 the fade. Both draw the timer first using $80:9F6C..9FD3, without its normal countdown update, then run $82:8B44..8BB8. The room pass includes locked Samus/shot handling, actors, effects, HUD, shaft work, room shake and elapsed play time. $80:8924..894C supplies the shared fade, zero delay and one brightness step here. $89:AD1A..AD5E rotates the plane and $A6:F850..F91F continues beacon and platform during the wait and fade. |
| DepartureHandoff | $82:83A7..8410, $88:829E..82BF and $80:82C5..82D5 wait for an active vertical interval to end, clear DMA/HDMA work and end the shaft's counter interrupts. Set gameplay mode with only objects on the main display and select the Zebes route both for flow and for capture. P2's $81:8000..8084 writes the payload and its checks. After that call returns, install the pending explosion, clear timer status and shaft-stage flags and issue the four requests. The final requested NMI runs $80:959E..95F4. |

The B8 producer owns the geometric edges, including the asymmetric horizontal bounds.
Constructed F4 cases consume its typed event and do not infer a reach from position or
replace the shaft's state machine. The S2/S5 lock holds her centre and starts a matching
standing animation, while the locked room continues to run. This witness stays at (144,75)
and retains 24 energy of 99. The timer finishes the ordinary countdown at 49.14 seconds on
164 and holds that value and its position. Inactivation on 241 clears status only. The 17 timer
sprites at the start of OAM match the imported glyphs and the modeled timer on 79/79 cuts.
The timer is still present in 241's prepared room picture even though its activity has
cleared before the commit. The next scene owns its later removal.

The rotation index and wait at $07E1/$07E3 run once per room pass through 239.
The affine matrix and centre at $78..83 stay carried when the rotating flag clears.
$A6:F851/$F8F1 selects a four-byte tilemap low-lane patch using bit 1 of the incoming
finished count. Its Mode 7 queue at $02D0 has length 9 in $0334 on both overruns.
$A6:F854..F86F selects the six beacon colours from the incoming count and writes
them to the palette shadow. The last room pass on 239 prepares both the platform and
the regular room upload queue. The live VRAM, palette and OAM hold through both
overruns. The queued platform patch publishes four changed low bytes at VRAM words
$060E..0611 on 241. The Mode 7 queue still drains because the interrupt mode mirror
$56 remains 7 even though the main mode mirror is 9. Hold the high bytes and the
unwritten graphics. Keep the normal room batches for the input to the commit.

Use $05B6 for finished work, $09DA..09E1 for elapsed play time, $05E5 for RNG and
$09C2/$09C4 for current and maximum energy. Time and RNG advance through room work
on 239. They hold on continuation at 240 and 241. The two overrun instruction sites
are $80:82CE (waiting) and $81:8066 (inside save). The display mirror at $51 requests
forced blank on 239, but the live INIDISP remains 1 until the requested NMI at 241.
On the plateau the mirror's unused brightness high bits differ from live INIDISP.
Compare forced blank and the low brightness nibble, not those unused mirror bits.
The saved setup mirrors and live per-line IRQ configuration can differ during the
two interrupts. Leave exact raster-phase reconstruction to T1/E3. The settled
publication has Mode 1 and sprites, with forced blank on. Pending OAM and regular
uploads, the full raster history and queued room effects are independent E1/E2/E3 inputs.

## Save and audio joins

The [Bank 81](https://patrickjohnston.org/bank/81/) capture follows destination
selection, before timer retirement or cinematic initialization. Runtime Ceres area
and its current station are the captured location. The stored restart becomes the
Zebes flight. The player still has 24 current energy, capacity 99 and play time
0:00:51.55. Do not substitute the later fully healed or Crateria location. The active
cartridge image and slot carry from the previous save. At revision `e086aa1c7`,
`production::save_files::Input::Write` accepts the live capture and a write stage,
and `FinishWrite` completes the two checks without another capture. The earlier
P2 proof holds the payload before the check copies. Reuse those entries once that
branch is available. Use the independently observed P2 write progress on lag cuts
until T1 can produce it.

$80:8FC1 queues music stop with an 8-interval delay, then $809021/$8090A3/$809125
queue the library 1, 2 and 3 silences, with allowed queue depth 15. Verify the
ordered request intents, and the queue additions at 241 using the previous write
positions. Music's slot value is zero, so the changed delay and write position
establish the request. Prior music and sound state belong to U1. Raw sound port
timing is outside the check. The explosion's own later stop, data and song at
cut 283 belong to the next scene.

At `d45f84b0d`, `played::explosion::State::start` already accepts the completed
blackout with finished count, Health, destination, subtitles, palette and OAM. Its
first blank state is ready at 241, after the interrupted save and the pending room
publication. Hold the palette, OAM, VRAM, affine state and carried battery outside
that scene constructor when the joined scheduler needs them. The existing
`game_time::Clock`, RNG, world-quake and video commit APIs own their parts.
The missing clean scene rule is the one-shot lock, hold/fade and ordered
blank/save/ready report sequence. R2/T1 production owns work dispatch and resumes
it across interrupts. The P2 and timing inputs never determine shaft geometry.

## Reference and witnesses

The independent reference starts at 163. It reconstructs the reach threshold
using the after-motion receipt, countdown decrement using the existing timer
table, room clock and RNG, carried rotation, beacon and platform art using
imported assets, and the complete battery using carried seed profile and maps
with generated play time. It matches 79/79 State cuts versus a held 1/79,
including 79 complete palette and battery comparisons and 79 platform byte
comparisons. The separate full-VRAM step check carries the preceding published
image from 238, projects the final four writes and holds the rest. It covers
239–241 without claiming a full independent E2 room replay. The power-on
integrator still supplies the full preceding room state from production.

The save restore runs from the suspended 240 state with the pending battery
payload and final room write carried. It completes the same cartridge image and
scene reports as continuous advance. Borrowing the overrun and save-completion
boundary is scoped T1 input. The whole-screen display itself, OAM assembly,
ordinary room play, and the rest of the E2 upload sequence remain their
owners' obligations. No cut copies the final blackout record as an input.

Faults dimming at the wait boundary, publishing blank on the first overrun,
dispatching explosion at the payload cut, and rerunning room work on an
interrupt first diverge at cuts 224, 239, 240 and 239 respectively. Ticking the frozen timer first changes the digits and timer picture at 165. Constructed
cases restore all 17 queued timer sprites, retire the timer and publish those same
sprites. Cut 241 completes publication and leaves an explosion pending. Cut 242 is
the first new work opportunity eligible to dispatch it. Faults that clear queued
timer sprites, dispatch before or during publication, or dispatch twice fail the
corresponding constructed witnesses.

## Joined departure

The played run admits the interrupted picture at cut 239, movie 13,311. It
carries room placement, Health, progress, physical video, the queued room
transfers and the cartridge before the payload write. The scroll mirrors
at $B1 through $BF supply the four queued scroll pairs. At the end of the
vertical interval the active indirect HDMA channels reload their first
ROM table entries before the shaft disables them. The reload sets the
table address, indirect address and line counter. The replay compares
every DMA channel on cut 240.

Cut 241 publishes the final room picture and the completed save. The
blackout dispatch on cut 242 opens the station's first work opportunity
with the carried destination, Health, count and picture. The same run
advances the flight and the Landing Site loader. It compares the cartridge
on every cut, including cut 2124, movie 15,196. Cut 2125 requires the live
sprite source beyond the ROM bank.

The preceding room pass on cut 239 prepares the regular upload queue and the
platform patch. A run starting before that pass still needs the locked-room
drawing producer. The after-motion elevator reach on cut 164 remains B8's
input to the separate blackout scope.
