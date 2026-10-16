# One new collector: Terminator exposed energy tank

Status: bounded source research complete; root retains selection and live authoring.
Owner: `/root/live_tank_route_recovery` (canonical tool identity; no session UUID exposed).
Recovered task: original researcher errored; original deadline 2026-10-16 16:56:50 UTC retained.

**Select room `8F990D`, exposed energy header `EED7`, permanent item argument `0008`.**
This is a concrete current-inventory candidate. The cartridge establishes its placement,
ungated door connection and collector; the proposed reverse traversal still requires
ordinary-input live validation. This note neither claims a completed route nor supplies pads.

## Exact retained endpoint and source authority

Pinned recovered C is commit `578f90b3cc49557bb70060ad033bb90b8cf8ac50`, retained at
`/home/jim/sm-decomp-restart/restart/out/scroll/native/2c9c9a3cf51a059220afac1ee04727c956e45d067e10e17405bf3adbd268fc7f/source/src/`.
Reads below use that source and bounded `od` reads of pristine `rom/sm.sfc`, whose
previously authenticated SHA256 is `12b77c4bc9c1832cee8881244659065ee1d84c70c3d29e6eaf92e6798cc2ca72`.

Accepted `first-missile/replay-01/result.toml` records checkpoint pad24832, message2
finished24814, health99, missiles/current capacity5. Its `final.wram` has roomA107
at079B, Samus x0066/y008B at0AF6/0AFA, pose0009 at0A1C; equipped/collected
items0004, beams0, energy/current maximum99, supers/power bombs/reserves0.
Persistent item bytes D870..D877 are `00 00 00 04 04 00 00 00`; target D871 bit01
is clear. Preserve all24832 pads of `first-missile/author-05/inputs.txt`, SHA256
`43d67592cb3da2a9dd9c1568f609aa19b60bcb555a28b643453427677908be69`.

## Exact placement and item predicates

Room8F990D is area0/index18hex, six screens wide by three high. Its default state
8F991A points to PLM population8F8432. That population starts with the six bytes
`D7 EE 07 2A 08 00`, then the zero terminator: headerEED7, **x block07, y block2A**,
argument0008. Pixel cell is x112..127/y672..687. Room-width96 gives PLM block
byte index `2*(42*96+7)=1F8E`. Coordinate interpretation is C/ida_types.h:69
(`RoomPlmEntry`), state loading C/sm_82.c:3650 and C/sm_84.c:111.

Bank84 headerEED7 contains setupEE4D/scriptE099. The exact fixed-tank chain is
already in [the family packet](next-ordinary-item-scope.md): E099 item-bit branch,
linkE0B1, PreInstrDF89, four-tick A2DF/A2E5 draw loop, E0B1 SetItemBit, music2,
collector848968 with operand pointer84E0B8 containing0064, then GotoDFA9/Delete.
Draw records84A2DF/A2E5 are `0001,B04A/B04B,0000`.

C/sm_84.c:2833 sets item variable8 through EE4D/EE5F; EE64 sets BTS45 and initially
clears the block type. The timed draw installs B04A/B04B, making the exposed item
a special collision block. C/sm_94.c:790/806 dispatches BTS45 through table949139
to header84EED3; that header is setupEEAB/scriptAAE3. EEAB (C/sm_84.c:2889)
requires `time_is_frozen_flag==0`, finds the item slot by matching block index and
sets its timer low byteFF. DF89 (line2782) then jumps to the pickup link.
The trigger/collector has **no specific pose, missile, bomb, super, power-bomb,
Hi-Jump or beam-upgrade predicate**: physical collision with this exposed cell
while unfrozen is sufficient. Unmorph for tall traversal; Morph remains available
for previously visited low passages. No shot-block revelation or Chozo shell is
required for this header.

Argument0008 maps through PrepareBitAccess (C/sm_80.c:53) to D871/mask01.
It must remain clear before contact; SetItemBit changes it before the collector.
New body848968 (C/sm_84.c:482) raises max09C4 by100, copies it to current09C2,
calls existing music delay0168, queues message1 and returnsE0BA. With retained
maximum99 the result is **max/current199** regardless of intervening damage.

## Smallest supported route proposal

Backtrack the already visited corridor and add the direct Parlor→Terminator edge:

| From → to | Exact outgoing door definition (bank83) |
|---|---|
|A107 FirstMissile →9F11 Construction|8FA6|
|9F11 →9E9F Morph|8EC2|
|9E9F →97B5 elevator|8EB6 (elevator flagsE0/direction03)|
|97B5 →975C Pit|8B92|
|975C →96BA Climb|8B7A|
|96BA →92FD Parlor|8B3E|
|92FD →990D Terminator|895E|

The ordinary door records have zero door bitflags; they do not encode equipment
requirements. **This does not substitute for inspecting live door caps/enemies.**
The final direct edge is exact: Parlor door list8F9362 entry0 is895E, whose record
is `0D 99 00 05 5E 06 05 00 00 80 00 00`; it enters Terminator at right-side
cap block(5E,06), destination screen(05,00), direction05. Terminator door list
8F9934 contains8BE6/8BF2;8BF2 returns directly to Parlor. Terminator's complete
PLM population contains only the exposed tank, so there is no placed colored
door/item-shell PLM gate in that population.

Proposed inputs: release after the retained message tail, face/move left and use
ordinary beam shots for blue caps; ascend Construction, cross the retained Morph
passage, take the elevator upward, climb the already visited Pit/Climb corridor,
then take Parlor's west door. Use ordinary jumps and, if needed, native wall jumps;
descend/walk west through Terminator to cell(07,2A), contact the tank, acknowledge
message1 and demonstrate resumed motion. Current inventory supplies Morph and
ordinary beam/jump; native wall jump does not require Hi-Jump (C/sm_90.c:1010,
1501). Exact jump timing, reverse-climb feasibility, cap opening, enemy survival
and Terminator terrain approach remain **proposed navigation**, not retained
live evidence. The bounded investigation did not decompress or catalog terrain.
This is the smallest supported door-graph continuation; it is not a claim of a
shortest frame count. If reverse ascent fails, the next coherent milestone is a
prefix-preserving return to Parlor with a measured room/pose/block failure; do
not silently substitute a hidden Blue Brinstar tank outside the exposed scope.

## Live acceptance to close the remaining gap

Future author/replay must preserve the entire24832-pad prefix and its accepted
endpoint WRAM, observe the above ordered room transitions, and retain an
uncollected EED7/0008 slot in990D before contact. Require material Rust ownership
of new848968 at actual operand84E0B8/0064 with full-WRAM and ordered services;
loading/shared-loop callbacks alone do not count. At pickup require D871 bit01,
max/current199, message1, and unchanged Morph/ammo capacities except ordinary
ammo use before pickup. After message completion require consumed ordinary-input
polls and movement, then one strict replay of the exact extended tape.
Original-ROM parity is a separate future verdict; this source note establishes none.
Root selects the next scope; this research does not gate20 or auxiliary21.

## Completion and cleanup

One note written; no source changes, route authoring, build, test, emulator,
capture, new framework, sub-agent or run producer. No run directories or retained
dependencies created: finished-run cleanup is not applicable; reclaimed0bytes.
Existing immutable evidence/source dependencies remain in place. Available disk
at inspection:100871483392bytes. Report published before the original deadline.
