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

Current source6dd3: production111941, legacy17353, unsafe2713, verification40385; warnings165.
Core sm82 persists actor/media identities and saved ROM patches; RAM/native frame dependencies remain explicit.
Historical f158c44fac33eac7 qualification retains39 contributors/652740 source bytes;
prepared110/initialized329+5,14 captures189475 frames and full22 byte-identical restore at original source.
Native startup now returns success; pixel preparation borrows the current GameOwner, no retained Snes pointer.
Native3 source6dd3/library/frontend33 checks+569 inputs/DLLfe9e passes; five local rows10967 selected+272 setup/0.
Codec2854b408fa453744 has39 contributors/652841 source bytes; prepared110 source lines/boot329+5
retain accepted dce54 attribution. No full14/full22/full86, startup/device or full ownership claim.
Cost +3 engine Rust, -1 native header/-1 installer support; unsafe/legacy/verification unchanged.
Required sm82 AST8.06min:3 options kills; unsafe gameplay omission unavailable.
Blackout37/native failure variants remain unobserved; hardware entry contracts stay explicit.
sm9b retired: scoped AST3 kills3453 selected/0 setup; original target750d/0cdf attribution retained.
Own-sm9b941 full86 passes136526 selected+38827 setup/0; all14 originals and bothKago14350 retained.
Logo/page2 passes10967 selected+272 setup/0; boot has replay/source support, exact page2 callback remains unobserved.
Launcher cleanup passes local2187 selected+272 setup/0 and Linux10 existing before/after entry runs; -36 verification, engine unchanged.
Display shadows pass current10967 selected+272 setup/0: 49 static raw byte sites retired; line counts unchanged.
Particle axes pass10967 selected+272 setup/0: five authored steps shared; -12 engine/-12 legacy, native diagnostics retained.
Sky byte references pass2787 selected+272 setup/0: three raw accesses/two mutable bindings retired; initializers remain source-scoped.
Next: joint stable WRAM ownership; Linux backend storage, Windows Game/RAM and actual consumers, cloud banks coordinated first.
Linux M: map-menu selection/safe callbacks -25 engine/-16 unsafe metric; raw pointers unchanged, six unsafe entries retained.
Linux M: native room diagnostic/store +1 engine/-1 unsafe metric; Bomb request dispatch -39 engine/unsafe0.
Combined source2d5c passed eight hosted rows17968 selected+21384 setup/0 and ordinary CI; tested=landed88b3.
Keep low-address failure, fresh native pointer capture and CrystalFlashStart source scope explicit.
Linux M: share four owned projectile axis kernels -23 physical/-24 substantive engine; unsafe/interfaces unchanged.
Source1bf7 passed six rows2429 selected+272 setup/0 and ordinary CI; tested=landedae0f.
Exact spike/DBF2/lavaquake kernels remain source-only.
Linux M: setup14 shares fresh native-word reads -51 physical/-52 substantive engine; source review pass.
Keep seven fresh indices/five multi-read blocks; source9964 passed16 rows16977 selected+1713 setup/0 and ordinary CI.
Linux M: six more signed8.8 projectile blocks reuse unchanged axis helper -43 physical/substantive engine.
Sourceb5aa review/format/library pass165 warnings; eight related+smoke rows3924 selected+272 setup/0;
Linux tested=landed7aa8 author receipt6432458588; exact variants remain source-only.
ROM logger substitution is held for CRT buffering/output/error/flush equivalence.
Native player/DSP/APU and input Vec/FILE/Cart are owned; GameOwner does not promise Rust thread sharing.
Keep SDL/native/audio quiescence and CRT order; invalid SIZE_MAX/malloc0 callbacks
remain outside universal equivalence. Whole Session and independent media remain unfinished.
First WRAM cut retains the backend aligned allocation and private, nonreplaceable Game binding.
Close all actual consumers, at least169 files including reserved sm_a7 reference-update; coordinate that cloud cohort first.
Preserve each byte/block failure and device order; bind full Game implementation to codec and make genuine captures.
Bus APIs stay unsafe until owned links, recursion/native/observer/reset/lifecycle closure; no slice/Index facade.

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
Start compilers above3GiB free; reserve1.4GiB/library or1.8GiB/DLL and leave >=2GiB.
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
