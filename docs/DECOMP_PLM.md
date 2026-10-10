# Native collision PLM setup

The readable bank84 setup module implements native `SpawnPLM` allocation and
collision-related terrain behavior. It searches the original 40 slots from high
to low, initializes every shared PLM field, reads setup/instruction pointers from
the actual ROM header and runs native setup synchronously. `supports_header`
classifies immutable ROM data before a live adapter chooses native dispatch or
measured C fallback. Unsupported setup functions return errors; they do not
invent carry flags or effects. A full pool returns zero without accessing the
header, preserving original behavior.

Native setup covers extension/scroll blocks, scroll partner wakeup, plants,
ice, quicksand and sandfalls, respawning speed/crumble/shot/bomb/powerbomb/super
missile blocks, grapple flags/damage, wall changes and selected generic room
setups. Seven original null setups preserve their actual no-op behavior.
Stations, gate triggers, scripted cutscene and enemy-spawn setups remain C-owned.

The public `setup` interface takes an explicit movement amount reference. Native
quicksand preserves the overlapping 24-bit fractional-word writes and the
original `$FF0000FF` displacement mask. PLM state uses canonical WRAM aliases;
block byte offsets remain distinct from block indices. Errors in `spawn` commit
no RAM or displacement writes, so an integration adapter can report or route an
unsupported header without observing partial slot allocation.

Valid domains are aligned even PLM slot offsets 0–78, aligned block byte offsets
within the level/BTS extent, valid projectile offsets 0–18, nonzero room width
up to 255 for wall traversal and terminating scroll partner search. Malformed
C inputs are not correctness oracles.

Verification commands:

```sh
python3 tools/port_family.py verify --manifest tools/port/plm.json --output out/port/plm
python3 tools/port_plm.py --manifest tools/port/plm-spawn.json --output out/port/plm-spawn
python3 -m unittest discover -s tools -p test_port_plm.py
```

The generic manifest compares 44 actual C setup symbols over 352 full-WRAM cases.
The native allocation manifest compares 673 real-ROM header/state cases,
including quicksand displacement mutation and occupied-slot exhaustion. The
special allocation harness compiles the actual complete pinned C tree, installs
`cur_coll_amt32` only on the C reference, and compares native Rust flags,
displacement and all canonical WRAM. It rejects CPU fallback and records zero
remaining C gameplay callbacks in native subject execution. A wrong-amount
candidate and tampered retained WRAM are rejecting controls. Five compiled ABI
controls reject unaligned displacement, aliasing displacement, wrong WRAM
extent and overlapping ROM/WRAM buffers before creating Rust references.
Admission rechecks the pinned source identity, current Rust dependency inventory
and every retained case output. All evidence is
restricted to declared valid fixture inputs; live exercise and original-ROM
parity require separate integration verification.

## Room and hardcoded allocation

`spawn_room` reads six-byte bank8F room entries, including the special door-closing
WRAM entry at 0x12. `spawn_hardcoded` receives byte coordinates and a bank84
header. Both preserve highest-free-slot selection, original Mult8x8 truncation,
room arguments, instruction/draw/timer initialization and synchronous native setup.
Room preinstruction is 0x84e6; hardcoded preinstruction is 0x8469. Full pools
return before reading entries or headers. Supported setup errors commit no writes.

`supports_positioned_header` excludes collision-context setup headers from positioned admission; their
terrain evidence remains separate. In particular, quicksand surface B requires
a displacement context in actual C. Other untranslated setups
remain explicitly counted C ownership. The PLM frame interpreter, rendering and
message coroutine are not replaced by these entry points.

`tools/port_plm.py` seals separate terrain/room/hardcoded fixture counts. The live
consumer requires those authenticated counts for `--room-plm`, renames only the
original C allocator definitions so same-bank callers also reach the adapter,
and keeps their original bodies for measured unsupported-setup ownership.
Independent C/mixed processes compare full state and ordered allocation effects.
Room and hardcoded counters separately report total, C, Rust and actual native
setup calls; full-pool calls do not count as native setup execution.

Use `tools/port_batch.py run --room-plm` for cumulative verification and admission.
For focused admission, provide current system-family and PLM allocator verdicts
to `tools/port_live.py --room-plm`. A passing focused run establishes the new
allocation seam with the accepted decompressor/RNG; it does not refresh every
previous gameplay family's source seal or prove original-ROM parity.

The observed boot hardcoded header 0xb7eb additionally owns native
`PlmSetup_B7EB_EnableSoundsIn32Frames`: area6 starts the original32-frame timer,
other areas start240, and preinstruction0xb7dd remains executed by the C engine.

Fresh full-WRAM actual-C allocation evidence:
`out/port/room-plm-final/20261010T223633.137790Z/verdict.json`:1,969cases
(673terrain,648room,648hardcoded), fiveABIrejections and zeroCPUfallback.
The independent18,000-frame run
`out/port/room-plm-live/20261010T223747.065046Z/verdict.json` matches four
C/mixed processes and executes one native hardcoded setup in each mixed process.
Room and terrain allocation have zero calls in this prefix and remain offline.

Final current-tooling live evidence:
`out/port/room-plm-current-live/20261010T224032.887967Z/verdict.json`, four
fresh9,000-frame processes, one native hardcoded setup per mixed process,
matching full state/returns/effects and zero measured CPU fallback. The original
18,000-frame comparison predates only the counter-parser correction and host
source generator extraction; the final run validates both tooling fixes.

## Door, open-gate and escape-room setup expansion

Native setup additionally owns grey/colored/blue doors, open upwards/downwards
gates, Mother Brain escape doorway extensions, critter escape terrain, solid
four-cell door columns, the unused B974 no-op and Shaktools scroll initialization.
Grey doors extract condition bits from the high room-argument byte while keeping
remaining bits; blue doors preserve the negative projectile sentinel and power
bomb deletion. Gate BTS writes preserve neighboring bytes and reject invalid
column extents before mutation. Shaktools writes canonical scroll cells0xcd20.
Closed gates and projectile-spawning escape preinstructions remain unowned.

Exact actual-C full-WRAM evidence:
`out/port/plm-door-setups/20261010T224620.249147Z/verdict.json`:54routines,
432cases including80new branch fixtures. Twelve independent public PLM tests,
full port tests, formatting and Clippy pass. These ten additions are offline
verified; previous hardcoded live admission belongs to its earlier exact source
snapshot. New module seals require refreshed consumers before installation.

## Colored gate trigger closure

All eight colored gate triggers now execute native projectile filtering and
adjacent-gate wakeup. Highest matching PLM slot is selected independently of
header occupancy; only a zero gate timer becomes1. The trigger is deleted and
carry1 is returned even when no partner exists. Green/red projectile masks,
right-yellow exclusion and recovered left-yellow/blue fallthrough remain intact.
Dud sound queues use the native audio module, compared against actual bank80 C.
Malformed projectile slots reject before mutation.

Actual-C evidence `out/port/plm-gate-triggers/20261010T224905.433245Z/verdict.json`
verifies62setup routines/576cases, including144new gate cases covering projectile
kinds, absent/present partner and nonzero timers. Fourteen independent PLM tests,
full Rust tests and Clippy pass. Gate trigger additions are offline verified;
closed gate projectile creation and the PLM frame/rendering engine remain C-owned.

## Door preinstructions

Native preinstructions cover ordinary/power-bomb/missile/super-missile shot
conditions, unconditional link wakeup, dud sound, boss/miniboss/Torizo state,
enemy death quota, Tourian statue and critter event checks, plus boot sound timer
expiry. System/event scratch writes and native audio queues are preserved.
Super-missile hit state119 belongs to the distinct plm_variables overlay0xdf0c,
not setup variable0x1e17. The bounded control frame dispatches these handlers;
unsupported preinstructions/rendering/coroutines remain explicit errors.

Actual-C evidence `out/port/plm-door-preinstructions/20261010T225127.240913Z/verdict.json`
passes75routines/680cases (104new preinstruction cases). Fifteen independent
PLM tests, full Rust suite and Clippy pass. New preinstructions are offline
verified; no additional live or original-ROM parity claim is made.

Frame ordering is now verified against actual `PlmHandler_Async`: a deleted PLM
still decrements its instruction timer and executes an expiring script; a link
wakeup runs in the same frame. All twenty C dispatcher no-ops are supported,
including the hardcoded allocation default. Evidence
`out/port/plm-frame-preinstructions/20261010T225440.281820Z/verdict.json`
passes 756 full-WRAM/raw-return cases, including 408 frame cases. Independent
deletion regression, full Rust tests and Clippy pass. Rendering and coroutine
ownership remain with C.

## Native drawn frames

`plm_render` now owns block coordinates and the complete `DrawPlm` tilemap path:
horizontal/vertical camera clipping, quadrant flips, two/four-entry VRAM queues,
screen crossing, chained signed offsets and shared tilemap capacity stops.
`plm_instructions::process_frame` executes drawn instructions through level-data
updates, coordinates and queued rendering. The host still executes hardware
transfers and message coroutines; unsupported preinstructions reject.

Actual pinned C evidence
`out/port/plm-rendering/20261010T225759.092054Z/verdict.json` passes 1,100 cases
across six seams, including 256 renderer cases, 48 coordinate cases and 40
drawn frames. Seven independent renderer tests and a literal full-frame example,
the full Rust suite and Clippy pass. This expands offline ownership; live
rendering and original-ROM parity remain unproven.

## Room gameplay conditions

Twenty-one further preinstructions own lava progression, speed-booster escape,
Shaktool scroll/event changes, escape liquid motion, triggered blocks, Samus
position/proximity, door bits, Metroid quotas and keypress wakeups. Missile room
arguments and trigger low-byte conditions retain distinct behavior. Native
single-block drawing builds the RAM script and queues rendering; proximity
instruction division now matches the SNES byte divisor, including zero.

Actual-C evidence `out/port/plm-room-conditions/20261010T230206.177568Z/verdict.json`
passes96routines/1,184cases. Frame evidence
`out/port/plm-room-frames/20261010T230207.358573Z/verdict.json` passes1,368cases,
including252new condition-frame cases and16block-draw cases. Independent Rust
examples, full suite and Clippy pass. Escape-shaft wall projectile spawning and
message coroutine ownership remain open; additions are offline verified.
