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
Push every commit to remote immediately after committing (user, 2026-10-26).

## Review directives (user direction, 2026-10-24; reset 2026-10-25 12:30)

Read [REVIEW.md](REVIEW.md) next; its **Milestone plan** governs work order 1-2 and
the user gave the reviewer authority to change goals and process (2026-10-25).
Lanes (reset 3, 23:25): **A** CI speed (A2: boot-replay rows onto seeds; A1 done, 44/44);
**M** unsafe outside bank files; **B1-B6** retire banks in the order REVIEW names.
Subjects start `A:`, `M:` or `B:sm_XX`; bodies end with the review's `Score:` line.
Speed rule: the **Fast loop** below replaces the old validation strategy. Land
verified work at once; work held unlanded over one hour is a stall. No docs-only
commits without a code commit beside them. Verification is `segments.toml` rows,
not new Rust fixtures or production hooks. Keep this plan near 200 lines.
Direct user instructions win; Git history holds no current rules.
**Reviewer authority (user, 2026-10-25 19:00):** instructions from the external
reviewers (the hourly review check-in and the cloud Claude agents' REVIEW.md
directives) are to be obeyed as given. Disagree in one line under the item, then do it.

**Progress picture (user, 2026-10-25):** the interactive map is retired. The only
progress view is `tools/treemap.py`'s image, published by Pages on every main push and
embedded in README.md. Nobody edits that image; no other dashboards.

**Tooling cleanup (user, 2026-10-26):** mobile portrait map, essential replay docs,
upstream references, runtime-only restores and shared Python installers. Measured
selected restore: 114178386 to 523438 bytes with identical runtime assets. Keep
CSV/TOML provenance and checks; support savings are separate from engine savings.
Spark `ab94761f` replacement ratio is 83.51%; its body incorrectly says 84.54%.

**Landing (user decision 2026-10-25 23:25, given live to the reviewer; amends 18:05).**
A Windows agent that ran its `Rows:` locally on a tip rebased onto current `origin/main`
pushes straight to main (`git push origin HEAD:main`; rejected push = fetch, rebase,
rerun only rows the rebase touched, push). Hosted `Segments` on main is the backstop: a
red main is fixed or reverted by its author before anyone else lands. Cloud agents (no
ROM) keep `ci/<lane>-<topic>` branches with auto-land on green. Seeds/lag logs in CI
(C2), cloud agents (C3) and the Linux verifier (L1) stand. Details: REVIEW.md reset 3.

## Work order

1. **Current - lane A coverage.** 44/44 required scenarios have passing local evidence.
   Golden Torizo passes 1,187 frames, fresh defeat at 1,066 and the full 120-frame tail;
   its normalization and projectile-parameter regressions add 1 and 235 passing frames.
   The original Snes9x seed preserves all five input ports and independently proved
   VRAM read-buffer, CGRAM-low and Mode7 latch values. No invented register defaults.
   Existing required rows and expectations remain intact; hosted verification and
   the combined full corpus are separate obligations, not inferred from this result.
   **A2 complete:** 69/83 rows use original seeds, thirteen use hash-bound native/oracle
   checkpoints, and only the full 8780-frame intro boots with zero setup. The existing
   runner and encrypted 22-asset bundle preserve full inputs, original windows,
   guards, both Kago setup encounters and all twelve previously distributed assets.
   These route starts are not original TAS seeds. Converted-row configured setup
   falls from 1459878 to 28700 frames; original seeded warmups remain separate.
   The final six rows plus smoke pass ordinary CLI 10/10, 7034 selected/zero setup.
   Seven unused recipes removed; net support reduction is 340 lines across both A2
   packets. Speed remains for the full intro input. A continues the next unclaimed
   bank in REVIEW's queues; coverage and bank retirement do not finish owned Session.
2. **Current - lane B retirement.** B1 sm_88/sm_a0/sm_81 and B3 sm_8f/sm_b2 are
   retired. M2 sm_a8, eleven B2 cinematic families and B3 Goofball are integrated;
   all 83 corpus rows passed on aebfee7d after bank-A2 deletion; later work uses scoped rows.
   B1 sm_80 is retired with checked gameplay and retained backend contracts;
   its affected rows pass, and B1 continues sm_82. B2 continues sm_8b after
   consolidating the checked credits interpreter; cloud-motion rows are `credits`,
   `credits-postroll-wave` and the four smoke rows. B3 sm_a2 is retired after
   the complete Snail interaction; B3 continues sm_a3. Non-enemy banks aim at most 80% replacement size.
   Windows M is now B4: retire sm_a8 families, then sm_b3.
   B3 sm_a2 retirement passes 39 local rows and the scoped bank mutation campaign.
   The new fixed 300-frame Gunship row detects bobbing beyond the entry tail;
   The 8-frame crawler slope regression passes with the full 800-frame bomb-jump
   window and both 14350-frame Kago setups; all 53 scoped rows pass after its fix.
   Ending and enemy continuations pass all 30 scoped rows on the published source.
   the full 83-row corpus passes on main with zero failing frames. Goofball macro sensitivity is open.
   No further in-place bank codemods; cloud M moves to unsafe outside bank files.
   A2 seeds remaining authored rows; bundle shard grouping is integrated.
   B5/B6 cloud queues continue sm_a9/sm_a6, then sm_aa/sm_a7.
   RAM remains authoritative; bank moves alone do not establish owned Session state.
   Preserve full identities, ordered failures/effects, native/LE access and ROM provenance.
   BG2 consolidation still needs scroll_tiles.row.bg2. Reject prototype 42c59cb's
   whole-RAM aliases and false-safe Load24; retain unsafe backend contracts.
3. **Then - finish owned gameplay.** Migrate remaining complete producer/consumer
   interactions, actual callers and legacy retirement. Close persistent ownership,
   content/topology/options boundaries and unsafe removal across the whole engine;
   preserve vanilla behavior and the Map Rando contracts throughout the migration.
4. **Then - prove the independent engine.** Complete our renderer/audio with small
   platform libraries, remove the game runtime's SNES-backend dependency, and verify
   boot-to-credits plus custom sprites, reordered rooms/doors and opt-in mechanics.

Persistent whole-engine ownership and backend independence remain open. Larger
reductions need live caller migration; the unused-bank scan found no removable family.
Core15 passes all 84 local rows; Core16 passes 25 affected/smoke/control rows.
Qualified Core16: production 112368, legacy 22107, unsafe 3085, verification 39963.
Core15's shared actor closeout costs 17 production lines; bank moves remain
transitional rather than proof of owned Session state. The existing comparison
normalizes 611..616; initializer continuation also relies on source review.
Do not auto-rename macro-shared parameters. Held scrolling consolidation needs
`scroll_tiles.row.bg2` coverage, absent in Ceres.

Mutation-driver audit: deleting the old room driver breaks retained local capture
callers. Existing bank campaigns do not replace its unusual alias, failed-upload and
custom-content checks; keep those checks until their consumers and coverage migrate.

M1a (`tools/codemod/strip_unsafe_fixpoint.py`) keeps all 22 unsafe production contracts
and the verifier bridge from [the review](M1A-REVIEW.md); bounded review found no remaining
concrete violation in its 86 conversions. All 64 hosted rows passed at 3484808d.
The rerun conservatively rejects imported aliases, validates Windows dep-info roots
and preserves source newlines. All required unsafe contracts survive. Keep the predecessor rejected.

Checkpoint cache finding: whole Session/codec hashes invalidate lossless scalar
migrations despite an unchanged RAM payload. A future format transition needs
Session round-trip checks and defined actor-identity persistence; keep guards now.

The local corpus saves matching manifest starts during setup and shares checkpoints
across recipe namespaces only
when SHA-bound canonical input prefixes match. Source, ROM, full-input and loader
guards remain intact. A donor-only smoke trial passed all four rows with zero
setup frames; Morph retains its full 1,200-frame window.

Morph keeps every required hit/expectation and its fixed 1,200-frame window.
The original room seed observes acquisition at 590 and message completion at 977,
leaving 222 following frames. The former authored window observed 506 frames after
the message; this conversion retains the full window, not that longer relative tail.
Do not add a 120-frame early stop to this row.

## Implementation rules

- `sm/src` is authoritative; there is no C to consult or regenerate. Preserve quirks.
  Resolve meaning with the oracle, `ram_map.rs` and [bank logs](https://patrickjohnston.org/bank/).
- Before changing boundaries, ownership or backend interfaces, read the accepted
  [architecture and migration policy](target-architecture/README.md).
  [State migration](ENGINE_STATE_MIGRATION.md) collects the ownership, aliasing,
  reset/load, replay, CPU-write and oracle-adoption constraints. Read detailed
  target contracts only for the interaction being changed.
- Keep `g_ram` byte-compatible with SNES layout; preserve numeric dispatch/identity,
  ROM formats, provenance, native/LE words, ordered effects, lazy reads, aliasing,
  suspension and failure prefixes. Use descriptive gameplay names and one authoritative
  state per legacy interval, never a mutable owned copy beside writable authoritative RAM.
- Move complete producer/consumer interactions into owned state before retiring
  their RAM communication. Completed cores forbid unsafe and need no Game, RAM,
  raw pointers or backend modules. Safe wrappers and package moves are transitional.
  Do not turn `word!` into a whole-RAM borrow while retained raw pointers remain;
  migrate offsets/handles and use checked, scoped access with overlapping reads intact.
- Retirements add no unsafe to non-bank files. Only hardware/IO register functions
  may retain unsafe in the existing platform owner, named in the commit body.
  Otherwise convert to checked access or retain the function in its bank. New owned
  modules forbid unsafe; every B body gives its ratio, with a reason above 90%.
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
   before its file is deleted: scoped `cargo-mutants` (<=30 min); the full corpus runs
   on main after the deletion lands (20:52 rule below), not on the branch.
   Partition independent movie groups to use spare CPU; keep shared prefixes and
   current source/codec checks. The full corpus also runs at each review waypoint.
   A failure from a full run gets a
   regression row with its fix. Never weaken a comparison or a row's expectations.
   On Windows, generate scoped AST mutants with cargo-mutants and execute them
   through the existing DLL rows; its Cargo/nextest runner cannot use this host.
   Stable GNU instrument-coverage lacks profiler_builtins (private probe E0463);
   installing LLVM tools alone does not enable local coverage on this toolchain.
4. Body <=8 lines: what changed, rows run and result, legacy/production delta and
   ratio, `Score:`. Re-run a flaky step once; a second failure is real.

**Hosted mode (user decision 18:05; live since 1f57df1, 19:07 CET):**
cloud agents only (Windows agents: Landing above). Name the rows in a `Rows:` body
line and push each commit immediately to one live `ci/<lane>-<topic>` branch.
`Segments` runs the union of `Rows:` over `origin/main..HEAD` plus the smoke set;
green auto-lands on main (rebased; a conflict fails the land and the author rebases).
A red run is that author's next task. Main's full corpus runs every 3h, on bank-file
deletion and at waypoints (dispatch `Segments` on main with empty `segments`), with
at most six shards at once. Routine lands start no full run.
**Landing (23:25):** Windows agents land on local green (above); no stacks, no
`ci/windows-*` branches. Existing `ci/b*` stacks: the owner rebases once, runs its rows
locally, pushes to main. Free runners only. Failures get a regression row as above.
Agents without the ROM (cloud) use hosted mode only. Mechanics: `Rows:` ids are comma/space
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
Start above 3GiB free and leave 2GiB after reserving 1.8GiB per DLL build or 1.4GiB
per library check plus admitted jobs' remaining growth. Stop the newest below 2GiB
or on resource errors. Frozen, formatted source may compile while its final review runs;
Core10's DLL process reached 1764MiB; the 1.8GiB budget replaces the older 1.7GiB estimate.
DLL/publication use the reviewed final pin. Do not leave ready checks waiting on prose.
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
The progress image updates itself from main; never edit it by hand.

Pre-transpile rules (`.scratch/`, gates, pointers, scopes) are inactive in `archive/pre-transpile-2026-10-17`.
