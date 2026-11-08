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
(the transitions of bank `$91` that set `0B14`, which is samus-motion's `camera_from`), 69 are the
shinespark's clamp (`$90:D1D6`..`$90:D1FE` and `$90:D2A3`..`$90:D2B9`, previous position within
15 px across and 14 px down), and the rest the grapple (`$9B:BB25`.., the ±12 px clamp) and the
Landing Site's gunship (`$A2:A925`). With the y the camera saw taken as her y at the scroll, the
113 pose changes match too.

`gen.py` picks the recorded vectors (at most two per outcome, from different rows, from frames
the model matches, with `0B10`-`0B16` at N+1 equal to her position `0AF6`-`0AFC`, outside movement
types 16h, 1Ah and 1Bh: grapple, grabbed by Draygon, shinespark; the one it picked from
samus-square-slope-reserve was dropped, its row name being a lexicon match), `cons.py` writes the
constructed ones (the results come from the same model) and `vecrun.py` runs every vector of
`scenarios/vectors/camera.toml` through it: 64 of 64 agree. `rise0.py` writes R35, the one
recorded rise at 0 (`0B36` = 1 with `0B2C`/`0B2E` = 0 at the scroll): all 7 such frames are
shinespark frames (movement type 1Bh), and in samus-shinespark-left 214 the clamp didn't fire
(the model matches with the previous position as record N left it) and the down scroller decides.

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
| rising | `0B36` | = 1 (up); 0 (none, the moonfall's) and 2 (down) are not rising |
| her start y | `0AFA`.`0AFC` of record N | as is |
| `camera_from` | samus-motion's output | its pixel part is what the placements write to `0B14` (the pixel word only; `0B16` is kept); where it equals her start y's pixel part nothing is written |
| the room's screens | `$7E:CD20`, 32h bytes, row-major, `07A9` across (`w`), `07AB` down (`h`) | 0 `Closed` (red), 1 `Short` (blue), 2 `Open` (green); the bytes from `w × h` on are the `below` rows, `w` to a row, any value but 0 and 1 read as `Open` (only 0 and 1 are ever tested) |
| the aim heights | `07AD` (up scroller), `07AF` (down scroller), from the room header (`$82:DEC1`, `$82:DECA`) | `hn` = `07AD`, `hr` = `07AF`; 70h/A0h or 90h/A0h in the records |
| lead setting | `0941` (camera distance index) | 0 `Ahead`, 2 `Near`, 4 `KeepLeft`, 6 `KeepRight`; the tables `$90:963F` (facing right) and `$90:9647` (facing left): 60h/A0h, 40h/50h, 20h/20h, E0h/E0h. Set by Crocomire (`$A4:8ABD` = 2, `$A4:97F6` = 6) and Kraid's room (`$A7:A9E7` = 2), cleared at `$88:8347`, `$A4:90D8`, `$A4:9B8D`, `$A7:C594`; always 0 in the records |
| time stopped | `0A78` | ≠ 0 (time frozen: x-ray, the reserve tanks' refill); set in 704 record frames of six rows; in the 95 that scroll, none where an ease would have moved |

The spec's enum orders are alphabetical: `Closed`, `Open`, `Short` against the RAM's 0 red, 2
green, 1 blue; `Ahead`, `KeepLeft`, `KeepRight`, `Near` against 0, 4, 6, 2.

## Sources per entry

Each entry lists the routines or branches that carry its content. `scan.sh --mirror` finds 2 of
14 entries (14 %) single-routine, `LeadingHer` and `AimingHeight`; `DownwardLooksAhead` is one
routine's two reads. All three are marked *.

| Entry | Source |
| --- | --- |
| FollowingHer | `$90:95A0` and `$90:964F` (moved: `0B10` ≠ `0AF6` by pixel parts, `$90:95A3`, `$90:9652`; at the aim: `$90:9606`, `$90:9687`; the move: `$90:960D`..`$90:9636`, `$90:968E`..`$90:96B7`), `$90:96C0` and `$90:96FF` (the speeds, plus `$90:9EAD` = 1), the aim stops at `$80:A651`, `$80:A6CB`, `$80:A8DA`, `$80:A936`'s start (pixel set, subpixel `STZ`) |
| LeadingHer* | `$90:95B2`..`$90:9600` with the tables `$90:963F` and `$90:9647` |
| AimingHeight* | `$90:9666`..`$90:9681` |
| Remembering | `$90:956E`..`$90:9583` (after both axes); the placements (samus-motion's `camera_from`): bank `$91`'s writes of `0B14` in pose setting, pose initialisation and the block collision due to a pose change (`$91:E340`, `$91:E44A`, `$91:E50A`, `$91:ED0E`, `$91:F595`, `$91:FF42`, `$91:FF6F`, `$91:FF93`..`$91:FFE9`; `$91:80AB` in the crouching pose input; `$91:F3A1`, super special pose change command 5, y + 5), and `$90:ECB1` (aligning her bottom with the old pose). `$90:92E4` is in an unused routine (`$90:92D6`). Pose change command 4 (`$91:F37C`: `$91:F386` x and `$91:F393` y, her position set to the view's centre) is not samus-motion's: masked |
| CreepReadsRightward | `$90:96C0` and `$90:96FF` (the direction by `CMP`/`BMI` on the pixel parts, the bank log's note at `$96C0`), read by `$80:A58F` and `$80:A7D3` |
| RoomEdges | `$80:A662` (`(07A9 − 1)` swapped, the last screen), `$80:A6D9` (negative to 0), `$80:A8EB`..`$80:A8FA` (the bottom with `0933`), `$80:A936`'s negative test; the back-off at the bottom `$80:A91E` |
| ClosedScreens | `$80:A671`..`$80:A6B5` (right), `$80:A6E3`..`$80:A72B` (left), `$80:A8FC`..`$80:A930` (down), `$80:A936`.. (up); the row and column from `0915 + 80h` and `0911 + 80h` through the multiplier `$4202`/`$4203` with `07A9` |
| ShortScreens | `$80:A746`..`$80:A77A` and `$80:A8A3`..`$80:A8D7` (`0933` = 0 when the upper scroll is blue, else 1Fh) |
| Settling | `$80:A528` (across) and `$80:A731` (down): the clamps `$80:A53D`..`$80:A552` and `$80:A77D`..`$80:A796`, the left/upper red case `$80:A57E`..`$80:A5D7` and `$80:A7C2`..`$80:A81B`, the right/lower red case `$80:A5D9`..`$80:A638` and `$80:A81D`..`$80:A88A` |
| TimeStopped | `$80:A52C` and `$80:A735` (time frozen `0A78`/`0A79`: return) |
| DownwardLooksAhead* | `$80:A89D`..`$80:A8D7` (the upper scroll's index `$14` from `0916` after the speed's add, before `$80:A8DA` caps it at the ideal), read again at `$80:A8FC` |
| ScreensRunOn | `$80:A5D9` (`INX`: the right scroll is index + 1), `$80:A686`..`$80:A68D` (`SEC; ADC`: screen + 1 + row × width), `$80:A5B2`..`$80:A5BA` (`INC`), all on the row-major table; the bank log's note at `$80:A528` |
| BelowTheRoom | the scroll table `$7E:CD20` (32h bytes; `$82:EA73` loads `w × h` of them), read past the room at `$80:A8FC`, `$80:A8C8` and `$80:A823` |
| PushCarries | the second `ADC #2` after `ADC 0DA2`/`0DA6` and the second `SBC #2` after `SBC 0DA2`/`0DA6` (carry and borrow kept): `$80:A58F`, `$80:A5F0`, `$80:A6A7`, `$80:A71D`, `$80:A7D3`, `$80:A842`, `$80:A922` and the up routine's |

## Findings behind the spec's choices

- **Positions compare as `before`.** Every compare that decides a branch is `CMP` then `BMI`/`BPL`
  (the sign of the difference), except the room's right edge and bottom (`BCS`/`BCC`), the forward
  pushes' `min` (`BCC`) and the settles' forward reach (`BCS`). Those see only values from 0 to
  the room's far edge, where both agree, so the spec states one compare.
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
- **The settles' third case.** Easing back (`$80:A5FB`, `$80:A84D`), the original tests whether the
  screen the eased position is in is red and, if so, puts it on the next screen's edge. From a
  camera inside the room that screen is the camera's own, which isn't red there, so the spec
  drops it; only a camera past the room's far edge, which no frame starts with, could reach it.
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
  16h (grapple: `$9B:BB25`.., the ±12 px clamp; `$91:F3AA`, the release's clamp) and 1Bh
  (shinespark: `$90:D1D6`..`$90:D2B9`), and `0CF8` ≠ 0 (slow grapple scrolling), both axes.
  Single writers of where the camera saw her, both axes unless named:
  - `$A0:B7A1`, the cap on scrolling speed (`$A0:B7C4` y, `$A0:B7E8` x), called by Draygon (which
    holds her in movement type 1Ah) and the yapping maw: mask while either holds her;
  - Crocomire's collision (`$A4:8C95`); Kraid's body collision (`$A7:B0F3`) and its restriction of
    her x to the first screen (`$A7:C9EE`, x); Ridley (`$A6:E501`); Mother Brain moving her towards
    the ceiling or floor (`$A9:BBFD`, y);
  - the gunship (`$A2:A925`, `$A2:A9BD`, `$A2:AC1B`), the suit pick-ups (`$91:D4E4`, `$91:D5BA`),
    pose change command 4 (`$91:F37C`, x and y);
  - the cutscenes of bank `$8B` (`$8B:AEFB`..`$8B:AFC4`) and the door transitions of bank `$80`/`$82`
    (outside game state 8 or at a room change, so already skipped).
  Writers of the camera after its turn: `07E9` ≠ 0 (the scrolling-finished hook, layer 1 Y only:
  mask y, keep x), the room's main code (`$8F:C1E6`, the Dachora room's scroll right, x;
  `$8F:E8CD`, Crocomire's room shaking, x), the Shitroid waiting for the camera (`$A9:EFE6`, x,
  and it closes two screens).
  A pose change between N and N+1 is not masked: `camera_from` for that frame comes from running
  samus-motion's model on it. Taking N+1's `0B14` instead is wrong: it is her y at the scroll by
  construction, which differs from the placement when she moves on after it (samus-motion's PF1,
  the lift after the push).
- **Null model:** the camera doesn't move (6,271 of 14,890 frames).
- **k out of sample:** the old code scores 2,273.5 over this spec's 42.3 units, 53.7 a unit
  (samus-motion's 65.8); its own 0.55 would give k = 29.6 against the frozen 36, so the 70% cap
  binds in follow. The old code lacks the knockback's turn of the lead and time stopped.
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
