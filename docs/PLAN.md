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

Current message source5787: production117936, legacy17793, unsafe2588,
verification44030. Actual incremental1 no-default library retains200 warnings,
zero added identities against the identical-profile renderer baseline200.
The verifier-only reset successor transfers all1013 actual compiler inputs unchanged.

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
