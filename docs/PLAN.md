# Plan

Goal: a compact Super Metroid engine playing boot to credits with owned, safe Rust
gameplay and clear modules. Remove unsafe/raw-memory operations from gameplay,
retained native interfaces and verification; hiding them in wrappers is incomplete.
Target below 100,000 production lines; 90,000 is a stretch, not promised deletions.

Map Rando requires custom sprites/palettes, reordered rooms/doors and opt-in re-spin/QoL.
Separate content, topology and options from platform code and ROM/RAM compatibility;
vanilla is the first fidelity profile. The [target](target-architecture/README.md)
is an owned deterministic Session, our renderer/audio and small platform libraries,
independent of SNES emulation. Windows migration may retain the emulated backend/oracle.

Git concurrency: never reserve or hold index/ref locks. Use Git's own short-lived
locks only during actual writes; no locks during builds, checks, reasoning or waits.
Recheck HEAD, index and other work before every Git write; reconcile changes and
preserve others' work. Do not delete another process's lock or force-push main.

## Review directives (user direction, 2026-10-24; reset 2026-10-25 12:30)

Read [REVIEW.md](REVIEW.md) next; its **Milestone plan** governs work order 1-2 and
the user gave the reviewer authority to change goals and process (2026-10-25).
Lanes: **A** coverage via room-seeded segments (A1) and CI rows (C0, C2); **M**
mechanical safety codemods; **B1/B2/B3** retire banks in the order REVIEW names.
Subjects start `A:`, `M:` or `B:sm_XX`; bodies end with the review's `Score:` line.
Speed rule: the **Fast loop** below replaces the old validation strategy. Land
verified work at once; work held unlanded over one hour is a stall. No docs-only
commits without a code commit beside them. Verification is `segments.toml` rows,
not new Rust fixtures or production hooks. Keep this plan near 200 lines.
Direct user instructions win; HISTORY holds no current rules.

**Codebase map:** brief done (c1fe795). Pages generates the
[view](https://jimlundin.github.io/sm-decomp/) from HEAD; keep only the five-box
SVG in PROGRESS current. No other map work unless the user asks.

**Hosted validation (user decision 2026-10-25 18:05, given live to the reviewer;
replaces the earlier "do not use or observe CI" note).** The user lifted the
local-only rule: hosted CI is the main verifier, the Windows machine is not.
Agents push to `ci/<lane>-<topic>` branches, let `Segments` verify, and land on
main when it is green (see Fast loop). The user also approved: ROM-derived seeds and
lag logs in CI, encrypted with a repository secret (C2); cloud Linux agents with
auto-land on green (C3); a Linux verifier spike (L1). Details: REVIEW.md lane C.

## Work order

1. **Current - lane A room-seeded coverage.** 40/44 required checks pass;
   4 remain pending. Exact pre-read controller observations preserve the first
   input edge; Gravity and Grapple pass fresh awards, message completion and full tails.
   Any% 197 and 100% 331 room observations are captured; seeds/movies remain local.
   Hash indexes and regeneration recipes are committed. Existing seeded regressions
   remain passing. Mother Brain has 4,560-frame and 2,756-frame spawn regressions; defeat is pending.
   Botwoon passes a fresh defeat and full tail (945 frames) from the published 13%
   route. Spore Spawn is absent from any%/100%; Golden Torizo is skipped in 100%.
   The 100% Mother Brain 10,000-frame trial has zero oracle differences but misses
   original defeat/exit; seed/route fidelity needs diagnosis. No new hand routes.
2. **Current - lane B bank retirement.** `sm_94`, `sm_91`, `sm_89` and `sm_90` are deleted.
   B1 continues sm_81 menu drawing (1,668 lines); B2 continues sm_88 (321 lines); B3 continues
   Crocomire sm_a4 (1,676 lines); attack and return scripts share checked actors. Player input, rendering, movement and dispatch share existing
   owners; save/load now shares checked slot and map transfers.
   Bank removal is not persistent whole-engine state ownership: camera
   compatibility and the unchanged Ceres elevator callback still use raw RAM.
   Room FX loading uses Room/Visuals. Weather and haze now share their update logic.
   Retire production observer hooks with named segments; no new Host layers or fixtures.
   Preserve full u16 identities, unknown values, failure timing and ROM provenance.
   Natural Gravity, boss and grapple coverage remains incomplete. BG2 tile
   consolidation waits for `scroll_tiles.row.bg2` coverage, absent in Ceres.
   Report production and legacy deltas together; growth needs a reason. Reuse existing
   interfaces and remove touched inline verification differences. The next size
   target is below 110,000 production lines; try scoped cargo-mutants before text patches.
   M1 checked caller-side byte access passes all 59 runnable rows locally, including
   78,108 selected and 100,697 setup frames; 123 redundant unsafe blocks are cleared.
   M2 Ripper/Jet Power Ripper now share existing Actor operations: two raw getters
   and 28 dereferences removed; eleven rows pass. Next: remaining common-enemy families. Prototype 42c59cb is not safe to adopt: its whole-RAM
   borrows can invalidate retained enemy pointers, and its signature stripping
   exposes raw-pointer Load24 as safe. Passing replays do not resolve these defects.
   We retain this checked M1 ahead of the revised REVIEW's proposed M1b deferral:
   it creates no RAM reference or slice, directly addressing the alias concern.
   HDMA backend calls retain unsafe contracts: safe Game construction leaves public
   hardware pointers null. Typed arguments and checked RAM do not prove backend validity.
3. **Then - finish owned gameplay.** Migrate remaining complete producer/consumer
   interactions, actual callers and legacy retirement. Close persistent ownership,
   content/topology/options boundaries and unsafe removal across the whole engine;
   preserve vanilla behavior and the Map Rando contracts throughout the migration.
4. **Then - prove the independent engine.** Complete our renderer/audio with small
   platform libraries, remove the game runtime's SNES-backend dependency, and verify
   boot-to-credits plus custom sprites, reordered rooms/doors and opt-in mechanics.

Persistent whole-engine ownership and backend independence remain open. Larger
reductions need live caller migration; the unused-bank scan found no removable family.
Do not auto-rename macro-shared parameters. Held scrolling consolidation needs
`scroll_tiles.row.bg2` coverage, absent in Ceres.

Mutation-driver audit: deleting the old room driver breaks retained local capture
callers. Existing bank campaigns do not replace its unusual alias, failed-upload and
custom-content checks; keep those checks until their consumers and coverage migrate.

Checkpoint cache finding: whole Session/codec hashes invalidate lossless scalar
migrations despite an unchanged RAM payload. A future format transition needs
Session round-trip checks and defined actor-identity persistence; keep guards now.

The local corpus saves matching manifest starts during setup and shares checkpoints
across recipe namespaces only
when SHA-bound canonical input prefixes match. Source, ROM, full-input and loader
guards remain intact. A donor-only smoke trial passed all four rows with zero
setup frames; Morph retains its full 1,200-frame window.

Morph row keeps every required hit/expectation. Its fixed 1,200-frame window ends
at 28,700, 506 frames after the hit at 28,193. The old 120-frame early stop ended
at 28,314 (814 frames); retaining it would truncate the extended observation.

## Implementation rules

- `sm/src` is authoritative; there is no C to consult or regenerate. Preserve quirks.
  Resolve meaning with the oracle, `ram_map.rs` and [bank logs](https://patrickjohnston.org/bank/).
- Before changing boundaries, ownership or backend interfaces, read the accepted
  [architecture](target-architecture/README.md) and [migration](target-architecture/MIGRATION.md).
  Consult [engine design](ENGINE_DESIGN.md) and [RAM ownership](RAM_OWNERSHIP.md)
  for the changed interaction; [state migration](ENGINE_STATE_MIGRATION.md) governs
  reset/load, replay, CPU writes and oracle rewind/adoption.
- Keep `g_ram` byte-compatible with SNES layout; preserve numeric dispatch/identity,
  ROM formats, provenance, native/LE words, ordered effects, lazy reads, aliasing,
  suspension and failure prefixes. Use descriptive gameplay names and one authoritative
  state per legacy interval, never a mutable owned copy beside writable authoritative RAM.
- Move complete producer/consumer interactions into owned state before retiring
  their RAM communication. Completed cores forbid unsafe and need no Game, RAM,
  raw pointers or backend modules. Safe wrappers and package moves are transitional.
  Do not turn `word!` into a whole-RAM borrow while retained raw pointers remain;
  migrate offsets/handles and use checked, scoped access with overlapping reads intact.
- Scope each responsibility: API, owner, setup/update/failure paths, callers and retirement.
  Adopt caller migration with deletion; preparation needs named consumers and a deletion step.
  Shared consumers or forwarding facades alone are not completion.
- Reuse checks; add only named missing behavior/ownership/order/failure coverage, no subsystem harness.
  Parallel work uses fixed interfaces, isolated snapshots and one integrator.
- Count all production helpers, tables, runtime adapters, registration, callers and native code.
  Separate verification, support/installers, dependencies and combined totals. Document mixed-file
  exclusions reproducibly; compare like-for-like classifications, not historical path totals.
- Moves, code packing, format compression and relocating code outside `sm/src` are not savings.
  Count overlap once; check tokens/text against packing. Never add independent draft savings
  or claim gross retirement as net savings. Disclose necessary safety growth and its full cost.
- Retain backend/shader/audio/oracle behavior, masks, patches, state and snapshots until
  consumers and equivalent checks migrate. Dependencies mean the backend is not retired.
- Never commit ROMs, movies, checkpoints, save states or ROM-derived content.
  Do not build process frameworks, evidence stores, registries or dashboards.
  Improve the existing verifier when the loop is too slow/weak; record the issue here.
- Make small commits with one coherent idea. If stuck on an item for over one
  hour of work, record the findings under it and stop.

## Fast loop (reviewer, 2026-10-25; replaces the 2026-10-21 validation strategy)

Every routine commit runs exactly this, then commits:
1. `cargo fmt --check`; `cargo check --no-default-features --lib` (no new warnings);
   one DLL build (`python tools/verify_dll.py --build-only`).
2. `verify_dll.py --corpus movies/segments.toml` with `--segment` for every row that
   exercises the change (named in the body) plus the smoke set `door-loading-lag-nmi`,
   `tas-ceres-four-doors`, `area-crateria`, `item-morph-ball`. Zero failures.
   No row reaches the change: lane A adds one; lane B moves the code unchanged and
   writes "uncovered: <function>" in the body.
3. Nothing else per commit: no mutation runs, independent controls, frontend metadata,
   optimized/release rebuilds, clean-source reruns or full replays. Once per bank,
   before its file is deleted: the full corpus plus scoped `cargo-mutants` (<=30 min).
   Partition independent movie groups to use spare CPU; keep shared prefixes and
   current source/codec checks. The full corpus also runs at each review waypoint.
   A failure from a full run gets a
   regression row with its fix. Never weaken a comparison or a row's expectations.
   On Windows, generate scoped AST mutants with cargo-mutants and execute them
   through the existing DLL rows; its Cargo/nextest runner cannot use this host.
4. Body <=8 lines: what changed, rows run and result, legacy/production delta and
   ratio, `Score:`. Re-run a flaky step once; a second failure is real.

**Hosted mode (user decision 18:05; live since C3 landed, 1f57df1, 19:07 CET; all lanes):**
local work is step 1 plus at most one smoke row. Name the rows in a `Rows:` body
line, push to `ci/<lane>-<topic>`, and start the next change at once on a new
branch; do not wait on the run. `Segments` runs the `Rows:` rows plus the smoke set
on the branch; green auto-lands on main (rebased; a conflict fails the land and
the author rebases). A red run is that author's next task. Main runs the whole
corpus as a backstop; its failures get a regression row as above. Agents without
the ROM (cloud) use hosted mode only. Mechanics: `Rows:` ids are comma/space
separated, a trailing comma continues the list, text after a colon is ignored and an
unknown id fails the plan. The branch must touch `sm/`, `tools/`, `segments.toml`
or the workflow, or no run starts. Workflow edits never auto-land (GITHUB_TOKEN
cannot push them): their green branch is fast-forwarded onto main by hand.
**L1 (go, 19:15):** the Linux `.so` verifier passes area-crateria and item-morph-ball
with every coverage line identical to the Windows shard (run 40218156987), but its
replay is ~1.6x slower (crateria 8340..13600: 40.1s vs 24.5s), so shards stay on
Windows until a Linux shard is shown faster end to end. Linux job: `ci/l1-*` pushes
or dispatch with `linux`.

Commands and checkpoint rules: [movies/README.md](../movies/README.md).
When touched, inspect `cpu_infra.rs` masks, `PatchBugs`, hardware/audio limits and
persistence behavior. Do not weaken comparisons to make a replacement pass.

## Toolchain and Windows constraints

Use edition 2024, stable `1.99.1 (58cf8f9c3 2026-10-13)`, target `x86_64-pc-windows-gnu`.
Nightly `2026-10-22` remains blocked by Defender ASR event 1121; migration is unfinished.
Do not change security settings or spoof hosted environment variables. Standalone tests
and denied compiler/build-script launches stay restricted. Use permitted DLL/ctypes;
denials never pass. khronos_api blocks frontend linking; cached frontend metadata and
direct DLL builds work. Keep crate-root unsafe-op enforcement.

Measured concurrency (2026-10-25): two isolated builds with Cargo -j1 completed
matched work in 76.06s versus 114.12s serial (1.50x). Three compilers reached
2.088GiB free; a further speedup is unproven. Use separate budgets for compilation,
replays and source work, not a blanket heavy-job count. Count accepted launchers
before rustc appears; release compiler admission immediately after compilation.
Start above 3GiB free and leave 2GiB after reserving ~1.6GiB per new compiler plus
admitted jobs' remaining growth. Stop the newest below 2GiB or on resource errors.
Refill useful ready work immediately; completed workers need explicit new tasks.
Measure CPU, free memory, paging, disk and completed useful work/time during real
jobs. Streaming TAS import reduced measured peak from 2.794GiB to 90.5MiB
with identical inputs (9.347s to 5.752s). Reserve 200MiB for this qualified path;
legacy whole-log imports still need 3GiB, separately from compiler reservations.
On pressure, shed the newest memory-producing job, not every lightweight reader.
Increase concurrency only when resources fit; compare the next useful batch
with the measured two-build result before retaining a higher setting. Three is an
observed trial, not a proven optimum. Disk monitoring must include accumulated
artifact storage and batch runway, not just whether one more job fits.
Never edit checked source or rebuild loaded DLLs. -j1 permits parallel code generation.
Use [the iteration guide](ITERATION.md#windows-shell-setup) for paths/commands.

Keep artifacts in their checkout; seed dependencies, not full target trees, and reuse checked builds.
Direct builds honor `CARGO_INCREMENTAL=0`; use it for disposable snapshots and preserve others' caches.
Check free disk and accumulated workspace/experiment build storage at session start
and before each batch; monitor free disk during artifact-producing jobs. Maintain
at least 50GiB free after budgeting the batch. Begin cleanup below 75GiB and
restore 100GiB free; below 50GiB, hold new builds, copies and captures until the
reserve is restored. Already-running bounded jobs may finish; source work and
checks using existing artifacts continue. The user raised this reserve on
2026-10-25: 10GiB was insufficient for a dedicated roughly 500GB workstation.
Budget copies, peak compiler temporaries, retained outputs and replay captures:
measured direct output is 76.8MiB plus no-default 22.6MiB; reserve 300MiB for a fresh
workflow or 200MiB remaining for an already-copied direct-only build, in addition
to the headroom above. Review obsolete generated caches when their work is adopted;
preserve source, results and reusable checkpoints. Assign cleanup ownership explicitly.
NTFS compression halves measured inactive rlib storage; verify unchanged hashes and
keep loaded DLLs untouched.
Before deletion, resolve paths and verify artifacts are unneeded. Never delete another worker's artifacts.
Prefer inspected, explicit compiler-file allowlists and nonrecursive deletion without
force; retain directories, locks, source, results, DLLs and checkpoints.
If agent policy rejects cleanup, report it promptly; do not let small compression
savings justify continuing below the storage budget or retry deletion through another tool.

## Definition of done for any change

The Fast loop passes; `sh tools/progress.sh` numbers are in the Score line, with any
rise explained in one line. Recheck HEAD/index, reconcile, commit to `main`. Update
this plan only for active-item, direction or blocker changes; results belong in commits.
Refresh PROGRESS's five-box SVG only when an item or bank completes.

Pre-transpile rules (`.scratch/`, gates, pointers, scopes) are inactive in `archive/pre-transpile-2026-10-17`.
