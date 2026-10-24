# Plan

Goal: a compact Super Metroid engine that plays from boot to credits, with
owned, safe Rust gameplay and clear module boundaries. Remove unsafe calls and
raw-memory operations, including retained native interfaces and verification;
wrappers that hide them do not complete the goal. Target below 100,000 production
lines; 90,000 is a stretch checkpoint, not a promised deletion budget.

Map Rando is a core requirement: custom sprites/palettes, reordered rooms and
door connections, and opt-in re-spin/QoL rules. Keep content, topology and gameplay
options separate from platform code and ROM/RAM compatibility. Vanilla is the
first fidelity profile. Follow the [accepted target](target-architecture/README.md):
owned deterministic Session, our renderer/audio and small platform libraries.
The final engine must run independently of SNES emulation; the current Windows
migration may keep its emulated backend and ROM oracle while replacing behavior.

Git concurrency: never reserve or hold index/ref locks. Use Git's own short-lived
locks only during actual writes; no locks during builds, checks, reasoning or waits.
Recheck HEAD, index and other work before every Git write; reconcile changes and
preserve others' work. Do not delete another process's lock or force-push main.

## Review directives (user direction, 2026-10-24)

Read [the external review](REVIEW.md) after this plan. Hygiene and process
reduction are done. Two tracks now (overrides "resume only after the corpus"):
(a) one agent continues checkpoint coverage as recipe data; (b) the others start
consolidation (work order 2) on subsystems whose changed behaviour the 23
passing checks exercise, naming those segments per commit. Production must
start falling; a docs-only commit needs a code commit beside it. Direct user instructions take precedence. Historical work is in
[HISTORY.md](HISTORY.md); its old queues, gates and results are not current rules.

**Codebase map:** the [simplified view](https://jimlundin.github.io/sm-decomp/)
shows gameplay responsibilities and owned / partly / legacy status, with support
files hidden and contextual dependencies. Desktop/phone checks pass locally.
Pages generates from HEAD; do not commit generated HTML or JSON. Current work
links to PROGRESS. One map agent owns any further user-requested changes.

**Local validation:** do not rely on or routinely observe CI. Use hosted checks
only for explicitly selected research/experiments that cannot run on this Windows
host. The review's known-red-main rule does not authorize CI polling; address any
already-known failing change before further adoption. Local results gate adoption.

## Work order

1. **Current - expand checkpoint coverage in parallel.** Recover reproducible
   route recipes and extend the existing verifier with per-segment expectations.
   Cover Crateria, Brinstar, Norfair, Wrecked Ship, Maridia and Tourian; every boss;
   main pickups, doors/elevators, save/load, death and credits. Report results and rooms.
   Commit only recipes. Preserve input/NMI timing; no guessed lag/offset changes.
   Each adoption names segments exercising changed behavior and downstream effects.
   Current recipes: 23/44 required checks pass; 21 remain pending. Three additional
   checks (in-memory save, upward elevator, Ice return) pass; the corpus is unfinished.
   Next coverage: Power Bomb pickup, then Crocomire. The ordinary approaches need
   power bombs or the far side of a green gate. The verified Ice return ends in
   Business Center; return through Red Tower's energy station toward A3AE's pickup.
   Existing verifier tests now execute locally: `python tools/verify_dll.py
   --unit-tests --direct` passes all 20 unchanged bodies. This is observer coverage,
   not additional gameplay checks. The runner changes checkpoint compatibility;
   regenerate from boot on this source. Preserve the old checked navigation build.
2. **Current - consolidate covered owned subsystems.** Name the existing corpus
   segments exercising each changed behavior; hold uncovered contexts.
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

Completed hygiene: [300eaf15](https://github.com/JimLundin/sm-decomp/commit/300eaf15)
reduces local library warnings 2,073 -> 482; full replay 148,406 frames / 0 failures,
55 rooms, no credits. PLAN/PROGRESS now use a short queue, graph and generated table.
Use `CARGO_INCREMENTAL=0` for comparable warning counts; do not auto-rename
macro-shared parameters.

Adopted player movement, transitions, settlement and minimap share owned Session
intervals; persistent whole-engine ownership and backend independence remain open.
The complete input draft `e956dffa` is held, not adopted. Preserve its checked
source/reference snapshots and audits under
`%LOCALAPPDATA%/sm-decomp-experiments/20261024/player-input-*`; do not rerun or
replace them without a source/coverage reason. Reprice/revalidate it against the
then-current base before adoption. Latest checked source: `5a4def1e`, shared actor
terrain movement (-18 production lines): 44 existing cases, all four movement
branches in Bomb Torizo (1,664 frames, zero failures), three compiling faults caught.
This is scoped adoption evidence; the full corpus remains unfinished.

## Implementation rules

- `sm/src` is the source of truth; no original C exists to consult or regenerate.
  Use the oracle, `ram_map.rs` and [bank logs](https://patrickjohnston.org/bank/)
  when meaning is unclear. Preserve defined behavior exactly, quirks included.
- Before changing boundaries, ownership or backend interfaces, read the accepted
  [architecture](target-architecture/README.md) and [migration](target-architecture/MIGRATION.md).
  Consult [engine design](ENGINE_DESIGN.md) and [RAM ownership](RAM_OWNERSHIP.md)
  for the changed interaction; [state migration](ENGINE_STATE_MIGRATION.md) governs
  reset/load, replay, CPU writes and oracle rewind/adoption.
- Keep `g_ram` byte-compatible with SNES layout. Preserve numeric dispatch/identity,
  ROM formats, content provenance, native/LE words, ordered effects, lazy reads,
  aliasing, suspension and failure prefixes.
  Keep descriptive gameplay names. A legacy interval has one authoritative state;
  never leave a mutable owned copy beside writable authoritative RAM.
- Move complete producer/consumer interactions into owned state before retiring
  their RAM communication. Completed cores forbid unsafe and need no Game, RAM,
  raw pointers or backend modules. Safe wrappers and package moves are transitional.
  Do not turn `word!` into a whole-RAM borrow while retained raw pointers remain;
  migrate offsets/handles and use checked, scoped access with overlapping reads intact.
- Plan a complete responsibility: public API, owner, setup/update/exceptional paths,
  callers and retirement. Adopt actual caller migration and legacy deletion together.
  Small preparation needs named consumers and an explicit deletion step; isolated
  shared consumers or forwarding facades are not subsystem completion.
- Reuse prior checks and fixtures; add verification only for a named uncovered
  behavior/ownership/order/failure contract. Do not grow a harness per subsystem.
  Implementation and independent review/check preparation may run in parallel
  on fixed interfaces and isolated files, feeding one integration/build lane.
- Count all production helpers, tables, adapters, registration, callers and
  maintained native code. Separate verification Rust, support/installers, external
  dependencies and combined maintained totals. Runtime adapters are production;
  mixed-file verification exclusions must be identified explicitly and reproducibly.
  Compare like-for-like classifications; path-only historical totals differ.
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
   Repair before reuse: `SETTLEMENT_EARLY_CONSUME` has a stale anchor;
   `room_export_before_binding_check` has an ambiguous anchor.

Commands and checkpoint rules: [movies/README.md](../movies/README.md).
When touched, inspect `cpu_infra.rs` masks, `PatchBugs`, hardware/audio limits and
persistence behavior. Do not weaken comparisons to make a replacement pass.

## Toolchain and Windows constraints

Use edition 2024, stable `1.99.1 (58cf8f9c3 2026-10-13)`, target
`x86_64-pc-windows-gnu`. The installed `nightly-2026-10-22` compiler remains blocked
by Defender ASR (event 1121); the nightly migration is unfinished.
Do not change security settings or spoof hosted environment variables. Standalone
test executables and denied compiler/build-script launches remain restricted.
Use permitted DLL/ctypes execution; denied launches never count as passes.
Cargo frontend linking is blocked by the khronos_api build script; cached-dependency
frontend metadata and the direct DLL path pass. Keep crate-root unsafe-op enforcement.

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

Applicable final-source checks pass and changed behavior executes with no new failing
frames. Run `sh tools/progress.sh`; explain every increase and report complete
production/verification/support accounting. Recheck HEAD/index, reconcile and commit
the focused change to main with commands, probes/frame ranges, results and limits.
Update this plan only when the active item, direction or blocker changes. Publish
PROGRESS with adopted milestones (or explicit user requests): the short graph plus
production/verification/unsafe/warning/checkpoint metrics, distinguishing held drafts.
Generate the table with `sh tools/progress.sh --markdown WARNINGS [PASSED/TOTAL]`;
use current measured values or leave unknowns explicit. The path split includes inline hooks.
Do not refresh atlas/architecture reports unless the user asks. Do not invent progress
percentages from files/nodes or claim unadopted savings.

Pre-transpile work is archived in `archive/pre-transpile-2026-10-17`; its
`.scratch/`, readiness gates, acceptance pointers and translation scopes are inactive.
