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

Live admission investigation: ticket73 records dispatcher evidence authentication,
actual-symbol registration and pre-mutation support selection required to admit
bank86 coverage. Read-only parallel audit confirms existing live tooling cannot
consume these dispatcher seals; no new live ownership claim is made.

Enemy death allocation frontier: ticket72 adds native EF89 position/metadata,
respawn flag and five death scripts.961 actual-C cases match; cleanup/evidence
recorded in ticket72. Drop/pickup scheduling and live ownership remain open.

Bank86 live admission progress: ticket73/commit7b37fcb adds authenticated
dispatcher leaf contracts and requires subsystem-specific Rust hits. Ticket74
seals385 actual-C movement cases.9000/18000 prefixes match state/effects across
eight processes but leaves execute zero times; actual leaf ownership remains
unproven and ticket73 stays claimed.

Enemy drop-selection frontier: ticket75/commitbd5ad1d ports RandomDropRoutine
weighted selection, inventory eligibility and health-bias hysteresis.1,794actual-C
cases include every RNG zero-byte retry seed. Full tests/fmt/Clippy pass;
cleanup reclaimed503,840,768bytes and retained evidence authenticated.
Live bank86 admission replay of all129712supplied frames is underway separately.

Full-movie bank86 admission attempt: commit e51e552/ticket73 extends bounded
replay timeout and attempts all129712inputs with fresh385-case leaf evidence.
Reference C crashes at43188before mixed processes: game_stateffff from SoftReset
indexes45-entry game-state table at65535. Ticket76 records exact debugger
reproduction/backtraces and remaining correct runtime-reset work. No live leaf
ownership claim. Diagnostic and replay cleanup measured in tickets73/76.


Enemy death-drop transition: ticket77/commitb6a874c connects native weighted
selection to pickup scripts, preserving slot-zero RNG/exclusion, timer/lifetime
and property behavior.2,434actual-C opcode/scheduler cases pass, with full
crate checks. Cleanup319,160,320bytes; evidence retained/authenticated.
Subsequent pickup collection and host-coordinate drop allocation remain open.

Reset repair: ticket76 mirrors the pinned comparison runtime's explicit
soft-reset entry in the C-only headless host. Two45000-frame C runs pass the
old43188crash and match through terminalstate1; zero CPUfallback. Fullmovie
C/mixed replay remains underway under ticket73, without an ownership claim.


Original movie timing frontier: ticket78 records the existing bounded controller
diagnosis (unsupported prelude poll19, controller mismatch272) and separates
actual input polling from gameplay dispatch. Raw fullmovie C replay diverges
from legacy original-ROM checkpoints before9000; corrected reset/C-mixed parity
cannot establish original-ROM movie progression.


Bank86 leaf admission: full129712-frame four-process C/mixed replay passes
with current-source evidence after commitsb6a874c/33ef42c. Native movement-X
executes11times and Y13times per mixed process; all RAM/effects/progress match
and CPUfallback0. Combinedmovement0hits remains unexercised. Ticket73resolved
for safe leaf integration; broader scheduler/allocator and original-ROM timing
remain open. Cleanup reclaimed19,355,467,749bytes; retained hashes authenticate.


Pickup collection frontier: ticket79 extends EFE0 through expiration, grapple
proximity, Samus-radius collision, all five inventory rewards, reserve handling
and sound refusal. Initial2561C comparisons pass; final2804case evidence includes
orthogonal grapple-delay/radius/inventory/audio boundary cases and matches C.
Ticket79resolved in commitdc8b934; both verdicts authenticate after cleanup
reclaimed703,987,712bytes. Live pickup ownership remains unproven.

Controller timing diagnosis: ticket78 records actual event ordering: first
original completion1025 precedes poll1030 at frame272, while diagnostic poll-as-
step replay assigns that future poll to its corresponding C completion. New
causal ledger retains prelude polls and boot clearing; full mapping remains
nonaccepting until missing boot/NMI polling and reset-clear boundaries are modeled.


Explicit projectile host context: ticket80/commiteb9f1ab represents actual
point/rectangle/r22/enemy-header C globals outside WRAM and adds stationary
fireball explosion947F, dust/explosionE468 and dropEF29 allocation.769 actual-C
cases match complete WRAM, raw return and mandatory host sidecars. Independent
Rust checks/fullcrate checks/11harness tests pass. Cleanup361,103,360bytes; final
evidence authenticated. Ticket35hostinitializer gap resolved in0214bac; host
context propagation through callers/scheduler and live ownership remain open.

Callable controller foundation: commitbca5ad0 verifies160original-ROM controller
polls includingprelude19 using actual register samples and entry/return captures.
151cases differ only in one explicitly validated native callstack byte;9match
fullWRAM. All otherWRAM matches, with wronginput/unrelatedWRAM rejectioncontrols
and3publictests. Retained evidence authenticates after cleanup0bytes. Ticket78
remainsclaimed: bootclearing/dispatch scheduling and originalmovie parity remain
unfinished; ordinary replay remains nonaccepting.


Host-aware caller coverage: ticket81/commitaf6dbd2 propagates mutable projectile
host state through Ridley fireball terrain impact940E and descending frames.
Horizontal collision has priority; impact publishes coordinates, frees/reuses a
slot, allocates947Fexplosion and preserves sound gates.585 actual-C cases match
WRAM/returns/host globals; independent tests/fullcrate checks pass. Cleanup
394,264,576bytes; retained final evidence authenticated. Live admission remains
open; unrelated opcode host propagation remains incomplete.

Causal first update: commite354c34 restores native NMIpoll ownership and suppresses
relocated main-entry poll in a separate bounded C subject.150actual prelude
samples execute in original order before real reset clearing. First completed
update matches all compared semantic fields at originalframe272/event1025,
Cstep151/state1/RNG758/inputs0; poll1030 occurs afterward. ChangedRNGcontrol
yields759;3publictests pass. Final retained evidence authenticated after cleanup
0bytes. Ticket78claimed: boot video timing/later waits and full movie parity
remain unproven; broad replay remains nonaccepting.


Host-aware drop instructions: ticket82 extends instruction/context scheduling
through AB8A,9270,8C68 and DC61, publishing externalpoint/header and invoking
actual SpawnEnemyDrops/F337 native drop selection. Internal raw enemygfxoffset
preserves C projectile-index use without loosening existing alignedenemy APIs.
Final536actual-C cases match WRAM/return/host globals across real script words
and all four scheduler callers. Ticket82resolved in commitcd32007; cleanup
207,618,048bytes, final evidence authenticates and fullcrate checks pass.
Live instruction ownership remains unproven.

Causal update prefix: commitbdd617f executes160original polls and matches all
ten completed-update semantic comparisons through nativeframe316. Each update
checks its entire preceding poll sequence/count; repeated heldvalues cannot
hide shiftedpollordinals. Allten changed-RNG comparisons detect errors;3public
tests pass. Retained source/binary/states/logs authenticate after cleanup0bytes.
Ticket78stillclaimed: native video/APUwait timing and fullmovie parity unproven.


Host-aware spores: ticket83/commit1b14934 connects DC77 spawning to DC8D
initialization and nativeDCEE movement in descending frames.768 actual-C cases
match completeWRAM/rawreturn/hostsidecars with actualDE88spawner/scriptDC0E,
all256movementphasebytes and coordinate/allocation boundaries. Fullcrate checks
and7independenthosttests pass. Cleanup218,103,808bytes; retained evidence
authenticated. Live spore ownership remains unproven.


APU wait frontier: commit771ab14 retains passive original uploads at boot
frames168–254 and music316–355, with278147/140879portreads and zero NMI/controller
polls inside each interval. Two actual cross-unit C uploads complete instantly;
hardware/SPC handshake remains unmodeled. Extended640capture verifies49causal
completed updates through nativeframe393; ordinal50/frame396 firstpollcount
mismatch199C/200native.82pairedsemanticmatches are diagnostic only. Completion
counts231C/229native differ; overallverdict remains nonaccepting. Prefixhost
uses existingrealsoftreset helper and completes raw640run.3public320tests pass.
All5finishedruns cleaned0bytes; finalsealed artifacts authenticate. Ticket78
remainsclaimed; broader originalmovie timing/ownership is unfinished.


Mother Brain room turret lifecycle: ticket84/commit9e60825 adds BFDF viewport/
destruction handling, hostpoint/dust allocation, rotation and firing timers and
bullet spawning through host-aware frames.577actual-C fullWRAM/return/host cases
match, with fullcrate checks and9independenthosttests. Cleanup104,464,384bytes;
finalevidence authenticates. Legacyhostless frames preflight BFDF beforewrites;
live lifecycle ownership remains unproven.


Within-update wait seam: commitc3d2c13 restores actual polling at the patched
WaitUntilEndOfVblankAndClearHdma call in a separate causal subject. NestedNMI
uses existingwaitingflag and consumes sourceordered samples without another
gameupdate. Verifiedcausalprefix49→51 through originalframe397; omittedwait
control reproduces oldordinal50failure199C/200native. One actualhelperhit;
nextordinal52remainsunresolved.3public320tests and independentRNGcontrol pass.
Bothfinishedruns cleaned0bytes; retained binary/source/states/controllogs
authenticate. Ticket78claimed: hardwareframe/APU/laterpoll timing unfinished.


Screen-enable polling seam: commit a3b3226 restores the native ScreenOnWaitNMI
continuation at all three pinned ScreenOn call sites. The source-stable 640-frame
capture matches 229 causal completed updates through original frame 639, with
229 candidate/native completions and exact preceding controller poll sequences.
Omitting only this seam reproduces ordinal 52 failure (202 versus 203 polls).
Three public tests pass; finished-run cleanup reclaimed zero bytes and retained
evidence authenticates. Ticket 78 remains claimed: hardware timing, full WRAM
and full-movie parity remain unproved.


Pirate host allocation: ticket 85 ports A009 laser point/r22 selection and A098
claw rectangle coordinates with wrapping. Final 549 actual-C cases match full
WRAM, returns and all eight host words, including sound quota refusals and
independently seeded script/flight frames. Crate tests, formatting and strict
Clippy pass. Cleanup reclaimed 414,056,448 bytes; evidence reauthenticated.
Live pirate caller ownership and allocation-to-flight replay remain unproved.


Controller frontier extension: a fresh source-stable 960-frame original capture
verifies 516 causal completed updates and all 668 polls, with matching completion
counts and no new semantic or poll provenance failure. Omitted HDMA and screen
wait controls still fail at ordinals 50 and 52. The completed run was cleaned
(zero bytes) and authenticated. Ticket 78 stays claimed for broader original
replay, full WRAM and hardware scheduler ownership.


Pirate caller ownership: ticket 86 translates native bank-B2 left/right laser
instructions and the claw instruction into host-aware allocation, including
enemy graphics and real B2 operand/return pointers. All 577 actual-C full-WRAM,
return and host comparisons pass. Independent Rust checks and 12 Python harness
tests pass, with crate tests, formatting and strict Clippy. Scoped cleanup
reclaimed 178,388,992 bytes; root reauthenticated final evidence. Whole pirate
AI and live original-ROM caller exercise remain unproved.
