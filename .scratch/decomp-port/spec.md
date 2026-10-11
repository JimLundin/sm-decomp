# Broad readable Rust gameplay port

Status: open

Implement the user's goal: translate large parts of the existing decomps into
maintained readable Rust. Follow docs/DECOMP_PIVOT.md. Primary source is the
pinned snesrev/sm; consult reviewed fork corrections and original assembly for
disagreements. Foundation acceptance remains immutable predecessor evidence.

Integration branch: `decomp-port-integration`.

## Contracts and verification seams

Continue the established full canonical WRAM routine boundary: explicit little
endian values, wrapping widths, alias-preserving offsets, full before/after state
and return-value comparisons against compiled actual upstream C. This is the
existing approved foundation test seam. Each family declares valid input bounds;
malformed C accesses are never used as a correctness oracle. Preserve failure
evidence and distinguish translated, offline-verified and live-exercised routines.
Live verification uses independent C/mixed processes with no state repair,
observable hardware effects, Rust hits and fallback measurements.

## Task graph

- 01: reusable family harness and source inventory, no blockers.
- 02: system state routines, no blockers; verification uses 01.
- 03: music and SFX queue routines, no blockers; verification uses 01.
- 04: integrate and exercise family replacements, blocked by 01, 02, 03.
- 05: Samus movement physics and ROM tables, in progress.
- 06: generic enemy geometry and solid-block mechanics, in progress.
- 07: harness assets, captures and 32-bit values, in progress.
- 08: complete Samus animation/input family, in progress.
- 09: generic enemy instruction handlers, in progress.
- 10: live failure controls and dependency seals, in progress.
- 11: actual PLM collision callback harness, in progress.
- 12: complete Samus collision/movement closure, in progress.
- 13: native PLM allocation and terrain setup, offline verified; live integration underway.
- 14: projectile initialization/damage/instructions and missile behavior, in progress.
- 15: common flying enemy AI and shared movement collision, offline verified.
- 16: projectile terrain flight and firing, live verified.
- 17: Norfair Rio state machines, offline verified.
- 18: Sidehopper/Desgeega jump states and script handlers, offline verified.
- 19: bombs/spread/SBA/speed echoes, offline verified; power-bomb path partial.
- 20: room/door data and camera scrolling, offline verified.
- 21: PLM instruction handlers and bounded frame-control engine, offline verified.
- 22: Geemer, Zoomer, Zeela and Sova crawler closure, offline verified.
- 23: power-bomb allocation and afterglow lifecycle, offline verified.
- 24: cumulative authenticated gameplay live admission and evidence storage cleanup, resolved.
- 25: native room/hardcoded PLM allocation, offline verified; hardcoded sound-enable setup live-exercised.
- 26: native doors, open gates and escape-room PLM setups, offline verified (ten additions).
- 27: all eight colored gate triggers and adjacent-PLM wakeup, offline verified.
- 28: door shot/room-condition preinstructions and bounded-frame dispatch, offline verified.
- 29: actual-C PLM frame preinstruction ordering, deletion timers and no-op closure, offline verified.
- 30: native PLM tilemap rendering and drawn-frame execution, offline verified.
- 31: native lava/room/trigger PLM conditions and single-block drawing, offline verified.
- 32: enemy-projectile allocation, nine room-effect initializers and PLM spawning, offline verified.
- 33: enemy-projectile bytecode, shared movement, room-effect preinstructions and frame scheduling, offline verified.
- 34: enemy-projectile terrain collision, slopes, extension chains and eye-door/room collision preinstructions, offline verified (1,232 collision and 940 dispatcher cases).
- 35: native Crocomire projectile/bridge and delayed fireball spawning, terrain transitions and scheduled frames, offline verified (354 cases); explosion host-state seam and further boss behavior remain open.
- 36: enemy-projectile random-radius opcode and scheduled execution, offline verified (384 cases).
- 37: directional fireball split/repeat spawning, area properties and collision transitions through scheduled frames, offline verified (528 cases).
- 38: Phantoon starting-fireball radial allocation, landing/pause/bounce lifecycle and scheduled transitions, offline verified (449 cases).
- 39: Phantoon destroyable-fireball modes, expanding/contracting orbits and delayed fall through scheduled frames, offline verified (809 cases).
- 40: Kraid spit/ceiling/rising rocks, collision flight and acceleration through scheduled frames, offline verified (513 cases).
- 41: Mini-Kraid spit and spike allocation, extra enemy-RAM aliases and collision flight through scheduled frames, offline verified (401 cases).
- 42: walking lava seahorse fireball spawn, trajectories and vertical-first collision flight through scheduled frames, offline verified (241 cases).
- 43: pirate laser/claw script callbacks and camera-relative flight through scheduled frames, offline verified (644 cases); host-global initializers remain open.
- 44: Bomb Torizo low-health drool initializers and probabilistic script branch through frames, offline verified (384 cases).
- 45: Torizo drool impacts, explosion placement/shared parameter mutation and position restore, offline verified (292 cases).
- 46: Bomb/Golden Torizo orb spawning, collision/bounce lifecycle and direction branch, offline verified (384 cases).
- 47: Torizo sonic-boom allocation, flight/impact snapshots and position reset, offline verified (196 cases).
- 48: Golden Torizo egg allocation, bounce/launch/impact lifecycle and facing branch, offline verified (436 cases).
- 49: Golden Torizo super-missile allocation, saved-slot tracking, aiming and collision flight, offline verified (368 cases).
- 50: Golden Torizo eye-beam allocation, three-call RNG velocity and collision transitions, offline verified (193 cases).
- 51: lava seahorse fireball allocation connected to verified fractional arc/apex/deletion frames, offline verified (241 cases).
- 52: Tourian statue eye-glow palette, souls and particle/tail allocation, offline verified (193 cases); effect scheduling remains open.

This is the first frontier, not the completion boundary of the user's goal.
Expand tickets into substantive gameplay subsystems after this frontier, using
observed dependencies and fixtures. Full ownership remains the direction; do not
equate a count of short helpers with large parts of the game being translated.

## Verified progress

Five-family live run out/port/gameplay-control-live/20261010T201048.927361Z/verdict.json
matches four independent 9,000-frame C/mixed processes, full periodic game/hardware
checkpoints, terminal WRAM/SRAM, per-call captured states, ordered PPU/APU effects
and raw-bit returns. 143 translated symbols have declared offline fixtures;
48 are live-exercised with 48,033 Rust calls plus 23 accepted Rust decompressions
and 8,783 Rust RNG calls per process. Measured CPU fallback is zero in this
prefix; untranslated C still owns scheduling and many game systems. The user's
goal remains active: collision/movement, bank-specific enemy behavior, projectiles,
PLMs, rooms/doors and further gameplay must continue before broad ownership can
be claimed. Counts are inventory, not a percentage of complete gameplay.

The later four-process 18,000-frame comparison at
`out/port/collision-plm-extended-live/20261010T203333.584044Z` passed all state,
effect and gameplay-progress comparison gates before rejecting the additional
native PLM execution requirement: zero PLM calls in every process. Each mixed
process executes 186,736 Rust gameplay calls across 50 symbols, 37 Rust
decompressions and 17,594 Rust RNG calls with zero measured CPU fallback.
Native normal movement executes 10,791 times and enemy Goto 9,241 times.
This is an exercised comparison observation, not accepted PLM live integration
or original-ROM movie completion. The current public crate additionally owns
offline-verified projectile core (34 routines/3,656 cases) and common flying
enemy AI/movement (17 routines/454 cases), with 63 independent Rust tests.
Projectile terrain flight/firing and Norfair Rio expansion are active tickets16/17.

Normal projectile flight/firing now has accepted live replacement at
`out/port/flight-fixed-live/20261010T204643.443282Z/verdict.json`: four fresh
9,000-frame C/mixed processes match and execute434 Rust flight/firing calls
across four symbols, with the accepted Rust decompressor/RNG retained and zero
CPU fallback. Native PLM has no calls in this prefix and stays offline-verified.
Public crate includes Norfair Rio's18 state routines/162 C fixtures; Sidehopper
and Desgeega jump closure is active ticket18. Overall full Rust ownership and
original-ROM gameplay parity remain unproven.


Latest cumulative admission: batch`20261010T213631.753519Z`, live verdict
`replays/20261010T215856.742725Z/evidence/20261010T215856.841221Z/verdict.json`
passes four independent18,000-frame processes.76 symbols execute235,952 Rust
gameplay calls per mixed process plus37 decompressions and17,594 RNG calls;
measured CPU fallback zero. Collision/movement ticket12 is exercised and
resolved. Zero-call bank-specific enemies, special projectiles and PLMs remain
offline; crawler/power-bomb expansions are offline verified. Full gameplay
ownership and movie completion remain active goals.


Room/hardcoded allocation frontier: ticket25 implements both entry points with
1,969 actual-C full-WRAM cases. Final9,000-frame live evidence at
`out/port/room-plm-current-live/20261010T224032.887967Z/verdict.json` matches
four independent processes and exercises one native hardcoded setup per mixed
process, retaining Rust decompression/RNG and zero measured CPU fallback.
An18,000-frame run also matches. Room and terrain allocation remain unexercised
in these prefixes; whole PLM engine/rendering and original-ROM parity remain open.

Tourian scheduling frontier: ticket53 ports particle/soul/splash/statue updates,
water-crossing allocation and particle/tail script spawns. 538 actual-C cases
match, including descending same-frame scheduling. Offline verification only;
live admission and full gameplay ownership remain active.

Lavaquake rock frontier: ticket54 adds native allocation and the complete
rising/falling arc with quadratic ROM half-steps and inclusive camera culling.
437 actual-C cases match. Offline coverage; live admission remains open.

Shaktool circle frontier: ticket55 ports direction/offset allocation and
front/linked terrain scheduling, including initial no-op script phases.
1,159 actual-C cases match; live admission and full gameplay ownership stay open.

Mother Brain turret projectile frontier: ticket56 ports layout/bullet allocation,
rotation/random timers and bullet terrain scheduling plus script switch/palette
operations. 1,281 actual-C cases match. Turret destruction host-state seam and
live gameplay ownership remain open.

Enemy-linked allocation frontier: ticket57 adds BB30 decoration setup and
BB92 Nuclear Waffle body extended-RAM linkage/fraction copying. 385 actual-C
cases match. Live gameplay ownership remains open.

Spore projectile frontier: ticket58 adds stalk/spawner allocation and native
ROM spore motion/random countdown. 1,185 actual-C cases match. Actual spore
creation host-state seam and live gameplay ownership remain open.

Nami/Fune projectile frontier: ticket59 ports fireball and DBF2 allocation/
horizontal motion with ROM speed selection and inclusive camera culling.
769 actual-C cases match. Drop-spawn host seam and live ownership remain open.

Lavaman thrown-lava frontier: ticket60 adds native allocation and horizontal
scheduling, with 385 actual-C cases plus 769 shared-path regression cases.
Live gameplay ownership remains open.

Shot gate/electricity frontier: ticket61 adds native PLM-linked allocation
and gate fractional movement/timer scheduling. 513 actual-C cases match.
Live gameplay ownership remains open.

Skree frontier: ticket62 ports four particle allocations and motion/gravity
with exclusive screen bounds. 705 actual-C cases include dust visibility
regression. Live gameplay ownership remains open.

Enemy falling-particle frontier: ticket63 adds 8A39/8A7D random allocation
and unsigned endpoint movement. 305 actual-C cases match; live ownership
remains open.

Paired projectile motion frontier: ticket64 ports Draygon shared E73E
16.16 movement and E722 fixed extent predicate. 321 actual-C cases match;
full Draygon scheduling and live ownership remain open.

Draygon scheduling frontier: ticket65 adds power-bomb collision and turret/gunk
paired movement/proximity updates. 897 actual-C cases match. Allocation,
attached gunk and live gameplay ownership remain open.

Draygon attached-gunk frontier: ticket66 ports attach opcode and Samus
follow/release scheduling. 1,281 actual-C cases match including regression.
Draygon allocation and live ownership remain open.

Botwoon body frontier: ticket67 adds native allocation and extended-RAM
body linkage. 193 actual-C cases match. Lifecycle scheduling and live ownership
remain open.

Botwoon lifecycle frontier: ticket68 ports main/flash/death delay/fall and
landing scheduling. 833 actual-C cases match; live ownership remains open.

Botwoon spit frontier: ticket69 adds native paired motion and inclusive
camera removal. 833 actual-C cases match. Allocation/angle conversion and
live ownership remain open.

Parallel allocation frontier: tickets70/71 add Botwoon spit quarter-circle
velocity setup and Yapping Maw body linkage.257+385 actual-C cases match.
Final combined tests/fmt/Clippy pass; live gameplay ownership remains open.
