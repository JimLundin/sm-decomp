# Ticket19 Morph material handoff

Owner `/root/material`, native owner `/root/native`, integration owner `/root`.
Worktree `/home/jim/sm-decomp-restart-morph-rust`, branch
`restart/morph-instructions-rust`, baseline
`549ef7775a44e9379a4e5d2fc930f2455a57fe9e`. Owned paths are only
`restart/item-instructions/`. No original production claim was opened or changed.

## Frozen source and public interface

Research source:
`/home/jim/sm-decomp-restart/restart/out/morph-instructions/planning/scope-research.md`,
SHA256 `124d6c6608f1201933b4bf0b16f1e05e84109bd32c7552cfa794b2402c66ae5c`.
Selected `.scratch/restart-pipeline/morph-instruction-interface.md` SHA256
`a234caf3b21c3f85fd69234d835ec530239bc4f65f323b9035aba843f2a84120`.
The recovered C pin and retained source path/hashes remain those in HANDOFF.md:
`578f90b3cc49557bb70060ad033bb90b8cf8ac50`.

Source module closure adds `src/morph.rs` and `src/morph_fixtures.rs` alongside
the existing `lib.rs`, `memory.rs`, `fixtures.rs`, and `tests.rs`.

Entry values 13..16 append LoadItemPlmGfx, DrawItemFrame0, DrawItemFrame1 and
PickupEquipmentAndShowMessage. Service values 3..5 append
ProcessPlmDrawInstruction, CalculatePlmBlockCoords and DrawPlm. Each new trait
method receives `(&mut self, ram: &mut [u8], slot: u16) -> Result<(), String>`.
Common is private Rust code, called locally by both frame roots; it is not a
scheduled entry or a generic native frame service. The twelve scheduled Morph
roots are 0,1,3,5,6,8,11,12,13,14,15,16. Together with Common they implement the
thirteen-function family.

The runtime admits only EF23/001A at the selected sites, seals the full Morph
script, cleanup helper, graphics/frame tables and all referenced draw records,
and uses D873 for the Morph item bit. EF2F/0022 retains its prior sites and D874.
Graphics writes preserve seven-byte queue packing and ring/tile-table indexing.
Equipment pickup preserves OR semantics, music-before-message ordering and
message9. Common preserves timer4/IP writes and Process(root), then reloads the
global slot for Calculate(saved) and Draw(saved), including odd Continue values
elsewhere. Post-service unsafe state or service failure is fatal, never C retry.

## Finite fixture inventory

The exported functions are
`fixtures::morph_body_fixtures(seed, rom)` and
`fixtures::morph_refusal_fixtures(seed, rom)`, returning existing `Vec<Fixture>`.
Inputs use the same frozen ticket18 revealed seed and environment overrides
documented in README.md; no new capture or private runtime input is introduced.
Builders authenticate the seed before overlaying slot68/EF23/001A, block298A,
room width128, camera1024/640, queue/cursor0 and fixed item/music fields.
Direct-write expectations omit native effects; real actual-C/full-WRAM and
ordered service snapshots are required independently.

| IDs | Distinguishing body behavior |
| --- | --- |
| M01..M04 | Graphics rings0,2,4,6; queue tails0,7,14,601; packed seven-byte writes, eight tile words, ring wrap |
| M05..M12 | Frame0 then Frame1 for each ring0,2,4,6; all eight draw records and local Common |
| M13 | Frame0, cameraY0 clips blockY41; Process/Calculate/Draw still requested |
| M14 | Frame1, full draw queue tail480; valid no-draw |
| M15 | Frame0, full tilemap cursor512; valid no-draw |
| M16 | Equipment OR preserves equipped0040 and collected0200, then music/message9 |
| M17 | Already ORed equipment remains unchanged while service/message still execute |
| M18 | Equipment pickup with game state40 retains native music no-op behavior |
| M19,M20 | Clear/set Morph item-bit conditional paths, ContinueE3FF/E419 |
| M21,M22 | Set Morph item bit from clear/already-set; preserve other bits |
| M23 | Set linkE40F at operandE401 |
| M24 | Set preinstructionDF89 at operandE405 |
| M25,M26 | GotoE407 and GotoDFA9 |
| M27 | Delete at zero-operandDFAF |
| M28 | Clear music queue, queue track2 from operandE413 |
| M29,M30 | Low-byteFF triggers; high-byte-onlyFF does not; both waiting IP sites |
| M31 | Root slot68 with saved global slot70, sealed alternate pointerA315 |

| IDs | Atomic refusal boundary |
| --- | --- |
| MR01,MR02 | Foreign header/argument return Refused, unchanged |
| MR03 | Wrong graphics operand site |
| MR04,MR05 | Changed script/frame table seal |
| MR06,MR07 | Graphics ring8 and frame ring1 |
| MR08 | Graphics queue tail602 overruns the packed queue region |
| MR09 | Required drawing services unavailable |
| MR10,MR11 | Odd block offset and blockFFFE crossing WRAM end |
| MR12,MR17 | Room-width divisor0 and256 |
| MR13 | Global slot80 |
| MR14 | Draw queue tail609 |
| MR15,MR18 | Tilemap cursor520 and unaligned4 |
| MR16 | Alternate global slot with unsealed draw pointer |
| MR19 | Truncated ROM backing |
| MR20 | Morph trigger with missile linkE49F |

Original B01..B25 inputs/expectations are unchanged. R07's unknown raw entry
changes from13 to17 because13 now denotes LoadItemPlmGfx; the other legacy
refusal definitions are unchanged. Native was informed of this ABI adjustment.

## Safety bounds and verification status

Complete128KiB WRAM and even slots0..78 are required. Graphics ring values are
0,2,4,6; graphics queue tail is at most601, so seven bytes end before0330.
Drawing uses only sealed one-block records. Both root and saved global block
offsets must be even and at mostFFFC; level_data starts10002. Width is1..255.
Draw queue tail is at most608 and tilemap cursor is aligned8 in0..512. These
bounds retain valid clipping/full-capacity no-draw outcomes. A distinct saved
slot must already name one of the eight sealed draw records. Root's pointer is
assigned from the frame table before Process. Bounds and required capabilities
are preflighted before any write and rechecked after foreign effects before
the next unsafe service.

Four additional unit tests cover the finite body/refusal matrix, each missing
draw capability and ordered service failure, and global-slot reload/save
semantics under explicit service mutations. Six prior tests remain, adapted
only for the expanded trait and enum. Formatting and `git diff --check` passed.
No build, test, native control or gameplay was run by this worker. Root owns all
gates with one Cargo job, source freeze, stage publication and actual-C/live
acceptance; no verified or live-exercised stage is asserted here.

Next root action after native integration and shared registration: run the
item crate tests, the new Morph controls plus justified changed-ABI legacy
controls, then the one selected fixed24832-input pair with only Morph ownership
varying. Keep the missile callback stream and old artifacts comparable to18.
Native owns the real service52 already-collected graphics-first control and
draw-service transaction/fatal controls. Original-ROM parity remains untested.

## Cleanup

No private producers, build trees or run outputs were created by this worker;
local reclaimed bytes0. Available filesystem bytes at inspection102956670976.
Root owns both final cleanup passes, authenticatable retained evidence and
implementer retirement. Keep this source worktree until integration and required
evidence-source retention are complete.
