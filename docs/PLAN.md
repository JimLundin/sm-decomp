# Plan

Goal: a compact Super Metroid engine playing boot to credits with owned, safe Rust
and clear modules. Remove unsafe/raw-memory operations from gameplay, retained
native interfaces and verification; hiding them in wrappers is incomplete.
Target below 100,000 production lines; 90,000 is a stretch, not promised deletions.
REVIEW owns the detailed size and waypoint targets.

The [accepted destination](target-architecture/README.md) is one deterministic owned
Session, our renderer/audio and small platform libraries, independent of SNES emulation.
Map Rando needs custom sprites/palettes, configurable rooms/doors and opt-in re-spin/QoL.
Keep content, topology and options separate from platform and ROM/RAM compatibility;
vanilla is the first fidelity profile. Windows migration may retain the backend/oracle.

## Authority, lanes and Git
Read [REVIEW.md](REVIEW.md) next and its Milestone plan at each item's start; it
supersedes this work order. The reviewer may change goals/process (user, 2026-10-25).
Follow external reviewers' instructions, including cloud REVIEW directives: record
one line of disagreement under the item, then do it. Direct user instructions win.
Git history holds no current rules; pre-2026-10-17 workflows are archived.

Lanes: A finishes seed/recipe speed work, then takes the next unclaimed bank;
M removes unsafe outside banks; B1-B6 retire banks in REVIEW's order. No more
in-place M bank codemods. Current bank queues are B1 sm_82, B2 sm_8b, B3 sm_a3,
B4 sm_a8/sm_b3, B5 sm_a9/sm_aa and B6 sm_a6/sm_a7; sm_b3 is already retired.
Freed workers take sm_a5, sm_9b, then sm_a4. Keep cloud claims intact.
Non-enemy replacements aim <=80%; every B body gives the full ratio and a reason
above 90%. Necessary safety growth must be counted and explained, not relabeled.

Never reserve or hold Git index/ref locks; Git's own short locks cover actual writes
only. Before every Git write, fetch/reconcile current origin/main, recheck HEAD/index
and preserve others' work. Never delete another process's lock or force-push main.
Windows has one implementation/Git integrator under the new oversight session.
Linux development is stopped by the user (2026-10-27); preserve its frozen handoffs.
Windows source owners may push coherent temporary migration branches with pending
checks clearly named (user/oversight grant, 2026-10-27). Main requires qualification.
Run named local rows on a tip reconciled with current origin/main, commit to main
and immediately `git push origin HEAD:main`. A rejected push means fetch/rebase,
rerun only affected rows, then push. Keep one Git writer and no force pushes.
Land verified work promptly; holding it over an hour is a stall. No docs-only commits
without a code commit beside them. Update PLAN only for active direction or blockers.

Subjects start A:, M: or B:sm_XX. Body <=8 lines: change, exact Rows/results, full
production/legacy/verification deltas, B ratio/reason and final Score. Use four actual
UTF-8 middle dots, not question marks; measure the values locally with progress.sh:
`Score: checks 44/44 · legacy N · production N · unsafe N · verification N`.

Keep replay docs essential, upstream references and CSV/TOML provenance/checks;
shared Python installers/runtime-only restores are support work, counted separately.

## Current work and limits

SOURCE37 native Samus content and lifecycle cohort is qualified.
Source 5d73ade6 / DLL eaab5768 passes fresh library in 27.175 seconds,
with 200 unchanged warning identities and 0 added. Final direct DLL: 88.295 seconds;
all 81 units and source/native guards pass; minimum RAM: 2.772 GiB.
All 26 assigned rows pass (23/44 required), including full Morph 1200 and
BOTH independent Kago 14350 setups. Runtime: 199063 comparisons / 0 failures,
in 2058.957 seconds. Genuine initialized 329 + resume 5, save 55650 and reserve
91623 bind codec bd6b62db27d15b56. Its 78 contributors bind 1301655 raw /
1292518 normalized bytes. The failed
SOURCE36 fixture/type check remains preserved; SOURCE37 changes one tuple type.
Returning-Die short replay-load failure is source-reviewed, not runtime-executed.
Net versus main 1da27463: production +1417 / legacy 0 / unsafe-containing +15 /
verification +1272; support reporter 80 separate. No retirement or savings claim.
SOURCE38 driver is on a separate migration branch; its SOURCE39 witnesses,
compiler and new-codec execution remain separately attributed and pending.
Actual persistent Session/Display/start/adoption, binary checkpoint,
Reference/audio, portability, boot-to-credits and size acceptance remain open.


SOURCE36 connects the explicit Game Samus content binding to native body,
position, tile and upload consumers. Scoped Runtime/CPU/reset/load adoption
retains selected registrations through actual late normalized settlement.
Empty patches stay no-ops; returning short replay loads preserve their original
continuation and leave authority revoked. Seven actual Game witnesses join the
four role witnesses: 81 units are registered, not executed. Source branch only;
normal library/warnings/DLL/units and genuine new-codec captures/replay are pending.
Actual prepared start, persistent Session/Display/SamusHalf, binary checkpoint,
Reference/audio, portable driver and size acceptance remain unfinished.


SOURCE34 shares the exact allocation-free full asset-byte validator. State
retains payload-only admission; tile requests retain their original explicit
identity/revision/offset prefix and first0/second0 transfer rules. Both methods
are pub(crate) for the real content cohort. Net -20 production; 70 registered
units remain raw-identical to SOURCE33. Source review/format/inventory pass;
DLL/runtime remain pending on the migration branch. Codec 6bf62feb168ff63b has
76 contributors; no intermediate capture/compile or old-codec transfer. Role
module R1 and actual Game/G/native consumer/lifecycle remain active dependencies.


SOURCE33 adds four independently authored catalog admission/lifecycle checks:
missing content behind original failures, malformed/valid changed catalogs,
rejected reset owner preservation and supplied Arc transfer through reset/reopen.
All prior cases and zero-prefix checks remain; 70 units are registered. Exact
source review/format/inventory pass; DLL/runtime remain pending on the migration
branch. No intermediate compiler/capture; complete actual role/consumer cohort
precedes the next qualification freeze. Net +409 verification versus qualified
main, production +21 from the retained catalog donor; no ownership/size closure.


SOURCE32 seals supplied Samus catalog content against an independently expected
catalog and carries it through open/reset/checkpoint-to-start metadata. Existing
constructor/reopen fixtures use explicit nonempty media-valid catalog declarations;
all original assertions and zero-prefix cases remain. Format/source inventory
pass; final DLL/runtime qualification is pending on this migration branch. The
new codec requires genuine captures after the complete content/consumer cohort
is frozen. No compile/capture is run for this intermediate checkpoint. Actual
persistent G, native role content, full checkpoint and backend independence remain
open; catalog admission alone does not establish their ownership.


Zero-producer prefixes now retain the original Scene error in both existing
Samus/marker units. Exact successor DLL and all 66 units pass; all 76 raw codec
contributors remain identical to qualified SOURCE30. Original 26-row runtime
attribution is preserved; no replay/capture repeated. Net +6 verification, all
other code classifications unchanged. Catalog constructor adaptation and the
actual persistent Session/content/consumer/lifecycle remain open.


SOURCE30 atmospheric/Samus producer cohort is independently reviewed and qualified.
Source d9daaf02 / DLL6768ee76 binds shared atmosphere, body/position/tile upload,
resident media publication and all66 units. Production library26.194s retains
SOURCE23 attribution; complete unchanged-input transfer reaches final SOURCE30.
Final direct DLL85.259s leaves sampled minimum2.973GiB RAM; no added warnings.
All26 assigned rows pass (23/44 required), including fullMorph1200 and BOTH
independentKago14350 setups. Actual runtime199063 comparisons/0,
fresh initialized329+resume5 and save55650/reserve91623 bind the final codec.
All76 raw contributors are identical across fixture successors; codec
f6e7c80dc575e620 binds1257015raw/1246178normalized bytes. Original64/66
failure receipts and superseding complete Tick0/1/2/producer25 corrections remain.
Zero-Samus namespace-prefix coverage remains unclaimed; no restamped checkpoint.
Net versus mainSOURCE08: +2752production/+2986verification/+13unsafe-containing/
legacy0. All retained adapters/dependencies count; no net-size saving or backend
retirement claim. Support+228 includes collector220 and role guides8; docs separate.
Actual G/start/adoption,Reference/audio,portable/full slice remain.
Next: reconcile the externally claimed Samus catalog with the frozen constructor
inventory, then the complete persistent Session/player-Scene consumer and lifecycle.
These completed final receipts supersede earlier pending statements below without
changing any historical execution or frozen artifact attribution.


SOURCE30 corrects both failed SOURCE28 fixtures without production changes.
Settlement independently checks literal Tick0/1/2 and carries actual settled
metadata through reset/continuation/reopen; namespace producer25 reaches Identity.
All prior assertions,64 passing bodies,66 units and76 raw codec leaves remain.
Independent source review PASS; original64/66 failure receipt is preserved.
Zero-Samus namespace-prefix coverage remains unclaimed; producer0 fails Scene first.
Final SOURCE30 DLL and66/66 units PASS;26-row fresh-codec runtime remains pending. Net verification+4; production/legacy/unsafe0.
Actual G/start/adoption,Reference/audio,full slice and engine-size goals remain open.


SOURCE28 corrects eleven new host-only private media lookups through existing
public iterators, preserving full MediaRef equality and every assertion. Source
4e7cf0d3 retains all66 units and all76 raw codec leaves f6e7c80dc575e620.
The original SOURCE27 sm library succeeded; its host failed11 E0624 after77.300s.
Keep that failure and its byte-bound library. Actual corrected host metadata
PASS0.914s against the retained unchanged library interface; source review PASS.
This is not final DLL/runtime proof. Final qualification19 DLL compiled; 64 of66 units pass, with two earlier fixture
failures under diagnosis. Original results remain preserved.
A four-line reviewer clause records the actual compiling crate/cfg and visibility.
Net verification+44 includes24 helper and20 formatter lines; production/legacy/
unsafe unchanged, support+4. No retirement/savings claim; G/full slice remain open.


SOURCE27 adds six independent owned upload/resident units (66 total), preserving
all prior assertions and the SOURCE25 move-order correction. Its two new source
faults use literal573 against the actual576-byte atlas; expected prefixes are
unchanged. Draft SOURCE26 and both original author checkpoints are preserved.
Final source90e3eb79 retains all76 codec leaves at f6e7c80dc575e620; fmt/source/codec
checks pass. Independent witness review is active; DLL/runtime remain unrun.
The prior SOURCE25 DLL was terminated below2GiB RAM reserve after43.698s;
its admission is released. Prepared retries require3.25GiB growth plus2GiB reserve
and current external reservations. Actual G/start/adoption/full slice remain open.


SOURCE25 fixes the cfg-verifier E0382 found by the actual SOURCE24 direct rlib:
capture a selected SceneObject's order before moving it into the indexed array.
The original failed30.164s execution/source/diagnostics are preserved; compiler
admission is released. New sourcec0a113f8 keeps all60 unit assertions and all76
codec contributor bytes unchanged at f6e7c80dc575e620. The one settlement helper
prefix differs explicitly; no cloned record, changed assertion or codec restamp.
Exact normal-library dependency transfer retains SOURCE23's proof. A new frozen
DLL/60-unit build is running; runtime/G/raster/start/adoption remain pending.


SOURCE24 adds independently authored position/table/Ceres failure literals to
the actual shared core. All60 units are registered; old55 assertions/native
prefix remain raw-identical. Every76 codec contributor byte is unchanged from
SOURCE23, codec f6e7c80dc575e620. Exact normal-library dependency transfer retains
SOURCE23's26.194s/200warnings/0 added attribution. Final DLL/units is running
at frozen e9f5804b; native/replay/genuine new-codec captures and actual G/raster/
start/catalog provenance/selected adoption remain pending.


SOURCE23 composes independently reviewed Samus NMI upload and resident media
publication into existing native/owned consumers. Shared upload preserves
VMAIN/flags/top/bottom and lazy descriptor/source/count/destination order;
resident media preserves aliases/colors before validation, success-only IDs,
post-raster callback sampling and complete ordered immutable row publication.
Source4883c95f passes fmt/source guards and normal library in26.194s,200
unchanged warnings/0 added. Codec76/f6e7c80dc575e620 requires genuine captures.
Original55 unit bodies remain; position/Ceres independent literals are active.
Final DLL/native/runtime/G/raster/start/media identity/adoption remain pending.


SOURCE22 composes the 55-unit common namespace/marker witness and its narrow
shared-position/tile Inputs successor. All existing unit assertions and native
prefixes remain; injected failures retain actual owner poisoning/OAM/cursor
prefixes. Source0ce1aa31 shares all74 raw codec contributors with SOURCE20,
codec a074b292792a6474. New test additions do not invalidate that codec.
Normal-library type/warning proof retains SOURCE20's original27.174s/200/0
attribution; exact cfg-only and include_bytes dependency transfer passes.
Final DLL/native/runtime and genuine main-to-new-codec captures remain pending.
Actual G/start/catalog provenance/lifecycle/adoption and full owned slice stay open.


SOURCE20 composes reviewed Samus position/tile cores into the native and owned
body consumers with explicit selected source/request identities and lazy effects.
Source077424d4 passes fmt/source guards and normal library in27.174s with200
unchanged warnings/0 added. Marker verifier adaptation and final DLL/native/
runtime/fresh captures are pending. New leaf is in source inventory and codec.
Native fallback/aliases/cinematic failure and pointer/flag ordering are retained;
actual G/start/selected-content provenance/adoption and full owned slice remain.


SOURCE19 source-branch checkpoint composes independently reviewed SAMUS-BODY01,
the corrected 51-unit settlement witness, and a source-guarded verifier/test seam.
Private coverage_checks moves with identical tokens into verification; its full
DLL source guard remains. All 73 state codec contributors stay registered;
only verify.rs bytes change in the seam. Test-only edits no longer change codec,
while payload edits still do; qualified codec calculator cross-check passes.
Source b4fc22fa / codec ab464e76d5931817 is formatted/source-guarded only.
Seam independent review, final library/DLL/units/native/replay and genuine captures
remain pending. No old checkpoint restamp, size saving or complete-G claim.


SOURCE16 repairs the actual player-detached dependency graph on the same migration
branch. One pure atmospheric slot/protocol leaf and one pure DisplayError leaf
serve native/owned display and the existing detached entry through exact aliases.
Definitions and ordered algorithm bodies are unchanged; no duplication or savings.
Source774172ec passes fmt and focused normal library+player_detached compilation
in40.183s. Library200 warnings/0 added; example1319 warnings recorded separately
without a transferred baseline. This is type/dependency evidence, not executed
detached cases, final DLL/runtime or full backend-independent gameplay proof.


SOURCE15 reviewed atmospheric/Samus producers and shared lifecycle namespace are
published on migration/scene-producers-source15, based on qualified main9d02b6ef.
Exact source4a669759 passes fmt and normal no-default library with200 unchanged
warnings; original201-warning predecessor is preserved. Direct DLL/native/replay,
settlement/persistence witness and genuine new-codec captures remain pending.
Schema4 seals supplied atmospheric catalog/variants/context/history/common IDs;
Samus is added to the literal source inventory and checkpoint codec contributors.
Native/default adapters and actual owned producer entry share the ordered core;
actual G/lifecycle/effective-context/import/selected-state adoption remains open.
This source branch is not main qualification or a full owned playable-slice proof.
Preserve original SOURCE08 proofs without repeating or restamping their source.


Current SOURCE08 component source dfdd6120: production 123,622; legacy 17,793;
unsafe-containing 2,621; verification 46,940. Final library/DLL/source/native
guards pass with 200 unchanged warnings. Original source 959bd651 retains 48
units and 199,063 comparisons/0; final 105/106 and identical 69-codec transfer pass.

Scene/display #20 component and native sharing cohort is independently reviewed and qualified.
Final source edbe402d / DLL d02d370b binds typed display/schema, native OAM, HDMA,
IRQ and atmospheric tile consumers; full owned gameplay/Scene authority remains open.
Original SOURCE07 ce55ff2f / DLL d02a8acc retains 46 units and all 26 distinct rows:
21,784 selected + 29,672 setup + 147,273 fresh captures + 334 initialized = 199,063 comparisons/0.
Full Morph 1,200 and BOTH independent Kago 14,350 setups pass. Genuine initialized
329+5 and fresh save 55,650 / resumed reserve 91,623 retain original source and inputs.
Final library 35.137s, direct DLL 97.327s, format/source/native guards pass; warnings 200/0 added.
Native 100-102 retain SOURCE07; final 103/104 and atmospheric 4051-4082 pass.
Original 4049/4050 complete the 34 distinct atmospheric cases; no unsupported CPU-oracle pair.
Original failed 4051 is preserved: its fixture set byte $05B5 instead of sampled word $05B6.
The one-line counter fix keeps all original sound/stream/alias/failure assertions; physical delta 0.
Codec 67/40ed4a0f14b5bd09 binds 1,280,285 raw / 1,269,448 normalized bytes, identical throughout.
Net +4,939 production / +2,591 verification / +31 unsafe-containing / legacy 0 / support +7.
State/schema/adapters and independent checks grow code; raw/backend retirement 0, no savings claim.
Next: separately frozen production SOURCE08 delivery-state/owned-HDMA/settlement integration;
actual G ownership, selected-state adoption, semantic media/history, initial Scene/Reference/audio
and all eight full-slice proof groups remain required. Preserve all frozen predecessors.

SOURCE08 delivery/settlement component and its native verifier successor are qualified.
Final source dfdd6120 / DLL 7fa1a335 shares owned HDMA channels, delivery state and
frame settlement. Actual G ownership and full initial/selected-state adoption remain open.
Original SOURCE08 source 959bd651 / DLL 8049aa51 retains 48 units and all 26 rows:
21,784 selected + 29,672 setup + 147,273 fresh captures + 334 initialized = 199,063 comparisons / 0.
Full Morph 1,200 and BOTH independent Kago 14,350 setups pass; genuine initialized 329+5,
fresh save 55,650 and resumed reserve 91,623 retain their actual source/input provenance.
Final library 38.541s / direct DLL 91.356s, format/source/native guards pass; warnings 200/0 added.
Native 100-104 retain original SOURCE08; final 105/106 pass independent literal register,
cached/per-layer routing, counter/latch and live/written OAM checks. No old unit/runtime repeat.
All 69 raw codec contributors are identical; codec c7f30cadd1121f08 binds 1,314,039 raw /
1,303,202 normalized bytes. New production retains genuine new-codec checkpoint lineage.
Net +747 production / +319 verification / +2 unsafe-containing / legacy 0 / support +2.
Safety/state/schema and checks grow code; the unsafe metric adds two forbid attributes.
Raw/backend retirement is 0; no savings claim.
Next: separately frozen SOURCE09 atmospheric semantic producer and guarded verifier adaptation;
actual G/room/raster/B-bus/adoption, complete media/history/Reference/audio and eight slice groups remain.
Preserve original SOURCE07/SOURCE08 receipts and all frozen SOURCE09 predecessors.

Prepared room cohort9787 qualifies Levels/Tiles/Palette/Background/Fx catalogs and
application room-pack selection before native startup. Selected Windows desktop ROM
paths and relative-search cwd are ASCII; verifier and pack paths retain Rust Unicode.
Actual05 fmt, external-host/library/frontend/direct DLL and loaded source/native
guards pass,160 unchanged warnings. Original04 retains14 rows20875+272/0, initialized
329+resume5,23 units, actor guard,3 level cases,7 selected pack cases and fresh prepared
checkpoint roundtrip/reset/continuation plus separate changed-world rejection.
Background38/38 combines30 original04,5 capacity-corrected04 and3 corrected05 cases;
full assertions remain. Fixture34..36 now expect the existing clears at0782/0612.
All34 original jobs are collected. Codec45/a239 and its45 contributor bytes are unchanged
by the two-line fixture correction; accepted04 proofs retain their original lineage.
Net+1909 production/+1039 verification/+34 unsafe metric/+67 support/legacy0.
Catalog/schema/pack validation and compatibility adapters grow code; no raw-operation
or backend retirement claim.
Shared renderer C01/V03 final06 c942 passes independent source review, scoped final
format, direct DLL/source/native guards, 24 units and isolated cases40/42. Mode1/Mode7
pixels, sprite limits/aliases/failure prefixes and actual EXTBG/enabled-mosaic
dispatch use independent literal expectations; original39 and corrected40/43 remain.
Final DLL b01c retains 48 codec contributors, 951201 raw/940208 normalized bytes and
codec2cdd23f3f10e7e57. All contributors are byte-identical to qualified03; its 17 rows,
26638 ordinary comparisons/0, full Morph1200, initialized329+resume5 and prepared
checkpoint/reset/continuation/rejection retain original df9b/5921 attribution.
Final06 actual no-default check passes at incremental1: 200 warnings versus202
same-profile9787, zero new. Original05 incremental0 158/160 and the failed cross-profile
comparison remain preserved. No source, warning suppression or DLL/runtime repeat.
Net +657 production/+506 verification/+7 unsafe-containing/legacy0; new 944-line
core/prepared/records and 288-line PPU reduction are counted together. Four formatter
joins are not savings; raw/backend retirement0 and full-engine claims remain open.
Message composition #22 source5787/DLL8d58 is independently source-reviewed and qualified.
One safe core serves native drawing and the actual prepared dialogue/display consumer;
live queue consumption, stale completion, authored media IDs and0782 waits are preserved.
Actual26 units and26 distinct rows pass: 21784 selected +29672 setup/0.
Setup includes charge700, Ceres249+lag23 and BOTH independent Kago14350 approaches.
Full Morph1200 ran once; genuine initialized329+resume5 and authored55650+91623 captures pass.
Final codec54/d3e20adab0249af3 binds1024520 raw/1013226 normalized bytes; no old restamp.
Independent full-display/glyph/restoration pixels,360 waits, input/effects, checkpoint,
changed/missing media/content and reset checks pass. Frozen predecessors remain preserved.
Net +866 production/+357 verification/+2 unsafe-containing attributes/legacy0/support0.
Catalog/validation/resumable media and independent fixtures exceed forecasts and count in full.
Raw-operation/backend retirement0; native audio/device/ROM and full Session/Scene remain open.
Next: Windows source owner01a17eee advances the released #20 Scene/display cohort. Its complete
Scenario/Checkpoint schema, initial Scene/audio and Reference profile remain required.
Qualified WRAM/Execution: Game/Backend share stable WorkRam; private DMA/Input
storage and actual byte/getter/macro consumers close with exact RAM layout.
Initialized329+5, full86 (136526 selected+38827 setup),14 genuine captures189475,
full22 byte-identical restore and both Kago14350 pass at recorded source/native identities.
Prepared110/native-startup proofs retain original dce54/f158 attribution; no startup/device,
full Session/backend independence or universal native/failure claim. Prior receipts and
mutation witnesses remain in [qualified history](https://github.com/JimLundin/sm-decomp/blob/43117e058f0de398858dfd8be5db4f3d75e9a0d8/docs/PLAN.md#current-work-and-limits).
Net WRAM cut versus84bdd: +2320 production/+1106 legacy/-75 unsafe/+1443 verification;
+188216 canonical raw bytes. Checked consumer/value safety grows engine code, not deduplication.
B2 continuations4ece:3 callbacks/12 resumes pass19 rows62432 selected+272 setup/0,
fmt/library/DLL160 unchanged warnings and complete source/native/input/seed guards.
Net -5 production/-25 legacy/+1 unsafe metric from two safety attributes; bank remains.
Source-list preflight569 entries adds42 support lines; original loaded guards retained.
AA04 shares Shaktool segments and Torizo palettes:9 rows6180 selected+1713 setup/0,
fmt/library/DLL160 unchanged warnings; Golden five ports/latches and source/native/input guards retained.
Net -73 production/-162 legacy/-4 unsafe/+4 verification; new palette owner88 lines, bank remains.
AA05 DB59 reversal shares ordered segment operations:5 rows3093 selected+272 setup/0,
fmt/library/DLL160 unchanged warnings; -62 production/-62 legacy, unsafe/verification unchanged.
Exact incremental route preserves AA04 palette/stride/broadcast proofs and all X resets before Y.
Fixed HDMA allocation uses bounded DMA bytes with the original low/observer/high/observer protocol.
32 declarations and fixed initializer/caller cohort close; generic interpreter/bus/native contracts remain.
Final17f fmt/library/DLL160 unchanged warnings and570 compiled source/native bindings pass.
Qualified0e29 unique rows49235 selected+33413 setup/0; genuineb2d six captures183971+329/5,
exact affected9-member encrypted restore and both Kago14350 approaches retain original source/profile lineage.
Final AA05 five joint rows bind17f; no accepted capture/29-row repeat or full86/14/22 claim.
Net -49 physical production/-4 legacy/-59 unsafe/verification0;12 lines are formatter reflow, not size savings.
Whole Session/backend independence and source-only allocation/observer/mapper/CrystalFlash variants remain unfinished.
Phantoon figure-eight: six functions/fourteen constants/two callers share ordered speed/movement/animation;
six raw native actor captures removed. Source9fe fmt/library/DLL160 unchanged warnings/source571/native pass.
Phantoon two rows plus four smokes:6234 selected+272 setup/0; full Morph1200 retained.
Net -98 production/-266 legacy/-9 unsafe/+4 verification; new167-line module and callers fully counted.
Native allocation/diagnostic/lazy ROM order preserved; full/odd/off-table and failure variants remain source-scoped.
CorpseRotting setup/scheduler/39 dispatch leaves share checked native records and tile/packed-VRAM protocol.
Six native captures/seventeen raw word loads/one MemCpy and36 unused wrappers removed; both finish names retained.
Sourcef427 fmt/library/DLL160 unchanged warnings/source571/native/all41rawcodec pass;8rows11183 selected+272 setup/0.
Net -60 physical production/-60 legacy/-23 unsafe/verification0;21 address joins excluded,39 substantive lines removed.
Whole-word mover preflight, native failure prefixes and packed queue store order retained; unobserved species/failure variants remain source-scoped.
Kraid frame cohort:12 raw capture forms/13 expanded,20 word-load expressions and11 offset forms removed; four native calls remain.
Actualjoinedeba0 fmt/library/DLL160 unchanged warnings/source571/native/all41rawcodec pass;5rows3348 selected+272 setup/0.
Net -36 physical production/-36 legacy/+1 unsafe from a forbid attribute/verification0; -37 substantive lines, formatter+4 excluded.
Ordered native/callback/queue/failure contracts retained; original5560 budget/both120 tails and full Morph1200 preserved.
Mother Brain22 callback outcomes/both frame interpreters/three sprite-map consumers use existing checked native records.
Ten raw native captures/21 word loads/one byte load removed; unsafe entry/backend/spawn/palette contracts retained.
Source93a fmt/library/DLL160 unchanged warnings/source571/native/all41rawcodec pass;7rows10883 selected+272 setup/0.
Net -51 production/-51 legacy/-1 unsafe/verification0; uncovered: DeadTorizo_MainGfxHook D761 at296,187,0.
Historical README SVG/treemap publishing is retired by the user; do not regenerate
progress images or Pages/image CI. Earlier support+57/publisher-51 earned no engine-size credit.
Shared EnemyScript replaces the duplicate projectile cursor/record/position/byte layer across both hosts, lifecycle, commands and verifier adapters.
Sourced672 fmt/library/DLL160 unchanged warnings/source570/native/all41rawcodec pass;21rows32443 selected+1713 setup/0.
Three missing old-codec starts replaced by genuine3ac captures18742 comparisons/0; inheritedb2d provenance remains distinct from currentd672 execution.
Net -57 production/legacy0/-1 unsafe from duplicate forbid removal/+18 verification;11 formatter joins excluded,46 substantive lines removed.
Fresh loaded-image/three-byte operand/zero-target/lazy callback and failure prefixes retained; internal checked-offset panic wording uses the shared reader.
Crocomire21 reactions/choose-spit policy use a pure owned module; projectile init reads the live typed actor key after both velocity stores.
Source32c9 fmt/library/DLL160 unchanged warnings/source571/native/all41rawcodec pass;7rows6453 selected+272 setup/0.
Net +37 production/legacy0/+1 unsafe from new forbid attribute/+4 verification binding; no raw operation retirement or size-saving claim.
Actor storage/native contracts remain; typed getter preserves private selected-field access, full/odd keys and separate lazy x/y reads.
Existing b2d/d672 genuine starts reused with original provenance; no capture repeat.
Botwoon pending-hit/active-death/tail-ready/previous-health/body-landed protocol has one owned Session authority.
Complete combat/tail/history/head/body and projectile callers retain native/LE full/odd aliases, unknown words and original deletion/reset behavior.
Source92da fmt/library/DLL160 unchanged warnings/source572/native/all42rawcodec pass;5rows3132 selected+272 setup/0.
Genuine initialized329+resume5 and dying capture480 to945 (465 resumed comparisons) pass; exact12-member encrypted restore is byte-identical under codec247e1e71060271a5.
Explicit extra phase480 to481 retains active1/head9A46/delay248 to249;4684 actual comparisons/0 includes this one repeated phase comparison, not4684 unique frames.
Net +169 production/legacy0/+1 unsafe from a forbid attribute/+60 verification; raw-operation retirement0, ownership growth counted in full.
Maw low/high independent stores retain failure prefixes, adopting only ten protocol bytes; export prefix precedes asset/room conversion. Existing checked runtime/Maw/eproj27 pass.
Actual detached composition passes at02 with193 unchanged dependencies transferred to03; native/backend storage and whole Session/media independence remain open.
Prepared room TileTables/shared CRE now import immutable encoded streams into a validated, versioned catalog and pack with fixed world/profile identity.
Actual initial2/3 and door4/5 GameOwner consumers bind before SnesInit; both fixture rooms use cre2. Legacy cursor and live/unproduced RAM references are preserved.
Source3845 fmt/library/DLL160 unchanged warnings/source573/native/all43rawcodec pass; actual detached example transfers194 unchanged dependencies.
Nine jobs start within0.573s; six rows11216 selected+272 setup/0, genuine initialized329+resume5, total11822 ordinary comparisons/0.
Actual prepared GameOwner checkpoint roundtrip/continuation, exact changed-catalog revision rejection before adoption, pack malformed/immutability and both destination checks pass at codec3e92b93347c1e182.
Fresh prepared checkpoint13002415 bytes and initialized checkpoint retain source/native/full-input guards; no old247e loads/restamps or encrypted distribution/restore claim.
Net +545 production/legacy0/+1 unsafe from a forbid attribute/+210 verification; raw-operation retirement0. Catalog/schema/validation/identity/adapters exceed the180..400 forecast, counted in full.
Engine delta: new module526 - old decompression39 + world43 + compatibility14 + registration1 =545; this relocates the shared decoder and adds catalog behavior, with no decoder savings.
Verification210: room resources159 + composed fixture16 + corrected route14 + cases14 + detached2 + codec1 + registry4. No new verification framework.
Session::open/application pack selection, remaining Levels/Tiles/Palette/Background/Fx/transition consumers and independent media/backend remain open.
Next: the external reviewer selects goals and milestone tasks in the issue tracker; preserve assigned source boundaries.

Source-only: Blackout37, exact page2 callback, sky initializers, CrystalFlashStart,
spike/DBF2/lavaquake and additional signed8.8 kernels; MotherBrain magnitude,
health/bands and seal-wall store-order variants. Preserve setup14's seven fresh indices
and five multi-read blocks, low-address failure and fresh native pointer capture.
Keep CRT logger buffering/output/error/flush hold.
Native player/DSP/APU and input Vec/FILE/Cart owners retain SDL/native/audio quiescence,
CRT order and GameOwner's non-sharing contract; SIZE_MAX/malloc0 cases remain excluded.
sm82 AST:3 options kills/8.06min, unsafe gameplay omission unavailable. sm9b AST:3 kills,
original750d/0cdf attribution and own941 post-deletion full86 retained.
Fixpoint96 unnecessary blocks/94 flattened bodies strips no unsafe function signatures;
conservative taint and reuse of only unchanged accepted compiler results remain required.
Checker+13 support lines; old timing is not a cross-workload speedup.

Coverage is86 manifest rows/44 required scenarios, not universal behavior proof.
Keep full inputs/guards/assets: intro8780, Morph1200, bomb-jump800, crawler8, Gunship300,
X-ray2131 with original pickup, and BOTH independent Kago14350 setup windows.
No Morph120-frame early stop. Authored starts are not original TAS seeds. Golden Torizo
keeps all5 input ports and observed VRAM read-buffer/CGRAM-low/Mode7 latch values.
Only historical Core37 C43 entry0 is observed; seven native angles, launch/clamp leaves,
release/mosaic/postcredits134/138/143/145 and other native/failure variants remain scoped.
Goofball macro sensitivity remains open; held scrolling needs scroll_tiles.row.bg2,
absent in Ceres. Never auto-rename macro-shared parameters or invent emulator defaults.

sm_b3 passed scoped mutation before deletion (11.40min:3 AST kills,2 unobserved
pipe-flight survivors,1 separate dispatch kill;945-frame windows) and post-push full86
(136,526 selected+38,555 setup/0 failures). Do not claim surviving paths were reached.
Reflec's native printf remains in sm_a3 pending a contract-preserving output owner.
Keep existing room mutation drivers until alias/failed-upload/custom-content consumers
and equivalent coverage migrate; bank campaigns alone do not replace those checks.
M1a's [audit](M1A-REVIEW.md) retains19 historical contracts and the verifier bridge's
separate closures; keep imported alias exclusions, source newlines and Windows dep-info guards.

## Implementation contracts

- sm/src is authoritative; no C to consult/regenerate. Preserve quirks; use the oracle,
  ram_map.rs and [bank logs](https://patrickjohnston.org/bank/) when meaning is unclear.
- Before boundaries, state ownership or backend changes, read the accepted destination
  and [state migration](ENGINE_STATE_MIGRATION.md), plus linked contracts for that interaction.
- Keep g_ram byte-compatible with SNES layout and one authority per execution interval.
  Preserve full/odd identities, unknown values, wrapping arithmetic, native versus LE
  access, ROM/native allocation provenance, off-table aliases, suspension and failure prefixes.
- Keep captured identities distinct from fresh selectors; preserve every lazy query,
  ordered effect and Runtime/Drop epoch. Do not predecode, batch or sort effects for savings.
- Migrate complete API/owner/create/setup/update/callback/failure/reset/load/delete/reuse
  and actual consumers before retiring RAM communication. Export before legacy/CPU/native
  writers; import the actual selected normalized image afterwards, including partial failures.
- Completed cores forbid unsafe and need no Game/RAM/raw pointer/backend modules.
  No always-live owned mirror beside writable authoritative RAM, whole-RAM borrow while
  raw aliases remain, forwarding facade or package move presented as completed ownership.
- Retirements add no unsafe outside banks. Only hardware/IO register functions may retain
  unsafe in their existing platform owner, named in the body; otherwise check or keep in-bank.
  New owned modules forbid unsafe. Retain unsafe entry contracts for unchecked callees.
- Keep backend/shader/audio/oracle behavior, masks/PatchBugs and snapshots until actual
  consumers and equivalent checks migrate. Dependencies mean the backend is not retired.
  The611..616 normalization means continuation also needs source proof, not byte comparison alone.
- Reuse the existing runner; no new production probes/observers/verify cfg blocks,
  fixture modules/process frameworks/evidence stores/registries. Improve the verifier itself.
  REVIEW permits a constructed check only for behavior no movie reaches: under100 lines,
  with the reason in its first line. Otherwise add named missing behavior/order/failure rows.
- Count all helpers/tables/adapters/registration/callers/native runtime code. Separate
  verification, support/installers, dependencies and combined totals; document mixed-file
  exclusions reproducibly. Inline verification stays production; compare like classifications.
- Moves, formatting/packing/comment deletion and production-to-verification relocation
  are not savings. Count overlaps once, check tokens/text, never sum independent drafts
  or claim gross retirement as net. Unsafe deny attributes count as lines, not operations.
- Never commit ROMs, movies/checkpoints/saves or derived game data. Only encrypted
  release assets use the existing ci-data route; verify complete restore/member identities.
- Small commits, one coherent idea. If stuck over an hour, record findings here and stop.

## Fast loop

1. cargo fmt --check; cargo check --no-default-features --lib with no added warnings;
   one DLL build: python tools/verify_dll.py --build-only. Denied work never passes.
2. verify_dll.py --corpus movies/segments.toml --segment for every affected named row,
   plus door-loading-lag-nmi,tas-ceres-four-doors,area-crateria,item-morph-ball; zero failures.
   No reached row: A adds one; B moves unchanged and writes uncovered: <function> in the body.
3. Nothing extra routinely: no mutation/independent controls/frontend metadata, optimized
   rebuilds, clean-source reruns or full replays. Once before deleting a bank, scoped
   cargo-mutants <=30min; full corpus runs on main AFTER deletion, also at review waypoints.
   Windows uses AST mutants through existing DLL rows; Cargo/nextest cannot run here.
   Stable GNU coverage lacks profiler_builtins (E0463); LLVM tools alone do not fix it.
4. Every comparison failure gets a checkpoint regression row with its fix; never weaken
   comparisons/expectations. Retry a flaky step once; a second failure is real.

For assigned sequential phases, collect/release the prior handle and budget, sample RAM/disk and other reservations,
then immediately activate each exact diagnostic-group owner with followup_task; record actual activation and send frozen source pins before inventory/report prose.
Declare the next budget and notify the integrator/Compute, then launch without waiting for a prose ACK.
Start with >3GiB free, leave >=2GiB after remaining growth, and keep >=50GiB disk including outputs; new overlaps share one ledger.
Default source handoff is one lossless canonical patch and complete inventory against actual main; use a64MiB scoped host source budget.
Keep four bank consumer owners and one primitive owner disjoint. Transfer accepted proofs by exact hashes instead of repeating campaigns.
Use bounded local Windows owners for source/review/checks; do not reactivate the
stopped Linux host. Windows DLL/replay still bind final source/native identities.
After each qualified cycle, reassess1h landed unsafe/legacy/production changes and source/review/build/replay/landing time, then address the measured bottleneck.
Partition independent groups/use spare CPU; preserve shared prefixes and all identity
guards. Run ready original-seed subsets during serial capture dependencies, then complete
checkpoint rows without repeating accepted work absent a source change/new concern.
Codec changes require genuine initialized machine/payload/continuation checks and fresh
captures, never restamps or wrong-codec loads. Normalized source length is not payload size.
Cross-recipe reuse requires SHA-bound identical canonical input prefixes; keep source,
ROM/native/full-input/loader guards and save matching manifest starts during setup.
Use [movies/README](../movies/README.md) for commands; inspect masks/hardware/audio/persistence when touched.

No routine CI observation/reliance on Windows (direct user); hosted automatic backstop
and cloud mode remain separate. Fix/revert known red changes from your push before landing.
Cloud no-ROM agents use one ci/<lane>-<topic> branch, push each commit, auto-land on green
with rebase; conflicts/red are the author's next task. Free runners only; main full corpus
runs every3h/deletion/waypoint, at most6 shards, not each routine push. Rows are comma/space
IDs; trailing comma continues, colon text is ignored, unknown IDs fail. A branch must touch
sm/,tools/,segments.toml or workflow. Workflow edits require manual landing, not GITHUB_TOKEN.
L1 stays Windows until Linux is measurably faster end-to-end; CI seeds/lag guards remain.

## Toolchain, resources and completion
Edition2024, stable1.99.1 (58cf8f9c3 2026-10-13), x86_64-pc-windows-gnu.
Nightly2026-10-22 is ASR1121-blocked; do not alter security/spoof hosted variables.
Standalone tests/denied compiler and build scripts remain restricted; use permitted
DLL/ctypes. Ordinary frontend linking passes with the exact dependency-only seed.
Bundled verifier cfg still reaches denied khronos_api scripts; use the separate normal
frontend and existing direct DLL with unchanged source/native/cache guards.
Keep crate-root unsafe-op enforcement; paths in [ITERATION](ITERATION.md#windows-shell-setup).

Use isolated snapshots/fixed interfaces/one integrator. Frozen formatted source may
compile during final review; DLL/publication bind the reviewed final pin. Never edit
checked source/rebuild loaded DLLs. Explicit followup_task wakes completed workers;
send_message alone does not. Refill ready work without waiting on review prose.
Budget compilers/replays/source separately, including accepted launchers before rustc
and admitted jobs' remaining growth; release admission as soon as compilation ends.
Start compilers above3GiB free; reserve1.6GiB/library or2.225GiB/direct DLL and leave >=2GiB.
Below2GiB/resource errors stop newest producer, not every lightweight reader.
Measure CPU/RAM/paging/disk/useful throughput: two -j1 builds76.06s versus114.12s serial;
three is unproven. Raise concurrency only when budgets
fit and the next useful batch improves measured throughput; -j1 permits parallel codegen.
Streaming TAS imports reserve200MiB (measured90.5MiB), old whole-log imports3GiB.

Check disk/storage runway at session start, before batches and during artifact jobs.
Keep >=50GiB after budgets; cleanup below75GiB to restore100GiB. Below50GiB hold new
builds/copies/captures; bounded running jobs may finish and existing-artifact/source work continues.
Budget copies/temporaries/retained outputs/captures:300MiB fresh workflow or200MiB
remaining for copied direct-only work, ABOVE reserve. Seed dependencies, not full targets;
reuse checked builds and keep incremental caches enabled, bound to toolchain/profile
and source/native/output identities. Preserve others' caches and compare warnings
in the same profile.
Assign cleanup ownership; preserve source/results/reusable checkpoints/loaded DLLs.
Resolve paths/verify unneeded artifacts first; explicit compiler-file allowlists, nonrecursive
non-force deletion preferred. Never delete another worker's artifacts/locks/directories.
NTFS compression needs unchanged hashes. Report policy rejection immediately; never retry
through another tool or use tiny compression gains to continue below the reserve.

Done: Fast loop passes; run sh tools/progress.sh, explain increases, put measured Score
last, fetch/reconcile/recheck HEAD/index, commit to main and immediately push each commit.
Update active PLAN beside code; no hand-maintained progress image or extra readiness gates.
Pre-transpile .scratch/gates/pointers/scopes are inactive in archive/pre-transpile-2026-10-17.
