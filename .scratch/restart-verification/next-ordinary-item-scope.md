# Candidate after18/19: exposed resource-tank instructions

Status: bounded source research complete; advisory, no ticket20 claim
Owner: `/root/main_reuse_audit`; root retains selection
Date: 2026-10-16

**Yes: the exposed resource-tank family is a coherent next Translation scope.**
Energy, missile, super-missile, power-bomb and reserve tanks share the existing
item lifecycle and add only **four pickup bodies and two named native HUD
services** after18/19. Their complete instruction family has17 functions:
13 reused, four new;16 scheduled roots plus local DrawItemFrame_Common.
This would expand translated/actual-C-verified behavior, but the unchanged
24,832-input tape cannot establish live execution of the four new collectors.

The remaining ordinary pickup opcode set also contains beam, grapple and X-ray
collectors. They need distinct equipment/beam graphics semantics; there is no
reason to require those alongside the tank family. This packet chooses one
complete exposed-tank boundary, not a catalog of every item presentation,
Chozo shell, shot-block variant or suit acquisition sequence.

## Source and reuse authority

`C` denotes retained recovered C578f90b3cc49557bb70060ad033bb90b8cf8ac50 under
[/home/jim/sm-decomp-restart/restart/out/scroll/native/2c9c9a3cf51a059220afac1ee04727c956e45d067e10e17405bf3adbd268fc7f/source/src/](</home/jim/sm-decomp-restart/restart/out/scroll/native/2c9c9a3cf51a059220afac1ee04727c956e45d067e10e17405bf3adbd268fc7f/source/src/>).
Use [18's interface](/home/jim/sm-decomp-restart/.scratch/restart-pipeline/item-instruction-interface.md),
[19's selected scope](/home/jim/sm-decomp-restart/.scratch/restart-pipeline/issues/19-rust-morph-instructions.md)
and [the Morph source packet](next-morph-item-ownership.md) for known reuse.
Active19 implementation was not inspected or changed.

Bounded `od` reads of existing pristine [sm.sfc](/home/jim/sm-decomp/rom/sm.sfc)
establish the header/script rows below. Its SHA256 was already authenticated
in the preceding Morph research:
`12b77c4bc9c1832cee8881244659065ee1d84c70c3d29e6eaf92e6798cc2ca72`.
These observations do not seal a future prepared runtime ROM.

## Four new bodies; exact observable behavior

All take an actual bank84 operand pointer P and return Continue(P+2). The sealed
tank amounts are100 for health/reserve and5 for ammo. Arithmetic stores wrap
to16 bits; none of these routines sets the room's persistent item bit itself.
All queue their message by the existing local write to0DC8 **after** services.

| New C body/address | Ordered body effects before returning |
|---|---|
|`PlmInstr_CollectHealthEnergyTank`,848968; C/sm_84.c:482|Add100 to max health09C4; copy resulting max to current09C2; existing PlayRoomMusic(0168); queue message1. This fills health to the new maximum, not old current+100.|
|`PlmInstr_CollectHealthReserveTank`,848986; :490|Add100 to max reserve09D4; if mode09C0 is0, increment it to1; existing PlayRoomMusic(0168); queue message19hex. Current reserve09D6 is unchanged. Nonzero mode is preserved.|
|`PlmInstr_CollectAmmoSuperMissileTank`,8489D2; :509|Add5 to capacity09CC, then current09CA; AddSuperMissilesToHudTilemap(); existing PlayRoomMusic(0168); queue message3.|
|`PlmInstr_CollectAmmoPowerBombTank`,8489FB; :519|Add5 to capacity09D0, then current09CE; AddPowerBombsToHudTilemap(); existing PlayRoomMusic(0168); queue message4.|

Primary definitions:
[C/sm_84.c:482](/home/jim/sm-decomp-restart/restart/out/scroll/native/2c9c9a3cf51a059220afac1ee04727c956e45d067e10e17405bf3adbd268fc7f/source/src/sm_84.c:482).
Field authority:
[C/variables.h:412](/home/jim/sm-decomp-restart/restart/out/scroll/native/2c9c9a3cf51a059220afac1ee04727c956e45d067e10e17405bf3adbd268fc7f/source/src/variables.h:412).
Message queuing remains
[C/sm_85.c:425](/home/jim/sm-decomp-restart/restart/out/scroll/native/2c9c9a3cf51a059220afac1ee04727c956e45d067e10e17405bf3adbd268fc7f/source/src/sm_85.c:425),
with real presentation owned by the native scheduler/message engine.

New services are **AddSuperMissilesToHudTilemap/809A0E** and
**AddPowerBombsToHudTilemap/809A1E**, not generic “collect item” delegation.
Both call AddToTilemapInner, conditionally writing four HUD words only when the
first tile's low10 bits equal15. Super writes HUD indexes14,15,46,47; power
bomb writes17,18,49,50. Native source:
[C/sm_80.c:1295](/home/jim/sm-decomp-restart/restart/out/scroll/native/2c9c9a3cf51a059220afac1ee04727c956e45d067e10e17405bf3adbd268fc7f/source/src/sm_80.c:1295),
`AddToTilemapInner` at1311; HUD baseC608. Keep the existing service pre/post
full-WRAM observation and inventory-before-HUD-before-music-before-message
order. Health/reserve add no new service. Existing music's state>=40 no-op
remains part of its behavior; it does not suppress the collector's other writes.

## Complete bounded family, without a per-room catalog

Headers/scripts below are exact cartridge data. P is the collector operand
offset; cleanup is the existing `DFA9: 0001,A2B5,86BC`, then Delete atDFAD.

| Item/header | Native setup / script span | Pickup link / collector P / amount | Already-collected or post-pickup Goto |
|---|---|---|---|
|Energy EED7|EE4D / E099..E0BD|E0B1 / E0B8 /0064|E0BA→DFA9|
|Missile EEDB|EE52 / E0BE..E0E2|E0D6 / E0DD /0005|E0DF→DFA9|
|Super EEDF|EE57 / E0E3..E107|E0FB / E102 /0005|E104→DFA9|
|Power bomb EEE3|EE5C / E108..E12C|E120 / E127 /0005|E129→DFA9|
|Reserve EF27|EE64 / E41D..E449|E43D / E444 /0064|E446→DFA9|

The four37-byte fixed-tank scripts share one exact layout: item-bit branch,
SetLinkReg, PreInstrDF89, alternating four-tick timed draw records, Goto loop,
SetItemBit, ClearMusicQueueAndQueueTrack(2), collector, GotoDFA9. They introduce
no new interpreter primitive. Their timed draw pairs are respectively
`A2DF/A2E5`, `A2EB/A2F1`, `A2F7/A2FD`, `A303/A309`.
Native setupEE4D/EE52/EE57/EE5C sets `plm_variables` to8/10/12/14 and delegates
toEE64's contact-block setup; this is existing native setup behavior, not a
new Rust completion ([C/sm_84.c:2833](/home/jim/sm-decomp-restart/restart/out/scroll/native/2c9c9a3cf51a059220afac1ee04727c956e45d067e10e17405bf3adbd268fc7f/source/src/sm_84.c:2833)).

Reserve's45-byte script is the Morph-shaped variant: LoadItemPlmGfx with
source9000 and eight zero palette bytes, item-bit branch, SetLinkReg, PreInstr,
Frame0/Frame1 loop, SetItemBit, ClearMusic(2), CollectReserve(100), GotoDFA9.
LoadGfx runs **before** the already-collected branch, just as for Morph.
Its gfx and frame services reuse19; no new rendering subsystem is needed.

The13 reused functions are Delete, PreInstr, Goto, GotoIfItemBitSet, SetItemBit,
SetLinkReg, ClearMusicQueueAndQueueTrack, GotoLinkIfTriggered,
CollectAmmoMissileTank, LoadItemPlmGfx, DrawItemFrame0, DrawItemFrame1 and local
DrawItemFrame_Common. All five selected header families must include their
initialization, loop, trigger, collector/message continuation and cleanup,
including the item-already-collected branch. Do not advertise broader ownership
for headers outside this table merely because they invoke the same collectors.

Admission delta: add these exact header/site/script combinations after19;
retain slot, ROM seal, service-capability and atomic-refusal contracts. A coherent
proposed parameter domain is nonnegative permanent item IDs0000..01FF, whose
index fits the64-byte D870..D8AF item map. It must be explicit, rather than
silently replacing18/19's single observed arguments. The bit helper computes
the actual argument's byte/mask and scratch05E7; it cannot retain hardcoded
D873/D874. Alternatively, root can select a concrete placed tank argument
with its future live route while retaining this complete header family.
No placed-room argument or future route is claimed by this note.

## What remains outside this candidate

There are **seven** collector opcode bodies not selected by18/19: the four
above plus PickupBeamAndShowMessage/8488B0, PickupEquipmentAddGrappleShowMessage/
84891A and PickupEquipmentAddXrayShowMessage/848941
([C/sm_84.c:441](/home/jim/sm-decomp-restart/restart/out/scroll/native/2c9c9a3cf51a059220afac1ee04727c956e45d067e10e17405bf3adbd268fc7f/source/src/sm_84.c:441)).
Beam pickup enforces Spazer/Plasma equipment exclusions and calls
UpdateBeamTilesAndPalette/90AC8D, which queues256 graphics bytes and writes
16palette words ([C/sm_90.c:2425](/home/jim/sm-decomp-restart/restart/out/scroll/native/2c9c9a3cf51a059220afac1ee04727c956e45d067e10e17405bf3adbd268fc7f/source/src/sm_90.c:2425)).
Grapple/X-ray require their respective HUD services. Generic Equipment pickup
is already19's body, but other scripts/arguments are additional domains.
Suit scripts may also invoke CallFunction's named Varia/Gravity procedures
(C/sm_84.c:318); these are not implicitly covered by inventory OR semantics.
None is required to finish the exposed-tank family above.

## Reuse, compact verification and the remaining decision

Main193cad9 has no direct implementation of the four new collector names or the
two HUD services. Reuse18/19's adapted runtime first. Main's
[plm_instructions.rs](../../crates/port/src/plm_instructions.rs), `instruction`
and `process_draw_data`, and
[plm_render.rs](../../crates/port/src/plm_render.rs), `draw`, remain useful source
references for shared control/draw behavior. Focused existing counterexamples
include `draw_rows_columns_and_negative_origin_offsets` and
`full_tilemap_stops_before_following_segment_and_full_queue_does_not_draw` in
their matching test files. Enemy-drop restoration code is **not** a tank
collector replacement: it clamps to existing maxima and can overflow into
reserves ([enemy_projectile_instructions.rs:1700](../../crates/port/src/enemy_projectile_instructions.rs:1700));
tank acquisition changes maxima with different ordering and refill rules.

Implementation can stay in the existing item gameplay crate and item-only native
adapter. Add four bodies, two named services, declarative header/site domains
and differentiated ownership records. Root owns shared registration, source
freeze and execution. The compact actual-C delta should distinguish health
refill, reserve-mode zero/nonzero and untouched current reserve, wrapping ammo/
capacity, HUD blank/already-present branches, and complete selected script
chains including already-collected initialization. Compare typed results,
fullWRAM and ordered service images; derive cases from these behavioral
differences, not a numeric fixture quota. No new framework or additional
transport suite is proposed.

The unchanged tape SHA256
`43d67592cb3da2a9dd9c1568f609aa19b60bcb555a28b643453427677908be69`
can check that18/19's Morph/Chozo-missile behavior and existing gameplay stay
unchanged. It **cannot prove material live execution of these new collectors**.
Its accepted [replay result](/home/jim/sm-decomp-restart/restart/out/first-missile/replay-01/result.toml:245)
records Morph then missile acquisition only; bounded reads of
[final.wram](/home/jim/sm-decomp-restart/restart/out/first-missile/replay-01/final.wram)
show equipment/collection4, beams0, health/max99, missiles/cap5,
supers/cap0, power bombs/cap0 and reserve mode/max/current0. No old callback
trace establishes live entry into the four collector bodies. Merely loading an
uncollected tank or recording its shared Goto/PreInstr cannot fill that gap.

**Remaining root decision:** is the next milestone explicitly source translation
plus actual-C family verification, or must it add a newly reached live tank?
For the latter, one concrete placed tank and ordinary-input reach evidence are
still missing; choosing/authoring that route is separate work, not justification
to expand this research or rerun the current tape as proof of new collection.
Keep source, actual-C fixtures, live ownership and original-ROM parity separate.
This packet establishes no new parity or gameplay completion and must not gate
active19 review/closeout.

## Completion and cleanup

One bounded source/data investigation; only this note written. No19 code audit
or changes, build, test, game, capture, tool/framework, recurring process or
ticket20 claim. No run artifacts or retained dependencies added; cleanup0bytes.
Free space at inspection102,830,837,760bytes. Root retains selection.
