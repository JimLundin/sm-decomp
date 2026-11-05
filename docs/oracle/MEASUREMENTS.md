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
