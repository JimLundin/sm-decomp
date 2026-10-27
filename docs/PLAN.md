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
Windows agents run named local rows on a tip rebased onto current origin/main,
commit to main and immediately `git push origin HEAD:main`. A rejected push means
fetch/rebase, rerun only affected rows, then push. No Windows stacks or ci/windows branches.
Land verified work promptly; holding it over an hour is a stall. No docs-only commits
without a code commit beside them. Update PLAN only for active direction or blockers.

Subjects start A:, M: or B:sm_XX. Body <=8 lines: change, exact Rows/results, full
production/legacy/verification deltas, B ratio/reason and final Score. Use four actual
UTF-8 middle dots, not question marks; measure the values locally with progress.sh:
`Score: checks 44/44 · legacy N · production N · unsafe N · verification N`.

The sole progress view is tools/treemap.py's mobile portrait image, published by
Pages on each main push and embedded in README. Never edit it or add dashboards.
Keep replay docs essential, upstream references and CSV/TOML provenance/checks;
shared Python installers/runtime-only restores are support work, counted separately.

## Current work and limits

Current sourcef427: production113897, legacy17880, unsafe2543, verification41836; warnings160.
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
Next: prepared Kraid frame responsibility on this parent, pending independent review/final local qualification.

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
Use available Linux capacity for local format/type checks when Windows compiler budgets do not fit; Windows DLL/replay still bind final source/native identities.
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
DLL/ctypes. khronos_api blocks ordinary frontend linking; direct DLL builds work.
Current cached-rlib frontend/Config DLL research is bounded and permitted, not routine validation.
Keep crate-root unsafe-op enforcement; paths in [ITERATION](ITERATION.md#windows-shell-setup).

Use isolated snapshots/fixed interfaces/one integrator. Frozen formatted source may
compile during final review; DLL/publication bind the reviewed final pin. Never edit
checked source/rebuild loaded DLLs. Explicit followup_task wakes completed workers;
send_message alone does not. Refill ready work without waiting on review prose.
Budget compilers/replays/source separately, including accepted launchers before rustc
and admitted jobs' remaining growth; release admission as soon as compilation ends.
Start compilers above3GiB free; reserve1.6GiB/library or2.1GiB/DLL and leave >=2GiB.
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
reuse checked builds, CARGO_INCREMENTAL=0 for disposable snapshots; preserve others' caches.
Assign cleanup ownership; preserve source/results/reusable checkpoints/loaded DLLs.
Resolve paths/verify unneeded artifacts first; explicit compiler-file allowlists, nonrecursive
non-force deletion preferred. Never delete another worker's artifacts/locks/directories.
NTFS compression needs unchanged hashes. Report policy rejection immediately; never retry
through another tool or use tiny compression gains to continue below the reserve.

Done: Fast loop passes; run sh tools/progress.sh, explain increases, put measured Score
last, fetch/reconcile/recheck HEAD/index, commit to main and immediately push each commit.
Update active PLAN beside code; no hand-maintained progress image or extra readiness gates.
Pre-transpile .scratch/gates/pointers/scopes are inactive in archive/pre-transpile-2026-10-17.
