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
Lanes: **A** (one agent) coverage, now via room-seeded TAS segments (A1); **B1/B2/B3**
(one agent each, at most three compilers) retire banks in the order REVIEW names.
Subjects start `A:` or `B:sm_XX`; bodies end with the review's `Score:` line.
Speed rule: the **Fast loop** below replaces the old validation strategy. Land
verified work at once; work held unlanded over one hour is a stall. No docs-only
commits without a code commit beside them. Verification is `segments.toml` rows,
not new Rust fixtures or production hooks. Keep this plan near 200 lines.
Direct user instructions win; HISTORY holds no current rules.

**Codebase map:** brief done (c1fe795). Pages generates the
[view](https://jimlundin.github.io/sm-decomp/) from HEAD; keep only the five-box
SVG in PROGRESS current. No other map work unless the user asks.

**Local validation:** do not rely on or observe CI except for explicitly selected
research/experiments unavailable on Windows. Fix already-known failing changes
before further adoption; the review's red-main rule does not authorize CI polling.

## Work order

1. **Current - lane A Crocomire, then pending pickups/bosses.** A0 is parked until
   18:00 CET per review; afterward align published any%/100% and map expectations.
   Cover Crateria, Brinstar, Norfair, Wrecked Ship, Maridia and Tourian; every boss;
   main pickups, doors/elevators, save/load, death and credits. Report results and rooms.
   Commit only recipes. Preserve input/NMI timing; no guessed lag/offset changes.
   Each adoption names segments exercising changed behavior and downstream effects.
   Current recipes: 26/44 required checks pass; 18 remain pending. Three additional
   checks (in-memory save, upward elevator, Ice return) pass; the corpus is unfinished.
   Measured wait-entry inputs align 9,266 original TAS steps. Options/door timing
   fixes requalified all 24 previously runnable required recipes. At step 9,266,
   original audio upload reads input during 41 acknowledged NMIs; native upload
   has no elapsed video clock. Read-collapse leaves $05B6 forty counts behind at
   step 9,266; the old 9,291 comparison mixed phases. Elapsed-NMI experiment reaches
   first visible-state mismatch at 10,555: Ridley countdown/setup is four updates late;
   RNG differs by 10,440; first onset is unlocated. Model stays local: one active-IRQ
   upload sample; equal-size uploads take 27/28 NMIs. Crocomire's shared $0E20 drop
   regression is fixed: 167,281 fresh setup frames plus the four-frame row pass;
   20 original-CPU cases and two mutations check the restored interaction. Continue
   the fight from the checked frame-167,285 checkpoint; defeat remains pending.
2. **Current - lane B bank retirement.** `sm_94` and `sm_91` are deleted.
   Player input, demos, suits, equipment, commands, initialization, Mother Brain
   reactions, X-ray and direct damage use owned state. Natural X-ray, Gravity,
   boss and grapple movie coverage remains incomplete despite bounded checks.
   X-ray controls/tables are consolidated; sm_88 has 1,454 lines left. sm_90 has 907.
   Pause sm_88 until sm_90 is deleted; retire production observer hooks with named segments.
   Crystal Flash consumption and Draygon grab/release are consolidated.
   Unused player completion hooks are removed; next: Crystal Flash setup, then
   the remaining complete sm_90 families.
   Use named enum/function handlers as families move; raw identities stay at
   compatibility boundaries. Full u16 unknown values and failure timing survive.
   Report added production and deleted legacy together; growth needs a reason.
   Get production below 110,000 before starting another subsystem. Reuse existing
   interfaces, remove touched inline verification differences, and try scoped
   cargo-mutants before text-patch mutations.
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

Checkpoint cache finding: whole Session/codec hashes invalidate lossless scalar
migrations despite an unchanged RAM payload. A future format transition needs
Session round-trip checks and defined actor-identity persistence; keep guards now.

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
   before its file is deleted: the full corpus plus scoped `cargo-mutants` (≤30 min).
   The full corpus also runs at each review waypoint. A failure from a full run gets a
   regression row with its fix. Never weaken a comparison or a row's expectations.
4. Body ≤8 lines: what changed, rows run and result, legacy/production delta and
   ratio, `Score:`. Re-run a flaky step once; a second failure is real.

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
jobs. Increase concurrency only when resources fit; compare the next useful batch
with the measured two-build result before retaining a higher setting. Three is an
observed trial, not a proven optimum. At 12:19 CET, disk was binding: one admitted
workflow, 5.92GiB RAM free and 1.30GiB disk free. Replays/source work overlap.
Never edit checked source or rebuild loaded DLLs. -j1 permits parallel code generation.
Use [the iteration guide](ITERATION.md#windows-shell-setup) for paths/commands.

Keep artifacts in their checkout; seed dependencies, not full target trees, and reuse checked builds.
Direct builds honor `CARGO_INCREMENTAL=0`; use it for disposable snapshots and preserve others' caches.
Budget disk against the job: measured direct output 76.8MiB plus no-default 22.6MiB.
A single bounded workflow reserves 300MiB additional space and leaves 1GiB for Windows;
monitor growth and budget copies/replays too. A fixed 2GiB free-disk floor is unnecessary.
NTFS compression halves measured inactive rlib storage; verify unchanged hashes and
keep loaded DLLs untouched.
Before deletion, resolve paths and verify artifacts are unneeded. Never delete another worker's artifacts.

## Definition of done for any change

The Fast loop passes; `sh tools/progress.sh` numbers are in the Score line, with any
rise explained in one line. Recheck HEAD/index, reconcile, commit to `main`. Update
this plan only for active-item, direction or blocker changes; results belong in commits.
Refresh PROGRESS's five-box SVG only when an item or bank completes.

Pre-transpile rules (`.scratch/`, gates, pointers, scopes) are inactive in `archive/pre-transpile-2026-10-17`.
