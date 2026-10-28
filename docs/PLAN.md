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
Root 01a17ecd takes implementation, coordination, Git, builds and the resource
ledger only after 01a17c50's explicit RELEASE and verified terminal writer/turn
state. Preserve current writer/jobs until then; main is the default, isolation optional.
Linux development is stopped by the user (2026-10-27); preserve its frozen handoffs.
Only the primary writes Git; optional branches can carry coherent source with
pending checks named. Main requires qualification.
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

Primary root owns implementation/Git/build/resources after verified integrator
release. Only main is remote and Jim's conversation #6 remains open. Delegate
source, review and qualification through the focused skills; keep work moving
while CI runs. Qualified ci16 source 54727767 / DLL 6eb76bb5 composes numeric, actor
and Display state into the gated six-component Session checkpoint envelope.
Format, normal-library input transfer, native/source guards and 87/87 units pass.
Fresh initialized/save/reserve checkpoints and 26 distinct rows pass 199,063
comparisons/0, including full Morph 1200 and BOTH independent Kago 14350 approaches.
Coverage 23/44; full Session/backend independence and engine-size acceptance remain
open. Net +2095 production/+1603 verification/+2 forbid attributes/legacy 0;
actual raw-operation retirement 0. Original ci16 receipts/artifacts are preserved.
Audio-only ci17 source 475b05d3 / DLL 4905f1a7 passes actual 87/87 units,
including wrapped queues, rejection, binary restore and ApuQueue witnesses.
All 85 raw codec contributors are unchanged; retain original ci16 runtime proof.
Net +435 verification/+1 forbid attribute; production/legacy unchanged.
OpenGL upload source 61b7a7f3 is independently reviewed and qualified through
the actual frontend library/binary (182.783s), with regenerated config/shader
and desktop archives. Owned Vec pixels and presenter storage retire malloc/free,
two associated pointer casts and three raw global captures. Net +21 production,
unsafe-containing -1; the existing OpenGL warning is unchanged. All 85 CPU codec
contributors are unchanged; preserve original unit/replay attribution. GL device
callbacks remain unexecuted. CI passes on both Session/audio commits.
Config-input source b9178ea8 passes independent review, regenerated native archives
and actual frontend library/binary (184.214s). Owned buffers preserve native parser
views, teardown order and the original non-Send/Sync owner; input malloc and raw
sentinel add/write retire. Net +7 production; all 209 frontend warning causes and
85 CPU codec contributors are unchanged. Include-path C ownership remains.
Standalone workflow Python has been published. GL01 source 415f0350 passes native
guards and actual frontend library/binary qualification in 188.374s, with all 209
warning identities unchanged. Its 42 atomic slots retire two Rust global writes;
native GL reads still require serialized access on the owning context thread.
Net +8 production/-1 unsafe-containing; original units/runtime attribution stays.
INCLUDE-PATH01 source review passes;
utility/headless/frontend qualification remains pending. Complete those actual
paths before making their qualification claims.
Prefer Python for portable automation, with standalone scripts instead of inline
Python in GitHub Actions. Reserves 2 GiB/50 GiB.

### Qualified history

The current milestones and qualified ci16/ci17/frontend results are above.
Superseded SOURCE15 through 38 progress, failure, cfg/warning and codec-transfer reports
remain in the [immutable 4c302a50 history](https://github.com/JimLundin/sm-decomp/blob/4c302a50/docs/PLAN.md#L100-L446).
Preserve each original source/DLL/native/input/loader attribution and the recorded
alias, read-order and failure contracts when touching those owners; history does
not assign new work or authorize repeating accepted campaigns.

The qualified SOURCE30 cohort is source d9daaf02 / DLL 6768ee76: 66 units,
26 assigned rows (23/44 required), 199,063 comparisons/0, full Morph 1200 and
BOTH Kago 14350 setups. Its 76 unchanged raw codec contributors bind
f6e7c80dc575e620, 1,257,015 raw / 1,246,178 normalized bytes; original failed
fixtures and genuine initialized/save/resume attribution remain in the history.
Room, renderer, dialogue and Scene/delivery evidence is summarized in
[REVIEW](REVIEW.md#current-publication-and-acceptance).

Earlier [WRAM/Execution, bank, fixed-HDMA, enemy and room-catalog qualifications](https://github.com/JimLundin/sm-decomp/blob/4c302a50/docs/PLAN.md#L370-L446)
retain their exact scoped rows, counts, mutation outcomes and exclusions.
Moves, formatter changes and source-only work earn no qualification or savings.

### Retained qualification constraints

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

The primary publishes qualified small commits to main and fixes known CI failures.
Observe the actual CI workflow and commit; continue independent implementation
while hosted checks run. Free runners only; Windows full corpus runs every 3h,
after bank deletion and at review waypoints, at most 6 parallel shards. Routine
pushes run CI without a separate no-op replay run. Manual row IDs are space-separated;
unknown IDs fail. Keep seed/lag/source guards; Linux development remains stopped.

## Toolchain, resources and completion
Edition2024, stable1.99.1 (58cf8f9c3 2026-10-13), x86_64-pc-windows-gnu.
Nightly2026-10-22 is ASR1121-blocked; do not alter security/spoof hosted variables.
Standalone tests/denied compiler and build scripts remain restricted; use permitted
DLL/ctypes. Ordinary frontend linking passes with the exact dependency-only seed.
Bundled verifier cfg still reaches denied khronos_api scripts; use the separate normal
frontend and existing direct DLL with unchanged source/native/cache guards.
Keep crate-root unsafe-op enforcement; paths in [ITERATION](ITERATION.md#windows-shell-setup).

Use fixed interfaces and one writer; isolate when useful. Frozen formatted source may
compile during final review; DLL/publication bind the reviewed final pin. Never edit
checked source/rebuild loaded DLLs. Explicit followup_task wakes completed workers;
send_message alone does not. Refill ready work without waiting on review prose.
Budget compilers/replays/source separately, including accepted launchers before rustc
and admitted jobs' remaining growth; release admission as soon as compilation ends.
Start compilers above3GiB free; reserve1.6GiB/library or3.25GiB/uncapped direct DLL.
Leave >=2GiB after remaining growth and other reservations.
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
