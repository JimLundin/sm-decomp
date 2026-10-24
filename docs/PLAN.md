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

Read [the external review](REVIEW.md) next. **The [milestone plan](REVIEW.md#milestone-plan-2026-10-24)
there is binding and executes work order items 1-2.** Two lanes only: lane A (one
agent) aligns the published any%/100% TASes (A0) and maps checks onto them; lane B (all others, one agent per
bank) retires legacy `game/sm_*.rs` banks in its bank order, deleting each file. Every
commit subject starts `A:` or `B:sm_XX`, and its body ends with the scoreboard line
the milestone plan defines. A docs-only commit needs a code commit beside it.
Verification is movie segments as data, not new Rust fixtures or production
hooks; see the review's size goal and verification policy. Keep this plan
under ~200 lines; results belong in commit messages (~8 lines). Direct user
instructions take precedence; [HISTORY.md](HISTORY.md) holds no current rules.

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
   Current recipes: 24/44 required checks pass; 20 remain pending. Three additional
   checks (in-memory save, upward elevator, Ice return) pass; the corpus is unfinished.
   A0.1: full original any% on main verified 129,712 frames / 0 failures, but only
   six Ceres rooms, final DF8D and zero credits frames (source identity 1f25f453).
   A0.2: original lsnes rr2-beta23/bsnes085 smoke logged 300 frames; all recorded
   inputs matched the movie. State 1 begins at index 271 versus native 150: a boot
   timing observation, not the first route divergence. Next: full reference capture.
   Keep raw counters, resets, subframes and latched input; flat NMI counters also
   occur during coroutine waits. No input removal or guessed offsets yet.
   `python tools/verify_dll.py --unit-tests --direct` runs 20 observer tests, not
   gameplay checks. Regenerate incompatible checkpoints from boot; preserve the
   old checked navigation build.
2. **Current - lane B bank retirement.** Retire `sm_94`; parallel drafts own `sm_91` and `sm_90`.
   The obsolete collision record/bridge is retired; finish the remaining `sm_94` families and delete its file.
   Report added production and deleted legacy lines together; net growth needs a
   reason. Get production below 110,000 before starting another subsystem. Reuse
   existing Host/effects/adapters unless they demonstrably cannot serve. Route
   probes through a no-op production host method; move inline verification cfg
   differences behind that method in touched subsystems. Try `cargo-mutants`
   before adding text-patch mutation recipes; retain recipes only where needed.
3. **Then - finish owned gameplay.** Migrate remaining complete producer/consumer
   interactions, actual callers and legacy retirement. Close persistent ownership,
   content/topology/options boundaries and unsafe removal across the whole engine;
   preserve vanilla behavior and the Map Rando contracts throughout the migration.
4. **Then - prove the independent engine.** Complete our renderer/audio with small
   platform libraries, remove the game runtime's SNES-backend dependency, and verify
   boot-to-credits plus custom sprites, reordered rooms/doors and opt-in mechanics.

Persistent whole-engine ownership and backend independence remain open. Larger
reductions need live caller migration; the unused-bank scan found no removable family.
Do not auto-rename macro-shared parameters.
Held scrolling consolidation needs `scroll_tiles.row.bg2` coverage, absent in Ceres.
Keep input `e956dffa` / `%LOCALAPPDATA%/sm-decomp-experiments/20261024/player-input-*` held:
repriced at +25 production / +1,235 verification lines; no final-source movie found.
Snapshot separation `2a3c7792` is lower priority: 390 verification lines moved,
15 wiring lines added, no logic deleted.

## Implementation rules

- `sm/src` is the source of truth; no original C exists to consult or regenerate.
  Use the oracle, `ram_map.rs` and [bank logs](https://patrickjohnston.org/bank/)
  when meaning is unclear. Preserve defined behavior exactly, quirks included.
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
- Plan a complete responsibility: public API, owner, setup/update/exceptional paths,
  callers and retirement. Adopt actual caller migration and legacy deletion together.
  Small preparation needs named consumers and an explicit deletion step; isolated
  shared consumers or forwarding facades are not subsystem completion.
- Reuse checks/fixtures; add verification only for named uncovered behavior, ownership,
  order or failure contracts. No harness per subsystem. Parallel implementation and
  independent review use fixed interfaces, isolated files and one integration/build lane.
- Count all production helpers, tables, adapters, registration, callers and maintained
  native code. Separate verification, support/installers, external dependencies and
  combined totals. Runtime adapters are production; identify mixed-file exclusions
  reproducibly. Compare like-for-like classifications; historical path totals differ.
- File moves, code packing, formatting compression and moving code outside `sm/src`
  are not reductions. Count overlaps once and check tokens/text against packing.
  Do not add independent draft savings or claim gross retirement as net savings.
  Necessary safety work may grow source; disclose its complete cost and reason.
- Retain backend/shader/audio/oracle behavior, masks, patches, state and snapshots
  until their real consumers and equivalent checks migrate. Do not claim backend
  retirement while gameplay or verification still depends on it.
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

Only one heavy compiler/capture job at a time across the machine; root coordinates
integration. `-j 1` does not coordinate separate Cargo processes. Never rebuild a
loaded DLL. Workers can inspect callers, prepare isolated changes and review checks.
Use [the iteration guide](ITERATION.md#windows-shell-setup) for paths/commands;
its historical queues or examples do not override this plan.

Keep artifacts local to their checkout. Seed dependencies, not entire target trees;
preserve active caches and unrelated work. Use `CARGO_INCREMENTAL=0` for disposable
snapshots, bound disposable output and stream build logs to files. Reuse matching
source-checked builds. Before deleting disposable output, verify it is unneeded
and safely resolve the intended path. Never delete another worker's artifacts.

## Definition of done for any change

Pass applicable final-source checks and execute changed behavior without new failing frames.
Run `sh tools/progress.sh`; explain increases and full production/verification/support costs.
Recheck HEAD/index, reconcile and commit to main with commands, probes/frames, results and limits.
Update this plan only for active-item, direction or blocker changes; results belong in commits.
Publish PROGRESS at adopted milestones or user request: shared graph and measured metrics,
held drafts distinct. Generate its table with `sh tools/progress.sh --markdown WARNINGS [PASSED/TOTAL]`;
leave unknowns explicit (path counts include inline hooks). No atlas/architecture refresh
unless requested, invented percentages from nodes/files, or claims of unadopted savings.

Pre-transpile work is archived in `archive/pre-transpile-2026-10-17`; its
`.scratch/`, readiness gates, acceptance pointers and translation scopes are inactive.
