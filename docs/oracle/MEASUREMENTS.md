# Oracle measurements for the architecture lane

Measured on the oracle harness (`crates/oracle`, LakeSnes `9db90b8` with fix
0001) on 2026-11-05, for the decisions on door and loading timing, the line
program and sound. Ids, frames, counts, lines, dots and addresses only.

Two sources, marked on every result:

- **movie**: the any% movie (`movies/any.inputs`) from power-on, frames
  0-8499. The game state, room and Samus position match the lsnes lag log
  through frame 8566, so these frames are valid. The game logic is one
  frame behind lsnes from frame 8314, which becomes a desync in Ceres.
- **seed**: corpus rows from their lsnes room captures. A capture has no
  SPC700 or DSP state, so the harness resumes the sound driver at its idle
  loop (docs/oracle/HARNESS.md). In some rows the driver then does not answer
  the CPU's sound requests (see "Sound" below), which stalls door transitions
  and music uploads. Loop timing is the hardware's; anything that waits on
  the APU is not.

Lines are the PPU's V counter (0-261; 225 is the first vblank line) and dots
are the master-clock H position (0-1363). Commands: `oracle measure ROM LOCK
DATA ID...` and `oracle measure-movie ROM INPUTS FRAMES`.

## (a) Busy spans, decompression, uploads, door transitions

A busy frame is one whose NMI interrupted the main thread outside a wait site
(`game::WAIT_SITES`): a lag frame.

**Movie, frames 0-8499:** 432 busy frames in 31 spans.

| Frames | Length | Where the NMIs landed (address: count, most common) |
| --- | --- | --- |
| 168-256 | 89 | $80:8096 20, $80:8092 15, $80:8103 9, $80:808E 8 (APU upload loop) |
| 258-270 | 13 | $80:8528 3, $81:811B, $81:8114, $81:810F |
| 272-302 | 31 | $80:B23B 2, $80:B22D 2, $80:B1AA 2, $80:B193 2 (decompression) |
| 304-305 | 2 | $8B:9C1C 2 |
| 316-353 | 38 | $80:808E 6, $80:8096 4, $80:8092 4, $80:8091 4 (APU upload) |
| 394, 397-399, 432 | 1, 3, 1 | $81:8E43, $81:811B, $81:8116, $81:810F |
| 466-471 | 6 | $80:B234, $80:B1D7, $80:B1D1, $80:B185 (decompression) |
| 569-621 | 53 | $80:B193 3, $80:B188 3, $80:B237 2, $80:B22F 2 (decompression) |
| 639-672 | 34 | $80:8096 7, $80:8091 5, $80:809F 4, $80:808E 4 (APU upload) |
| 980-1003 | 24 | APU upload ($80:8092, $80:8096, $80:8091, $80:808E) |
| 1307-1341 | 35 | APU upload ($80:8103, $80:8096, $80:808E, $80:8091) |
| 2219-6415 | 13 single frames | $8B:8CF1-$8B:8D14, $8B:A886, $8D:C538/$8D:C53B, $80:8A3A, $80:898A (intro cinematics) |
| 7513-7543 | 31 | $80:B1AA 6, $8B:BD25 2, $80:B18E 2, $80:B16F 2 (decompression) |
| 7553-7580 | 28 | APU upload ($80:809F, $80:8096, $80:8092, $80:808E) |
| 8315 | 1 | $81:8056 |
| 8438-8465 | 28 | APU upload, room DF45 (Ceres), door pointer $AB58 |

**Movie decompression calls** (`$80:B119` to RAM, `$80:B271` to VRAM; output
size and command mix parsed from the compressed stream; duration from entry
to RTL):

| Frame, line | Entry | Output bytes | Commands (0-6: count) | Lines |
| --- | --- | --- | --- | --- |
| 5, 259 | B119 | 16384 | 0:939 1:610 2:787 3:2 4:113 5:6 6:166 | 3406 |
| 272, 87 | B119 | 16384 | 0:968 1:1317 2:120 3:57 4:374 6:500 | 4363 |
| 288, 258 | B119 | 4096 | 0:69 1:101 3:49 4:12 6:7 | 178 |
| 289, 175 | B119 | 16384 | 0:939 1:610 2:787 3:2 4:113 5:6 6:166 | 3414 |
| 302, 183 | B119 | 1024 | 0:91 1:57 2:2 3:8 4:4 6:57 | 261 |
| 466-470 | B119 | 5 x 2048 | 0:39-63 1:4-7 2:26-43 4:5-15 6:13-39 | 254-299 each |
| 569, 195 | B119 | 32768 | 0:1784 1:768 2:1464 3:5 4:453 5:92 6:212 | 7410 |
| 598, 8 | B119 | 2304 | 0:187 1:72 2:129 4:26 6:86 | 624 |
| 600, 109 | B119 | 2048 | 0:30 2:26 6:5 | 246 |
| 601, 93 | B119 | 8192 | 0:145 1:4 2:164 4:3 6:76 | 1457 |
| 606, 240 | B119 | 9216 | 0:378 1:232 2:128 3:5 4:49 5:3 6:289 | 1914 |
| 614, 59 | B119 | 2048 | 0:29 2:10 4:4 6:26 | 233 |
| 616, 166 | B119 | 4608 | 0:396 1:27 2:293 4:134 5:1 6:175 | 1653 |
| 7513, 7 | B119 | 16384 | 0:1367 1:1563 2:67 3:73 4:483 6:452 | 4337 |
| 7529, 153 | B119 | 4096 | 0:7 1:26 3:32 | 309 |
| 7530, 200 | B119 | 16384 | 0:505 1:330 2:162 3:3 4:91 5:32 6:78 | 2513 |
| 8318, 199 | B271 | 12288 | 0:677 1:343 2:464 3:5 4:147 5:13 6:219 | 4108 |
| 8334, 115 | B271 | 32768 | 0:665 1:4 2:216 4:1348 6:1003 | 17516 |
| 8401, 77 | B119 | 256 | 0:11 1:6 6:7 | 61 |
| 8473, 8 | B119 | 3842 | 0:139 1:77 2:43 4:37 6:83 | 1017 |
| 8476, 105 | B119 | 8192 | 0:109 1:3 2:93 4:7 6:33 | 1197 |

Decompression to RAM costs about 210-270 lines per KiB of output (a 16 KiB
block takes 13-17 frames). To VRAM it costs 340-550 lines per KiB, the most
where dictionary copies (command 4) dominate.

**APU uploads (movie):** the busy spans marked "APU upload" are the
`$80:8059` send loop (`$80:808A`-`$80:8106`). The boot upload, frames 168-353,
is about 1,218 CPU port writes per frame for 10 frames before its last span.

**Door transitions (seed):** every door row measured stalls in its door
transition because the resumed sound driver does not answer: door-transition
(frames 117-372, room E06B), elevator-traversal (71-859, 97B5),
power-bomb-red-tower-refill-door (155-412, A253), chain-0006-E0B5 (1610-1776,
E0B5). Their door-transition lengths are therefore not hardware lengths;
they wait for harness-native seeds. No door transition happens in movie
frames 0-8499.

**DMA bytes under forced blank:** not yet measured correctly. The harness
counts DMA register writes when INIDISP at the cut has forced blank, which
misses transfers inside a blanked interval. It needs a per-write forced-blank
check; open.

## (b) Busy frames per row (coverage)

**Seed:** from `oracle corpus` over the 601 rows that ran, 176 rows have
busy frames, 2,231 in total. The rows with the most:

| Row | Busy frames | Ticks | Video frames |
| --- | ---: | ---: | ---: |
| options-menu-tour (from boot) | 283 | 2310 | 2614 |
| chain-any-0097-B32E | 129 | 2672 | 2801 |
| area-maridia | 126 | 300 | 426 |
| chain-0170-CEFB | 123 | 190 | 313 |
| chain-0232-D5A7 | 64 | 632 | 696 |
| chain-0118-B32E, boss-ridley | 50 each | 2209, 2100 | 2259, 2150 |
| boss-phantoon, phantoon-exit-spawn-parameter, chain-any-0165-CD13 | 47 each | 1989-2215 | 2036-2262 |
| chain-0190-CD13 | 41 | 2281 | 2322 |
| terrain-lower-norfair-hand, enemy-shaktool, chain-0238-D8C5 | 40 each | 1391-2479 | 1431-2519 |

**Movie:** frames 0-8499 have 432 busy frames, all at boot, the title,
the intro and the Ceres load.

## (c) Line timing (tier a)

**HDMA object handler** (`$88:84B9`, its RTL at `$88:8516` or `$88:851B`),
the line at which it returned:

| Source | Frames | Min | Median | Max |
| --- | --- | --- | --- | --- |
| seed, area-brinstar | 250 | 241 | 247 | 248 |
| seed, item-morph-ball | 802 of 1200 | 6 | 246 | 249 |
| seed, door-transition | 373 | 244 | 244 | 248 |
| seed, elevator-traversal | 860 | 240 | 245 | 250 |
| seed, chain-0006-E0B5 | 1777 | 246 | 247 | 252 |
| movie, frames 0-8499 | 7741 | 61 | 251 | 258 |

It runs in vblank, right after the NMI returns to the main loop. The handler
finishes inside vblank on every frame except a few frames during loads, where
it ends early in the next display.

**IRQ handler register writes** (line, register: count, dot range). The two
HUD split IRQs:

| Source | Line | Register writes (dot range) |
| --- | --- | --- |
| seed, area-brinstar (250 frames) | 0 | $2109 (1044-1084), $2130 (1068-1108), $2131 (1092-1208), $212C (1138-1244) |
| seed, area-brinstar | 31 | $2130 (1052-1100), $2131 (1096-1214), $2109 (1142-1258), $212C (1186-1302) |
| seed, item-morph-ball (1200 frames) | 0 | $2109 (1044-1098), $2130 (1068-1196), $2131 (1092-1230), $212C (1130-1266) |
| seed, item-morph-ball | 31 | $2130 (1052-1106), $2131 (1096-1234), $2109 (1142-1278), $212C (1186-1322) |
| movie (18 frames in Ceres) | 0 | $2109 (446-1068), $212C (530-1152), $2130 (470-1092), $2131 (494-1116) |
| movie | 31 | $2130 (1056-1072), $2131 (1100-1116), $2109 (1144-1160), $212C (1188-1204) |

Every write lands in the last third of its line (H-blank begins at dot
1096), one write per register per IRQ. The line-0 IRQ's writes spread
further in the movie's Ceres frames.

## (d) Sound: port traffic and the driver's echo latency

Echo latency: from a CPU write of a value to port N until the SPC700 writes
the same value to port N, in SPC700 cycles (1,024,000 per second; about 17,060
per frame).

| Source | CPU writes (ports 0-3) | SPC700 writes | Echo latency, cycles (median, range) |
| --- | --- | --- | --- |
| movie, frames 0-8499 | 164844, 164978, 168, 1487 | 170608, 8407, 8400, 8400 | port 0: 21 (12-70224) over 164837 echoes (uploads); ports 1-3: 8354-8690 (50-85713) over 139, 125, 801 |
| seed, item-morph-ball | 3, 715, 224, 741 | 1093, 1250, 1250, 1250 | ports 1-3: 8674-8844 (1635-13945) over 25-26 |
| seed, chain-0006-E0B5 | 2, 1773, 1744, 1712 | 1851, 1850, 1850, 1850 | ports 1-3: 3876-4044 (250-16287) over 18 |
| seed, area-brinstar | 0, 0, 0, 0 | 227, 260, 260, 260 | no requests |

The driver answers a sound request in about half a frame (median about 8,500
cycles), at worst within one frame; upload bytes echo in about 21 cycles.
music-fir-selector10 has no faithful lock entry, so its row was not run;
item-morph-ball and chain-0006-E0B5 stand in.

In the stalled seeded rows (credits, Mother Brain, death, save, the door
rows), the CPU writes the same request every frame and the resumed driver never
echoes it. That is the migration seed's missing SPC state, not the driver.

## (e) lsnes: the any% movie, resuming, and the stalled rows

**How far lsnes runs the any% movie.** With the cut callback of `97ae09a2d`,
lsnes ran all 19000 frames from power-on, without saves, at 32 frames/s, and
logged no errors. From native seed 16 (frame 17829) it ran to 18300, past 18068,
with no errors either. With saves (`--native-seeds`) from seed 16, it cut seeds
17, 18 and 19 (frames 18109, 19514 and 19809), logged 1180 `nil value`
callback errors and hung before 21829. So the exit at 18068 is lsnes failing
inside its callback dispatcher, which the saves make more likely; the game is
not the cause (docs/oracle/HARNESS.md, "lsnes crashes").

**Resumed from seed 16:** the lag log matches in every column except
`lagged`, frames 17829-17900.

**The 27 stalled rows, on lsnes native seeds: all 27 complete.** They stalled
on LakeSnes (31 lines, credits rows sharing seeds), where the migration seeds
have no SPC state. Cut from the movies on lsnes (`$TEMP/ohx/cut_seeds.sh`) at
these captures, the seeds carry the sound CPU's state too:

| Capture | Frame | Rows |
| --- | --- | --- |
| 100-credits:336 | 222657 | 14 (credits-*) |
| 100-input-phase:27 | 24079 | 1 |
| 100-input-phase:104 | 72927 | 1 |
| 100-input-phase:305 | 196264 | 1 |
| 100-input-phase:324 | 209945 | 2 |
| 100-mb-phases:324, 325, 329 | 209945, 210553, 218608 | 4 |
| 100-boundary:259 | 169581 | 1 |
| low-input-phase:129 | 100731 | 2 |
| any-boundary:110, 190 | 73446, 116359 | 3 |
| scenario-one-energy-crocomire:142 | 99412 (100-input-phase:142, edited) | 2 |

`corpus --oracle lsnes --native-seeds DIR --only ROW...` at `9db02d2e3`, 13
processes (two re-run after a start-up failure, see below), 2026-11-05.
Every row ran to the lock's tick count; none stalled. Streams equal to the
lock: sram on 25 rows; wram, vram, cgram, oam and ppu on none, as on the other
native rows (the old lock came from the patched oracle). video and apu are
redefined, so not compared.

| Row | Ticks | Frames | Overrun | Without NMI | Equal |
| --- | --- | --- | --- | --- | --- |
| boss-draygon-final-whips | 4480 | 4568 | 75 | 13 | sram |
| boss-draygon-later-swoop | 2650 | 2736 | 73 | 13 | sram |
| boss-draygon-tail-whip | 3500 | 3586 | 73 | 13 | sram |
| boss-mother-brain | 1380 | 1542 | 132 | 30 | sram |
| chain-0324-DD58 | 10134 | 10630 | 123 | 373 | sram |
| chain-any-0190-DD58 | 10175 | 10428 | 120 | 133 | sram |
| credits | 11900 | 11967 | 55 | 12 | sram |
| credits-completed-spawn | 2902 | 2903 | 0 | 1 | sram |
| credits-ending | 15900 | 15967 | 55 | 12 | none |
| credits-explosion-afterglow-slot | 1652 | 1653 | 0 | 1 | sram |
| credits-explosion-slot | 1508 | 1509 | 0 | 1 | sram |
| credits-postroll-wave | 11790 | 11857 | 55 | 12 | sram |
| credits-silhouette-spawn | 1500 | 1501 | 0 | 1 | sram |
| credits-time-hours-spawn | 3170 | 3171 | 0 | 1 | sram |
| credits-time-hours-units-spawn | 3178 | 3179 | 0 | 1 | sram |
| credits-time-minutes-spawn | 3194 | 3195 | 0 | 1 | sram |
| credits-time-minutes-units-spawn | 3202 | 3203 | 0 | 1 | sram |
| credits-time-separator-spawn | 3186 | 3187 | 0 | 1 | sram |
| credits-time-spawn | 3084 | 3085 | 0 | 1 | sram |
| credits-zebes-spawn | 1112 | 1113 | 0 | 1 | sram |
| death | 1800 | 2137 | 219 | 118 | sram |
| file-copy-clear | 3612 | 3954 | 223 | 119 | none |
| mother-brain-beam-spawn-parameters | 4560 | 4813 | 120 | 133 | sram |
| mother-brain-escape-door | 1500 | 1684 | 132 | 52 | sram |
| mother-brain-fake-death | 1360 | 1397 | 35 | 2 | sram |
| mother-brain-glass | 760 | 809 | 0 | 49 | sram |
| mother-brain-purple-breath-parameter | 2756 | 2968 | 2 | 210 | sram |

- `credits-ending` differs in sram too: the ending saves the game, so its sram
  is written during the row.
- `death`: Samus dies. `--watch-words 0998,09C2,09D6` (game state, energy,
  reserve energy): energy 1 and reserve 0 at frame 0; at frame 277 energy
  becomes 0 and the game state `$0998` goes from 8 (main gameplay) to `0x13`
  (death sequence), then `0x14` to `0x19`, and `0x1A` (game over) at 604. The
  row's movie words then leave the game over menu: state 0 at 746, the title
  (1), the file select (4 at 884, 2, 5), loading (6, 7), and main gameplay (8)
  again at frame 1771 with energy 99.
- `file-copy-clear` completes its 3612 ticks but does not do what it is for.
  It follows `death` frame for frame to frame 1771, back in gameplay. Its
  authored part starts after the death row's 1,800 movie words and expects
  the game over menu (tools/authored_inputs.py); here the game is in
  gameplay, so its presses pause the game (state `0x0C` to `0x0F` from frame
  2462), and no copy or clear happens. Hence sram differs. The recipe was
  timed on the old patched oracle, where the game was presumably still at the
  game over menu at word 1,800; on lsnes the movie words already pass it. The
  recipe needs re-timing against the unpatched game (prefix shorter than 604
  frames, or start it from the game over state).
- Speed: about 18 to 23 frames/s per process with 13 at once on 14 cores
  (about 260 in total), 40 frames/s for one or two at once.
- Start-up race: two of 13 lsnes processes started in the same second died
  with `FATAL: Configuration directory '...\AppData\Roaming/lsnes' is not
  writable` and then did not exit, so their corpus processes waited at step 0.
  Killed by PID, the harness reported `lsnes exited at step 0` as designed,
  but the panic aborts the whole process (non-unwinding), losing its other
  rows. Started a few seconds apart, the re-runs completed. Fixed since: each
  lsnes has its own configuration directory, a silent lsnes is killed after
  60 s, and a failed lsnes fails only its row (14 processes started within
  0.5 s all completed; docs/handoff/oracle-harness.md).

**Native-seeded rows that already run on lsnes:** chain-any-0001-DF45,
chain-any-0006-E0B5 and chain-any-0012-91F8, 3220 ticks in all, at 39 frames/s.
sram matches; wram, vram, cgram, oam and ppu differ, as they do under LakeSnes.
These ran with an interim script that wrapped the cut callback in `xpcall`.

## (f) Cross-oracle report: LakeSnes against lsnes, any%, frames 0-4800

`oracle cross ROM INPUTS --frames A..B [--ranges] [--ranges-at A..B]` steps
LakeSnes (left, `9db90b8` with fix 0001) and lsnes (right, rr2-beta23) from
power-on with the same pads, and prints per step `equal` or each differing
part with its byte count (fields for the cut, frame, pad and CPU). A memory
that differs is also compared with the other backend's previous step: when
that count is under half the unshifted one, the step is a one-frame lag
(`right a frame behind`), otherwise `content`. The video frame counter is
compared against its offset at step 0 (lsnes counts from 1, LakeSnes from 0).
4801 steps take 170 s (lsnes, all memories, is the pace).

**PC at the cut: settled.** Both backends record the same interrupted PC
whenever their NMIs interrupt the same instruction. The CPU's PC after the RTI
was one past it on lsnes: `$80:8344` and `$80:8347`, against LakeSnes's
`$80:8343` and `$80:8346`. The ROM at `$80:8343` is `AD B4 05` (`LDA $05B4`)
and at `$80:8346` `D0 FB` (`BNE $8343`), so `$8344` and `$8347` lie inside
instructions: the stacked return address (LakeSnes's value) is the true
interrupted instruction. lsnes now records it (`2da4d4ae6`). This changes
bytes 34-35 of every lsnes record with a returned NMI, so ladders and native
seed records cut before differ there from records cut after.

**First difference per part** (frames are 0-based steps):

| Part | First | Steps differing (of 4801) | Nature |
| --- | --- | --- | --- |
| sram | 0 | 4800 | Power-on contents: LakeSnes 0x00, lsnes 0xFF. From about 250 the game's writes leave 3292 bytes different (`$0000-$000F`, `$1324-$1FDF`, `$1FF0-$1FFF`). Not behaviour. |
| pad | 0 | 5 | Meaning: LakeSnes records the word auto-joypad latched (0 before the game enables it), lsnes the pad it was given. |
| cpu | 0 | 2588 | Steps 0-20 have no NMI and are cut at each emulator's own frame boundary, at different instructions. Afterwards the CPU differs only where the cut does. |
| wram | 4 | 4684 | Step 4: the boot clear loop is cut at a different index (no NMI yet). Steady state 1-21 bytes: `$0641` (the APU upload's acknowledgement polls, sound timing) and stale bytes in the stack page `$1Fxx`. |
| cut | 21 | 2536 | Timing inside a frame: the NMI lands on the other instruction of the two-instruction wait loop (`$8343` against `$8346`), on about half the steps. Same kind, same frame. |
| frame | 303 | 2 | Sampling: LakeSnes reads its counter at the cut; an NMI that ends after the next frame began counts one more. |
| vram | 303 | 699 | One frame: lsnes's NMI outlasts its frame (`nmi-running`, PC `$80:8CC1`), so its uploads land a step later. |
| joypad-reads | 307 | 24 | The same: an lsnes lag frame where LakeSnes polled. |
| cgram | 307 | 221 | Right a frame behind: the title palette arrives on lsnes one step later. |
| oam | 480 | 1519 | Right a frame behind (2 bytes at the first). |

**The NMI that outlasts vblank (from 303, 396, 472, 1290, 1342).** lsnes cuts
at its frame boundary with the handler still running (`nmi-running`, at
`$80:8CC1`); LakeSnes runs on to the RTI and records a requested NMI. The next
step realigns: wram is back to 5 bytes two steps later at 398. This is where
the lag log stops matching (it matched through 395). It is a cut-definition
difference between the backends, not a different game.

**The first lasting lag: 1342, timing, sound.** Steps 1307-1341 are the APU
upload (`$80:8070-$80:8104`, overruns on both): lsnes is about 7 bytes ahead
in X but takes one frame longer to finish it, and at 1342 LakeSnes is back at
the wait loop while lsnes's NMI is still running. From there the game logic
on lsnes is one frame behind LakeSnes whenever the frame's work moves bytes:
at 3100 the frame counter `$05B6` is 0x23 against 0x22 and the NMI counter
`$05B8` 0x1C against 0x1B, and right at step k equals left at k-1 but for a
byte. Before 1342 the game logic agrees; the memories differ only in the
bytes above.

**What this means for the lock (D25).** No content difference in the game's
own state was found through 4800. Every difference is one of: power-on SRAM,
the cut's definition (no NMI; NMI outlasting vblank; frame counter sampling;
the pad's meaning), sub-instruction NMI timing in the wait loop (`cut`,
`cpu`, the stack page), and sound timing (`$0641`, then one frame from 1342).
So LakeSnes's memories are lsnes's only up to these, and only up to 1341
without a shift.

## (g) MesenCE as a third vote: frame 1342 and the later lag

MesenCE is the maintained fork of Mesen 2 and GPL-3, so we run its released
binary as a separate process and never link or copy its code
([ACCURACY](ACCURACY.md), "What would settle 1342").

**Binary.** MesenCE 2.2.1 (tag `2.2.1`, commit
`20ba206cef5ba207c21203176d02cb9f43dda9fb`, released 2026-06-05),
`https://github.com/nesdev-org/MesenCE/releases/download/2.2.1/Mesen_2.2.1_Windows.zip`,
zip sha256 `c19e10b3b3865eb0e86e4d01bf09a5027a4452081c04b5bd5b6cbbf76e346fd6`,
`Mesen.exe` sha256 `8ef403d6b9af32075416193914d7993ce8480b347fe1f20a0cfd9815374933e7`.
Unpacked in `%LOCALAPPDATA%/sm-decomp-tools/mesence-2.2.1/`; it runs from a
copy in `mesence-2.2.1-run/` beside a `settings.json` of `{}`, which makes it
portable (without one it opens its setup window).

**Command.** [`tools/oracle/mesen_lag_log.lua`](../../tools/oracle/mesen_lag_log.lua)
feeds the `.inputs` pads from `inputPolled` (pad k to the k-th poll) and
writes one row per frame at `startFrame` (line 0) in the column layout of
`tools/lsnes_lag_log.lua`, plus `$0641`, the line of the frame's last `$05B8`
write and the line the main loop entered its NMI wait (`$80:8338`):

```
rm -rf mesence-2.2.1-run/Saves
SM_MESEN_INPUTS=movies/any.inputs SM_MESEN_LOG=out.csv SM_MESEN_FRAMES=9001 \
  mesence-2.2.1-run/Mesen.exe --testRunner --timeout=150 --doNotSaveSettings \
  --debug.scriptWindow.allowIoOsAccess=true --snes.port1.type=SnesController \
  --snes.ramPowerOnState=AllZeros --snes.spcClockSpeedAdjustment=40 \
  tools/oracle/mesen_lag_log.lua rom/sm.smc
```

9001 frames take 76 s (about 120 frames/s; lsnes is at 29). With `Saves/`
cleared each run, two runs give the same file. Mesen keeps the cartridge's
SRAM in `Saves/sm.srm` between runs, and a kept save sends the boot through
other SRAM checks in bank `$81` from frame 268: clear it every time.

We compared the rows with the lsnes lag log of the movie
(`any-boundary/lag.csv` from the seeds bundle) and, through
`oracle movie ROM INPUTS --frames 9001 --oracle lakesnes --lag-log out.csv`,
with LakeSnes (`9db90b8`, fix 0001). Column: `$05B6`/`$05B8`/lag flag after
the step. Line: Mesen's line of the `$05B8` write (`-1`: no write that frame).

| Step | lsnes | LakeSnes | Mesen +40 | Mesen +41 | Line (+40/+41) | Agrees with |
| --- | --- | --- | --- | --- | --- | --- |
| 303 | 1/46/1 | 1/46/1 | 1/46/1 | 1/46/1 | -1/-1 | all |
| 396 | 51/139/1 | 52/140/0 | 52/140/0 | 52/140/0 | 249/249 | LakeSnes |
| 400 | 52/143/1 | 53/144/0 | 53/144/0 | 53/144/0 | 249/249 | LakeSnes |
| 472 | 117/215/1 | 118/216/0 | 118/216/0 | 118/216/0 | 249/249 | LakeSnes |
| 1290 | 824/1033/1 | 825/1034/0 | 825/1034/0 | 825/1034/0 | 249/249 | LakeSnes |
| 1341 | 841/1085/1 | 841/1085/1 | 841/1085/1 | 841/1085/1 | 225/225 | all |
| 1342 | 841/1085/1 | 842/1086/0 | 842/1086/0 | 842/1086/0 | 249/249 | LakeSnes |
| 8465 | 7779/8125/0 | (as +40) | 7778/8125/1 | 7779/8125/0 | 225/236 | +41 with lsnes |
| 8469 | 7782/8128/1 | (as +40) | 7782/8129/0 | 7782/8128/1 | 236/-1 | +41 with lsnes |

Through frame 9000, LakeSnes and Mesen +40 agree on all three words but at
8421 (`$05B8` one less on LakeSnes); the other rows the harness prints are
game-state and position samples a step apart at transitions. +40 and +41
agree with each other through 8464, `$0641` aside.

**1342 is lsnes's cut, not a lag.** At every step from 21 to 9000, Mesen +41
equals lsnes when its `$05B8` write is on line 240 or earlier (2005 steps), and
is one ahead in both counters when the write is on line 241 or later (6975
steps). There is no exception. So lsnes records the frame at line 241, before a
long NMI's counter writes and joypad read (its lag flag is set on those
steps); LakeSnes and Mesen record after the RTI. Both counters move together,
so no NMI is lost or gained: the "one frame behind for good" from 1342 is
the NMI after the boot upload running to line 249 on every step, which
straddles lsnes's cut, on all three emulators. The upload itself ends with
room to spare: Mesen's main loop reaches its wait at line 183 (+40) or 175
(+41) of step 1342, 42 lines before vblank, so the SPC rate does not decide it.

**The real lasting lag is at 8465, and it is the SPC clock.** At 8465 Mesen
+40 has a lag frame that lsnes and +41 don't. +40, like LakeSnes, then
desyncs from the movie: Samus's X differs from step 8643 (LakeSnes: 8567).
+41 stays in sync with lsnes through 9000: two single-step samples at room
transitions (8506, 8788) differ beyond a one-step shift. lsnes's 32040.5 Hz
lies between +40 (32040) and +41 (32041), and here the movie needs the
upper side.

**Verdict, by the rule in ACCURACY.** At 1342 MesenCE agrees with LakeSnes,
so we do not need a LakeSnes fix 0002 for port sync: the difference is lsnes's,
and it is in its cut, not in bsnes v085's game timing. We record it as
`LEGACY(oracle)`: lsnes rows are cut at line 241, and a NMI whose writes
pass that line shows one step later. A comparison that cuts at the RTI, or
at a fixed line after vblank, removes it. The lag that persists, at 8465,
goes with the SPC rate. The candidate for LakeSnes is a rate closer to
32040.5 Hz (a revision of fix 0001), not a port latch. Steps 3 and 4 of
ACCURACY's list (the rate sweep and ares) are still open. The threshold
between 32040 and 32041 Hz at 8465 already shows that lag there depends on
the console's clock.

## (h) LakeSnes at bsnes v085's SPC rate: fix 0002

**Fix.** `crates/oracle/native/fixes/0002-apu-clock-bsnes-v085-exact.patch`,
applied after 0001. bsnes v085 (lsnes rr2) clocks the SMP at 24 607 104 Hz
(32040.5 · 768) against a 21 477 272 Hz CPU, so the SPC gains
32040.5 · 32 = 1 025 296 cycles per 21 477 272 master clocks. LakeSnes's
`double apuCatchupCycles` becomes `int64_t apuCatchupDebt`, the SPC cycles
owed scaled by the master rate: each 2-clock step adds 2 · 1 025 296, and a
catch-up runs `debt / 21 477 272` cycles and subtracts what ran times
21 477 272. The rate is exact and does not drift. PAL keeps upstream's
32040 Hz against 1364 · 312 · 50, also as an exact ratio. The save state still
holds 8 bytes there (`sh_handleLongLongs`). A separate 0002, not a revised
0001: records name their fixes in `OracleId::fixes`, so records cut at
32040 Hz (`0001` alone) keep an identity that still says what they ran.

**Command.** `oracle movie ROM movies/any.inputs --frames 36000 --no-picture
--oracle lakesnes --lag-log any-boundary/lag.csv --positions-only` (166 s,
217 frames/s), and a per-step dump of the seven lag-log words from the same
run to classify each step against lsnes's rows k-1, k and k+1.

**Boot: unchanged.** The full lag log (all seven columns) matches exactly
through step 395, as with 0001 alone; the first rows that differ are 396,
397, 400, 472-476, the NMI-outlasts-vblank cut of (f), the same rows as
before. The boot upload around step 352 is unaffected.

**Counters, steps 21-17999.** `$05B6`/`$05B8` equal lsnes on 5955 steps and
are one NMI ahead on 12015 (lsnes's line-241 cut of (g)). Nine steps are
neither: 8421 and 15235 (`$05B8` one less, as before), and 17967-17988, a
door transition (below). At 8465 and 8469 LakeSnes now has lsnes's lag flag
(7779/8125/0 and 7782/8128/1), as Mesen +41 has, where 0001 had +40's.

**State, room and position.** They match lsnes through step 18107, but for
single steps. Every step in 1-18107 equals lsnes's row k, or k-1 (the cut a
frame apart at a transition), except 12 door-transition samples (8905, 9187,
9532, 9732, 9796, 10052, 11894, 12353, 12684, 13027, 17334, 17398), where X
is one pixel short of lsnes's row k and equal to it at k+1: the cut catches
the door's position step at another point. So 8314 (0001: one frame behind
from there) is now one sampled step, and 8567 (0001: the Ceres desync)
passes.

| Steps | LakeSnes against lsnes |
| --- | --- |
| 0-395 | exact, all seven columns |
| 396-17984 | state, room, position equal (single-step samples above) |
| 17985 | `$05B6` one behind with `$05B8` equal: LakeSnes has one more lag frame, in the door transition from room `97B5` into `9E9F` |
| 18108-18586 | state, room, position equal to lsnes's previous step: one game frame behind |
| 18587 | Samus's X differs beyond a shift; from then LakeSnes stays in room `9E9F` (20000, 30000, 35999) while the movie goes on |

**Mesen +41 to 18300**, same Lua and settings as (g): it leaves the movie at
16065 (20 consecutive steps differing beyond a one-step shift), before
LakeSnes does, so it does not vote on 17985.

**Reading.** At bsnes v085's exact rate LakeSnes follows the any% movie
through Ceres, the escape and Crateria to the door out of room `97B5`, 2.1×
as far as at 32040 Hz (8566). The lasting lag at 17985 is again a lag frame
at the end of a door transition, the kind that rides on the CPU-SPC
handshake. The rate is now lsnes's, so what is left is how the communication
is modelled: LakeSnes catches the SPC up in whole opcodes at each port
access, bsnes v085 syncs its cothreads on every access (ACCURACY, section 3).
Not tried here: a sub-opcode catch-up, or the rate sweep around 32040.5 on
LakeSnes.

## (i) lsnes cuts at the NMI's RTI also when the NMI outlasts its frame

(g) found lsnes's record cut at its frame boundary (line 241) whenever SM's
NMI was still running there, before the NMI's counter writes and joypad
read. The cut is now always the NMI handler's RTI (`0:9601`), as D20
asks; a frame without an NMI keeps the boundary cut (`9777b468a`, lane
`lane/oracle-lsnes-rti`, on `794ac546d`, LakeSnes with fixes 0001 and 0002).

**Why the server needs the next pad.** lsnes calls `on_input` right after
`on_frame_emulated`, at line 241, and commits the frame's input there;
`input.set` anywhere else returns without error and has no effect (probe:
Up set at the RTI of lsnes frame 1344 never reached `219`; set in
`on_input` of 1346 it was read by the NMI at line 249 of 1347). An NMI
running past 241 returns at line 249 of the next lsnes frame, whose input is
already committed. So the client announces the next step's pad with each
step (`S pad memories next`, `Lsnes::preview`, `Backend::preview`); at a
boundary inside an NMI the server commits that pad for the next frame and
sends the owed record from the RTI, and the next step must bring the same
pad. Each step's request still gets exactly one record. A save (`V`) is now
requested before its step and taken at the boundary that ends the step,
inside the NMI if it runs on; a resumed run restarts its counts at that
NMI's dangling RTI. The record's video frame is the lsnes frame the NMI
began in. `movie`, `cross` and `ladder-check` preview; `corpus` cannot (its
next pad depends on this step's cut), so corpus rows on lsnes keep the
boundary cut (`NmiStillRunning`) for such steps. So the cut difference (g)
recorded as `LEGACY(oracle)` is gone for `movie`, `cross` and the ladders,
and stays for lsnes corpus rows.

**Cross-oracle report, any% steps 0-1500** (`oracle cross`, 38.7 s):

| Part | First | Steps differing (of 1501) | Before (f) |
| --- | --- | --- | --- |
| sram | 0 | 1500 | power-on contents |
| pad | 0 | 5 | the pad's meaning |
| cpu | 0 | 834 | |
| wram | 4 | 1384 | |
| cut | 21 | 795 | |
| frame | 303 | 2 | 303, then every long NMI |
| vram | 303 | 2 | 303, 699 of 4801 |
| joypad-reads | 307 | 1 | 307, 24 of 4801 |
| cgram | 307 | 1 | 307, 221 of 4801 |
| oam | equal throughout | 0 | 480, 1519 of 4801 |

Steps 396, 472, 1290 and 1342 no longer show "right a frame behind": each
differs only in wram `/usr/bin/bash641`-`/usr/bin/bash642` (sound acknowledgement), the stack page,
`E:D7C0-DE1B` (zero against `0xFF`, copied from the power-on SRAM) and
where in the wait loop the NMI landed. With `--ranges-at 1340..1500` the
only wram ranges are `/usr/bin/bash641`(-`/usr/bin/bash64B`) and `FDD`-`FFF`: `/usr/bin/bash5B6` and
`/usr/bin/bash5B8` agree after 1342. Left: 303-307, the title's NMI (`B:9C..`),
where LakeSnes counts its frame one later and lsnes's uploads land a step
later (vram 304, cgram 307 "right a frame behind"); unchanged, not looked into.

**Resuming.** From native seed 16 (`movie --from 0016.lss 17829 --frames
18130 --ladder 50 --native-seeds room-seeds-any-boundary.csv`): no errors,
seed 17 cut, `ladder-check` 5 of 5 rungs equal. None of those boundaries
fell inside an NMI, so also from power-on with `--ladder 10` to 1420: 17 of
141 saves were inside an NMI (rungs 480-560, 1300, 1350-1410), and
`ladder-check --rungs 470..570` and `1290..1420` give 23 of 23 equal (the
resumed record, memories included, equals the straight run's byte for byte).

**Speed.** `movie --frames 3000 --oracle lsnes`, memories on, alternating
runs: 42 and 43 frames/s, against 44 and 43 with the previous server.

**Records change.** Every lsnes record of a step whose NMI outlasted line 241
changes (its cut kind, CPU, counts, memories and frame), and later saves
are requested a step earlier in the client. The lsnes ladders and the
native seeds' `.record` files cut before `9777b468a` must be regenerated before
they are compared with new records.

## (j) Ports in time order (candidate 0003): not the cause of 17985

**Candidate.** `crates/oracle/native/candidates/0003-apu-ports-in-time-order.patch`,
on top of 0001 and 0002. It is not in `fixes/`, so the build does not apply
it. It orders every CPU-SPC port access by master-clock time, as bsnes v085's
cothreads do:

- The SPC pays off the catch-up debt cycle by cycle, so each of its accesses
  knows its time against the CPU's.
- At a CPU access to `$2140-$2143` the SPC runs to the access time (reads 4
  master clocks before the end of the access, writes at its end, as bsnes
  v085's CPU does). It stops before an opcode that would touch `$F4-$F7` at
  or after that time; a dry run of the next opcode on a copy of the SPC
  registers tells. On a tie the CPU goes first.
- The SPC reads mid-cycle and writes at the end of its cycle, as bsnes v085's
  SMP does. A CPU write keeps the previous value and its time, so an SPC read
  made before the write still sees the old value. `$F1` port clears follow
  the same order.
- The port history goes into the save state. Applying it needs two bridge
  edits: `snes_catchupApu(snes, 0)` at the end of a step, and
  `oh_set_spc` copying the seeded ports into `apuPortBefore`.

**Command.** The same as (h): any% from power-on, the seven lag-log words
dumped per step, steps 0-35999 with 0003 and 0-18999 with 0002 alone, and a
per-step trace of steps 17950-17995 on both (wait entry, cut, CPU PC, the APU
port log and the SPC PC).

**Result: no change.** All seven words are equal on every step 0-18999 with
and without 0003. The classification is (h)'s: exact through 395, the boot
upload (352) and 8465/8469 unchanged, 17985 one more lag frame, one frame
behind from 18108 and the desync from 18587.

**Evidence.** The candidate is active: the SPC's own timing changes. Its port
writes land up to 257 SPC cycles (one timer-2 period of 16 ticks) apart from
0002's in the traced window, and its PC at the step ends differs. The CPU's
side does not change: the wait-entry beam, the cut and the CPU PC are equal on
all 46 traced steps. The door does not talk to the SPC. The CPU accesses the
ports only at 17950, 17952, 17958 and 17961, two to four times each. From
17962 to 17995 it makes none, and the SPC only writes its status. The extra
lag is a long room-load frame: the main loop reaches its NMI wait at line 251
on step 17986, past vblank (line 225), in both builds. lsnes is in room
`9E9F` a step earlier. So the extra time is in the CPU's own work for the
load (decompression, DMA, VRAM uploads), not in a CPU-SPC handshake.

**Speed.** The machine was shared with other lanes, so the numbers are
noisy. `oracle movie ... --frames 20000 --positions-only` ran back to back:
0002 at 56 frames/s, 0003 at 48 (about 15 % slower). Earlier, on a quieter
machine, 0003 alone ran at 295 frames/s. The dry run happens only within 16
SPC cycles of a port access, so the cost is probably the per-cycle debt
update.

**Reading.** At the exact rate, port order is not what LakeSnes lacks for the
any% movie through 18999. What is left at 17985 is CPU-side timing in the
room load. Next: find the load's time on both emulators. Log the master-clock
position of the load's phases on LakeSnes and lsnes (watched PCs in the
door's `$82:E2xx` and decompression routines, and DMA start lines) and
compare them. Then check the CPU timing models that differ: DMA setup and
per-byte cost, DRAM refresh, and HDMA during the load.

## (k) The 17985 lag is HDMA overhead on lines after the HUD, not the room load

**Command.** Watched PCs on both emulators over the same movie steps.
LakeSnes: `oracle movie ROM movies/any.inputs --frames 17996 --no-picture
--oracle lakesnes --trace 17955 17995 PC,...` from power-on (82 s). It prints
each step's game state, door function (`$099C`), wait entry, cut, HDMAEN and
DMAP, then every hit with its line and dot. The bridge now keeps up to 32
watched PCs and 16384 hits per step, and logs each `$420B` write as a pseudo
hit (`0x01000000 | MDMAEN`, with the first channel's DMAP, B-bus address and
size). lsnes: the same command with `--oracle lsnes --from 010000.lss 10000`
(the any% ladder rung, 190 s) and `OH_TRACE=PC,...`, `OH_TRACE_LOG=FILE`,
`OH_TRACE_FROM=FRAME` set. The server then logs `pc ADDR LINE DOT` at each
watched address and a `cut` line after each record. lsnes's `$420B` write
callback never fired, so the DMA starts are LakeSnes-only. lsnes fires exec
callbacks on operand fetches too (a watch on `$82:DA2B`, the operand of a
`BEQ`, hit), so watch instruction starts only. Lines are 0-261 and dots
0-1363 on both.

**Not the load.** Steps 17955-17986 are game state `$0B` with door function
`$E2DB` (fade out the screen). Each pass is `$82:D961` (advance the palette
fade by 1/12) and then the enemy routines and drawing. A pass takes 2-3 video
frames. The room load (`$E36E`, `$EA73`, decompression `$80:B119`) starts
after the fade, at 17989 on lsnes and 17990 on LakeSnes, so it is already a
step late. The wait entry (`$80:8338`) of each pass, line/dot:

| Step | lsnes | LakeSnes |
| --- | --- | --- |
| 17955 | 31/368 | 36/1200 |
| 17958 | 24/842 | 29/846 |
| 17961 | 24/960 | 29/1040 |
| 17964 | 17/54 | 22/170 |
| 17967 | 3/942 | 8/948 |
| 17970 | 248/1262 | 253/986 |
| 17973 | 242/1226 | 247/926 |
| 17975 / 17976 | 223/156 (step 17975) | 228/680 (step 17976) |

Both start each pass at the same beam: `$82:E288` at 246/1104 (LakeSnes) and
246/1108 (lsnes) at 17956. LakeSnes ends every pass about 5 lines later. The
game's own work per pass gets shorter, so the wait entry drifts up the frame.
At 17975 lsnes enters its wait on line 223 and the NMI on line 225 takes it:
a 2-frame pass. LakeSnes enters on line 228, after vblank started, so its
pass takes 3 frames. From there LakeSnes is one step behind: its last fade
pass, its `$E2F7` and its room load each come a step later. So 17985 is the
first step whose counters show the lag.

**Phase table**, one fade pass (master clocks between watched PCs; a frame is
262 x 1364):

| Phase (from, to) | 17953 LakeSnes / lsnes | 17956 | 17962 |
| --- | --- | --- | --- |
| `$E288` to `$E2DB` (state `$0B` dispatch) | 166 / 166 | 126 / 126 | 126 / 126 |
| `$E2DB` to `$D961` | 40 / 40 | 40 / 40 | 40 / 40 |
| `$D961` to `$E2DE`: **palette fade** | 728658 / 722210 (**+6448**) | 724462 / 717958 (**+6504**) | 714982 / 708430 (**+6552**) |
| `$E2DE` to `$E2E4`: which enemies to process | 8870 / 8910 | 8910 / 8910 | 8910 / 8910 |
| `$E2E4` to `$E2E8`: main enemy routine | 13248 / 11846 | 12946 / 12592 | 11398 / 11398 |
| `$E2E8` to `$E2EC`: draw | 36606 / 36580 | 23016 / 23040 | 23966 / 23608 |
| `$E2EC` to `$E2EF` | 150 / 132 | 132 / 132 | 132 / 132 |
| `$E2EF` to the wait | 5482 / 5708 | 6226 / 6232 | 6078 / 6072 |

The first phase that differs is the palette fade, by about 6,500 master
clocks (4.8 lines) per pass. The other phases differ by at most a few hundred
clocks. The fade runs from line 241 of one frame to about line 250 two frames
later, so it covers about 1.9 frames of display lines.

**Where in the fade.** We watched 16 PCs inside the fade's colour routines
(`$82:DA4A`, `$DAA6` and the division at `$DAC9`-`$DAE3`), steps 17953-17955.
For each pair of consecutive PCs we compared the clocks between them. In
vblank the two emulators agree (the sum of pair means is 2315.2 on LakeSnes
and 2317.6 on lsnes, ratio 0.999). On display lines LakeSnes is slower (2366.0
against 2355.6). On short pairs (base 30-40 clocks) the extra time is a step at
dot 1076-1110, the HDMA point:

| Lines | LakeSnes | lsnes |
| --- | --- | --- |
| 0-31 (HUD) | +72 per line (6 samples) | +72 per line (4 samples) |
| 32-224 | **+12 or +18** per line (27 samples, lines 37-223) | nothing (no sample) |
| 225-261 | nothing | nothing |
| 0, dot 18 | nothing | +36 or +42 (HDMA init) |

The +40 of DRAM refresh (dot 536) shows equally on both. During the fade one
HDMA channel is enabled: HDMAEN `$80`, channel 7 with DMAP `$43` (indirect,
mode 3, 4 bytes a line to BG3 scroll): the HUD's scroll split. It transfers on
the HUD lines and then ends. On lines 32-224 lsnes charges nothing for it,
and LakeSnes charges 12-18 clocks every line. 193 lines x about 15 clocks is
about 2,900 clocks a frame. Over the fade's 1.9 frames of lines that is
about 5,600-6,000 clocks, close to the measured +6,450 to +6,550 per pass.

**Mechanism (hypothesis).** LakeSnes's `dma_doHdma` runs on every display line
when any channel's HDMAEN bit is set: `hdmaActive` there is the enable bit
only, terminated channels included. It aligns to an 8-clock boundary, adds 8
clocks of overhead and re-aligns to the CPU's cycle, which makes 12 or 18 at
6-clock FastROM cycles. Its per-channel work (`do all copies`, `do all
updates`) skips terminated channels, but the overhead is still charged. bsnes
v085 counts a channel as HDMA-active only while it is enabled and not
completed, and charges no overhead on a line where no channel is active. That
is what lsnes shows from line 32. This is a reading of the measurement against
how bsnes and Mesen describe HDMA (a per-line overhead, plus 8 clocks per
active channel and 8 per byte, while a channel is still running); no GPL code
was read or copied. The ruled-out candidates: DMA cost per byte and DMA setup
(no general DMA in the main thread during the fade; the NMI's `$420B`
transfers start at 225/950-970 every pass and do not differ), DRAM refresh
(equal), slow/fast ROM and WRAM access cycles (vblank pairs are equal to the
clock), and the APU (no port accesses, (j)).

**Mesen** was not run: at +41 it leaves the movie at 16065 ((h)), so it cannot
vote at this door.

**Next.** A LakeSnes fix, `0004`: in `dma_doHdma` (and the HDMA run inside
`dma_waitCycle`), skip the sync and overhead when every enabled channel has
terminated. Then rerun (h)'s any% command and the per-step classification:
17985 should be one-step-equal, and the boot (exact through 395), 8465/8469
and 1342 must stay unchanged. Check whether 18108 and 18587 move. Hardware
also counts here: if Mesen or the timing documents (anomie, fullsnes) charge
the overhead on lines after termination, the fix follows lsnes for the movies
and is listed as such.

## (l) Fix 0004: no HDMA overhead on lines after every channel terminated

**Fix.** `crates/oracle/native/fixes/0004-hdma-overhead-only-while-a-channel-runs.patch`,
after 0001 and 0002. LakeSnes's `dma_doHdma` (called from `dma_handleDma`
and from `dma_waitCycle`, so both paths) now returns before the alignment,
the 8-clock overhead and the re-alignment when every enabled channel has
terminated for the frame; an enabled channel still cancels its general DMA
there, as upstream does. A line with no channel enabled was already free.
The source is anomie's SNES timing doc (revision 1126, section "HDMA"): "For
each scanline during which HDMA is active (i.e. at least one channel has not
yet terminated for the frame), there are ~18 master cycles overhead." So
hardware and lsnes agree here, and the fix is not a movie-only setting.

**Frame-start init: unchanged.** The same section describes the init as "For
all active channels, the HDMA registers are initialized at about V=0 H=6",
with ~18 clocks plus 8 per direct and 24 per indirect channel. LakeSnes's
`dma_initHdma` already returns with no overhead when HDMAEN is `$00` and
charges only for enabled channels, so it has no counterpart of the per-line
issue. fullsnes and the nesdev wiki (Timing, DMA registers) give no HDMA
clock counts.

**Command.** (h)'s run, `oracle movie ROM any.inputs --frames 36000
--no-picture --oracle lakesnes --lag-log any-boundary/lag.csv`, with the
mismatch print limit lifted in a local, uncommitted build, once with
0001+0002 and once with 0001+0002+0004 (same rlib, same session). Each step
was then classified against lsnes's rows k-1, k and k+1.

**Steps 0-17974: unchanged.** All seven lag-log words are identical between
the two builds on every step before 17975: exact through 395, the same rows
396, 397, 400, 1342 (`$05B6` one ahead, no lag flag), 8421 and 15235, and
8465/8469 equal to lsnes as in (h). Only wait-entry dots move, on frames
with display-line HDMA.

**17985 passes.** From 17975 the fade-out passes take lsnes's 2 frames, and
17985-18031 are exact in all seven words. 18108 and 18587 pass too.

**State, room and position** now match lsnes through step 35199, but for 50
single steps that match again on the next step: 34 door-transition samples
(game state `$0B`) where Samus's position is taken while the game moves her
into the next room, and 16 steps whose state or position equals lsnes's row
k-1. The counters keep (g)'s pattern (one NMI ahead at the line-241 cut).

| Steps | LakeSnes (0001+0002+0004) against lsnes |
| --- | --- |
| 0-395 | exact, all seven columns |
| 396-35199 | state, room, position equal, but for the 50 single steps above |
| 35175-35199 | `$05B8` one ahead with `$05B6` equal, in the door transition from room `9AD9` into `9CB3` |
| 35200 | lsnes leaves the transition (state `$08`), LakeSnes a step later (35201) |
| 35202- | Samus's X differs beyond a one-step shift (49 against 43 at 35202, 467 against 332 at 35300); by 35800 lsnes has left for room `9D19`, LakeSnes stays in `9CB3` |

**The new divergence** is the same kind as 17985: the door transition from
`9AD9` (Green Brinstar main shaft) into `9CB3`, after the load, with passes
of the game loop that end around vblank start. LakeSnes's wait entries are on
lines 222-229 from 35162, and at 35199 on 229/686, after vblank started, so
its last pass takes one more frame and the room starts one step late. Then
Samus's movement no longer lines up with the inputs. Not localized here: a
`--trace` of both emulators over 35150-35200, as in (k), would show which
phase of the pass is longer.

**Speed.** 271 frames/s with 0004 (133.0 s for 36000), 260 frames/s for
0001+0002 in the same session (138.5 s); the run with 0004 plays more of the
game, so the two are not the same work after 18587.

## (m) 35175: a 30-clock miss of the NMI request, from HDMA on the HUD lines

**Command.** As in (k), with fixes 0001+0002+0004 (a private DLL built from
`32eaa1838`). LakeSnes: `oracle movie ROM any.inputs --frames N --no-picture
--oracle lakesnes --trace FROM TO PC,...` from power-on (2.2-2.7 min). lsnes:
the same with `--oracle lsnes --from 030000.lss 30000` (the any% ladder rung)
and `OH_TRACE`, `OH_TRACE_LOG`, `OH_TRACE_FROM`. Three rounds: the main loop
over 35150-35204; the loop and every call of door function `$82:E737` (fade
in, run enemies) over 35160-35179; 16 PCs inside the fade's colour routines
(`$82:DA24`-`$DAED`, the division at `$DAC9`-`$DAE3`) over 35173-35175.

**(l) misread the step.** From 35152 every pass is `$E737`, and both
emulators enter their waits at the same beam, within about 40 clocks, pass
for pass: 233/358 and 233/360, 226, 226, 224/906 and 224/904, 222/1152 and
222/1160, 224/1260 and 224/1230, 224/316 and 224/360, then line 229 (four
passes on both), then 108/588 and 108/612 in gameplay. lsnes also enters on
line 229 from 35181; its record's `wait` field shows each wait a step earlier.
The step attribution differs from 35175 on. There LakeSnes enters its wait at
224/1260 and lsnes at 224/1230. The wait routine (`$80:8338`) is PHP, PHB,
PHK, PLB, SEP, `LDA #$01` and then `STA $05B4` (NMI request flag) at
`$80:8340`, about 104-120 clocks of work before the store. On lsnes the store
comes before the line-225 NMI: the NMI is `Requested` (interrupted at
`$80:8346`), a 2-frame pass. On LakeSnes the NMI interrupts at `$80:8340`,
before the store: the game counts it as a lag frame (`Overrun`) and waits one
more frame. From 35175 LakeSnes is one step behind (`$05B8` one ahead with
`$05B6` equal, as (l) saw). It enters gameplay a step late at 35200, and
Samus's movement then gets the wrong inputs. A 30-clock difference after a
2-frame pass decides it.

**Phase table**, the pass from the 35172 wait to the 35175 wait, master
clocks between watched PCs (LakeSnes / lsnes):

| Phase (from, to) | LakeSnes / lsnes | Difference |
| --- | --- | --- |
| wait to `$82:8948` (NMI, vblank) | 26220 / 26218 | +2 |
| `$8948` to `$894F` (HDMA objects) | 4662 / 4662 | 0 |
| `$894F` to `$8963` | 1494 / 1494 | 0 |
| `$8963` to `$E288` to `$E737` | 128 + 126 / the same | 0 |
| `$E737` to `$E73B` (animated tiles) | 1358 / 1358 | 0 |
| `$E73B` to `$E73F` (which enemies) | 14560 / 14560 | 0 |
| `$E73F` to `$E743`: main enemy routine, lines 258-27 | 43002 / 43012 | **-10** |
| `$E743` to `$E747`: enemy projectiles | 3012 / 3006 | +6 |
| `$E747` to `$E74B`: draw | 37052 / 37048 | +4 |
| `$E74B` to `$E74E` to `$E752` | 7800 + 182 / the same | 0 |
| `$E752` to `$E755`: **palette fade** (`$82:D961`), lines 63-224 and 0-220 | 215196 / 215160 | **+36** |
| `$E755` to the wait (`$E292`, `$896E`-`$897A`) | 58 + 144 + 1192 + 3620 + 344 + 54 / the same | 0 |

Clocks are beam distance (line x 1364 + dot); a phase that crosses a frame
counts the frame once on both. The pass starts 6 clocks early on LakeSnes
(`$E288` at 246/920 against 246/926) and ends 30 late. The three passes
before it show the same pattern: the fade +14, +44 and +32, the main enemy
routine -38, -28 and -34, the NMI +38, +20 and +8, and every other phase
equal to the clock. The two phases that differ every pass both cover lines
0-31, the HUD. The NMI phase (vblank, the NMI's `$420B` transfers) is not
localized here; in the deciding pass it is +2.

**Where in the fade.** 7099 consecutive watched-PC pairs from `$E752` to
`$E755`. Both PC sequences are the same once LakeSnes's duplicate hit is
dropped (it logs the PC an IRQ interrupts twice, before and after the IRQ).
The running difference (LakeSnes minus lsnes) moves only on lines 0-32:

| Lines | Running difference after them |
| --- | --- |
| 63-261, first frame | -2 |
| 0-32 (HUD), second frame | +24 |
| 33-261 | +24 |
| 0-32 (HUD), third frame | +36 |
| 33-220, to `$E755` | +36 |

Pairs that span the HDMA point (start between dot 600 and 1090, end after
1112, same line): on lines 33-224, 351 samples, all equal to the clock. On
lines 1-31, 33 samples: 13 equal, and the rest -6 (4), -2 (3), +2, +4 (2), +6
(4), +8 (2), plus 4 of +108 where lsnes's pause falls in the next pair
(cancelled there by -108 or -114). Net about +0.7 clocks per HUD line, with
2-8 clocks of noise either way. The DRAM refresh shows as a matched +40 and
-40 pair (the hit lands after the pause, dot 532 on LakeSnes against 578)
and cancels. During the fade HDMAEN is `$84`: channel 7 (DMAP `$43`,
indirect, mode 3, 4 bytes to the BG3 scroll, the HUD split) and channel 2
(DMAP `$42`, indirect, mode 2). Channel 7 transfers on the HUD lines.

**Mechanism (hypothesis).** On the lines where a channel transfers, the HDMA
pause rounds differently. It is not a fixed extra cost. anomie's SNES timing
doc (revision 1126, sections "HDMA" and "DMA"):

- HDMA starts "at dot 278 of the scanline (or just after, the current CPU
  cycle is completed before pausing)".
- Per line it costs "~18 master cycles overhead" while a channel runs,
  "another 8 master cycles" per active channel, "16 master cycles" for a new
  indirect address, and "8 cycles per byte".
- The pause is aligned as a DMA is. Before the transfer it waits "2-8 master
  cycles to reach a whole multiple of 8 master cycles since reset". After it,
  it waits "2-8 master cycles to reach a whole number of CPU Clock cycles
  since the pause", where "the speed of the next CPU cycle to execute after
  the DMA determines the CPU Clock speed".

LakeSnes (MIT) does this differently:

- It requests HDMA at master clock 1104 (dot 276, not 278) and starts the
  pause at the next CPU access.
- It charges 8 clocks of line overhead (not ~18), 8 per running channel (in
  its update loop), 8 per byte, and 8 + 8 for an indirect address.
- It aligns the start to a multiple of 8 since power-on and the end to the
  speed of the access that triggered the pause.

Any of the start point, the overhead constant or the alignment phase moves
the CPU-clock rounding by 2-8 clocks on a line, depending on where the CPU is.
That fits the pattern: noise with a small mean, only on lines where bytes
move. On lines 33-224 the pairs match exactly.

The other candidates do not fit:

- General DMA: there is none during the fade.
- DRAM refresh: matched.
- The division and multiply registers: the fade's division pairs are equal
  away from the HUD lines.
- WRAM and FastROM access speed: equal everywhere else.
- The APU: no port accesses here.

The H/V IRQ trigger point is a candidate on line 0 only. anomie puts a V-IRQ
at "1374 master cycles after dot 0.0 of the previous scanline" and an H-IRQ
at "14+H*4", and LakeSnes raises it at `hPos == HTIME*4`. The line-0 IRQ pair
is +6, a small part of the +36. fullsnes and the nesdev wiki give no HDMA
clock counts. No GPL code was read.

**Next.** The fix lane first measures the HUD lines, then changes one thing
at a time and keeps 0-17974 unchanged ((l)'s check):

1. Log each HDMA pause on lines 0-32 from LakeSnes: start clock, clocks since
   power-on mod 8, the speed of the triggering access, the work and the end.
   Use the 35173-35175 fade, beside lsnes's watched-PC pairs on the same
   lines. lsnes cannot log the pause itself.
2. Try variants one at a time, each scored by the HUD-line pair sum against
   lsnes (the target is 0 over the 33 samples): a line overhead of 18 instead
   of 8; the request at 1112 (dot 278) instead of 1104; the H/V-IRQ point per
   anomie.
3. Keep the variant that zeroes the HUD lines and rerun (l)'s any%
   classification. 35175 should become `Requested`. The boot (exact through
   395), 1342, 8465/8469 and 17985 must not move.

The margin here is about 30 clocks after a 2-frame pass. Any per-line error
left near vblank shows up as a one-step lag.
