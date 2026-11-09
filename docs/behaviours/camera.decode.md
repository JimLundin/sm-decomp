# The camera following Samus: decode notes

For the observer, the adversary and the domain harness's decoder only. Implementers never see this
file (MODEL §5). It holds every source the spec draws on: the original's routines, its RAM and its
codes. Addresses are the bank logs' (`$TEMP/banklogs/bank_*.txt`,
https://patrickjohnston.org/bank/).

## How the spec was made

Behaviour first: the 32 rows cached in `$TEMP/collab/more-caches/cache` were dumped per frame by
`$TEMP/collab/camera-spec/dump` (a cdylib over `oracle_record`, run by `dumprun.py`; output in
`$TEMP/collab/camera-spec/out`) and read over time. The bank logs were then read only to name the
cases and pin the arithmetic. An observer's reference model, `model.py` in the same directory, runs
each frame from record N to record N+1 by the original's rules. Over the 14,890 frames that ran
one frame of gameplay in one room (game state 8 at both, `05B6` advanced by one, the same room
`079B`, no slow grapple scrolling `0CF8` and no scrolling-finished hook `07E9`), it matches the
camera's position (`0911`.`090F`, `0915`.`0913`) on 14,704. A null model (the camera doesn't move)
matches 6,271 of them. The 186 it misses are frames where another routine moved the point the
camera measures from (`0B10`-`0B16`) after record N and before the scroll: 113 are pose changes
(the transitions of bank `$91` that set `0B14`, the placements samus-motion reports as
`placed`), 69 are the shinespark's clamp (`$90:D1D6`..`$90:D1FE` and `$90:D2A3`..`$90:D2B9`,
previous position within 15 px across and 14 px down), and the rest the grapple (`$9B:BB25`..,
the ±12 px clamp) and the Landing Site's gunship (`$A2:A925`). With the y the camera saw taken as her y at the scroll, the
113 pose changes match too. Over the 44-row cache (the 12 rows added since: the bosses, the
items, the event and four chain rows; `camdump3` on `$TEMP/collab/scenes/cache`, then
`$TEMP/collab/camera-r4/model.py` and `pfpose.py`), it matches 20,289 of 20,558 frames, the null
model 8,361. The 12 new rows add 5,668 frames, 5,585 matched: 81 of the 83 misses are pose changes
that match with the seen y taken as her y at the scroll, and 2 are in boss-kraid-defeated
(records 806 and 850, movement types 0Ah and 3, the camera's x subpixel only, its pixels matching):
Kraid's body collision moved where the camera saw her on both, and the harness masks the 2 frames
it fires on, by the push back's extra displacement it leaves (below); frame 672, where Kraid's arm's
touch calls the same push back, stays compared and matches.

`gen.py` picks the recorded vectors (at most two per outcome, from different rows, from frames
the model matches, with `0B10`-`0B16` at N+1 equal to her position `0AF6`-`0AFC`, outside movement
types 16h, 1Ah and 1Bh: grapple, grabbed by Draygon, shinespark; the one it picked from
samus-square-slope-reserve was dropped, its row name being a lexicon match), `cons.py` writes the
constructed ones (the results come from the same model) and `vecrun.py` runs every vector of
`scenarios/vectors/camera.toml` through it: 80 of 80 agree (`$TEMP/collab/camera-r4/vecrun.py`
reads `placed`, adversary 53's O3; camera round 3, adversary 49's O1:
`below` is one run continuing the room's screens, as the table is, and
`$TEMP/collab/camera-r3/vecrun.py` appends it to the room's screens unpadded, a screen past its
end read as `Closed`; round 2's version padded each below row to `w`, which let a short row stand
before another, a table the game can't hold; round 3 adds adversary 42's D1, D2, D6 and D7 and
C-push-carries-move, and round 4 adversary 43's D8, C-settle-up-from-below-short, each worked by
hand from the bank log as well: for D8, `$80:A74B` reads `0933` from row 2, below the room, so
Closed and 1F, before `$80:A788` clamps `0915` to 011F; row 1 is Short, not red, so `$80:A81D`
reads row 2, red, and `$80:A839` holds on 011F ≥ 011F; and adversary 51's F1, two vectors that
read the run past its first `w` and past the last row's right edge: C-below-second-row, where
`0915` after the speed's add is 0210, row 2, so the down scroller reads index 2 × 2 + 0 = 4
(Open, 1F rows) and `$80:A8FC` index 4 + `07A9` = 6, the run's third byte, Open, and the camera
stays on its aim 00F8 (red there gives 00F0 − 2 = 00EE); and C-wrap-into-below, where across the
row is (0100 + 80h) >> 8 = 1 and the camera's screen is index 1 × 2 + 1 = 3, red, its ease 0103
and the screen after it index 1 × 2 + 2 = 4, the run's first, Open, so it eases to 0103 (red
there drops it to 0100); down, column 1, index 3 red and index 5, past the run, red, so it holds
on 0100. Each Closed variant was run through the model as well and gives the stated other
result; and adversary 53's O1, C-edge-down-from-below: from 0150, seen 0200 and her 0202, the
speed is 3, `$80:A8DA` keeps 0153 (the aim 0192 is below it), the upper scroll's index 1 is Open,
so `0933` = 1F and `0937` = 011F; 011F < 0153 takes `$80:A8FA`'s branch to `$80:A91E`, which
proposes 0153 − 3 − 2 = 014E, and `$80:A928`'s signed compare keeps it over 011F; folding the
stop into 011F would give 011F). `rise0.py` writes R35, the one
recorded rise at 0 (`0B36` = 1 with `0B2C`/`0B2E` = 0 at the scroll): all 7 such frames are
shinespark frames (movement type 1Bh), and in samus-shinespark-left 214 the clamp didn't fire
(the model matches with the previous position as record N left it) and the down scroller decides.

**The dump's screen table** (camera round 3). `camera-spec/dump` wrote each of the table's 50
bytes in decimal with no separator, and `model.py` read it a digit a screen. Inside the room the
table holds only 0, 1 and 2, so the rooms' screens were right, but a byte past the room of 10 or
more (the bytes the loading copies from after the room's own, `$82:EB28`) became several screens.
`$TEMP/collab/camera-r3/sdump` writes them in hex, its output is `camera-r3/out3`, and
`camera-r3/model.py` reads it: the per-frame counts are unchanged (14,704 of 14,890, null 6,271),
since the screens each frame reads past the room happen to read the same. Two recorded vectors'
`below` were wrong past those screens and are now the true run, cut by `camera-r3/belowrun.py` to
the shortest that gives the recorded result: R3 (area-brinstar 414, table bytes past the room `12
01 80 12 00 80 ..`, the frame reads column 5, `80`, Open) "OSOOCO", was "SOSSOOSO"; R16 (r01 655,
`13 01 80 01 00`, column 3, `01`, Short) "OSOS", was "SOSSOOSC", whose last three screens lay past
the 50-byte table and came from the misread digits.

**Row names.** As in samus-motion's decode notes, the chain rows go by opaque ids, because a chain
row's name carries a room header's address:

| Id | Row |
| --- | --- |
| r01 | chain-0012-91F8 (Landing Site) |
| r02 | chain-0013-92FD |
| r03 | chain-0014-96BA |
| r04 | chain-0057-9FBA |
| r05 | chain-0059-A293 |
| r06 | chain-0097-ACF0 |
| r07 | chain-0100-B07A |
| r08 | chain-0106-AD5E |
| r09 | chain-0220-D017 |
| r10 | chain-0224-D0B9 |
| r11 | chain-0232-D5A7 |

## State

| Quantity | RAM | Decode |
| --- | --- | --- |
| camera position | x `0911`.`090F`, y `0915`.`0913` (layer 1's position) | as is |
| where it last saw her | x `0B10`.`0B12`, y `0B14`.`0B16` (Samus's previous position) | as is |

**RAM shape** (fields decoded as one word copied as-is): both, four words each. They are
positions, and the spec's quantity is the position itself: its pixel part and its subpixel part
are what the rules read and what the player sees (the view's corner, the point the next speed is
measured from). Nothing narrower would hold them.

Not State: the ideal positions `0B0A` and `0B0E` (the aims, worked out each frame), the camera
speeds `0DA2`.`0DA4` and `0DA6`.`0DA8` (worked out each frame from the position and the previous
position), the scratch `0933`-`0939`.

## Inputs

| Input | RAM (at the scroll, record N+1) | Decode |
| --- | --- | --- |
| her position | `0B10`-`0B16` of record N+1, which the scroll sets to `0AF6`-`0AFC` as it ends | as is; equal to N+1's `0AF6`-`0AFC` unless the room's main code or the next frame's start moved her |
| her facing | `0A1E` low byte | 4 `Left`; 8 `Right`; 0 `Screen` (the scroll tests only 4) |
| going backwards | `0A52` (knockback direction), `0A1F` low byte (movement type), `0B4A` (X acceleration mode) | `0A52` ≠ 0, or movement type 10h (moonwalking), or `0B4A` = 1 (turning around: the skid, samus-motion §1.2) |
| rising | `0B36` | = 1 (up), samus-motion's rise state: from the rise's start until `$90:8FDC`/`$90:9046` end it at the start of a frame, so a rise at 0 and the frame whose pull spends the rise (`0B36` = 1 with `0B2E` negative when `$90:9666` reads it) are rising; 0 (none, the moonfall's) and 2 (down) are not. Records: 3 usable spent frames (samus-bomb-jump-vertical 1035, samus-ceres-push 989, samus-grapple-release 325), none decisive; C-rise-spent is |
| the y a placement put her at, if any | samus-motion's output | its pixel part is what the placements write to `0B14` (the pixel word only; `0B16` is kept); with none, `0B14` is not written. Samus-motion reports it as an option (`placed`), a placement of 0 px included, which the harness's `placement` passes through and the bridge's `settle` writes to `0B14`'s pixel |
| the room's screens | `$7E:CD20`, 32h bytes, row-major, `07A9` across (`w`), `07AB` down (`h`) | 0 `Closed` (red), 1 `Short` (blue), 2 `Open` (green); the bytes from `w × h` on are the `below` run (index row × `w` + column, the room's rows continued), any value but 0 and 1 read as `Open` (only 0 and 1 are ever tested); past the 32h bytes, the RAM after it (docs/LEGACY.md) |
| the aim heights | `07AD` (up scroller), `07AF` (down scroller), from the room header (`$82:DEC1`, `$82:DECA`) | `hn` = `07AD`, `hr` = `07AF`; 70h/A0h or 90h/A0h in the records |
| lead setting | `0941` (camera distance index) | 0 `Ahead`, 2 `Near`, 4 `KeepLeft`, 6 `KeepRight`; the tables `$90:963F` (facing right) and `$90:9647` (facing left): 60h/A0h, 40h/50h, 20h/20h, E0h/E0h. Set by Crocomire (`$A4:8ABD` = 2, `$A4:97F6` = 6) and Kraid's room (`$A7:A9E7` = 2), cleared at `$88:8347`, `$A4:90D8`, `$A4:9B8D`, `$A7:C594`; always 0 in the records |
| time stopped | `0A78` | ≠ 0 (time frozen: x-ray, the reserve tanks' refill); set in 704 record frames of six rows; in the 95 that scroll, none where an ease would have moved |

The spec's enum orders are alphabetical: `Closed`, `Open`, `Short` against the RAM's 0 red, 2
green, 1 blue; `Ahead`, `KeepLeft`, `KeepRight`, `Near` against 0, 4, 6, 2.

## Sources per entry

Each entry lists the routines or branches that carry its content. `scan.sh --mirror` finds 2 of
14 entries (14 %) single-routine, `LeadingHer` and `AimingHeight`; `DownwardLooksAhead` is one
routine's two reads. All three are marked *. The name scan's 5 hits in the camera crate are prose
(axis.rs, follow.rs, room.rs twice, tests/spec.rs), and the enum-order scan's one hit is the
vectors test's TOML `Value`, a false positive; the root accepts both.

| Entry | Source |
| --- | --- |
| FollowingHer | `$90:95A0` and `$90:964F` (moved: `0B10` ≠ `0AF6` by pixel parts, `$90:95A3`, `$90:9652`; at the aim: `$90:9606`, `$90:9687`; the move: `$90:960D`..`$90:9636`, `$90:968E`..`$90:96B7`), `$90:96C0` and `$90:96FF` (the speeds, plus `$90:9EAD` = 1), the aim stops at `$80:A651`, `$80:A6CB`, `$80:A8DA`, `$80:A936`'s start (pixel set, subpixel `STZ`) |
| LeadingHer* | `$90:95B2`..`$90:9600` with the tables `$90:963F` and `$90:9647` |
| AimingHeight* | `$90:9666`..`$90:9681` |
| Remembering | `$90:956E`..`$90:9583` (after both axes); the placements (samus-motion's `placed`): bank `$91`'s writes of `0B14` in pose setting, pose initialisation and the block collision due to a pose change (`$91:E340`, `$91:E44A`, `$91:E50A`, `$91:ED0E`, `$91:F595`, `$91:FF42`, `$91:FF6F`, `$91:FF93`..`$91:FFE9`; `$91:80AB` in the crouching pose input; `$91:F3A1`, super special pose change command 5, y + 5), and `$90:ECB1` (aligning her bottom with the old pose). `$90:92E4` is in an unused routine (`$90:92D6`). Pose change command 4 (`$91:F37C`: `$91:F386` x and `$91:F393` y, her position set to the view's centre) is not samus-motion's: masked |
| CreepReadsRightward | `$90:96C0` and `$90:96FF` (the direction by `CMP`/`BMI` on the pixel parts, the bank log's note at `$96C0`), read by `$80:A58F` and `$80:A7D3` |
| RoomEdges | `$80:A662` (`(07A9 − 1)` swapped, the last screen), `$80:A6D9` (negative to 0), `$80:A8EB`..`$80:A8FA` (the bottom with `0933`), `$80:A936`'s negative test; the back-off at the bottom `$80:A91E` |
| ClosedScreens | `$80:A671`..`$80:A6B5` (right), `$80:A6E3`..`$80:A72B` (left), `$80:A8FC`..`$80:A930` (down), `$80:A936`.. (up); the row and column from `0915 + 80h` and `0911 + 80h` through the multiplier `$4202`/`$4203` with `07A9` |
| ShortScreens | `$80:A746`..`$80:A77A` and `$80:A8A3`..`$80:A8D7` (`0933` = 0 when the upper scroll is blue, else 1Fh) |
| Settling | `$80:A528` (across) and `$80:A731` (down): the clamps `$80:A53D`..`$80:A552` and `$80:A77D`..`$80:A796`, the left/upper red case `$80:A57E`..`$80:A5D7` and `$80:A7C2`..`$80:A81B`, the right/lower red case `$80:A5D9`..`$80:A638` and `$80:A81D`..`$80:A88A` |
| TimeStopped | `$80:A52C` and `$80:A735` (time frozen `0A78`/`0A79`: return) |
| DownwardLooksAhead* | `$80:A89D`..`$80:A8D7` (the upper scroll's index `$14` from `0916` after the speed's add, before `$80:A8DA` caps it at the ideal), read again at `$80:A8FC` |
| ScreensRunOn | `$80:A5D9` (`INX`: the right scroll is index + 1), `$80:A686`..`$80:A68D` (`SEC; ADC`: screen + 1 + row × width), `$80:A5B2`..`$80:A5BA` (`INC`), `$80:A613`..`$80:A61E` (the eased position's column, any value to FFh, plus row × width), all on the row-major table; the bank log's note at `$80:A528` |
| BelowTheRoom | the scroll table `$7E:CD20` (32h bytes), one table for the room and below it: every read indexes it by row × `07A9` + column (`$4202`/`$4203`, then `ADC $4216`), so a row past `07AB` continues the run, read past the room at `$80:A8FC`, `$80:A8C8` and `$80:A823`. `$82:EA73` loads it: a room with a scroll pointer copies all 32h bytes from it whatever the room's size (`$82:EB28`..`$82:EB39`), so past `w × h` the run is the ROM bytes that follow the room's screens; a room with preset scrolls fills `w × h` only (`$82:EB3D`..`$82:EB67`), leaving the rest as the last room left it. Either way it is one run from `w × h` on, so a short row can only be its last |
| PushCarries | the second `ADC #2` after `ADC 0DA2`/`0DA6` and the second `SBC #2` after `SBC 0DA2`/`0DA6` (carry and borrow kept): `$80:A58F`, `$80:A5F0`, `$80:A6A7`, `$80:A71D`, `$80:A7D3`, `$80:A842`, `$80:A922` and the up routine's |

## Findings behind the spec's choices

- **Two compares: `before` and `exceeds`** (round 2; adversary 40, S1). Every compare that
  decides a branch is `CMP` then `BMI`/`BPL` (the sign of the difference, the spec's `before`),
  except these, which are `BCS`/`BCC` (unsigned, the spec's `exceeds`):
  - the room's right edge (`$80:A667`) and bottom (`$80:A8F7`) after a move, and the settles'
    clamp to them (`$80:A54D`, `$80:A791`);
  - the left and up pushes' `min` (`$80:A723`, `$80:A99E`);
  - the settles' forward reach (`$80:A595`, `$80:A7D9`);
  - the "within `rows`" tests going down (`$80:A919`) and settling up (`$80:A839`).

  Round 1 stated one compare, reasoning that these see only values inside the room. That is
  wrong for three: a move from a camera left of or above the room reaches the right edge or
  bottom test with `FFF3` (C-edge-right-from-left, C-edge-down-from-above); the settles' reach
  is computed from the camera as the frame found it, before the clamp to 0
  (C-settle-reach-from-left); and the left push sums from where the move took the camera, which
  can be `8000` from the aim it stopped on (C-push-left-far). The spec now states them as the
  original makes them, rather than listing the difference in docs/LEGACY.md: the camera is
  placed by other domains (doors, the room rules), so a camera outside the room is a State the
  model can be handed, and stating the original costs nothing. `search.py` and `search2.py` in
  `$TEMP/collab/camera-r2` swap one compare at a time for `before` in `model.py`; each new
  vector differs under exactly its own swap. The two "within `rows`" tests, both down, and the
  settles' clamp can't differ by their compare: they see only values from 0 to the room's far edge
  (the push's test only after the bottom test, the clamp only after the `max` with 0).
- **No "within `rows`" test across** (round 3; adversary 42, A1). Round 2 stated one across as
  well, with `rows` 0, and said the settles "can't differ" from the original there, as no search
  found a case. That was wrong: the test exists only down (`$80:A919` pushing down, `$80:A839` settling up). Across, the right push
  (`$80:A69A`..`$80:A6B5`) and the ease back (`$80:A5E3`..`$80:A638`) have none: they compare
  `back` with the left scroll position by `BMI` and take the larger. Inside the room the two
  readings agree (a camera on its screen's edge backs off before the edge and is held there),
  which is why it looked unreachable; from a camera right of the room, clamped to the right edge
  at `$80:A54D` while `0939` keeps the unclamped position, they don't: C-settle-back-right-of-room
  (0150 eases to 014D, not 0100). `search.py` only swaps compares, so it could not find a test
  that shouldn't be there. The spec states the original: the test down only.
- **The row of a position above the room** (round 3; adversary 42, A2). Every screen read takes
  the row as the high byte of the 16-bit pixel position (`0916`, or `$14` from `0915` after the
  speed's add, `XBA` of `0915 + 80h` across) and the column likewise, times `07A9` plus the
  column. So `FFF3` is row FFh: past the room and its below rows, read as `Closed` (docs/LEGACY.md,
  "Scroll areas past the room's table"), and `0933` is 1Fh. C-edge-down-from-above-short: 011F,
  where reading the top as the first row (`Short`) would give 0100.
- **An aim `8000` away** (adversary 40, S2). The direction is `CMP` ideal, camera, then `BMI`
  (`$90:960B`, `$90:968C`): with the difference `8000` the camera moves left or up. The spec
  states it (C-aim-half-way).
- **The screen right of the last column** (adversary 37, F1). The original reads index + 1, the
  next row's first screen (or the first below row from the last row). An earlier version said
  `Closed`, reasoning that it is read only with the camera on the last screen's edge; that missed
  the ease forward: with the camera's own last-column screen closed (the cells can close it) and
  the next row's first open, the original eases past the room's right edge, 3 px a frame (C-wrap).
  No record reaches it; the spec now states the wrap (`ScreensRunOn`).
- **Below the room.** A move down's reads use `0915` after the speed's add (`DownwardLooksAhead`),
  so a move that crosses into the room's last row reads the row below it, from the leftovers. In
  2 rows (R3, R16) the leftovers decide: read as red, 5 frames would differ from the record. The
  model reading them as the table holds matches all.
- **Past the run** (adversary 51, F2; spec open question 1). `$TEMP/collab/adv51/past50.py` logs
  every table read at index 32h or more over `out3`: 409, all indices 50 and 51 (`$7E:CD52`,
  `$7E:CD53`), in samus-shinespark-left, -right, -up and -up-left, all in Landing Site (w 9, h 5,
  the run 45..49). They are the down settle's read of the screen under a camera on the room's
  bottom (`0915` = 0400, columns 5 and 6); the dump holds `00 00` there, red, so `Closed` agrees,
  and the settle holds there whatever the byte is. A top above the room (row FFh, index
  255 × `07A9` + column) also reads far past the table (C-edge-down-from-above-short).
- **The settles' third case.** Easing back (`$80:A5FB`..`$80:A631`, `$80:A84D`..`$80:A883`), the
  original reads the screen the eased position is in (column `093A`, the ease's high byte, `AND
  #$00FF`, plus the row times `07A9`) and, if it is red, puts the camera on the next screen's edge.
  Round 2 dropped it as unreachable. Across it is reached from a camera right of the room: from
  0250 in a room two screens across, the ease to 024D lands in column 2, which runs on to the next
  row's first screen; red there, it goes to 0300 (C-settle-back-far-right). Down it is not:
  `$80:A839` returns unless the clamped top is more than `0933` below its screen's edge, and a
  camera below the room is clamped to exactly that (`$80:A791`, with the same `0933`), while
  inside the room the eased top stays in the camera's own screen, which isn't red there. The spec
  states it for both axes, as the original has it.
- **The push's carry and borrow** (round 3; adversary 42, A3). The second `ADC #2` keeps the
  first add's carry, and the second `SBC #2` its borrow. The carry shows whenever the push sums
  from FFxx: a camera placed there (C-push-carries, settling from FFFF: 0003, not 0002), and also
  inside the room, when a move left or up wraps `0939` below 0 before the aim's `max` stops it
  (`$80:A6CB`, `$80:A936`), and the first screen is red (C-push-carries-move: 0006, not 0005).
  Round 2's "only from a camera left of or above the room, which a frame never starts with" missed
  that case. The borrow never shows: it needs `0939` under the speed, and every such push back is
  then before the edge it is compared with, which wins. A fuzz over 300,000 random frames of
  `model.py` with and without each (`$TEMP/collab/camera-r3/fuzz_borrow.py`) found the borrow
  change nothing and the carry change 503 frames; the spec states the carry as an arm and the
  borrow as a "So" line.
- **Time frozen.** The scroll right/left/up/down routines don't test it; she doesn't move while
  time is frozen, so only the settles' test is seen.
- **Out of scope here and masked in the harness:** the slow grapple scrolling (`0CF8` ≠ 0,
  `$90:94F7`..`$90:954D`, ±3 px towards a 60h..A0h by 70h..90h window), the scrolling-finished hook
  (`07E9` ≠ 0; Spore Spawn's `$90:9589`, layer 1 Y at least 1D0h), debug scrolling (`$80:8006`).

## The harness (for the integrator)

The camera domain is downstream of her motion: nothing in samus-motion reads the camera, so it can
be verified on its own from the records, with her motion's outputs decoded as inputs.

- **Frames:** every record pair N, N+1 of every row. Skipped, by name and never by outcome: game
  state other than 8 at N or N+1 (no main gameplay, so no scroll), `05B6` not advanced by exactly
  one (lag: no scroll or two), a room change (`079B` differs: the door's domain). Counted as a
  harness defect: a pair that doesn't decode.
- **Decoded from record N:** the State (camera position, where it last saw her).
- **Decoded from record N+1:** the inputs of §Inputs, at the scroll. Her position is N+1's
  `0B10`-`0B16`: the scroll writes it as it ends, so it is exactly what the camera read.
- **Compared at N+1:** camera x and y, pixel and subpixel parts. Where it last saw her is decoded
  from the same words as her position, so comparing it proves nothing: report it, don't count it.
- **Masks (fields, not frames):** where another domain moved the point the camera measures from
  between N and the scroll, or moved the camera after its turn, the axes it touches are
  unverified, reported apart. Whole states (documented as such, not single writes): movement types
  16h (grapple: `$9B:BB25`.. and `$9B:BE98`.., the ±12 px clamps; `$91:F3AA`, the release's clamp) and 1Bh
  (shinespark: `$90:D1D6`..`$90:D2B9`), and `0CF8` ≠ 0 (slow grapple scrolling), both axes.
  Single writers of where the camera saw her, both axes unless named:
  - `$A0:B7A1`, the cap on scrolling speed (`$A0:B7C4` y, `$A0:B7E8` x), called by Draygon (which
    holds her in movement type 1Ah) and the yapping maw: mask while either holds her;
  - Crocomire's collision (`$A4:8C95`); Kraid's body collision (`$A7:B0F3`), on the frames whose
    end holds the extra displacement its push back sets (`$A7:94A4`: `0B58` = 4, `0B5C` = FFF8h),
    except where the bottom lint's function (`10A8`) turns to B89Bh: Kraid's arm's touch
    (`$A7:9490`, `$A7:9496`; `$A7:9499` writes B89Bh) calls the same push back and moves neither her
    nor where the camera saw her; and its restriction of her x to the first screen (`$A7:C9EE`, x),
    under the six Kraid functions that call it (`$A7:C865`..`$A7:C924`, `0FA8`). The frame runs her
    movement (`$82:8B58`), then the enemies (`$82:8B65`), then her pose change (`$82:8B69`), so the
    restriction reads the x her movement left: mask x where her x at record N or at N+1 is 100h or
    past it (N+1's is 100h wherever it acted, unless a pose change placed her after it, which the
    placement masks cover);
    Ridley (`$A6:E501`); Mother Brain moving her towards the ceiling or floor (`$A9:BBFD`, y) and
    towards the wall (`$A9:BC3F`, x): mask Mother Brain's moves on both axes;
  - the gunship, while its function (enemy $D07F's) isn't the idle one (`$A2:A9BD`): it sets her x
    and where the camera saw her across (`$A2:A925` landing, `$A2:AA1D` as she enters) and moves
    her y (`$A2:A8DC`, `$A2:AC2F`, lowering and raising her): mask x and y (integrator, lane/wire-cam:
    an earlier version said y only);
  - the suit pick-ups (`$91:D4E4`, `$91:D5BA`), which centre her on the view across (`$91:D57D`,
    `$91:D580`, `0B10`) and down (`$91:D58A`, `$91:D58D`), and pose change command 4 (`$91:F37C`):
    mask x and y;
  - Samus's appearance (`$90:E86A`, writes `0B14`; her appearance when a game loads, owned by the
    game-state domain): mask y;
  - the cutscenes of bank `$8B` (`$8B:AEFB`..`$8B:AFC4`) and the door transitions of bank `$80`/`$82`
    (outside game state 8 or at a room change, so already skipped).
  Writers of the camera after its turn: `07E9` ≠ 0 (the scrolling-finished hook, layer 1 Y only:
  mask y, keep x), the room's main code (`$8F:C1E6`, the Dachora room's scroll right, x;
  `$8F:E8CD`, Crocomire's room shaking, x), the Shitroid waiting for the camera (`$A9:EFE6`, x,
  and it closes two screens).
  Frames samus-motion's harness defers (adversary 42, C2 and M4), as one rule: y masked where
  samus-motion defers y (a block-inside push, the quicksand's, C2; the knockback's tail, a frame
  starting with the knockback on under the normal handler, C3) and either her pose changes or
  samus-motion's step reports a placement. The camera takes her position from N+1, and from
  samus-motion only the placement (its `placed`); on a
  deferred frame without one, nothing the camera reads is unverified, so y stays compared
  (adversary 49, F1: 98 push and 5 tail frames masked, 247 more y compared, every one matching).
  x stays compared on both, since the lead's knockback turn is the camera's own `LeadingHer`.
  A pose change between N and N+1 is not masked: the placement for that frame comes from running
  samus-motion's model on it. Taking N+1's `0B14` instead is wrong: it is her y at the scroll by
  construction, which differs from the placement when she moves on after it (samus-motion's PF1,
  the lift after the push).
- **Null model:** the camera doesn't move (6,271 of 14,890 frames).
- **The push's 2 px: vectors only** (adversary 49, O3). No recorded frame backs off short of an
  edge: over the 32 rows, `camera-r3/o3model.py` (`model.py` with a tag for the push down that
  wins over the edge) finds no right, left, up or down push and no settle that stops short of its
  edge (the `*-red-back`, `*-red-step` and `*-red-both` tags, 0 each); every push the records
  reach ends on the edge. The push's figure (`ClosedScreens`, `PushCarries`, the settles' ease) is
  checked by the constructed vectors alone (C-push-*, C-ease-*, C-ahead, C-wrap).
- **The reads down from where the move took it: vectors only** (adversary 53, F2). No recorded
  frame decides `DownwardLooksAhead`: reading the screens from where the aim stopped the move
  leaves every row's output the same. In R3 and R16 the move crosses into the last row past its
  aim, but the view stops on its aim under either read. It is checked by C-ahead,
  C-below-closed and C-below-short-row, and the bank log (`$80:A8A6`..`$80:A8D4`).
- **k out of sample:** the old code scores 2,273.5 over this spec's 47.0 units, 48.4 a unit
  (samus-motion's 65.8); its own 0.55 would give k = 26.6 against the frozen 36, so the 70% cap
  binds in both areas (limits at 31.3 units). The old code lacks the knockback's
  turn of the lead and time stopped.
- **Free run:** from a seed frame, carry the camera's own State forward and take only the inputs
  from the records; report the first differing frame per axis. Re-seed at a room change.
- **Per-behaviour attribution:** `model.py`'s tags name the branch each frame takes; the entries
  each tag exercises are in `gen.py`'s `entries_of`.

## Size baseline (tokens, 2026-11-08)

`tools size` (the quality-terms counter, `$TEMP/collab/sizedll-quality`), at spine `cc505407c`,
over cuts of the old code in `$TEMP/collab/camera-spec/old`, measured together so the clumps are
counted across both:

| Area | Old code | Score | Logic | Types | Data |
| --- | --- | ---: | ---: | ---: | ---: |
| follow | `scrolling.rs`: `speed`, `ONE_PIXEL`, `moved`, `follow`, `horizontal`, `vertical`; `state/room.rs`: `CameraDistance::lead` | 715 | 763 | 18 | 16 |
| limits | `scrolling.rs`: `ScrollAreas` and its methods, `screen`, `middle_screen`, `screen_start`, `next_screen`, `negative`, `before`, `ahead`, `behind`, `settle_x`, `scroll_right`, `scroll_left`, `bottom_rows`, `settle_y`, `settled_y`, `scroll_down`, `scroll_up` | 1,558.5 | 1,586 | 41 | 0 |
| total | | 2,273.5 | 2,349 | 59 | 16 |

Not counted: `layer2` and `scroll_backgrounds` (252, layer 2 and the BG scroll registers, the
background's domain), the `CameraDistance` and `Layer2Scroll` codes' `TryFrom`/`From` (the
decoder's), and `tick.rs`'s building of `ScrollAreas` (the bridge's). The spec's header states
limits' old code as 1,559.

## Budget

Moved from the spec's header (MODEL §1 decision 10): the implementer is not told the score or
the budget. `tools spec-units` reads k and the areas here, and `tools report` the areas' old
figures and the commit.

Size is counted in logical tokens: each identifier, keyword, literal and operator is one, and so is
each bracket pair; comments, layout and tests are free. Two kinds of token are counted apart and
reported, not held against the target: the values of constants and statics (tables of the game's
data, such as the lead distances), and declarations (the `struct`, `enum` and `type` items and
attributes that define the model's shapes; define as many as the behaviour needs). The target is on
everything else, the logic.

**Budget (MODEL §1 decision 7).** k = 36 weighted tokens per spec unit, frozen on samus-motion and
tested here out of sample. An area's budget is k × its units, capped at 70% of its old code (`old`,
the `tools size` score of the old code that area replaces: `game/src/scrolling.rs` without its
layer 2 and background scroll, and the lead distances' function in `state/room.rs`, at spine
`cc505407c`). The old code lacks two of this spec's rules, the knockback's turn of the lead and time
stopped, so `old` slightly understates the domain. The budget is an indicator: the fit is score ÷
budget, and being over it costs in proportion to how far over, never as a wall; over an area's old
code is a failure.

- Area follow (old 715): FollowingHer, LeadingHer, AimingHeight, Remembering, CreepReadsRightward
- Area limits (old 1,559): ClosedScreens, RoomEdges, ShortScreens, Settling, TimeStopped,
  DownwardLooksAhead, ScreensRunOn, BelowTheRoom, PushCarries

Budget: 36 × 47.0 units, capped at 70% of old = 1,591 (follow 500 capped, limits 1,091 capped)
