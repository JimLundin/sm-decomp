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

## Review directives (user direction, 2026-10-24)

Read [REVIEW.md](REVIEW.md) next. Its milestone plan governs work order 1-2:
lane A (one agent) aligns TASes and adds coverage; lane B (one agent per bank) retires banks.
Subjects start `A:` or `B:sm_XX`; bodies end with the review's `Score:` line.
Docs-only commits need a code commit beside them. Verification is movie data,
not new Rust fixtures or production hooks. Keep this plan near 200 lines and results
in concise commit messages. Direct user instructions win; HISTORY holds no current rules.

**Codebase map:** brief done (c1fe795). Pages generates the
[view](https://jimlundin.github.io/sm-decomp/) from HEAD; keep only the five-box
SVG in PROGRESS current. No other map work unless the user asks.

**Local validation:** do not rely on or observe CI except for explicitly selected
research/experiments unavailable on Windows. Fix already-known failing changes
before further adoption; the review's red-main rule does not authorize CI polling.

## Work order

1. **Current - lane A published TAS alignment (A0).** Align the published any% and
   100% movies, then map the existing verifier's per-segment expectations onto them.
   Cover Crateria, Brinstar, Norfair, Wrecked Ship, Maridia and Tourian; every boss;
   main pickups, doors/elevators, save/load, death and credits. Report results and rooms.
   Commit only recipes. Preserve input/NMI timing; no guessed lag/offset changes.
   Each adoption names segments exercising changed behavior and downstream effects.
   Current recipes: 26/44 required checks pass; 18 remain pending. Three additional
   checks (in-memory save, upward elevator, Ice return) pass; the corpus is unfinished.
   Measured wait-entry inputs align 9,266 original TAS steps. Options/door timing
   fixes requalified all 24 previously runnable required recipes. At step 9,266,
   original audio upload reads input during 41 acknowledged NMIs; native upload
   has no elapsed video clock. The review's measured read-collapse experiment
   leaves a 40-count difference at RAM $05B6 on step 9,266 and first gameplay
   divergence at 9,291. A0 remains open; do not offset counters or mask differences.
   Under the accepted fallback, Power Bomb pickup and the closing-gate regression
   pass. Wave pickup passes; current: Crocomire. Regenerate incompatible checkpoints from boot
   and preserve the checked navigation build.
2. **Current - lane B bank retirement.** `sm_94` and `sm_91` are deleted.
   Player input, demos, suits, equipment, commands, initialization, Mother Brain
   reactions, X-ray and direct damage use owned state. Natural X-ray, Gravity,
   boss and grapple movie coverage remains incomplete despite bounded checks.
   Cannon control is owned; sm_90 has 1,306 lines left. Next: typed forced movement.
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

## Validation strategy (2026-10-21)

1. **Select before running.** Name changed routines, branches, callers, shared state
   and downstream effects. Use the shortest scenario that actually exercises them.
   Docs/comments/format-only edits need static checks; removal of demonstrably
   unused private code needs reference/compiler checks; frontend edits need UI
   checks. The hygiene review explicitly requires a full replay for its combined pass.
2. **Check final source.** Run root `cargo fmt` then `cargo fmt --check` with default
   configuration before every Rust commit. Run appropriate normal/backend-free
   compiler checks and a release build for behavior changes. New library source
   belongs in `sm/src/verify_source.rs`; keep source and inputs fixed while checking.
   The DLL launcher's embedded-source check must match the actual compiled bytes.
3. **Exercise behavior.** Require changed-routine/branch probes with `--require-hit`
   and downstream probes where available; missing hits fail even with zero frame
   differences. Use `--frames` or `--after-hits N` to observe meaningful callbacks,
   transitions, later frames and persistent writes. Keep normal RAM/oracle comparisons,
   rendering and audio advancement enabled. No new failing frames.
4. **Reuse only valid prefixes.** Checkpoint state must precede affected setup/calls
   and shared-state history. Runtime compatibility alone is insufficient. Cache a
   necessary prefix as setup, separate from changed-code coverage. Reuse builds and
   baselines only when source bytes and relevant context match. Removing temporary
   probes requires rebuilding and repeating the measured segment on clean source.
5. **Close coverage gaps.** Choose another route or a focused component/oracle check
   of the actual implementation. Constructed cases are not movie execution; identify
   their limits. Undefined/unaligned predecessor Rust is not a fidelity reference;
   use qualified ROM-oracle evidence instead. Source identity is not behavior proof.
   Hold cleanup of an uncovered context; do not extend unrelated green replays
   or claim full-game fidelity from one room. Earlier routes do not cover credits.
6. **Justify full movies.** Except for the explicit hygiene requirement, show why
   changed behavior needs the full route; name probes/frame ranges first. Size,
   elapsed time, commit count or subsystem completion alone is not justification.
7. **Mutate meaningful behavior.** Confirm the unchanged baseline passes, then check
   isolated compiling faults in boundaries, updates, ordering or ownership. Use the
   permitted Python-hosted DLL path; try scoped `cargo-mutants` before new recipes.
   Never adopt mutants. Compile failures, timeouts, crashes, unrelated assertions
   and unexecuted checks are not kills. Investigate survivors or show equivalence.
   Keep mutation support separate and disclose scope; a few kills are not comprehensive.
   Add ordinary `#[test]` coverage for pure owned logic; report blocked/unexecuted
   tests accurately rather than bypassing Windows restrictions.
   Repair stale `SETTLEMENT_EARLY_CONSUME` / ambiguous `room_export_before_binding_check` anchors before reuse.

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
matched work in 76.06s versus 114.12s serial (1.50x). Three are feasible; a speedup
beyond two is unproven. The integrator assigns ready slots explicitly, counting
accepted launchers before rustc appears. Restart completed workers with a new task.
Allow up to three compiler workflows; start above 3GiB free, account for ~1.6GiB
per compiler plus growth, and stop the newest below 2GiB or on resource errors.
Refill slots immediately and release after compilation. Replays may overlap.
Judge completed useful work/time, memory and paging, not CPU alone. Never edit
checked source or rebuild loaded DLLs. -j1 still permits parallel code generation.
Use [the iteration guide](ITERATION.md#windows-shell-setup) for paths/commands.

Keep artifacts in their checkout; seed dependencies, not full target trees, and reuse checked builds.
Preserve active caches and others' work. Set `CARGO_INCREMENTAL=0` for disposable snapshots;
bound output and stream logs. Before deletion, resolve paths and verify artifacts are unneeded.
Never delete another worker's artifacts.

## Definition of done for any change

Pass applicable final-source checks and execute changed behavior without new failing frames.
Run `sh tools/progress.sh`; explain increases and full production/verification/support costs.
Recheck HEAD/index, reconcile and commit to main with commands, probes/frames, results and limits.
Update this plan only for active-item, direction or blocker changes; results belong in commits.
Publish PROGRESS with adopted code: shared graph and `sh tools/progress.sh --markdown WARNINGS [PASSED/TOTAL]`.
Keep drafts/unknowns explicit; path counts include inline hooks. No unsolicited atlas refresh,
invented percentages or unadopted savings.

Pre-transpile rules (`.scratch/`, gates, pointers, scopes) are inactive in `archive/pre-transpile-2026-10-17`.
