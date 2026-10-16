# First missile pickup after Morph Ball

Status: research complete; conditional recommendation, not an implementation claim
Blocked by: restart ticket16 successful pickup, replay and closeout
Triage: ready-for-agent
Owner: recovery reserve `01a144fc-f095-7d71-aa00-10358e7f716e`
Date: 2026-10-16

Recommend one milestone: use Morph Ball under ordinary input, travel from its
room through Construction Zone to First Missile, reveal and collect the Chozo
missile tank, then resume material controller response. Preserve ticket16's
eventual successful input prefix and author only a bounded tail. The integrator
owns selection, bounds, implementation and execution. This note is the only
assigned write; current Morph policy, its diagnosis and all watches retain their
existing owners.

## Source authority and limits

**C** means the retained recovered-C source directory:
`/home/jim/sm-decomp-restart/restart/out/scroll/native/2c9c9a3cf51a059220afac1ee04727c956e45d067e10e17405bf3adbd268fc7f/source/src/`.
Its upstream pin is `578f90b3cc49557bb70060ad033bb90b8cf8ac50`, as recorded in
the preceding Morph source packet. Exact file hashes are below.

**CS** means
`/home/jim/sm-decomp/out/research/community/SuperMetroidDecomp/csharp/src/SuperMetroid.Core/`.
The actual repository root was checked, at commit
`216f3a1d38c2e9d82ec9c0e626a35415b74d47c9`; the consulted Rooms directory and
pose-rule source have no working-tree changes. These community-authored retail
data declarations supply the route, placement and input-table rows below.
They are secondary reconstruction evidence, not a fresh original-ROM byte or
runtime verification. The C routines establish the corresponding mechanics.
No ROM capture, disassembly, gameplay run or original-parity claim was made.

This distinction matters: exact ROM data corroboration of the proposed doors,
population and pose rows remains a small pre-authoring check if the integrator
requires cartridge authority. It does not require repeating the Morph research.
The source-location targets for that check are included below.

## Route and placement

| Leg | Source declarations | Door and anchor |
| --- | --- | --- |
| Morph `9E9F` → Construction `9F11` | CS `Rooms/RoomHeaderDefinitions.cs:141`; `DoorDefinitions.cs:137`, `:700` | Morph list `9EE5`, index1 → door `838EAA`. Source east cap from reverse `838EC2`: block `(126,38)`, pixel anchor `(2016,608)`. Destination left cap `(1,6)`. |
| Construction `9F11` → First Missile `A107` | CS `Rooms/RoomHeaderDefinitions.cs:143`, `:161`; `DoorDefinitions.cs:141`, `:158`, `:701`, `:710` | Construction list `9F57`, index2 → door `838EDA`. Source lower west cap from reverse `838FA6`: block `(1,22)`, anchor `(16,352)`. Destination right cap `(14,6)`. |

All three rooms are Brinstar, area1. Construction is one screen wide and two
high; First Missile is one by one. `DoorDefinitions.cs:631` defines list
indexing and its 12-byte physical stride. The source route is through the lower
west exit, not the upper-right exit to Blue Brinstar Energy Tank `9F64`.
Construction's population `8754` contains a red door at block `(14,6)` and
scroll triggers across row11 (`RoomPlmPopulationDefinitions.Generated.cs:417`).
That red door is not the recommended lower-west exit. The Morph-to-Construction
and Construction-to-First-Missile door records have no door ASM. The reverse
First-Missile-to-Construction record has `BE25`; returning is outside this one
pickup checkpoint.

First Missile state `A114` selects population `8802`
(CS `Rooms/RoomStateDefinitions.cs:183`). That population places exactly one
Chozo missile tank at block `(4,7)`, room argument `0022`
(`RoomPlmPopulationDefinitions.Generated.cs:461`). Its item block spans
X64–79/Y112–127; those are block bounds, not a tested Samus-center tolerance.
The header is `EF2F` (`RoomPlmHeaders.cs:306`), setup `EE52`, instruction list
`E47C` (`RoomPlmHeaderDefinitions.Generated.cs:58`).

Cartridge-data cross-check targets: bank8F headers `9E9F/9F11/A107`, door lists
`9EE5/9F57/A12E`, state `A114`, population `8802`; bank83 doors
`8EAA/8EC2/8EDA/8FA6`; bank84 header `EF2F` and instruction list `E47C`.
No platform collision geometry, tunnel aperture or exact navigation tolerance
has been established from these topology declarations.

## Ability and actual interaction predicates

Morphing is gated by `equipped_items & 4`: C `sm_91.c:3491`
(`Samus_MorphTrans`, `91F7CE`) otherwise restores the prior pose. The input
matcher consumes both newly pressed and held input (`sm_91.c:208`). The
reconstructed table rows require fresh Down from standing to crouch, then a
fresh Down from crouching to morph: CS `Game/SamusPoseInputRulesEarly.cs:74`,
`:666`, `:699`. Release between presses and observe the intermediate pose;
holding Down continuously does not establish the second edge. Avoid conflicting
new Up/turn/jump chords, which have higher priority in those tables.
Down is SNES `0400`, native pad `0020`, consistent with the existing Morph
runner's bit reversal. This is a source-derived input policy, not proven timing.

Require observed movement while morphed, rather than inferring use from inventory.
C `ida_types.h:301` names ground poses `1D/1E/1F`, and `:326` names `41`;
movement type4 is Morph Ball on ground (`:523`). Observe pose at WRAM `0A1C`,
movement type at `0A1F` (C `variables.h:445`), ordinary consumed directional
polls and changed position. This can prove material Morph use even though this
note does not prove which precise tunnel cells force the ability.

The missile is a **Chozo orb**, not an exposed pickup. CS
`Rooms/RoomPlmSystem.Collectibles.cs:141`, `:404` separates projectile-triggered
shell reveal from later visible-item touch (`:113`). Breaking the shell alone
does not set item persistence. Use the mapped ordinary uncharged shot after
standing where the actual collision geometry permits; no bombs or pre-existing
missiles are implied by this recommendation. The source does not establish a
shot location, projectile count or frame budget.

C `sm_84.c:2837` selects missile item graphics/setup through `EE52/EE5F/EE64`.
`sub_84EEAB` at `:2890` matches the collision block against the item's PLM and
writes timer255 when time is not frozen. `PlmPreInstr_GotoLinkIfTriggered` at
`:2782` recognizes low timer byte255. The existing projectile collision adapter
admits setup `EEAB` and calls the synchronous C SpawnPLM service
(`restart/projectile/src/collision.rs:52`, `:63`). These are useful existing
boundaries, not proof that the new orb was hit successfully.

## Material completion recommendation

Retain the prefix's actual missile count/capacity and item bit before authoring.
For the expected first pickup, require all of the following in order:

1. Material ordinary-input Morph movement after ticket16's completed prefix.
2. Ordered room progression `9E9F → 9F11 → A107`, with pickup in area1.
3. Clear-to-set item argument `0022`: `item_bit_array[4] & 04`, WRAM `D874`.
   C `sm_84.c:432` calls `PrepareBitAccess`; `sm_80.c:53` defines index `a>>3`
   and mask `1<<(a&7)`; `variables.h:1097` defines base `D870`.
4. Missile capacity and current ammo increase by5 at acquisition. Their words
   are `09C8` and `09C6` (`variables.h:415`). C
   `PlmInstr_CollectAmmoMissileTank`, `sm_84.c:499`, adds the instruction operand
   to both words, updates HUD and requests message2. CS
   `RoomPlmSystem.Collectibles.cs:645` supplies the reconstructed operand5.
   With a zero-ammo baseline and no missile firing, both final words should be5.
5. Message2 finishes, freeze/message continuation clears, normal state8 resumes
   in area1/roomA107 with positive health, then newly consumed ordinary controls
   cause material motion. Reuse the current runner's item/message observations;
   message09's earlier proof does not itself verify message2.
6. Replay the saved successful input sequence without policy, preserving the
   exact prefix and the same completion observations. Loading, orb destruction,
   inventory alone or tail exhaustion is insufficient.

No numeric tail allowance or deadline is inferred from source geometry. Preserve
the first incomplete attempt and fix the first observed dependency within the
integrator's selected bounds. The eventual ticket16 input identity does not yet
exist as a completed result for this note to freeze.

## Existing ownership and next coherent gameplay opportunity

Ownership reference: restart source inspected at
`892b87ff1187d15f30023b16c6aacc3edcebf739`, existing
`restart/projectile/README.md`, `PROJECTILE_ESCAPE.md`, `CAMERA.md`, and ticket16.

| Behavior needed | Current owner and useful boundary |
| --- | --- |
| Ordinary uncharged firing, projectile scheduling/terrain collision/trails and ordinary drawing | Existing Rust projectile consumer within its admitted domain. Orb/block reaction crosses synchronous C SpawnPLM; C still owns item PLMs. |
| Room/map scrolling, directional transition scrolling, room transition roots and normal camera | Existing Rust consumers with documented generic native services. The Ceres-door consumer does not establish Rust ownership of these Brinstar door actors. |
| Morph pose transitions, Morph movement and Samus terrain collision | Retained C (`sm_91.c:3491`, `sm_90.c:1944`, `:1993`), not the Rust route policy. |
| Orb reveal, touch, item persistence, ammo/HUD mutation, pickup message and continuation | Retained C item/PLM and presentation behavior. |
| Input authoring and completion observer | Rust tool policy; it adds no translated gameplay ownership. |

Merely acquiring missiles does **not** invalidate the current projectile
consumer: `projectile/src/lib.rs:35` guards equipped beams, Hyper Beam, echoes
and active projectile kinds, not missile ammo/capacity. Actually firing a missile
introduces a different active type outside the admitted ordinary beam/impact
pairs (`:54`); missile firing and flight remain outside this proposed pickup
checkpoint and outside existing Rust ownership. Do not turn an ammo increment
into an implicit claim of Rust missile support.

The most directly exposed next coherent translation opportunity is the ordinary
item pickup lifecycle: shell reveal through touch, permanent item bit, inventory
effect and message-return continuation. This would replace a concrete C service
boundary reached by existing Rust projectiles and compare the full interaction.
It is an advisory scope opportunity, not another assignment or a requirement to
translate item logic before proving the native milestone. Morph pose/movement is
also C-owned; this note does not launch that larger subsystem.

## Remaining source gaps

The route/data and pose rows are pinned community reconstruction declarations;
direct cartridge-byte corroboration was not performed. Narrow tunnel geometry,
usable shot position, exact message2 dismissal behavior and native route success
remain unproved. Do not substitute map anchors for activation tolerances. Source
topology supports this one recommendation; actual observations select any fix.

## Exact source identities and cleanup

SHA-256 of consulted C files:

```text
7a7da7a5c1a35033c8cac6d608cc9a3c043c52347504ab0f8609ac2c6acc095a sm_84.c
8a087a687e9611d1272797522f1b5fd20f9e344d851e0a6701af8b9052b5070f sm_91.c
38a661917219e6647826d25ea0bc45039c36bd54924f8cf968facb3f463389fa sm_90.c
e2ce28d07ceff91129e93417099f33f20f161c37e95e745c575d0e30b3fd5585 variables.h
```

SHA-256 of CS files, under the directory prefixes cited above:

```text
87eca378cc8080882d37cb2b8cb12be06ef2d567d019e3060a53cbe98003079d Rooms/RoomHeaderDefinitions.cs
3e504392322729df7469a93600a3903fc98566a2088027e86793b196ab40414f Rooms/DoorDefinitions.cs
62a0435ae18b4c24973f36620fc79500de554a06f73077794bdd66f3d98dfb53 Rooms/RoomStateDefinitions.cs
606c2ff9d3996d3fbe8f862f7a07bcfc02fa453de1fd1f68476a6bc3e984badf Rooms/RoomPlmPopulationDefinitions.Generated.cs
d86ae5ff8c2b2ea9dfabefef7d0b6c6fcd1183fa3f13cebbefa8066ab79dbf1e Rooms/RoomPlmSystem.Collectibles.cs
f0375fc5d0b2499f3a40e3def94ce3a26a1d03c6043251515595afa7e7a0ebdf Game/SamusPoseInputRulesEarly.cs
```

This assignment read source and wrote only this Markdown note. No gameplay,
tests, captures, project builds, worktrees or verification outputs were created;
run cleanup is not applicable, zero bytes reclaimed for this research. Warm
caches, other owners' source and all retained evidence remain intact. Available
disk at the source checkpoint: 103,596,990,464 bytes. No new gameplay stage is
claimed or published. The integrator retains the next selection and claim.
