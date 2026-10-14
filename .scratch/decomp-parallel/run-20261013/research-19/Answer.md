# Native SRAM save/load transaction

Research result: the smallest complete codec is four routines, `SaveToSram`,
`LoadFromSram`, `PackMapToSave` and `UnpackMapFromSave`. Keep these together
in the ticket's proposed persistence module and reuse canonical WRAM plus the
cartridge's 8192-byte SRAM. Save and load are separate lexical call closures;
both need genuine direct-root exercise. There is no current SRAM fixture
adapter or supported bank81 exercise receipt. Implementation admission therefore
depends on the single shared21 provenance extension and a reviewed persistence
adapter. This research earns no gameplay stage.

Exact claim: `ticket-e76fb87d41d3e1e6cab6286936a985d5b5c9d12bc1c0f99806ad86738940ba66`,
revision `3a80add9c71186647255feadd5855a494d6859612ea56cc4c4ee5cd5ae5829cd`,
attempt1, coordinator `decomp-run-20261013`. Actual worker thread:
`01a135f8-3634-7043-be4f-a5c4d96c15e5`. The research Job supplies no
`base-head`, writable source or isolated workspace. Observed source baseline
was `3a8b9ea28b716d90df55f798941bba68e1df6531`; this is an observation,
not an invented assignment input.

## Exact routine and file boundary

| Routine | Original address | Pinned C boundary | Defined dependency |
| --- | --- | --- | --- |
| SaveToSram(uint16) | 818000 | src/sm_81.c:40,37 body lines | PackMapToSave; RtlWriteSram host service |
| LoadFromSram(uint16) -> uint8 | 818085 | src/sm_81.c:78,37 body lines | UnpackMapFromSave |
| UnpackMapFromSave(void) | 8182E4 | src/sm_81.c:116,14 body lines | RomPtr_81 ROM access |
| PackMapToSave(void) | 81834B | src/sm_81.c:131,12 body lines | RomPtr_81 ROM access |

Proposed gameplay ownership, unchanged from ticket19:

- `crates/port/src/persistence.rs`
- `crates/port/tests/persistence.rs`
- `tools/port/persistence.toml`

Definition hashes and22 supporting routine records are inherited from
`tools/port/source-inventory.toml` in [routine-observations.toml](routine-observations.toml).
[scope-holding.toml](scope-holding.toml) is a custom holding document.
It is deliberately unsupported by `assign --scope`: there is no genuine
exercise path/hash with which to construct `translation-scope-v1`.

Pinned recovered source revision:
`578f90b3cc49557bb70060ad033bb90b8cf8ac50`; archive:
`69c78831351112887575f852dadf0281dd5c23cef890b07a268ae6cf60aa9193`;
lexical inventory:
`7842e9035e7360eef10a93a96e6abc1cca601888b8423ea92fb554bbb15abb25`;
source identity:
`e74864c229949fc6f1a6bd7528f339bdb3f9364f1e4f9544f08693881d10f111`.
These are retained declarations. The coordinator must authenticate exact current
captures through the existing evidence crate; this worker ran no hash command.

## Canonical state and operation order

Full WRAM is131072 bytes; SRAM is8192 bytes. `memory::Wram` supplies canonical
WRAM. A dedicated canonical SRAM accessor/persistence ABI is missing and remains
integrator-owned. Avoid a second struct that copies only named save fields.

| State | Byte extent | Meaning |
| --- | --- | --- |
| Live player copy | WRAM09A2..0A01 | 48 words /96bytes, including unnamed tail |
| Save buffer/payload | WRAMD7C0..DE1B | 814 words /1628bytes |
| Player section of payload | WRAMD7C0..D81F | exact96byte live-player copy |
| Events/boss/chozo/items/doors | D820/D828/D830/D870/D8B0 | canonical persisted bitsets |
| Unused/stations/maps/loading | D8F0/D8F8/D908/D914 | unused bytes; used stations/elevators; map flags; loading state |
| Saved station/area/item count | D916/D918/D91A | full16-bit values |
| Compressed map | D91C..DE1B |1280bytes; only first327bytes overwritten by pack |
| Active map | 07F7..08F6 |256bytes |
| Saved maps | CD52 onward | seven256byte area mirrors |
| Live area/station/slot/map flag |079F/078B/0952/0789 | distinct caller state |

The player block includes equipment/collected bits, controller bindings,
reserve mode, current/max resources, HUD, elapsed time, language, moonwalk,
debug/options and bytes beyond named variables ending09EA. Payload ends before
`plm_instruction_timer` atDE1C.

Save order is: backwards-copy48 player words; copy256 active-map bytes to
`CD52 + area*256` using the full area; pack six map areas; store station/area;
copy814 words to selected SRAM payload while summing with16-bit wrap; write
lower checksum, upper checksum, lower inverse and upper inverse; then issue
exactly one `RtlWriteSram`. Slot-selection metadata and `has_area_map` are not
changed by the codec. Ceres area6 receives an active mirror but is outside the
six-area pack loop. Reject unsupported area/pointer/count extents before any
public mutation.

Load first copies all1628 SRAM payload bytes to the save buffer, before
validating checksums. Accept a complete valid lower pair OR a complete valid
upper pair; a checksum from one copy cannot pair with the inverse of the other.
Success backwards-copies the96byte player block, unpacks maps, restores station
and area, and returns uint8 zero. Failure clears only the SRAM payload, zeros
live station/area, and returns1. It leaves checksum metadata and the just-loaded
corrupt WRAM save buffer unchanged; live player/map stay unchanged. Failure
issues no `RtlWriteSram`, so in-memory invalidation is not automatic disk
persistence. All-zero metadata fails; all-zero payload with checksum0 and
inverseFFFF succeeds. A checksum-valid payload may still hold unsupported
station/area/loading state: codec validity does not establish playable continuation.

## Slots, maps and refusal domain

The ROM slot table at81812B has three words:0010,066C,0CC8. Each payload is065C
bytes: slot0=0010..066B, slot1=066C..0CC7, slot2=0CC8..1323. The argument is
masked with3:4/5/6 alias0/1/2. Slot3,7 andFFFF select the nonexistent fourth
word, whose following bytes decode484A, outside8192 SRAM. This is unsafe C,
not a legal fourth slot. Rust must refuse before mutation; do not execute an
undefined-behavior C oracle control.

For slot s, checksums are at2*s and1FF0+2*s; inverse checksums at8+2*s and
1FF8+2*s. Completion signature1FE0..1FEB and selected-slot/complement1FEC/1FEE
are outside the payload.

Consumed map tables:

| Table | Original extent | Values |
| --- | --- | --- |
| Counts |818131..818136,6bytes |4A,48,4C,12,42,15 |
| Packed offsets |818138..818143,12bytes |0000,004A,0092,00DE,00F0,0132 |
| Unpacked-list pointers |8182D6..8182E1,12bytes |8146,8196,81E6,8236,8256,82A6 |
| Six map index lists |8146/8196/81E6/8236/8256/82A6 |74/72/76/18/66/21 bytes |

Ceres next entries/count8/list82C6..82CD exist but are not consumed.
Pack writes327bytes,D91C..DA62, and preserves the remaining953 compressed
bytes,DA63..DE1B; those preserved bytes still participate in payload copying and
checksum. Unpack clears1794bytes,CD52..D453, because its inclusive
1792-to-zero word loop clears two extra bytes beyond the seven-area extent.
It then restores only the six sparse lists. Preserve cleared holes and Ceres
bytes; a full-map lossless roundtrip is not the C behavior. A zero count causes
the original do-while to underflow; malformed table controls must refuse before
effects instead of running nonterminating/unsafe C.

Static comparison `cmp -s -i 32768:32768 -n 927 rom/sm.sfc
out/port/room-plm-current-live/20261010T224032.887967Z/c-0/rom-view.bin`
returned0 for818000..81839E. This is byte equality of code/tables only.
[rom-observations.toml](rom-observations.toml) records the exact ranges and
inherited image seals. Original code uses direct-page scratch0012/0014 and
pack/unpack0016/0018/001A, which the recovered C expresses as locals. Original
load status is carry(clear success/set failure), while C returns0/1. An original
comparison requires a reviewed register/scratch mapping; raw full-WRAM equality
cannot be silently claimed.

## Caller and continuation ownership

| Caller / companion | Exact observed behavior | Existing owner / next boundary |
| --- | --- | --- |
| GameState_33_BlackoutFromCeres828388 | DrawTimer/await gameplay/fade; NMI/VBlank/IRQ teardown; loadingD914=22 and state34; SaveToSram before cinematicC11B, flags and sound queues are cleared | milestone06 plus13host/22/23; codec supplies dependency only |
| CinematicFunction_Intro_Func738BC100 | loading/game31,area6,station0,then save | Ceres/cinematic owner06/20 |
| PlmInstr_ActivateSaveStationAndGotoIfNo848CF1 | poll message23; negative retry; result2 jumps/no save; yes spawns electricity before deriving station from global plm_id room argument&7, marking used station and saving |27whole PLM/29message/allocator/player; no canned prompt result |
| GunshipTop_7A2A987 / GunshipTop_13A2AB1F | landing sets loading5/station0/usedstation0 before save; prompt28 saves unless result2 | separate whole gunship enemy/player/message closure; no ship ownership earned |
| GameOptionsMenu_4_StartGame82EEB4 | debug branch sets loading5 then save; loading22 routes to cinematic | research18 options root |
| GameOverMenu_3819003 / _481912B | save then SoftReset; or reload selected slot | research18 file/menu successor plus held reset76 |
| FileSelectMenu_4_Main81A1C2 | persists selected slot/complement before Load/NewSave/LoadMirror | research18 file/file-map successor |
| FileCopy819A2C / FileClear819C9E | copy payload plus all4 checksum fields then persist; clear payload/metadata then NewSave/failed Load/slot-area mirror/draw/persist | same research18 successor; no duplicate UI ticket |
| VerifySRAM808261 | short-circuit Load0&&Load1&&Load2; all-invalid writes madadameyohn and persists; otherwise supermetroid signature enables4 demo sets |13boot owner with codec dependency |
| CopySuperMetroidString80824F | writes supermetroid signature and persists |20ending/credits owner |
| NewSaveFile81B2CB |99health/resource/controller/option defaults and selected bitset/map clears; no save | minimal companion research |
| Load/Save explored-map mirrors80858C/8085C6 |256byte full-area copy; Save performs overlapping16-bit OR00FF atD908+area if map flag set | same canonical companion research; public room hooks integrator |
| LoadFromLoadStation80C437 |14byte ROM record into room/door/screen/player/previouspos/scroll/low-byte area/minimap | companion research then accepted room initialization |

`LoadFromSram_`81A053 is a trivial forwarding wrapper owned with the file-menu
successor. Codec exercise cannot establish any of these callers.

NewSaveFile sets debug09E6/unused09E8 to1, includes duplicate item clears as in
original assembly, clears1792 map bytes and selected bitsets, and does not clear
events/boss arrays, loading state or the full save buffer.

After saved loading22, options routes to the Ceres cinematic; Intro_Func95
8BCADF eventually selects state6 and refills health. `InitAndLoadGameData_Async`
828000 maps loading1F to area6/station0 plus timer clear, and loading22 to
area0/station18 plus LoadMirror. It then performs IO/graphics/Samus/loadstation/
gameplay/HUD/enemy/NMI initialization. Codec success therefore is not arbitrary
snapshot restoration. The required continuation controls must state and preserve
canonical refill and initialization semantics.

Loadstation pointer table80C4B5 contains C4C5,C5CF,C6D9,C81B,C917,CA2F,CB2B.
Selected14byte records set room/door/screen; playerx=screenx+128+xoffset,
playery=screeny+yoffset; previous positions match, scroll is zero, room8F header
supplies low-byte area, and low-byte debug minimap is cleared. `door_bts` is
ignored by C. Ceres6/0 record80CB2B=`45df58ab00000000000048000000`;
Crateria0/18 record80C5C1=`f891fe8800000004000080000000`.
Room/decompression/assets and the complete start coroutine remain dependencies.

## Required persistence adapter and evidence

The current generic family fixture imports full WRAM plus at most eight
projectile host words; it has no SRAM snapshot contract and does not initialize
`g_sram`. Adding the gameplay manifest alone cannot establish actual-C parity.
Proposed adapter files:

- `crates/pipeline/src/persistence_adapter.rs`
- `crates/pipeline/src/persistence_model.rs`
- `crates/pipeline/src/persistence_host.c.in`
- `crates/pipeline/src/persistence_abi.rs.in`
- `crates/pipeline/tests/persistence_adapter.rs`
- `crates/pipeline/tests/persistence_live.rs`

The coordinator owns canonical memory/lib/game_host wiring, shared
family/family_model/live/live_model/main registration and batch admission, after
source-epoch/ownership review. Preserve legacy87/94/96/97,native137 and held141.
Workers are not alone in the codebase and must preserve other owners' edits.

The reviewed ABI must import/export full128KiB WRAM and8192 SRAM, raw return,
ordered memory/metadata changes and synchronous filesystem requests/results.
Preflight the whole transaction; refusal leaves memory/services unchanged.
After accepted memory changes, exactly one save service occurs. A service
failure is owned and fatal, with no retry into C after first effect. This reuses
the existing execution-once contract.

Pinned `RtlWriteSram`726 renames .srm to .bak, opens, writes8192 and closes;
only fopen failure logs, fwrite/fclose results are unchecked and return is void.
Separate gameplay memory success from disk durability/error reporting. Stronger
Rust durability behavior needs explicit review. `RtlReadSram`714 can publish a
short read and merely logs; missing files leave initialized memory. A fresh
Rust host must declare truncated/oversized input behavior rather than attributing
stronger guarantees to recovered C.

`SnesInit` aliases g_sram to cartridge RAM; cartridge load zeroes RAM while
cartridge reset preserves it. RtlReset0 clears SRAM; RtlReset1 preserves it.
Current `tools/decomp/headless.c` starts with RtlReset0 and never calls
RtlReadSram. Fresh subject directories and terminal SRAM comparison do not prove
file reopen. Movie recorder SRAM/full-snapshot injection is not fresh-process
ordinary-reset continuation evidence. Ticket76 SoftReset returns through
coroutine3/CPU808462 while preserving SRAM/hardware/movie; it is a distinct path.

Use independent RM_MINE C0/C1/mixed0/mixed1 subject chains: genuine save to an
isolated real file, process exit, new process importing only that file, genuine
ordinary reset/menu/load/continue, then compare canonical resources, room,
items, maps and state with declared initialization/refill. No candidate WRAM or
whole snapshot injection. RM_COMPARE restores snapshots and cannot substitute
for these independent chains. Original-ROM replay remains a separate gate,
including reviewed scratch/register status, real controller/NMI/APU waits and
visible fallback. Research18's original audio acknowledgment/loading gaps remain
dependencies; no new runtime comparison was performed here.

## Reviewable child proposals and gates

[children.toml](children.toml) proposes five bounded follow-ups: shared21
supplement; persistence adapter research; the complete4routine codec task
holding; minimal canonical initialization/map/loadstation companion research;
and composed Ceres-save/fresh-process continuation evidence. Reuse research18's
file/file-map successor and20credits rather than duplicate their ownership.
Coordinator publication must make08 depend on codec, companions, genuine caller
and fresh-process gates;06 needs the real GameState33 save boundary;
09 retains full ownership requirements. Read-only research satisfies none of
those gameplay gates.

Every proposed new test/module/manifest is unimplemented; all following commands
are unrun. Adapter/provenance review must register support and nonzero tests
before admitting a gameplay scope. Initial gates are capped at60seconds:

```sh
cargo test -p pipeline --locked --offline --test native_exercise
cargo test -p pipeline --locked --offline --test persistence_adapter
cargo test -p port --locked --offline --test persistence
```

Merged gates are capped at3600seconds with shared workers4:

```sh
cargo test -p pipeline --locked --offline --test persistence_adapter --test persistence_live --test family --test live
cargo run -p pipeline --locked --offline --bin sm-pipeline -- family-verify --manifest tools/port/persistence.toml --output out/port/persistence --case-jobs 4
```

The latter is conditional on real persistence adapter registration. Exact
supported subject-chain and original-ROM argv, reset/file/input/backend controls
and material captures must be an adapter deliverable; there is no invented
currently supported live/original command. The integrator separately freezes
current source, authenticates all inputs and runs the all-affected family gate.

## Evidence and completion

[inputs.paths](inputs.paths) identifies exact retained input paths.
[inherited-input-seals.toml](inherited-input-seals.toml) records available
inherited hashes and their origins; [input-seal-provenance.toml](input-seal-provenance.toml)
states current-seal gaps. Research18 retained evidence run
`2156e965ec6cd98bd54b381c5d490784d34af141ee1a8ddb4cebc729583f3062`
supplies the menu/audio/caller context; its Answer hash is
`45bd7e4152b6167a585ecd3805b1597c8ab6704a678aa8980f374668d463b366`.
Research17 run
`6ebd3637b5984438ab276489ee703d5132b522e01f701a3549694f53c45d49a1`
retains enemy/host ownership facts. Existing evidence readers/store/private
inputs must remain available. No manually computed new hashes are supplied.

No build, runtime, extraction, verification run or worktree was created.
Static bounded source reads, ROM `od` slices and one `cmp` produced only the
observations above. [cleanup.toml](cleanup.toml) records final scoped liveness,
disk space and0bytes reclaimed. The coordinator freezes/seals this package with
`sm-evidence ingest/inventory/verify`, publishes exact research result/tickets,
and records actual stages through `sm-pipeline progress`.
[manual-handoff.toml](manual-handoff.toml) preserves actual thread/claim/baseline,
limits and the concrete next action before handoff.
