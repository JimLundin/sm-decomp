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
