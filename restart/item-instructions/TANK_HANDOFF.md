# Ticket20 exposed-tank material handoff

Owner `/root/material`, native owner `/root/native`, integration owner `/root`.
Worktree `/home/jim/sm-decomp-restart-tank-rust`, branch
`restart/tank-instructions-rust`, baseline
`5012bcaccefe9c815a7ded975f20ee23bcc805c9`. Only
`restart/item-instructions/` is writable by this worker. No original production
claim or fence was changed; no new agent or private producer was started.

## Source and boundary

Selected research:
`/home/jim/sm-decomp-restart/restart/out/tank-instructions/planning/scope-research.md`,
SHA256 `3c921483432126a36a5e4ff454f49a5ae095fed28448b866978c3021a18a1314`.
Selected `.scratch/restart-pipeline/tank-instruction-interface.md` SHA256
`240b53d0445b13be30aeb76ed25b7dfe99bea079624c1c7cf56e6dd7884a52e2`.
Recovered C remains pin `578f90b3cc49557bb70060ad033bb90b8cf8ac50`, with the
retained source path/hashes from HANDOFF.md. Only bounded reads of the four new
collector definitions, two HUD definitions, and the declared script bytes were
needed after the supplied research. No broad investigation or new route occurred.

New source closure modules are `src/tanks.rs` and `src/tank_fixtures.rs`.
`morph.rs` now exposes its shared graphics seals separately from the Morph
script seal; reserve reuses the original graphics/body/Common helpers. No
renderer or timed-record interpreter was duplicated.

The five admitted headers are EED7/EEDB/EEDF/EEE3/EF27. Their item IDs are
0000..01FF, mapping to D870..D8AF with mask `1 << (argument & 7)` and scratch
05E7. Recognized headers with IDs0200..7FFF fail before effects. Negative IDs
8000..FFFF return safe unchanged refusal. Old EF2F/0022 and EF23/001A identities
and all their sites remain exact; their computed map byte/mask is unchanged.

Fixed script starts are E099/E0BE/E0E3/E108. Each exact 37-byte seal consists of
the pinned common layout, fixed addresses/draw pointers, the selected collector
opcode, and its amount100 or5. Reserve's complete 45-byte E41D script and common
DFA9 cleanup are sealed. Reserve also seals the existing graphics/frame tables
and draw records. Fixed timed draw records remain native scheduler dependencies.

Entry17..20 append CollectHealthEnergyTank, CollectHealthReserveTank,
CollectAmmoSuperMissileTank and CollectAmmoPowerBombTank. Service6..7 append
AddSuperMissilesToHud and AddPowerBombsToHud, each trait method taking only
`(&mut self, ram: &mut [u8]) -> Result<(), String>`. The full selected tank
family is17functions: sixteen scheduled roots0,1,3,5,6,7,8,11,12,13,14,15,17,18,19,20
and private Common reused through reserve's frame roots.

Energy wraps max+100 then refills current to that new max. Reserve wraps max+100,
changes mode0 to1, and preserves nonzero mode/current reserve. Super/power wrap
capacity then current+5, call their explicit HUD service, then music. Every
collector queues its message only after all services. None writes the persistent
bit itself. Services remain native and retain their complete WRAM effects.

## Frozen fixture API and inventory

Public `fixtures::tank_body_fixtures(seed, rom)` and
`fixtures::tank_refusal_fixtures(seed, rom)` return existing `Vec<Fixture>`.
They use the unchanged authenticated ticket18 missile-revealed seed and private
ROM path/overrides in README.md. No capture is committed or newly required.
They overlay slot68, the selected header/argument/link/waiting IP, block298A,
width128, camera1024/640, safe queue/cursor0 and explicit inventory/HUD canaries.
The item map starts with5A in every byte; native chains must explicitly clear or
set the selected bit for their normal/collected scenarios.

Direct-write expectations exclude native service effects. Full WRAM and ordered
native service pre/post snapshots are independent actual-C comparison requirements.

| IDs | Distinguishing body behavior |
| --- | --- |
| T01,T02,T03 | Energy refill99→199, wrapping FFEF→0053, native music state40 no-op |
| T04,T05,T06,T07 | Reserve mode0→1, preserve2, wrap max/preserveFFFF mode, state40; current37 unchanged |
| T08,T09,T10 | Super blank-HUD/present-HUD/wrapping ammo+capacity |
| T11,T12,T13 | Power-bomb blank-HUD/present-HUD/wrapping ammo+capacity |
| T14 | Existing missile collector at exposed-missile operandE0DD |
| T15,T16 | Energy argument0: clear branch, SetItemBit at D870/mask01 |
| T17,T18 | Missile argument7: set branch, idempotent SetItemBit at D870/mask80 |
| T19,T20 | Super argument8: clear branch, SetItemBit at D871/mask01 |
| T21,T22 | Power argument511: set branch, idempotent SetItemBit at D8AF/mask80 |
| T23,T24 | Energy SetLink and exposed-missile PreInstr sites |
| T25,T26 | Super loop Goto and power-bomb cleanup Goto |
| T27,T28 | Reserve DeleteDFAF and ClearMusic operandE441 |
| T29..T33 | One trigger case per header: lowFF, high-byte-onlyFF, second waiting site, idle, reserve second site |
| T34 | Reserve source9000 graphics load, ring6 wraps0, maximum safe queue tail601 |
| T35,T36 | Reserve Frame0/ring0 and Frame1/ring6 with full cursor512, local Common |

T08 is super slot68/headerEEDF/argument8/operandE102, HUD C624=2C0F.
T11 is power slot68/headerEEE3/argument511/operandE127, HUD C62A=2C0F.
They are the native new-HUD transport/error controls' inputs. T09/T12 use344B
at the first HUD tile to test the already-present branch. All other HUD words
are explicit canaries, and collector writes precede HUD, music and queued message.

| IDs | Atomic refusal boundary |
| --- | --- |
| TR01 | Foreign headerEED8, Refused |
| TR02,TR03 | Positive out-of-map IDs0200 and7FFF, Error |
| TR04,TR05 | Negative IDs8000 andFFFF, Refused |
| TR06,TR07 | Wrong collector opcode or another tank's operand site |
| TR08,TR09 | Fixed collector amount seal and reserve graphics-source seal changed |
| TR10,TR11,TR12 | Reserve graphics ring8, queue extent602, changed graphics table |
| TR13,TR14 | Reserve wrong pickup link and fixed tank wrong waiting IP |
| TR15,TR16 | Required super/power HUD capabilities unavailable |
| TR17 | Odd native music queue position |
| TR18,TR19,TR20 | Reserve frame services unavailable, table index8, width0 |
| TR21 | Truncated fixed-script backing |

The original B01..B25 and M01..M31 definitions are unchanged. The unknown enum
fixture R07 changes raw17 to21 because17 now names the energy collector. Existing
tests only gain required trait methods and the updated enum bound.

## Bounds, remaining verification and cleanup

Complete128KiB WRAM and even slots0..78 remain required. Script/entry/site/link
and bit-map bounds are checked before effects. Music queue indexes must be even
0..14; all required services are preflighted. Reserve retains the prior safe
graphics ring0/2/4/6, load queue<=601, draw queue<=608, aligned8 cursor<=512,
width1..255, even block offset<=FFFC, and valid saved-slot/draw-pointer guards.
Clipped/full-capacity drawing is valid. Post-service failures remain fatal and
must not retry through C. Negative item IDs are excluded even when their native
bit helpers would avoid map access.

Three new unit tests cover body full-WRAM/typed-output/service order, all changed
refusal boundaries, and each new HUD capability/failure before message. Thirteen
tests are authored in total. Formatting and `git diff --check` passed; this
worker ran no builds, tests, native controls or games. No passing verification
or live stage is asserted by the source submission.

Root's next actions are integrated tests and the native finite actual-C matrix,
real normal/already-collected chains for all five headers, and required new-HUD
controls. Normal chains stop at genuine pending presentation; cleanup bodies
have independent fixtures. Collected reserve still loads graphics first. One
unchanged24832-input candidate replay checks compatibility with19's retained
artifacts; it cannot prove live reach of the four new collectors. Original-ROM
parity and placed-tank live progression remain outside this ticket.

No worker-owned producer or build/run output exists. Local bytes reclaimed0;
available filesystem bytes at inspection102826213376. Root owns source freeze,
all gates/publication, final ordinary/dedup cleanup, evidence authentication and
worktree retirement after required small source retention.
