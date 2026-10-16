# Successor candidate: ordinary Morph Ball instruction ownership

Status: research complete; advisory after ticket18, no successor claim
Owner: `/root/main_reuse_audit`; root integrator selects the next scope
Date: 2026-10-16

**Candidate:** own the complete ordinary Morph Ball PLM instruction family for
header `EF23`, argument `001A`. It comprises **13 C functions: eight bodies
reused from ticket18 and five new bodies**, including graphics loading and item
animation. Pickup alone is not this scope. The existing 24,832-input missile
tape already includes Morph acquisition, message completion and later Morph
movement; no new route is needed.

The source/data boundary is established below. This is actionable after18, but
not a drop-in admission-table change: item-frame drawing needs three named
native service capabilities absent from18's frozen interface. Those are concrete
implementation dependencies within this candidate. There is no newly identified
missing route, room-loader, trigger or message engine to build. This note does
not audit, modify or block active18.

## Source identity and direct reached evidence

`C` below denotes the retained recovered C pin
`578f90b3cc49557bb70060ad033bb90b8cf8ac50` at
[/home/jim/sm-decomp-restart/restart/out/scroll/native/2c9c9a3cf51a059220afac1ee04727c956e45d067e10e17405bf3adbd268fc7f/source/src/](</home/jim/sm-decomp-restart/restart/out/scroll/native/2c9c9a3cf51a059220afac1ee04727c956e45d067e10e17405bf3adbd268fc7f/source/src/>).
Frozen18 reuse authority is
[item-instruction-interface.md](/home/jim/sm-decomp-restart/.scratch/restart-pipeline/item-instruction-interface.md)
and [semantics-contract.md](/home/jim/sm-decomp-restart/restart/out/item-instructions/planning/semantics-contract.md).
No active18 implementation or evidence was changed.

The following are primary retained outputs, not predictions from route notes:

| Observation | Exact source |
|---|---|
| Step22623: equipped/collected/item fields all clear. Step22624: `09A2=0004`, `09A4=0004`, `D873=04`, queued/index message9 and coroutine2. | Morph author03 [item-observations.csv:4553](/home/jim/sm-decomp-restart/restart/out/morph/author-03/item-observations.csv:4553), next row4554; same replay02 rows. |
| Message still pending at23021; queue/coroutine clear at23022; normal control reaches X1108/Y651 at23036 after14 consumed polls and16px left. | [CSV:4951](/home/jim/sm-decomp-restart/restart/out/morph/author-03/item-observations.csv:4951), rows4952/4966; [result.toml:247](/home/jim/sm-decomp-restart/restart/out/morph/replay-02/result.toml:247), reported398 message waits. |
| Acquired snapshot has actual slot34, byte-index `k=68/0044`: headerEF23, arg001A, block298A, preDFA8, IPE419, trigger00FF, instruction timer0, drawA315, linkE40F, gfx variable0. Global gfx index2 and first saved gfx operandE3F1. | Direct bounded word reads of [morph-acquired.wram](/home/jim/sm-decomp-restart/restart/out/morph/author-03/morph-acquired.wram); declared SHA256 `008656f708c4c30d17b046b74665cdd28d9692d3b686354e977ee11e91e53b1f`. Addresses:1C7B,1E0B,1CCB,1D1B,1D6B,1DBB,DE60,DEB0,DF00,DF50,1C2D,1C2F. This is a post-pickup seed, not an entry snapshot. |
| The accepted missile tape subsequently exercises actual Morph movement at23424, missile acquisition24420 and message return24814 through24832. | Missile author05 [missile-observations.csv:390](/home/jim/sm-decomp-restart/restart/out/first-missile/author-05/missile-observations.csv:390), rows1386/1780/1798; strict replay [result.toml:235](/home/jim/sm-decomp-restart/restart/out/first-missile/replay-01/result.toml:235). |

The full unchanged input is
[author-05/inputs.txt](/home/jim/sm-decomp-restart/restart/out/first-missile/author-05/inputs.txt),
SHA256 `43d67592cb3da2a9dd9c1568f609aa19b60bcb555a28b643453427677908be69`.
Its accepted prefix is all23,036 Morph pads. Existing results explicitly say
`original_rom_comparison = "not-tested"`; acquisition evidence grants no future
Rust instruction ownership or complete C/Rust parity.

## Exact ordinary script domain

After explicit coordinator authorization for bounded data lookup, the pristine
[sm.sfc](/home/jim/sm-decomp/rom/sm.sfc) was hashed once: SHA256
`12b77c4bc9c1832cee8881244659065ee1d84c70c3d29e6eaf92e6798cc2ca72`.
Only bounded `od` reads were used, without emulator or parser tooling. These
are **pristine cartridge data**, not a seal of a future prepared runtime ROM.

`8F:86DE` (file786DE) is `23 EF 45 29 1A 00`: EF23 at block69,41,arg001A.
`84:EF23` (file26F23) is `64 EE EF E3`: setupEE64, scriptE3EF.
Exact complete script46 bytes, `84:E3EF..E41C`:

```text
E3EF: 6487008700000000000000007c8819e4248a0fe4c18689df4fe067e0248707e49988dd8b02f3880400092487a9df
DFA9: 0100b5a2bc86
```

`DFA9..DFAE` is the complete referenced cleanup helper. `P` means the operand
offset passed to C, not the opcode offset; zero-operand instructions need not
read P. Do not derive P from the currently stored PLM instruction pointer.

| Instruction/body | Opcode site → P | Exact result/effect routing | Delta from18 |
|---|---|---|---|
|LoadItemPlmGfx/8764|E3EF → E3F1|gfx source8700 plus eight zero palette bytes; ContinueE3FB|New|
|GotoIfItemBitSet/887C|E3FB → E3FD|set bit → E419; clear → E3FF|Reuse; new site/arg|
|SetLinkReg/8A24|E3FF → E401|installE40F; ContinueE403|Reuse; new site|
|PreInstr/86C1|E403 → E405|installDF89; ContinueE407|Reuse; new site|
|DrawItemFrame0/E04F|E407 → E409|select frame0 draw pointer; local Common; Stop|New|
|DrawItemFrame1/E067|E409 → E40B|select frame1 draw pointer; local Common; Stop|New|
|Goto/8724|E40B → E40D; E419 → E41B|ContinueE407 or DFA9|Reuse; new sites|
|SetItemBit/8899|E40F → E411|setD873bit04; ContinueE411|Reuse; new arg/site|
|ClearMusicQueueAndQueueTrack/8BDD|E411 → E413|byte02; ContinueE414|Reuse; new site|
|PickupEquipmentAndShowMessage/88F3|E414 → E416|word0004, byte09; ContinueE419|New|
|Delete/86BC|DFAD → DFAF|header0; Stop; no dereference atDFAF|Reuse; new site|
|GotoLinkIfTriggered/DF89|preinstruction, no P|linkE40F; normal waiting stored IP E409/E40B; trigger low byteFF|Reuse; new pointer/link domain|
|DrawItemFrame_Common/E07F|local dependency of Frame0/1; P E409/E40B|timer4, stored IP=P, ordered draw services, Stop|New internal body|

The normal cycle is Frame0 → wait4 → Frame1 → wait4 → GotoFrame0. Triggering
DF89 sets preDFA8, IPE40F and instruction timer1. Pickup queues message9 and
returnsE419. The native scheduler storesE419 while awaiting presentation, then
GotoDFA9 performs one tick of empty draw before Delete. **Stop after a draw
means stop processing this PLM for this invocation; it does not mean delete.**

Already-collected initialization still executes **LoadItemPlmGfx before testing
the item bit**, then branchesE419→DFA9→Delete without item/music/pickup writes.
There is no Chozo shell, Call/Return, SetTimer or ClearPreInstr in this family.

Referenced cartridge tables/data to bind alongside the runtime ROM:

- `84:87CD..87E4`: VRAM destinations `3E00,3E80,3F00,3F80`; tile-table byte
  offsets `0470,0480,0490,04A0`; starting tile numbers `03E0,03E8,03F0,03F8`.
- Frame tables `84:E05F..E066 = A30F,A31B,A327,A333` and
  `84:E077..E07E = A315,A321,A32D,A339`.
- `84:A30F..A33E`: eight consecutive six-byte draw records,
  each `[0001, B08E+n, 0000]`, n=0..7. Cleanup's `84:A2B5..A2BA` is
  `[0001,00FF,0000]`.
- Graphics transfer depends on bank89 source8700 for256 bytes. Those artwork
  bytes were not separately read here; full runtime ROM identity already binds
  them. This candidate does not translate NMI DMA or artwork decoding.

All four legal gfx-ring values0,2,4,6 refer to the same sealed Morph script,
not four new item domains. The reached seed has gfx0. Malformed table indexes
or unsafe queue/draw-memory extents must not reach unchecked native accesses.

## Five new bodies and concrete adapter dependency

All word accesses are little-endian; `k` is an even byte-index0..78.

**PickupEquipmentAndShowMessage — `8488F3`, C/sm_84.c:453.** Read word(P)=4;
in order OR word09A2 with4, OR word09A4 with4, call existing18
`PlayRoomMusicTrackAfterAFrames(0168)`, then write word0DC8=9; returnP+3=E419.
Preserve unrelated equipment bits; OR is idempotent but the service/message
still occur on a direct repeated body invocation. The preceding SetItemBit
owns D873, not PickupEquipment. There is no HUD/beam-tiles service.
`DisplayMessageBox` is only the queued-word assignment
([C/sm_85.c:425](/home/jim/sm-decomp-restart/restart/out/scroll/native/2c9c9a3cf51a059220afac1ee04727c956e45d067e10e17405bf3adbd268fc7f/source/src/sm_85.c:425)).
Existing music service retains its state>=40 no-op behavior
([C/sm_82.c:3751](/home/jim/sm-decomp-restart/restart/out/scroll/native/2c9c9a3cf51a059220afac1ee04727c956e45d067e10e17405bf3adbd268fc7f/source/src/sm_82.c:3751)).

**LoadItemPlmGfx — `848764`, C/sm_84.c:347.** Save old gfx indexv from1C2D
to wordDF0C+k; write1C2D=(v+2)&6; record operandE3F1 in1C2F+v. At queue
tailt=word0330, write packed seven-byte entry at00D0+t: size0100, source8700,
bank89, destination table[v/2]; then tail=t+7. Write eight consecutive words
atA000+tile-offset[v/2], starting at the table's tile number and adding each
script palette byte<<10 (all zero here); returnE3FB. Preserve native order and
7-byte packing, not padded Rust struct layout. **No native service is needed
for this body.** NMI later consumes the WRAM queue.

**Frame0/Frame1 — `84E04F/E067`, C/sm_84.c:2813/2818.** Select the corresponding
sealed draw table by wordDF0C+k>>1, writeDE6C+k, then execute the Rust-owned
Common body locally. Do not delegate Common back to C or recursively reenter
the public Rust registration. Its internal ownership can be attributed to the
successful frame-root completion without pretending it is a separate scheduled
entry.

**Common — `84E07F`, C/sm_84.c:2802.** WriteDE1C+k=4, write1D27+k=P, then:

1. `ProcessPlmDrawInstruction(k)`.
2. Reload global `plm_id` (word1C27); `CalculatePlmBlockCoords(plm_id)`.
3. `DrawPlm(the saved plm_id)`; return Stop.

Those three named functions are the new synchronous native-service boundary,
with the same private invocation WRAM and pre/post observation machinery as18.
The owned common body's timer, pointer and call order stay in Rust. A generic
“execute item frame” C service would delegate the owned body and is unsuitable.
Sources: [C/sm_84.c:277](/home/jim/sm-decomp-restart/restart/out/scroll/native/2c9c9a3cf51a059220afac1ee04727c956e45d067e10e17405bf3adbd268fc7f/source/src/sm_84.c:277),
[75](/home/jim/sm-decomp-restart/restart/out/scroll/native/2c9c9a3cf51a059220afac1ee04727c956e45d067e10e17405bf3adbd268fc7f/source/src/sm_84.c:75),
[801](/home/jim/sm-decomp-restart/restart/out/scroll/native/2c9c9a3cf51a059220afac1ee04727c956e45d067e10e17405bf3adbd268fc7f/source/src/sm_84.c:801).

Service effects extend beyond inventory: Process writes level_data at10002+
block offsets; coordinates write1C2B then1C29 using room width07A5; Draw reads
camera0911/0915 and091D, tile tableA000, draw pointer/data and coordinates,
then writes tilemapC6C8, cursor1C25 and packed VRAM queue/tail. Queue/tilemap
capacity and screen clipping may produce a valid no-draw result. Full128KiB
WRAM plus ordered service images and the existing frame effects cover these
observables. Field authority:
[C/variables.h:909](/home/jim/sm-decomp-restart/restart/out/scroll/native/2c9c9a3cf51a059220afac1ee04727c956e45d067e10e17405bf3adbd268fc7f/source/src/variables.h:909),
99/101,397/398,704,1050/1061,1107–1110.

## Lifecycle reuse and limits

Reuse18's typed Continue/Stop/PreinstructionDone, preflight atomic refusal,
service availability/error rules, separate passive observer, room-transition
service52 context and child-exclusion/transaction transport. Extend the exact
header/argument/site/script admission after18 lands; do not broaden18's EF2F
claim or change its frozen report layout retroactively. Morph bit helpers write
scratch05E7=4 and use **D873**, whereas missile usesD874.

Room setup remains `sub_84EE64`: special-air BTS45 and loaded-item counter.
Native contact `sub_84EEAB` finds the matching block and writes trigger255;
frozen contact clears its temporary header. These are existing C dependencies,
not new instruction translations. Source:
[C/sm_84.c:2854](/home/jim/sm-decomp-restart/restart/out/scroll/native/2c9c9a3cf51a059220afac1ee04727c956e45d067e10e17405bf3adbd268fc7f/source/src/sm_84.c:2854),
[2889](/home/jim/sm-decomp-restart/restart/out/scroll/native/2c9c9a3cf51a059220afac1ee04727c956e45d067e10e17405bf3adbd268fc7f/source/src/sm_84.c:2889).
`SpawnRoomPLM` initializes the selected slot and script
([160](/home/jim/sm-decomp-restart/restart/out/scroll/native/2c9c9a3cf51a059220afac1ee04727c956e45d067e10e17405bf3adbd268fc7f/source/src/sm_84.c:160));
the existing native scheduler awaits message presentation and resumes stored IP
([224](/home/jim/sm-decomp-restart/restart/out/scroll/native/2c9c9a3cf51a059220afac1ee04727c956e45d067e10e17405bf3adbd268fc7f/source/src/sm_84.c:224)).
No new room-loader/trigger/message engine is indicated. Message9's existing398
wait observations fit18's512 presentation bound; no new longer limit is needed.

The concrete gap is new draw-service adapter plumbing and its full effects,
including calls during service52. Existing Morph CSV does not record each PLM
entry, so initial service52 ownership and both animation roots still need the
future callback observations. A retained pickup snapshot proves reached fields,
not all preceding callback inputs or source-level ownership. `LoadItemPlmGfx`
also has non-item reload callers (C/sm_84.c:48); safe foreign contexts remain C.

## Selective main reuse

Main193cad9 contains no implementation of the five new named bodies. It does
contain useful dependency algorithms and counterexamples:

- [plm_instructions.rs:288](../../crates/port/src/plm_instructions.rs:288),
  `process_draw_data`; [test:117](../../crates/port/tests/plm_instructions.rs:117),
  `draw_rows_columns_and_negative_origin_offsets`. Useful if later translating
  the draw service; this candidate need not expand ownership to that renderer.
- [plm_render.rs](../../crates/port/src/plm_render.rs), `calculate_block_coords`,
  `draw`; [plm_render tests](../../crates/port/tests/plm_render.rs),
  `coordinates_use_snes_eight_bit_divisor`,
  `full_tilemap_stops_before_following_segment_and_full_queue_does_not_draw`.
  These expose real width/capacity distinctions relevant to service observation.
- [plm.rs:669](../../crates/port/src/plm.rs:669), `preinstruction` DF89 arm,
  and [plm.rs test:348](../../crates/port/tests/plm.rs:348) demonstrate low-byteFF
  trigger preservation. [system.rs:58](../../crates/port/src/system.rs:58),
  `prepare_bit_access`, records scratch-mask behavior. Prefer18's adapted
  equivalents; do not import old crates or old acceptance labels.

## Minimal implementation split and acceptance delta

After root selection and18 integration, one gameplay owner extends only
`restart/item-instructions/`: add the five bodies/local Common and Morph's
declarative sites/data domain, reuse the eight shared bodies and existing
fixture-support interface. One native owner extends item-specific hooks/types/
preparation/control: exact new dispatch patching, three named draw services,
scope-specific counters/observer records and paired control cases. Root keeps
workspace/shared runtime registration, source freeze and integrated execution.
No edits to active18, generic renderer rewrite or separate framework are needed.

Use a finite distinguishing-case delta, not an arbitrary case-count target:

- Actual-C body pairs for Pickup's OR preservation/service-before-message/order,
  gfx ring/packed queue/eight tile writes, both frame tables and Common's
  timer/IP/ordered services/Stop. Cover valid clipping/capacity no-draw behavior
  when exercising the new native services; compare full WRAM and typed returns.
- Adapt shared-body cases to Morph sites/D873, including clear/set item branch,
  trigger low-byteFF versus high-byte-onlyFF and Delete atDFAF. Already-collected
  initialization must retain its **gfx load** while suppressing pickup/music.
- Atomic refusal only for changed admission boundaries: foreign header/argument,
  wrong script/site/table index, unavailable required draw service and unsafe
  memory/queue inputs. Do not execute malformed native indexes as reference
  cases. Reuse18's existing transport/fatal rules without multiplying its suite.
- One unchanged24,832-input no-policy comparison through the same Game observes
  Morph's init/animation/trigger/equipment/message/delete and18's missile family.
  Require material Rust execution for selected Morph roots, zero owned-C/refusal,
  explicit native-service accounting, and retained Morph checkpoint fields plus
  missile observations/final state unchanged. Observe callbacks over the Morph
  prefix as well as the missile tail; the old missile-specific CSV begins23036.
  Use the existing compact service52 already-collected control for the branch
  not demonstrated by this route. No additional full-route prefixes are proposed.

Keep claims separate: source translation is the13-function admitted domain;
actual-C fixtures prove only their declared cases; live evidence proves observed
ownership and compared state/effects along this fixed tape; pristine data lookup
proves data identity, **not original-ROM gameplay parity**. Scope outsideEF23/1A,
general PLM scheduling, native draw services and message presentation remain
explicit dependencies. Source/adapter integration and fresh evidence are future
work; no successful successor implementation is asserted here.

## Completion and cleanup

Only this Markdown file was written. Primary C, accepted output rows and bounded
WRAM/pristine data were read. No build, test, game, capture, emulator, new tool,
claim, shared registry or18 file was created/changed. No run cleanup needed;
0bytes reclaimed; free space103,317,815,296bytes at inspection. Retained evidence
dependencies were neither added nor changed. Final selection remains with root.
