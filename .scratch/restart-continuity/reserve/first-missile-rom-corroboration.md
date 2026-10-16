# First Missile cartridge-data corroboration

Status: complete within narrowed scope; item-script semantics remain with material
Owner: reserve `01a144fc-f095-7d71-aa00-10358e7f716e`
Date: 2026-10-16, bounded check started 14:24 UTC
Scope: additive corroboration for ticket17; no implementation ownership

The retained ROM directly corroborates the proposed room/door chain, missile
population/header and fresh-Down pose rows. No disagreement with those rows in
the preceding community-source note was found. This closes their cartridge-data
gap; it does not prove navigation, collision geometry, timing or gameplay success.
Early bytes were sent directly to integrator01a144ca before final publication.

## Identity and method

Read-only `sha256sum`, `stat` and `xxd -g1 -s OFFSET -l LENGTH` inspected
`/home/jim/sm-decomp/rom/sm.sfc` (3,145,728 bytes). SHA-256:
`12b77c4bc9c1832cee8881244659065ee1d84c70c3d29e6eaf92e6798cc2ca72`.
This equals the original-ROM identity retained by the prior Morph source packet.
No header adjustment is applied. LoROM mapping is
`file = (bank & 0x7F) * 0x8000 + (address & 0x7FFF)`.
Words below are little-endian. Hex offsets are absolute file offsets.

Structures are interpreted using the same pinned recovered C as the preceding
note, upstream `578f90b3cc49557bb70060ad033bb90b8cf8ac50`:
`restart/out/scroll/native/2c9c9a3cf51a059220afac1ee04727c956e45d067e10e17405bf3adbd268fc7f/source/src/`
under `/home/jim/sm-decomp-restart/`.
`ida_types.h` defines DoorDef/RoomDefHeader; `sm_91.c:208` matches pose rows.
The preceding note retains exact source hashes and semantic citations.

## Room headers and door lists

First11 header bytes are room index, area, map X/Y, width/height, remaining
header bytes and the little-endian door-list pointer:

```text
8F9E9F file079E9F: 0e 01 15 08 08 03 70 a0 00 e5 9e
8F9F11 file079F11: 0f 01 1d 0a 01 02 70 a0 00 57 9f
8FA107 file07A107: 18 01 1c 0b 01 01 70 a0 00 2e a1

8F9EE5 file079EE5: 9e 8e aa 8e b6 8e fc 88
8F9F57 file079F57: c2 8e ce 8e da 8e
8FA12E file07A12E: a6 8f
```

Thus all three rooms are area1; Morph is8×3 screens, Construction1×2 and First
Missile1×1. Morph list index1 selects8EAA; Construction index2 selects8EDA.
The Morph list's final88FC is its pseudo-door, not another physical door.

Each12-byte door is destination word, flags, direction, cap X/Y, screen X/Y,
distance word and door-ASM word:

```text
838EAA file018EAA: 11 9f 00 04 01 06 00 00 00 80 00 00
838EC2 file018EC2: 9f 9e 00 05 7e 26 07 02 00 80 00 00
838EDA file018EDA: 07 a1 00 05 0e 06 00 00 00 80 00 00
838FA6 file018FA6: 11 9f 00 04 01 16 00 01 00 80 25 be
```

These confirm Morph9E9F→Construction9F11→FirstMissileA107. Reciprocal source
cap anchors remain Morph `(126,38)` and Construction lower-west `(1,22)`;
destination caps are `(1,6)` and `(14,6)`. Only the reverse return from First
Missile has ASM BE25. Cap anchors establish no Samus-center tolerance.

## State, population and item header

First Missile header continuation at file07A112 begins `e6 e5`, the default
state selector. The following stateA114 bytes were read in the43-byte A107 dump:

```text
8FA114 file07A114:
50 fd c5 06 00 00 14 83 b2 9e e5 85 01 01 00 00
00 00 00 00 02 88 be ba d5 91

8F8802 file078802: 2f ef 04 07 22 00 00 00
84EF2F file026F2F: 52 ee 7c e4
```

The state's PLM pointer is8802. The six-byte population entry is headerEF2F,
block X4/Y7, room argument0022, followed by the zero terminator. The header
selects setupEE52 and instruction listE47C. Thus the prior note's item placement
and argument are now directly corroborated. `0022` maps to item byteD874 mask04
using pinned C `PrepareBitAccess` and `PlmInstr_SetItemBit` as previously cited.

An initial diagnostic read used file070802 by mistake; it showed a zero word.
That is bank8E:8802, not bank8F:8802. The corrected LoROM offset078802 above
contains the expected item. This was an inspection-address correction, not a
cartridge/community-source conflict.

## Pose-pointer rows and input edges

The table begins at91:9EE2. Its pose01/02 and27/28 pointers read:

```text
919EE4 file089EE4: ec a0 72 a1
919F30 file089F30: 6c a6 bc a6
```

The relevant six-byte rows are new-input, held-input and target pose:

```text
91A116 file08A116: 00 04 00 00 35 00
91A19C file08A19C: 00 04 00 00 36 00
91A68A file08A68A: 00 04 00 00 37 00
91A6DA file08A6DA: 00 04 00 00 38 00
```

Standing right/left therefore selects crouch transition35/36 on fresh0400;
crouching right/left selects morph transition37/38 on another fresh0400.
The full corresponding lists were read through theirFFFF terminators, including
higher-priority jump/aim/Up/turn chords. Release/repress Down after observing
crouch and avoid competing inputs. The rows corroborate the earlier source
recommendation; they do not assert how many host steps each pose transition takes.
The equipped-item gate remains established by pinned C `Samus_MorphTrans`.

## Item-script bytes handed to material

These bytes were read before continuity narrowed script interpretation to the
integrator's material worker, and were relayed verbatim. This reserve makes no
additional control-flow/operand claim from them:

```text
84E47C file02647C:
7c 88 a8 e4 2e 8a af df 2e 8a c7 df 24 8a 9f e4
c1 86 89 df 4e 87 16 04 00 eb a2 04 00 f1 a2 24
87 93 e4 99 88 dd 8b 02 a9 89 05 00 01 00 b5 a2
bc 86 7c 88 da e4 2e 8a af df 2e 8a c7 df 24 8a
```

Material owns exact EF2F/E47C shell-versus-revealed-item semantics and binding
the instruction operands to the +5/message2 effect. The preceding note already
cites C `PlmInstr_CollectAmmoMissileTank` for inventory/HUD/message mechanics.
There is no need for a second reserve interpretation or broader script audit.

## Cleanup and handoff

Only this note was written. Reads created no gameplay output, capture, build,
worktree or intermediate file; run cleanup is not applicable and reclaimed bytes
are0. Available disk at closeout: 103,551,410,176 bytes. All ROM/source/evidence
and compacting/live owners remain intact. Remaining
work is implementation-owned script semantics and actual bounded authoring;
tunnel/shot geometry and runtime timings remain unproved. No stage is published.
