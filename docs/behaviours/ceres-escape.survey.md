# Ceres from Ridley's flight to the landing on Zebes: observer survey

For the root, observers and integrators. Not a spec and not part of any export: it names the
original's routines where they pin a mechanism, as decode notes do. It says what plays from
Ridley's flight (`chain-any-0006-E0B5` cut 867) to the milestone's frame 15,196, who owns each
part, what is already specced or played, what is missing, and the order of spec rounds.

## How it was made

Behaviour first, from CI's lean record cache (`$TEMP/collab/ci-cache/rows`): rows
`chain-any-0006-E0B5` (from cut 860) to `chain-any-0011-DF45`, read per frame by a scratch reader
of the lean tier (`$TEMP/collab/escape-survey/scripts/`, local only): the game state, the room,
the station's stage, the timer's status, digits and place, the quake's type and frames left, the
mode 7 flag, the cinematic step, the FX type, every enemy slot's species and first parameter,
the enemy projectile and PLM slots, the HDMA objects, her handlers, pose, position and energy,
Ridley's slot (function, position, properties) and the music track. The bank logs
(https://patrickjohnston.org/bank/: 80, 82, 84, 86, 88, 89, 8B, 8F, 90 and A6; local copies
`$TEMP/collab/escape-survey/ref/`) then named the steps. The played runner
(`tools replay played`) was run from each row's seed.

**Frames.** A row's cut `c` is movie frame `seed + c`, as `scenarios/replay/stretches.toml`
counts them: `chain-any-0006-E0B5` 10,163, `0007-E06B` 11,940, `0008-E021` 12,199,
`0009-DFD7` 12,463, `0010-DF8D` 12,794, `0011-DF45` 13,073 (its cut 2,123 is 15,196, the
milestone, where the game turns to loading the Landing Site). The stretch "The escape"
(11,773–13,236) starts at room 6's cut 1,610, as she touches its door; the self-destruct
sequence before it is in the stretch "Ridley's room" (#753).

**Where the run stands.** Played alone, room 6 holds 867 cuts and stops at his take-off
(`RidleyFlying`). From their seeds, rooms 5 and 4 (0007, 0008) play their whole rows, passages
included; room 3 (0009) holds 136 cuts and room 2 (0010) 65, each stopping in `movement.pose`
where a steam puff touches her. The shaft's seed (0011) is taken, and its first frame is refused
by `Fault::Escape`, the shaft's overlay in the room; a seed in the rotating shaft is refused.
Each escape row can be worked from its own seed before room 6 joins it (#757's dependency
note).

## 1. Room 6 after the tail: Ridley's part 4c (cuts 867–1,609, movie 11,030–11,772)

| Cuts | What plays | Owner |
| --- | --- | --- |
| 867–993 | He takes off (rises fast, then slows), then hovers near the room's top right, his tail swinging; its tip hits her at 897 and 993 (both from the left), -15 energy and a knockback each | B8 4c-1a: his flight; 4c-1b: the tail in flight (the tip for 4b's `TailHits`); S4 knockback, hurt colours, H1 (as at 609–801) |
| 994–1,092 | She is down to 24: he gives up, rises straight off the top of the screen (to 128 px above the room) | B8 4c-2: the retreat |
| 1,093–1,157 | A hold of 64 frames; then his colours on the background and the walls' sprite row change, and the two getaway walls are placed at the room's sides (enemy slots 2 and 3) | B8 4c-2; the walls' running is already specced (`GetawayWalls`) |
| 1,158–1,269 | The getaway: stage `Getaway`; the screen turns to mode 7 and he flies away as a background picture, turning, zooming and moving by a table per frame, his wings and the baby animated, his colours dimming with the zoom; the getaway sound at its first frame. At its 105th frame (1,262) a strong shake of the back layer for 64 frames, and she is pushed out of his way (1,262–1,264: knockback pose, raised, pushed sideways until she meets a wall) | B8 4c-3: the getaway path; R3 the shake; S2 the push (samus-movement); E4 the plane |
| 1,270–1,585 | Back to the room's mode. The self-destruct sequence: the text colours are set; the timer's sprite and background tiles are uploaded over several frames; "EMERGENCY" is drawn and glows; the escape music is asked for (1,293); after 128 frames "SELF DESTRUCT SEQUENCE" is typed letter by letter (the English path skips the Japanese text) | B8 4c-4: the self-destruct sequence; E2 its tile writes; U1 the music request |
| 1,586 | The escape starts: stage `Escape`, the station's boss beaten, the timer set to one minute. The door of his room is let go and the walls hide (already specced: `LockedIn`, `GetawayWalls`, R-released, W-hides); each door's run now keeps the station shaking (`EscapeShakes`, `Back` in his room) | B8 4c-4 (the start); P1 `BossesDefeated`; H2 the countdown |
| 1,586–1,609 | The timer waits 16 frames, then runs; "EMERGENCY" keeps glowing while she is in the room | H2 countdown; B8 4c-4 (the glow) |
| 1,610–1,776 | She leaves by the door; the passage to room 5, the timer drawn and frozen | R1 world-doors (already draws the timer through a passage, `WhatRuns`, `TheFades`) |

Ridley's function runs the self-destruct sequence, so 4c-4 is B8's, not the HUD's; the HUD
owns the countdown from the frame it is started. Her energy does not change after 993. Two steam puffs touch her later (`chain-any-0009-DFD7`
cut 137, `chain-any-0010-DF8D` cut 66): each is a hit worth no energy, so she is knocked back,
flashes and is protected as after the tail's hits. No debris or quake touches her.

## 2. The escape, rooms 5 to 1 (movie 11,773–13,236)

Every room: the stage `Escape`; the timer runs during play and is frozen (drawn) during each
passage; the doors keep the station shaking (`EscapeShakes`: a 2-frame 1 px diagonal shake, now
and then a 4-frame 3 px one, of the whole screen); the haze in its red, Ridley-dead colour
(world-haze already takes any of the three colours); the rooms load their versions for Ridley
beaten (P1 `RoomStates`). Lag: the passages' overruns (borrowed until T1).

| Row, cuts (movie) | Room | What plays beyond the common part | Owners |
| --- | --- | --- | --- |
| 0007 0–95 (11,940–12,035); passage 96–258 | 5 (two doors) | the timer's slide into place begins at cut 71 (status "moving into place"): its sprite moves from off screen to its corner | H2; R3; B8 doors |
| 0008 0–99 (12,199–12,298); passage 100–263 | 4 (two doors) | the slide ends at cut 81 ("moved into place") | as room 5 |
| 0009 0–168 (12,463–12,631); passage 169–330 | 3 | five steam vents (species steam, four facings): each waits a random 1 to 32 frames from its placing, then shows its puff (seven 3-frame figures), hides 64 frames, and repeats, while it is in view; a puff touches her at cut 137: a hit worth no energy (knockback, hurt colours, protection) | B8 part 5: steam (with C1's running, drawing and contact); G2 one draw per vent at its placing; S4 knockback, hurt colours, H1 (hit worth 0) |
| 0010 0–111 (12,794–12,905); passage 112–278 | 2 (the pre-elevator hall) | eleven steam vents (one touches her at cut 66, as in room 3); falling debris: from cut 34, every 9 frames a piece of ceiling (light or dark by the random number's top bit) falls from y 42 at one of sixteen x places chosen by its low 4 bits, accelerating; landing on a block it becomes a small dust cloud with the falling-tiles sound | B8 part 6: debris and dust (with C1's projectile runner); G2; S4 knockback, hurt colours, H1 (hit worth 0) |
| 0010 passage into the shaft | 2 → 1 | the door into the shaft sets the shaft's mode 7 (identity, rotation index at its middle, 60-frame hold); in the passage's frames of work the shaft's overlay starts its rumble (dust clouds from cut 241) | R1 (the way into the shaft, now refused as a `DoorRule`); B8 part 7 |
| 0011 0–163 (13,073–13,236) | 1 (the shaft) | the overlay's rumble: 48 frames of explosions at four offsets round the door, one every 5 frames (small explosion or smoke by the random number), each with the big-explosion sound; then it hides and the shaft starts rotating (stage word's rotate flag, cut 34); the background swings by a table, back and forth; the beacon and the platform go on as before the escape (`BeaconCycles`/`PlatformTurns` say "no step" in the escape: open question 4 of the spec); eleven steam vents placed at the rotated room's positions; the door to room 2 is turned to solid blocks as the room is set up (PLM); at cut 164 she stands on the pad, still, near its middle: "made it to the elevator" | B8 part 7: the shaft in the escape; E4 the rotation's plane; W2 the PLM; R2 the room's set-up |

The countdown never runs out in this movie (49.14 s left at the elevator); its expiry is
original-only.

## 3. The explosion (movie 13,237–14,076)

| Row 0011 cuts (movie) | What plays | Owner |
| --- | --- | --- |
| 164–223 (13,237–13,296) | Made it to the elevator: she is locked; play goes on (the quakes, the shaft, the timer drawn but stopped) for 60 frames | the station's end: new F4 piece, or B8 part 7's last rule (open question 3); samus-movement for "locked" |
| 224–239 (13,297–13,312) | Blackout: play goes on while the screen fades out, 16 steps | same; the fade is the shared screen fade |
| 240 (13,313) | The game saves (SRAM, P2), the stage and the timer are cleared, the music stops and three silences are asked for | P2; U1 requests; the explosion scene's set-up |
| 241–1,003 (13,314–14,076) | Ceres goes boom: the scene is set up and waits for the music; small explosions fade in; the station explodes in growing explosion sprites (three spawners, the last a final explosion), asteroids and stars drift; the gunship flies away from the station; a 192-frame hold; the fade out | new F4 model `screens-explosion`; E4 the plane (screens-flight's `PlaneTransforms`, to be lifted); E1 the sprites; R4 any colour cycles; U1 |

## 4. The flight to Zebes (movie 14,077–15,196)

| Row 0011 cuts (movie) | What plays | Owner |
| --- | --- | --- |
| 1,004–1,045 (14,077–14,118) | the scene set up, a fade in, a mosaic transition | new F4 model `screens-zebes` (or a second scene of screens-flight, open question 4) |
| 1,046–1,429 (14,119–14,502) | Zebes and its stars placed; "PLANET ZEBES" fades in and out | same; E1 sprites; the label's fade |
| 1,430–1,984 (14,503–15,057) | the gunship flies toward Zebes: drifting right, turning left, drifting left (the same plane rule as the flight to Ceres) | same; E4 |
| 1,985–2,122 (15,058–15,195) | a 64-frame hold, then the scene slides away | same |
| 2,123 (15,196) | the game turns to loading the Landing Site: the milestone | the next milestone |

## 5. What exists, and what is missing

| Owner | Exists (spec or played) | Missing for this stretch |
| --- | --- | --- |
| B8 boss-ceres-ridley | doors incl. `EscapeShakes`, `TheDoorsColours` (escape set), `GetawayWalls`, `LockedIn`; the shaft's pad, cover, beacon and platform before the escape; Ridley placed, waiting, waking (4a); the tail at rest and its hits (4b) | 4c-1a his flight; 4c-1b the tail in flight; 4c-2 retreat; 4c-3 getaway; 4c-4 self-destruct and the escape's start; part 5 steam; part 6 debris and dust; part 7 the shaft in the escape (rumble, explosions, rotation, the beacon in the escape, the elevator reached) |
| R3 world-quake | the shakes: `QuietQuakePhase`, `ShakeStarts`, `ShakeCountsDown`, `ShakenLayers`, `ShakenActors` | only the getaway's start of the back-layer shake, which B8 4c-3 raises |
| H2 interface-hud | the countdown: `EscapeCountdown`, `CountdownBegins`, `CountdownWaits`, `CountdownTicks`, `CountdownRunsOut`, `CountdownSlides`, `CountdownShown` | only the start, which B8 4c-4 raises (the played runs take it from the records) |
| P1 run-progress | `BossesDefeated` (R-ceres-boss), `RoomStates` | the station's stage as State (Calm, Getaway, Escape, plus the shaft rotating) has no owner: today "held from the seed, but where Ridley's placing sets it" (open question 1) |
| R1 world-doors | passages, the timer drawn through them | the way into the shaft (mode 7 set-up), refused today as a `DoorRule` |
| R2 room loading | Ceres rooms' versions | placing steam (species refused today as other actors), the shaft's escape set-up (its door's PLM) |
| W2 world-blocks | block reactions, scroll cells | the PLM that turns the shaft's door into solid blocks during the escape |
| S2 samus-movement | knockback | pushed out of Ridley's way (3 frames, 1,262–1,264) |
| S4, samus-appearance, H1 | the hit worth `d`, knockback, hurt colours | nothing new: the tail's hits at 897 and 993 and the puffs' at 0009 137 and 0010 66 |
| G2 game-random | `AdvanceRandom` | the steam's and the rumble's extra draws are the consumers' |
| F4 | screens-flight (to Ceres), screens-arrival | the elevator's end and blackout, the explosion, the flight to Zebes |
| P2 | — | the save at 13,313 (the spec is P2's first piece of Ceres) |
| U1 | audio-transport | the requests of the stretch (getaway, music, explosions, silences), compared at frame cuts |
| E1, E2, E4, T1 | the sprite and tile encoders; screens-flight's plane rule | the timer's sprite; the self-destruct text; the getaway's, the shaft's, the explosion's and the flight's planes; the stretch's lag (borrowed) |

## 6. Proposed order of spec rounds

Ordered as the player meets it (domains.md §4, "Ceres in the order it plays"), each by
mechanism and verifiable on its own from its row's seed. Rounds on different domains run in
parallel; the order says which lengthens the back-to-back run first.

The quake's shakes and the countdown are on main; rooms 5 and 4 play whole from their seeds.

| # | Round (domain, piece) | Movie frames | Depends on | Indexed cases and played runs it adds |
| --- | --- | --- | --- | --- |
| 1 | B8 part 5: steam | 12,463 on | C1 running and drawing; the shared hit (samus-health's hit worth 0, knockback, hurt colours, on main) | vectors; played: `chain-any-0009-DFD7` 0–168 |
| 2 | B8 part 6: falling debris and dust (with C1's projectile runner) | 12,828 on | the projectile runner the pad uses | vectors; played: `chain-any-0010-DF8D` 0–111 |
| 3 | B8 part 7: the shaft in the escape (rumble and explosions, rotation, the beacon in the escape, the elevator reached); with R1's way into the shaft and W2's door-to-blocks PLM as integrator wiring | 12,906–13,236 | 2 (dust clouds), E4 for the plane (compared as the matrix) | vectors; played: `chain-any-0010-DF8D` 112–278 and `chain-any-0011-DF45` 0–163 |
| 4 | B8 4c-1a: his flight | 11,030 on | 4b | played: room 6 to 896 |
| 5 | B8 4c-1b: the tail in flight | to 11,156 | 4; 4b, S4, hurt colours | played: room 6 to 993 |
| 6 | B8 4c-2 and 4c-3: retreat, walls, getaway path; S2's push as a samus-movement amendment in parallel | 11,157–11,432 | 5; R3; E4 | played: room 6 to 1,269 |
| 7 | B8 4c-4: the self-destruct sequence and the escape's start | 11,433–11,772 | 6; H2 for the countdown; U1 request | played: room 6 to 1,610 and on through the passage into 0007 |
| 8 | F4: the station's end (elevator reached, blackout) and the save (P2) | 13,237–13,313 | 3 | played: 0011 164–240 |
| 9 | F4 `screens-explosion`: Ceres goes boom | 13,314–14,076 | screens-flight's plane rule lifted to E4 | a seeded row at the scene's start (cut 241), as the flight to Ceres was |
| 10 | F4 `screens-zebes`: the flight to Zebes | 14,077–15,196 | 9's lifted plane rule | a seeded row at cut 1,004 |

## 7. Open questions for the root

1. **Who owns the station's stage** (Calm, Getaway, Escape, and the shaft rotating)? It is
   written by Ridley (getaway, escape start), the shaft's overlay (rotate) and cleared by the
   explosion's set-up, and read by the doors, the haze's colour, the debris and the shaft.
   Proposal: P1 carries it as progress State; B8 and F4 raise the events.
2. **The countdown's start**: B8 4c-4 raises the countdown's start, H2's input event
   (interface-hud §5.1).
3. **The elevator's end (gs 20/21)**: B8 part 7 (the shaft decides she made it), or F4 (the
   station's end, with the blackout and the save)? domains.md puts "the elevator's end" in F4.
4. **The flight to Zebes and the explosion**: new models (`screens-explosion`,
   `screens-zebes`) or later pieces of screens-flight? screens-flight's header says its plane
   rule is to be lifted to E4 when the second scene needs it: that lift is a platform lane
   before round 9.
5. **Room 6 joins room 5**: the passage out (1,610–1,776) needs the stage `Escape` and the
   countdown frozen in a passage, which world-doors already draws; nothing new beyond the self-destruct round.
