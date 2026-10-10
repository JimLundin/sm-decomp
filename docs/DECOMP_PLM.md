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
