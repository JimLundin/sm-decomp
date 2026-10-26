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

1. **Lane A coverage complete:** 44/44 required scenarios have passing local evidence;
   hosted verification and full-corpus runs remain separate obligations. A2 uses
   69 original seeds and thirteen hash-bound native/oracle checkpoints; the full
   8780-frame intro still boots. Authored route starts are not original TAS seeds.
   Keep full inputs/windows/guards, both Kago setups and all distributed assets.
   Golden Torizo preserves all five Snes9x input ports and independently observed
   VRAM read-buffer, CGRAM-low and Mode7 latch values; no invented defaults.
   A continues the next unclaimed bank in REVIEW; coverage does not finish Session.
2. **Lane B retirement:** B1 continues sm_82, B2 sm_8b, B3 sm_a3, Windows M/B4
   sm_a8 then sm_b3. B5/B6 continue sm_a9/sm_a6 then sm_aa/sm_a7. Cloud M removes
   unsafe outside bank files; no further in-place bank codemods. Non-enemy
   replacements aim <=80%. RAM remains authoritative; bank moves are transitional.
   Cloud-motion rows: `credits`, `credits-postroll-wave` and the four smoke rows.
   Keep the 300-frame Gunship row, 8-frame crawler regression, full 800-frame
   bomb-jump window and both 14350-frame Kago setups. Goofball macro sensitivity
   remains open. Preserve full identities, ordered failures/effects, native/LE
   access and ROM provenance. Reject prototype 42c59cb's whole-RAM aliases and
   false-safe Load24; retain unsafe backend contracts.
3. **Then - finish owned gameplay.** Migrate remaining complete producer/consumer
   interactions, actual callers and legacy retirement. Close persistent ownership,
   content/topology/options boundaries and unsafe removal across the whole engine;
   preserve vanilla behavior and the Map Rando contracts throughout the migration.
4. **Then - prove the independent engine.** Complete our renderer/audio with small
   platform libraries, remove the game runtime's SNES-backend dependency, and verify
   boot-to-credits plus custom sprites, reordered rooms/doors and opt-in mechanics.

Persistent whole-engine ownership and backend independence remain open. Larger
reductions need live caller migration; the unused-bank scan found no removable family.
Qualified Core53: production111702, legacy19089, unsafe2853, verification39970; warnings179.
Native player/DSP allocations have actual owners: +25 engine lines, no unsafe-line savings; raw callback contracts remain.
Fresh4d25 checkpoints and actual rebuilt desktop archive pass source/native guards; old33e6 checkpoints remain incompatible.
Full86 passes136,526 selected+38,827 setup/0 failures;189,475 genuine capture frames, both Kago setups independent14,350.
All22 encrypted release members restore byte-for-byte; initialized machine payload/continuation passes, prepared Session identity remains open.
Retired sm_b3 separately passed post-deletion full86:136,526 selected+38,555 setup/0 failures.
Frame dispatch shares21 wrappers via45 bank-private descriptors: -12 engine/-19 unsafe lines; readonly table +360 bytes.
Independent full86:136,526 selected+38,827 setup/0 failures; captures retain Core50 provenance.
Grapple connection: -14 substantive engine/-101 legacy/-2 unsafe lines; five rows3338+272 setup pass, seven native angles remain unobserved.
Door-state dispatch: -14 engine/-14 legacy/-3 unsafe lines; seven rows4096+297 setup pass, Runtime Drop ordering unchanged.
Next: publish qualified grapple/door/cinematic families; complete ROM-load ownership/new476b verification runs in parallel.

Scoped mutation11.40min: 3AST kills, 2unobserved pipe-flight survivors, 1separate dispatch kill; all945-frame windows preserved.
Final retirement removes17 engine/48 legacy/4 unsafe/1 verification lines; full31/48=64.58%, LF conversion is not savings.
Complete Botwoon/all15 B3 preinstruction routing adds2 engine lines, retires104 legacy/15 unsafe; full106/104=101.92%.
GameOwner no longer promises Rust thread sharing; raw SDL userdata/gate/lifetime obligations remain explicit.
Powerbomb retires -34 legacy/-2 engine/-1 unsafe with no verification growth; full caller-inclusive replacement35/37=94.59%.
Escape animals share lava/rescue paths and migrate callers: -3 engine, -120 legacy, -4 unsafe; +4 verification, combined +1. All variants remain unobserved.
Melt graphics preparation/upload share their ordered queue encoder: -7 engine, -79 legacy, -2 unsafe, no verification growth.
Fish shares swim/turn logic: -4 engine lines, -123 legacy, -3 unsafe; +4 verification registry, combined zero. All Fish variants remain unobserved.
Reflec remains in sm_a3: its native printf has no safe output owner preserving the contract; mutation precedes bank deletion.
Fireflea retires 128 legacy lines and seven unsafe lines; four source-cache guard lines are separate verification.
Core39 passed eight Crocomire/control rows (6,457 selected); tilemap and native upload queues are checked.
Core38 passed 22 rows (66,513 selected); plume/melt and shared burst callers are checked.
Core37 passed all 86 rows (136,526 selected + 38,827 setup), 189,475 fresh captures and separate C43 105.
Eleven backend allocations now have stable owners; the snapshot observer blocker is closed.
Ownership adds 104 runtime lines (89 Rust + 24 native - 9 generated), removes 14 unsafe and 8 verification.
The fresh 33e6 checkpoint bundle is uploaded; all 22 restored members match byte-for-byte.
Four release, two mosaic and postcredits 134/138/143/145 remain unobserved; native/failure variants are source-reviewed.
Six read-only state projections preserve 57,621 bytes; exact native leaf variants remain source-reviewed.
The manifest remains 86 rows; hardware/Frame, external SpcPlayer ownership and backend independence remain open.
Retain the fixed 2131-frame X-ray activation/release row and original pickup.
The existing comparison normalizes 611..616; initializer continuation also relies
on source review. An unsafe deny attribute counts as a line, not an operation.
Do not auto-rename macro-shared parameters. Held scrolling consolidation needs
`scroll_tiles.row.bg2` coverage, absent in Ceres.

Mutation-driver audit: deleting the old room driver breaks retained local capture
callers. Existing bank campaigns do not replace its unusual alias, failed-upload and
custom-content checks; keep those checks until their consumers and coverage migrate.

M1a originally restored 22 production contracts and the verifier bridge from
[the audit](M1A-REVIEW.md); Core30 requalifies two and Core34 options start. The other 19
historical requirements and bridge retain their own safety closures. Keep imported
alias exclusions, Windows dep-info root checks and source newlines; the predecessor
remains rejected.

Checkpoint cache finding: whole Session/codec hashes invalidate lossless scalar
migrations despite an unchanged RAM payload. A future format transition needs
Session round-trip checks and defined actor-identity persistence; keep guards now.

Checkpoint reuse across recipe namespaces requires SHA-bound matching canonical
input prefixes; keep source, ROM, full-input and loader guards. Save matching
manifest starts during setup. Morph retains every hit/expectation and its fixed
1,200-frame window, with no 120-frame early stop. Its original seed leaves 222
frames after message completion, versus the former authored window's 506-frame tail.

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
