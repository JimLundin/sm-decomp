# Frame behaviour the foundation must reproduce

Research lane `research/foundation-frame` (2026-11-05), for the rebuild in
[REVIEW](../REVIEW.md) "Rebuild on a new foundation". It lists every way the original
game's frame behaviour shows in the outputs we compare (video, APU ports, VRAM, CGRAM,
OAM, PPU registers, SRAM, Session-space fields), and what each demands of the Tick,
the Scene and the effects in [ARCHITECTURE](../target-architecture/ARCHITECTURE.md)
and [CONTRACTS](../target-architecture/CONTRACTS.md).

Sources: the bank logs (`$BB:AAAA` addresses, <https://patrickjohnston.org/bank/>),
the oracle (`crates/sm_host/src/snes`: LakeSnes CPU and PPU running the ROM), and the
existing engine as a map only (`crates/sm_game/src/gameplay_frame.rs`,
`room_entry/protocol.rs`, `dialogue.rs`, `docs/handoff/session-authoritative-tick.md`).
Row ids are `scenarios/corpus.toml` ids. **R*n*** marks a requirement.

## 1. What a Tick is

The oracle steps one **video frame** (one NMI, 262 lines) per Tick, lag frames
included; the corpus input stream already records lag Ticks explicitly
([SCENARIO_HARNESS](../SCENARIO_HARNESS.md), `input` domain "lag flag"). The game's
main loop does not run once per frame: one iteration can span many frames, and
interrupts do frame-paced work while it runs.

- **R1.** The Tick is one video frame, not one main-loop iteration and not one
  gameplay pass. Every Tick produces a complete frame output, including Ticks where
  game logic made no progress.
- **R2.** The Session must be able to suspend game logic at a defined point and
  continue it on a later Tick (a typed continuation in the Session state, saved in
  Checkpoints). Section 6 lists the suspension points; some are inside the gameplay
  pass (an item message box opens from the PLM handler, `$82:8B71`, and the rest of
  the pass runs ~370 frames later: `xray-activate-release`, `item-*`,
  `probe:coroutine.plm.message.completed`).

## 2. Main loop and NMI

Main loop `$82:893D`, one iteration:

1. HDMA object handler `$88:84B9`: first the **music queue** (`$80:8F0C`), the pending
   power bomb spawn, then the six HDMA objects' instructions and pre-instructions,
   which write the HDMA tables in WRAM and the channel-enable mirror `$85`; then the
   layer blending handler `$88:8000`.
2. RNG `$80:8111` (one call per iteration).
3. Clear high OAM, OAM stack pointer, Samus tile-transfer flags.
4. Game state dispatch `[$0998]` (45 states, table `$82:8981`).
5. Sound handler `$82:89EF`, finalise OAM `$80:896E`, previous-input copy `$82:8AB0`.
6. Wait for NMI `$80:8338`: set NMI request `$05B4 = 1`, spin until the NMI clears it.

NMI `$80:9583`:

- **Requested** (`$05B4 != 0`), in this order: OAM DMA from `$0370` (220h bytes) and
  CGRAM DMA from `$7E:C000` (200h) `$933A`; Samus tiles `$9376` (two halves, two parts
  each, from `$071D..$0722`); animated tile objects `$9416`; register mirrors to PPU
  `$91EE` (INIDISP, OBSEL, BGMODE, mosaic, tilemap and tile bases, windows, TM/TS,
  colour math, COLDATA, scrolls, Mode 7 matrix when mode 7; copies `$69→$6A`,
  `$6E→$70`, `$71→$73` for the IRQ); HDMA table pointers from the object queue
  (`$18B4/$18C0/$18D8`); Mode 7 transfers `$8BBA` (only in mode 7); VRAM write table
  and scrolling DMAs `$8C83`; VRAM **read** table `$8EA2`; HDMA enable `$420C = [$85]`;
  controller read `$9459`; then `$05B4 = 0`, lag counter `$05BA = 0`, `$05B5`++ (8-bit)
  and `$05B6`++ (16-bit).
- **Not requested** (lag): none of the above. `$05BA`++, max lag `$05BB` updated.
- Always: `$05B8`++ (NMI counter).

Requirements:

- **R3.** The display update is committed at the frame boundary as one ordered batch
  (OAM, CGRAM, Samus tiles, animated tiles, registers, HDMA binding, Mode 7, tilemap
  queue, readbacks), and only on Ticks whose logic completed. On any other Tick the
  previous batch's state stays on screen.
- **R4.** The Session keeps three counters with exact semantics: completed frames
  (`$05B5`, `$05B6`), all NMIs (`$05B8`) and lag (`$05BA`, `$05BB`). Animation and
  palette timers key on the completed-frame counters. The message-box wait keys on
  the NMI counter.
- **R5. Input latch.** The controller is read at the end of a *requested* NMI, after
  auto-joypad completes. That word (`$8B`), its edge `$8F = $8B & ~$97`, the auto-repeat
  word `$93` (timers `$A3`, delays `$87/$89`) and the previous word `$97` are what the
  *next* main-loop iteration sees. That is the "closing NMI latches the next Tick's
  controller word" finding: `TickInput` carries the word sampled at this Tick's
  vblank (`docs/handoff/gate1-slice.md`, `wall-crawler-slope-step`). A lag NMI reads
  nothing, so a press and release that both fall in lag frames never reaches the game.
  Edges are computed against the last *read* word, not the last frame's.
- **R6.** Controller 2 and the debug branch (`$9497..`; soft reset on L+R+Select+Start
  only when the debug flag `$05D1` is set) are dead in vanilla. The foundation needs no
  soft-reset path for vanilla, but the game-over and file-select quits jump to the
  soft-reset entry `$80:8462`, so a full reset into boot must stay a Session transition.
- **R7. Other input sources.** The main thread also reads input directly, outside the
  NMI: the message box polls `$4218/$4219` in a busy loop (`$85:84A3`), and the save
  confirmation calls `$80:9459` itself every second frame (`$85:84CE`); demos replace
  the pad with recorded words (state 2Ah, `$82:8548`, `demo-attract-setup`). So
  `TickInput` must offer the raw pad state of *this* Tick's vblank, and the Session
  decides which consumer reads it and when. `tas-ceres-cross-frame-read` is a read that
  crosses adjacent video frames.

## 3. Lag frames

There are two kinds, with different causes and different visible results.

**Structural lag.** The code loops or blocks without reaching `$80:8338`: message
boxes (`$85:8136` waits on `$05B8`), door loading, music data uploads, pause loading,
`$80:8577`-style blocks. The count is fixed by the code (for example 2 frames
`$85:8574`, 360 or 10 frames `$85:8493`) or by a handshake (sections 7 and 8).

**Overrun lag.** One iteration's CPU time passes the next vblank: busy rooms, power bomb
table maths (Map Rando tech `canUsePowerBombLag`), decompression in loading states.
The count depends on 65816 cycle costs. In the oracle it falls out of LakeSnes timing.

What a lag frame shows:

- No NMI batch: OAM, CGRAM, scroll and other registers are the previous ones, and the
  HUD IRQ restores the stale `$6A/$70/$73` mirrors.
- HDMA still runs from the tables in WRAM. The handler rewrites them at the *start* of
  each iteration (in vblank). So a lag frame shows **new HDMA tables over old
  registers and OAM**.
- In LakeSnes the beam reads HDMA tables line by line while main code runs
  (`advance_beam`: line rendered at hPos 512, HDMA at 1024). A table written by
  overrunning code mid-frame tears at the line the beam had reached.

Requirements:

- **R8.** The Scene must separate *NMI-committed display state* (registers, OAM,
  CGRAM, video memory) from *per-line programs* (HDMA tables), which have their own
  commit point (the start of the iteration). A lag Tick re-presents the former and
  takes the latest of the latter.
- **R9.** Structural lag counts are behaviour: encode them as typed sequence steps
  (`Wait(n)`, `WaitNmi`, `WaitUntil(flag)`), as `dialogue.rs` does for message boxes.
- **R10. Overrun lag needs a policy before code is built.** Without a CPU, the
  engine cannot derive it. The options:
  (a) a calibrated cost model: per-routine cycle budgets checked against the
  oracle's lag log (`tools/lsnes_lag_log.lua` against bsnes; the oracle itself);
  (b) lag as recorded input: the replay's explicit `-` Ticks tell the Session
  "this frame overran". The Session then repeats display state (R8) and holds the
  iteration. This matches every corpus row cheaply but gives no lag in live play;
  (c) no overrun lag in the vanilla profile, with a LEGACY entry and video tolerance
  on those Ticks.
  Gameplay, RNG and IGT do not depend on overrun lag (one RNG call and one IGT
  increment, `$82:DB69`, per *iteration*), so (b) or (c) keeps gameplay exact. Video
  and Tick count on lag-heavy rows (power bombs, Kraid, Draygon, Mother Brain) do
  depend on it. This is a Reference-profile decision (VALIDATION Gate 0).
- **R11.** Overrun timing also decides whether the door IRQ interrupts main-thread
  multiplications (the hardware multiplier is clobbered by `$80:AE4E`; comment at
  `$80:9771`, only `$82:E91C` guards it). If the oracle shows such a corruption on a
  corpus row, the foundation needs (a) or a recorded marker. Unknown; see open
  questions.

## 4. IRQs, HDMA and per-line display

The IRQ handler `$80:986A` runs interrupt command `[$AB]` (table `$80:9616`). Each
command returns the next command and the next v/h target, always h = 98h. That is
dot ~231 on hardware, with jitter 95h..A3h. In LakeSnes the line is already rendered
at hPos 512 by then, so a write takes effect from the next line.

| Commands | Lines | Effect |
| --- | --- | --- |
| 4 / 6, main gameplay | 0, 1Fh | HUD: BG3 base 5Ah, TM = BG3, colour math off; at 1Fh restore `$5B,$6A,$70,$73` |
| 8 / Ah, door start | 0, 1Fh | HUD; then TM = sprites, or BG1+sprites (by CRE bitset) |
| Ch / Eh, Draygon | 0, 1Fh | HUD without the BG3 base change |
| 10h/12h/14h, vertical door | 0, 1Fh, D8h | HUD; at 1Fh **door scrolling** `$80:AE4E` and the deferred VRAM update; at D8h sets `$05B4 = 1` |
| 16h/18h/1Ah, horizontal door | 0, 1Fh, A0h | as above, with the VRAM update at A0h under **forced blank** (`$80:9632`) |

HDMA objects (6 channels, `$88:8435`) target BGMODE `$2105` (Ceres Ridley and
elevator: mode 7 playfield, mode 1 HUD and floor, `$88:D8DE`, `$88:D928`), BG1/BG2/BG3
scroll `$210D..$2112` (water `$88:C3FF`, lava and acid `$88:B279`, rain, spores, fog,
scrolling sky `$88:A7D8`, Tourian statue, the HUD's BG3 scroll `$88:D865`, the message
box BG3 Y scroll `$85:8363`), windows `$2126..$2129` (x-ray `$88:86EF`, power bomb and
crystal flash `$88:8AA4/$A2A6`, Mother Brain rainbow, suit pickup), TM `$212C`
(Draygon `$88:DF34`), CGADSUB `$2131` and COLDATA `$2132` (Bomb Torizo and Ceres haze
`$88:DD32/$DDC7`). None targets the Mode 7 matrix, which is set per frame (NMI) and
per Mode 7 transfer.

- **R12.** The Scene's display description is a **per-scanline program**: per line,
  the effective BG mode, scrolls, window edges, TM/TS, colour math and backdrop colour,
  formed from the frame base plus HDMA streams plus IRQ change points at line
  granularity. A per-frame register snapshot cannot express the HUD split, door
  scrolling, water, or Ceres mode switching (`boss-ceres-ridley`, `area-ceres`,
  `door-transition`, `elevator-up`, `volcano-room-lavaquake`, `xray-scan-*`,
  `samus-crystal-flash`).
- **R13.** Line granularity matches the oracle. Hardware applies the 1Fh writes
  mid-line (~dot 231), so a hardware-exact image would need (line, dot) events. Record
  the oracle's line rule as the Reference profile choice and list it in LEGACY.
- **R14.** Forced blank must be expressible for a line range inside a frame (the door
  VRAM update at A0h or D8h), and as a whole-frame state (loading).
- **R15.** Interrupt work is gameplay. During door transitions the IRQ moves Samus,
  layer 1 and the BG scrolls, runs the DMA, and grants the NMI request. The Tick must
  run this scheduled work at its line positions *within* the Tick, ordered against the
  main thread's suspended loading (section 6). It cannot be a side effect of drawing.
  Also, the door IRQ's `STZ $05B4`/`INC` in 16-bit mode zeroes the 8-bit frame
  counter `$05B5` on every door frame. That is observable state (R4).

## 5. Video memory: uploads, budgets, readbacks

- Uploads reach VRAM only in the requested NMI or under forced blank (door IRQ,
  loading with `$51 = 80h`, decompression to VRAM `$80:B271`). Main code never writes
  VRAM during display in vanilla paths we know of. A hardware PPU drops such writes;
  the LakeSnes oracle does not appear to model that.
- The NMI has no transfer budget check. The VRAM write table (`$80:8C83`), Samus tiles,
  animated tiles and OAM/CGRAM (~1 KB fixed) all run every requested NMI. An overrun
  past vblank would corrupt on hardware; we have no evidence of one in the corpus.
- **Readbacks.** The NMI's VRAM read table (`$80:8EA2`) copies VRAM into WRAM. The
  pause (Kraid's BG2: `boss-kraid-pause-*`) and x-ray background (`xray-scan-*`) read
  what was uploaded earlier, so video memory feeds gameplay state.

Requirements:

- **R16.** The Session owns the *content* of video memory that the game reads back or
  depends on (at least the BG2 and FX tilemaps and the readback sources). The renderer
  cannot be the only holder of tilemap state.
- **R17.** The VRAM, CGRAM, OAM and PPU-register streams are SNES-layout. Either the
  engine keeps a layout-exact video memory image (which the D9 typed Scene does not
  imply), or verification projects the typed Scene into that layout, as the RAM to
  Session decoder does for WRAM. One must be decided before the Scene is designed.
- **R18.** An upload has a commit point (an NMI batch, or a forced-blank line in a
  given Tick). The renderer applies it at that point. "Latest Scene wins" is not
  enough, because a lag Tick must keep showing old video memory (R3, R8).

## 6. Multi-frame sequences and their yields

| Sequence | Where | Frame structure |
| --- | --- | --- |
| Door transition | states 9-Bh, `$82:E169..E737`; functions in `$099C` | fade out; load the door header and delete HDMA (`$E2F7`); align the screen (`$E310`); load the room header and decompress level, scroll and CRE data (`$E36E`, overrun lag under black); set up scrolling; place Samus, load tiles, start the IRQ chain at vblank end (`$82:DF69`, `$E3C0`); load sprites, PLMs and enemies; **busy-wait on the IRQ's VRAM update** (`$E50D`, `$E06B`) and **on IRQ scrolling finishing** (`$0931`, `$E526`); one PLM handler pass and wait for NMI; animated tiles; wait for the music queue (`$E664`); nudge Samus; fade in. Rows `door-transition`, `door-loading-lag-nmi`, `tas-ceres-four-doors`, `elevator-*`, `power-bomb-*-entry` |
| Message box | `$85:8080`, called inside the PLM handler | ~375 frames of NMI-lag frames with only the music queue and sound handler per frame (`$85:8574`, `$8493`); direct PPU and HDMA writes; exits on any held input (polled) or a confirmation edge. Rows `item-*`, `pickup-*`, `save-memory` |
| Pause / unpause | states Ch-12h, `$82:8CCF..93A1` | fades; loading under forced blank; BG2 backup by readback; Kraid's unpause hook waits on NMI (`$A7:C2A0`). Rows `pause-menu-tour`, `boss-kraid-pause-*` |
| Death | states 13h-19h, `$82:DC80..DDC7` | per-frame state machine; 15h waits on the music. Rows `death`, `save-death-reload` |
| Reserve auto | 1Bh `$82:DC10` | per frame; races door loading (R-mode, G-mode) |
| Ceres escape | 20h-24h, timer `$80:9DE7` | timer HUD sprites; time-up white or black outs; Mode 7 rooms. Rows `area-ceres`, `boss-ceres-ridley`, `samus-forced-ceres` |
| Intro, Ceres boom, ending, credits | 1Eh/22h/25h/27h, bank `$8B` (`$8B:A35B`, `$8B:D443`) | cinematic objects; nested waits for NMI (`$8B:C11B`, `$8B:D480`); Mode 7 zooms; writes `supermetroid` to SRAM `$80:824F`. Rows `credits*`, `samus-scripted-intro`, `gunship-bobbing-cycle` |
| Boot, logo, file select | `$80:841C`, `$8B:9146` (six waits) | logo frames before the main loop exists; the SRAM check and clear |

- **R19.** The sequence vocabulary needs: wait N frames; wait for NMI; wait for a
  condition that *interrupt work* sets (door scroll done, VRAM update done); wait for an
  APU handshake (section 7); run a sub-step every frame while waiting (music queue
  and sound handler in message boxes); and resume inside a gameplay pass. Each cursor
  goes into the Checkpoint (R2).
- **R20.** Door transitions run two strands in the same Tick: interrupt-driven
  scrolling, VRAM updates and HUD at fixed lines, and the suspended main strand. The
  number of frames is set by whichever finishes last. Model both strands explicitly.
  Do not fold them into one fixed frame count. (The existing engine's "loading
  crosses one lag NMI before the first scrolling IRQ" in `room_entry/protocol.rs` is a
  fixed count. Check it per room against the oracle, R10.)
- **R21. Carry-over is behaviour.** Door transitions keep Samus's speed, pose, blue
  suit, shinecharge, X-ray and reserve state unless code resets them. Map Rando tech:
  `canCarryBlueSuit`, `canCarryFlashSuit`, `canDoorTransitionXMode`,
  `canRightSideDoorStuck`, `canDeepTransition`. G-mode and R-mode come from the reserve
  auto-refill losing a race with room loading, plus X-ray staying active, plus PLM
  queue exhaustion: there is one gameplay frame between the room finishing and the
  reserve trigger (TASVideos threads below). The foundation's transition must keep
  every field the original keeps, with the original's order of "room loaded", "first
  gameplay frame" and "reserve check". PLM and HDMA queue capacities are gameplay
  limits, not engine buffers (CONTRACTS "intentional original-game limit").

## 7. APU protocol and timing

Ports: `$2140` music, `$2141..$2143` the sound libraries 1-3.

- **Sound effects** `$82:89EF`, every iteration (and every message-box frame): per
  library a 5-state handshake. State 0 writes the queued sound. State 1 rewrites it
  until the APU echoes it on the read port (`CMP $2141,x`). State 2 waits 2 frames and
  writes 0. State 3 waits for the echoed 0. State 4 resets. Queues are 16 deep per
  library (`$0656`). `$80:9051` and the variants cap by "max queued", drop in demos,
  during power bombs and when sounds are disabled, and prefer the lower id on
  collision. Downtime `$0686` (8 after a music change) zeroes all three ports per frame.
- **Music queue** `$80:8F0C`, at iteration start (and in message boxes): an 8-entry
  queue with per-entry delays (8 frames minimum, `$80:8FC1`, `$80:8FF7`). A track writes
  `$2140` directly. Music *data* uploads the bank through the blocking IPL-style
  handshake (`$80:8024`, `$80:8059`: wait for AAh/BBh, kick, echo per byte), which takes
  many frames of lag paced by the SPC. Door transitions and death wait on the queue.
- **R22.** The APU's port replies feed back into game RAM (sound state `$0649..`,
  queue drain rate) and into the next writes. The APU port stream is compared per Tick.
  So the *driver's command-acknowledge timing* must be simulated deterministically
  inside Tick time (a sound-driver protocol model owned by, or in lockstep with, the
  Session). It cannot live only in the real-time audio callback, whose clock drifts
  (CONTRACTS "Sound engine and transport"). Samples and mixing can stay in the
  callback.
- **R23.** Music data uploads need a frame cost: frames of lag per upload, from the
  SPC protocol rate and the data size. Calibrate against the oracle as data, not as a
  constant (`music-fir-selector10`, every area row's first room).
- **R24.** Cue order is the APU write order within the Tick: music (iteration start)
  before sound effects (iteration end), libraries 1, 2, 3, then the next Tick's
  echoes. The effect trace keeps the original's queue semantics (caps, priority,
  drops). CONTRACTS' "never silently drop a cue" applies to engine buffers; the game's
  own drops are behaviour.

## 8. RNG

`$80:8111`: `r = r*5 + 111h` (with the high-byte carry quirk), using the **hardware
multiplier** `$4202/$4203`. One call per main-loop iteration (`$82:894F`), plus ~50
call sites in enemy AI, enemy projectiles, drops, rain and bosses (52 `next_random` sites
in `crates/sm_game/src`; banks `$A0`, `$A2..$B3`, `$86`, `$88:D981`, `$90`). Seed 97 at
boot (`finish_boot`).

- **R25.** RNG is one Session value advanced in the original's call order. A lag Tick
  does not call it (per iteration, R10). Its consumers' order (actor order, projectile
  order) is part of the schedule: the Tick's phases (CONTRACTS stage 2-6) must be
  calibrated so that the call order equals `$82:8B44`'s pass order (enemy
  selection, palette FX, Samus current state, Samus/projectile interaction, enemies,
  Samus new state, enemy projectiles, PLMs, animated tiles, collisions, power bomb,
  scrolling, drawing, HUD, BG scroll, room ASM, health and time, room shaking, hurt
  timers).
- **R26.** Map Rando enemy manipulation (`canManipulateMellas`) and TAS routes rely on
  exact call counts. A Checkpoint holds the RNG word. A call-count observation (the
  `rng` domain) is worth having to localise drift.

## 9. OAM and sprites

- 128 entries in the order drawn. `$80:896E` parks unused entries at Y = F0h. Writers
  stop at the 200h end. Draw order (`$A0:884D`: Samus, projectiles, enemies, enemy
  projectiles by layer) is priority: a lower index wins. The per-line hardware limits
  (32 sprites, 34 8-pixel tiles) drop sprites visibly; the existing renderer models
  `Overflow`.
- **R27.** The Scene's sprite list is ordered and bounded at 128, with the original
  overflow drop, and the renderer applies the per-line range and time limits in that
  order. Sprite order is not sortable.
- **R28.** OAM reaches the screen only through the NMI batch (R3). A lag Tick shows the
  previous OAM, so sprites freeze for a lag frame while HDMA effects move.

## 10. Save and SRAM

- Save `$81:8000`: copies `$09A2..$0A01` and the map into `$7E:D7C0`, then writes slot
  words to `$70:xxxx` with a sum checksum at `$70:0000+2s` and `$70:1FF0+2s`, and
  complements at `+8`. It is synchronous inside one frame (the save message's
  confirmation, `save-memory`, `samus-gunship-save`). Load `$81:8085`: on a checksum
  mismatch it **clears the slot** and resets the station and area. File select copies
  and clears (`file-copy-clear`). The game-completed marker `$70:1FE0` ("supermetroid")
  sets the number of demo sets (`$80:8261`).
- **R29.** SRAM is a compared stream at Tick granularity. The save effect must carry
  the SRAM image (or the exact slot bytes and checksums) of the Tick it happened in.
  The in-game save flow does not wait for host durability (the original has none), so
  the `HostResult` save receipt must not change game behaviour in the vanilla profile.
- **R30.** The original's corrupt-slot behaviour (clear the slot) conflicts with
  CONTRACTS' "do not silently reset a corrupt slot". The vanilla profile must
  reproduce the clear for SRAM-format saves. The host-file policy applies to the
  container around them. Decide this and record it.

## 11. Fades, brightness, game-over and display-only state

INIDISP brightness `$51` steps per completed frame (`$80:8924`, `$80:894D`) and is
written in the NMI. Debug "spare CPU" `$82:8AB0` is dead in vanilla. The display
resolution and interlace mirror is `$77`. Overscan is off: 224 visible lines of 262
(NTSC).

- **R31.** The cadence is NTSC, 262 lines, non-interlaced: an exact rational from
  the master clock (21.477 MHz / (1364 × 262) ≈ 60.0988 Hz, minus the oracle's
  short-line handling if any). Pin it in the Reference profile (ARCHITECTURE "Scope").

## 12. Known quirks the trick libraries depend on

| Quirk | Mechanism | Foundation demand |
| --- | --- | --- |
| Frame-perfect windows (wall jump, short charge, mockball, X-ray climb, CWJ, moonfall) | one input read per completed frame, edges against the last read word (R5) | the Tick input model of R5; no input buffering or smoothing |
| Lag-dependent (`canUsePowerBombLag`) | overrun lag gives an input-free frame and freezes OAM | R10 policy; R8 display split |
| Door carry-over, door stuck, X-mode through doors | transition keeps player state (R21) | explicit keep/reset table per field |
| G-mode / R-mode | reserve auto versus room load race; X-ray kept; PLM queue full makes PLMs inert | R20 two-strand transitions; PLM queue limit as behaviour |
| Out of bounds, off-screen movement | camera and collision outside room bounds read whatever the original's level data access returns | collision must define out-of-range reads as the original's (wrapping, or data past the room) and flag them in LEGACY |
| Pause abuse (`canPauseAbuse`, spark interrupts) | the pause states freeze gameplay while timers in some domains continue | the in-game pause is a Game mode, per CONTRACTS; list which domains advance per pause state |
| "Red/green doors appear blue" (Crateria → Red Brinstar) | the last scroll update beats the PLM draw (`$82:E4A9` comment) | R20 ordering inside the final door Tick |
| Door IRQ multiplier clobber | IRQ maths during main-thread multiplication (R11) | open question |

## Decisions this forces on the foundation

1. **Tick = video frame.** Game logic is a resumable program with typed
   continuations inside it, including mid-pass. Checkpoints hold every cursor (R1,
   R2, R19).
2. **Two display commit points.** An NMI-committed batch (registers, OAM, CGRAM,
   video memory uploads, on completed frames only) and a per-iteration per-line
   program (HDMA tables). The Scene carries both, and a lag Tick re-presents the
   former (R3, R8, R18, R28).
3. **The Scene is a per-scanline display program** with IRQ change points, HDMA
   streams, BG mode switches and forced-blank line ranges, at oracle line granularity
   (R12-R14).
4. **Interrupt work is scheduled gameplay inside the Tick** at line positions: door
   scrolling, door VRAM updates, NMI-request grants (R15, R20).
5. **Video memory content is Session state** wherever the game reads it back. The
   VRAM, CGRAM, OAM and PPU streams need a layout-exact image or a verification
   projection (R16, R17).
6. **The sound-driver handshake runs in Tick time**, deterministically, apart from the
   real-time mixer. Music uploads have a modelled frame cost (R22, R23).
7. **Original limits are behaviour**: sound queue caps and drops, 128 OAM entries, 6
   HDMA objects, the PLM queue (R24, R27, R21).
8. **Lag policy is a Reference-profile input**, chosen before the spine (R10).

## Open questions

1. Overrun lag: cost model (a), recorded lag (b), or none (c)? Is (b) acceptable for
   Gate 2/3 rows and the movies?
2. Per room and door, does the oracle's door loading lag vary (decompression size,
   enemy graphics)? Measure it over `tas-ceres-four-doors` and both room chains before
   fixing R20's step costs.
3. VRAM/CGRAM/OAM/PPU streams: does the typed Scene keep a layout image, or does
   verification project it? (D9 and R17.)
4. Does any corpus row show mid-frame HDMA tearing or the IRQ multiplier clobber in the
   oracle? If so, (a) is needed for exact video on those Ticks.
5. The APU ack latency: is it a fixed number of Ticks in the oracle for every command,
   or does it depend on the SPC driver's own loop phase? (This sets whether R22 needs
   a driver model or a latency rule.)
6. Hardware versus oracle: mid-line IRQ writes (R13), VRAM writes outside vblank,
   IRQ jitter. Which of these does the Reference profile adopt from LakeSnes, and
   which are checked against Mesen and hardware (REVIEW #250)?
7. SRAM corrupt-slot clear versus the CONTRACTS no-silent-reset rule (R30).

Sources for the G-mode mechanism: TASVideos forum threads
[466184](https://tasvideos.org/Forum/Posts/466184) and
[466183](https://tasvideos.org/Forum/Posts/466183). Map Rando tech names come from
<https://maprando.com/logic>.
