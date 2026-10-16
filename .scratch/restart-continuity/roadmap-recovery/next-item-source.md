# Next item source recommendation: Morph Ball

Status: bounded source research complete; advisory for ticket16 selection after15
Owner: `/root/projectile`; sole writable path is this note; main index not owned
Date: 2026-10-16

Recommend one milestone: continue the proven Parlor checkpoint (area0, room92FD,
Samus x1211/y139) through Climb, Pit Room and the elevator, collect Morph Ball in
area1/room9E9F, then demonstrate ordinary control after the pickup message ends.
This is the first compact item objective supported by the inspected room links
and pickup data. It is not a completed native route or a new claim on ticket16.
Root chooses the next implementation scope after ticket15 closeout.

The earlier [progression packet](../../restart-collaboration/next-progression-milestone.md)
and [recovery recommendation](next-progression-milestone.md) already own Landing
Site and Parlor arrival. This note begins in Parlor and does not repeat that
geometry or propose changes to its accepted prefix.

## Sources and identity

Primary room, door, enemy and PLM data were read directly from the retained
unheadered ROM `/home/jim/sm-decomp/rom/sm.sfc`, SHA-256
`12b77c4bc9c1832cee8881244659065ee1d84c70c3d29e6eaf92e6798cc2ca72`.
ROM offsets below are file offsets; SNES addresses are also given. LoROM mapping
is `(bank & 0x7f) * 0x8000 + (address & 0x7fff)`.

Pinned recovered C source, abbreviated **C** below, is retained at:

`/home/jim/sm-decomp-restart/restart/out/scroll/native/2c9c9a3cf51a059220afac1ee04727c956e45d067e10e17405bf3adbd268fc7f/source/src/`

This is the packet's pin `578f90b3cc49557bb70060ad033bb90b8cf8ac50`. The main
`deps/snesrev-sm` checkout now reports `193cad941713b3539d23e08238a79e7fb09dde61`;
final code citations here use the retained C path, avoiding an assumption that
the moving dependency checkout is the pinned restart reference.

Names are corroborated by retained community
`out/research/community/SuperMetroidDecomp/csharp/src/SuperMetroid.Core/Rooms/RoomHeaderPointers.cs:9–25`.
The ROM owns the connections and coordinates. Its structure is decoded by
**C/ida_types.h:260–270** (`DoorDef`), **:889–900** (`RoomDefHeader`), and
**C/sm_84.c:160–184** (`SpawnRoomPLM`).

## Exact room sequence and door waypoints

Door-list indices below are zero-based. Door cap positions are 16-pixel block
anchors, not Samus center coordinates or a collision tolerance. For source-room
caps, the reciprocal door record names that room and gives its cap position.
Screen positions in outgoing records describe the destination.

| From → to | Outgoing list entry and door record | Source-room waypoint | Destination data |
| --- | --- | --- | --- |
| Parlor92FD → Climb96BA, area0 | `$8F:936A` / file `0x7936A`, list9362 index4 → `$83:898E` / `0x1898E` | South cap block `(0x16,0x4D)` = `(22,77)`, pixel anchor `(352,1232)`; reciprocal `$83:8B3E` / `0x18B3E` | Direction06 (down); cap `(22,2)`, screen `(1,0)` |
| Climb96BA → Pit975C, area0 | `$8F:9725` / `0x79725`, list971F index3 → `$83:8B62` / `0x18B62` | East cap block `(0x1E,0x86)` = `(30,134)`, pixel anchor `(480,2144)`; reciprocal `$83:8B7A` / `0x18B7A` | Direction04 (right); cap `(1,6)`, screen `(0,0)` |
| Pit975C → elevator97B5, area0 | `$8F:97A3` / `0x797A3`, list97A1 index1 → `$83:8B86` / `0x18B86` | East cap block `(0x2E,6)` = `(46,6)`, pixel anchor `(736,96)`; reciprocal `$83:8B92` / `0x18B92` | Direction04 (right); cap `(1,6)`, screen `(0,0)` |
| Elevator97B5 → Morph9E9F, area1 | `$8F:97FC` / `0x797FC`, list97FA index1 → `$83:8B9E` / `0x18B9E` | Down elevator described below | FlagsF0, direction02; destination screen `(5,0)`; reverse `$83:8EB6` / `0x18EB6` has flagsE0/direction03 |

Room headers at file `0x792FD`, `0x796BA`, `0x7975C`, `0x797B5` and `0x79E9F`
give areas `0,0,0,0,1` and widths/heights in screens `5×5,3×9,3×2,1×1,8×3`.
The third elevator-room list entry is pseudo-door88FC, not a physical header.
These bytes independently corroborate the community door catalog at
`.../Rooms/DoorDefinitions.cs:28,64–72,656,672–674,700`.

Practical spatial sequence: move west from current upper-right Parlor to the
shaft leading to its south exit; descend Climb to its low east exit; traverse
Pit east; align with the elevator; descend; then move left to the item. The
exact platforms, jump inputs, shot timing and safe descent path still require
ordinary native input authoring. Door topology alone does not establish them.

## Elevator and item position

Elevator room default state97C6 points to enemy population `$A1:8B61`
(file `0x108B61`). Its first record is enemyD73F at `(0x80,0xA2)` = `(128,162)`,
parameter1=0, parameter2=0140. **C/sm_a3.c:491–505** initializes its direction;
**:514–524** checks a fresh input edge. Table `$A3:94E2` / file `0x1194E2`
contains `0400,0800`: this downward elevator requires fresh Down (`0400`).
**:567–575** aligns Samus to enemy X and enemy Y minus26, giving `(128,136)`.
Release/repress Down when needed to create the edge; no pad duration is inferred.

Morph room default state9EB1 points to enemy population `$A1:93AC`
(file `0x1093AC`). Its elevator record at `$A1:93CC` / `0x1093CC` is D73F at
`(0x580,0x2C2)` = `(1408,706)`, parameter1=1, parameter2=0018. Thus its normal
end alignment is Samus `(1408,680)`; entry motion uses the same native elevator
state machine, including its temporary initial position. Do not treat that
endpoint as the first observable post-transition position.

Default room state `$8F:9EB1` has PLM population867E (pointer at `$8F:9EC5`,
file `0x79EC5`). The population's item entry at `$8F:86DE` / file `0x786DE` is
exactly `23 EF 45 29 1A 00`: PLM **EF23**, block **(69,41)**, argument **001A**.
Its tile spans pixel x1104–1119/y656–671, left of the elevator. This is a tile
anchor, not an exact required Samus center.

Header `$84:EF23` / file `0x26F23` is `64 EE EF E3`: setupEE64 and scriptE3EF.
**C/sm_84.c:2854–2858** makes its tile a special-air contact block, BTS45.
Script `$84:E3EF` / file `0x263EF` checks the existing item bit, installs
trigger preinstructionDF89 and pickup linkE40F. **C/sm_84.c:2782–2789** follows
the link when the timer's low byte isFF; **:2889–2905** supplies that trigger
for contact with an unfrozen item block. The pickup sequence at `$84:E40F`
(file `0x2640F`) includes SetItemBit8899, then PickupEquipment88F3 with
equipment word0004 and message09. This confirms Morph Ball, not merely an item
sprite or arrival in its room.

Initial route states remain pre-awakening/default states. Pit and elevator
state selectors use `RoomDefStateSelect_MorphBallMissiles` at
**C/sm_8f.c:731–737**: both Morph bit4 and nonzero max missiles are required
for their alternate state. Acquiring Morph alone does not meet that condition.
This recommendation ends before collecting missiles or revisiting those rooms.

## Proposed material pickup predicate and scope limit

Before pickup, retain the absence of Morph and its room-item bit. After observed
contact/message progression, require all three present:

- `word(WRAM09A2) & 0004 != 0` — equipped Morph Ball.
- `word(WRAM09A4) & 0004 != 0` — collected Morph Ball.
- `byte(WRAMD873) & 04 != 0` — item argument001A persisted in the item bit array.

**C/variables.h:397–398,1097** owns these addresses.
**C/sm_84.c:432–439,453–460** sets the item bit and ORs both equipment words;
**C/sm_80.c:53–56** computes byte index `001A >> 3 = 3` and mask
`1 << (001A & 7) = 04`. Require normal state8, area1, room9E9F, positive health,
and a short observed ordinary-input response after the message, rather than
accepting the first inventory mutation during the pickup sequence.

Reuse the frozen successful ticket15 input prefix and existing runtime capture,
replay, state and image comparison mechanisms. Append only an ordinary-input
tail. The authoring owner should stop on the first unexplained reached failure
and retain it; this source note adds no independent execution or verification
gate and proposes no new framework. It grants no Rust ownership for room/item,
elevator, message, Samus collision, or remaining native behavior. The unchanged
native route, successful pickup and reached dependencies still need execution.
No legacy TAS timing, pads, synchronization or route feasibility is inferred.

## Cleanup and handoff

Only this Markdown file was written. Read-only source/ROM inspection finished;
no code, gameplay run, capture, build, prepared native package, new tool or run
intermediate was created. Reclaimed bytes0; no run cleanup utility applies.
Available disk at handoff:104139300864 bytes. Existing evidence, main index,
other agents' files and claims remain untouched. Ownership of this note is
released to root/recovery for synthesis; continuity owns its routine watch.
