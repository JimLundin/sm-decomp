# Plan

Goal: Super Metroid as a readable, maintained Rust program that plays from
boot to credits, with emulated SNES hardware for video and audio. Map Rando
support comes after vanilla.

Git concurrency (user direction, 2026-10-19): agents must not manually
reserve or hold Git locks, including index.lock and branch ref locks.
Use Git's normal short-lived locks only for the actual Git write. Builds,
verification, reasoning, reviews and waits must run without a held lock.
Use isolated snapshots when needed; recheck HEAD and reconcile changes
before committing instead of preventing other agents from committing.
An agent holding a reservation must release its own reservation immediately,
preserving source changes and verification work. Do not delete another
process's lock; tell its owner to release it.

## Active direction (2026-10-20)

User direction: reduce maintained Rust below **100,000 lines**, preferably
further, by tackling large structural changes before individual cleanup.
Use **90,000 lines as a stretch planning checkpoint**, not a demonstrated
forecast. This work order replaces earlier reduction and cleanup queues.

Keep Game ownership, byte-compatible g_ram, ROM formats and observable
behavior, including quirks. sm/src is the source of truth; use ram_map.rs,
the oracle and https://patrickjohnston.org/bank/ when meaning is unclear.
The pre-transpile project remains archived and is not the work queue.

Design the complete replacement before splitting it into commits. A unit
of planning is a whole subsystem or behavior family, including setup,
updates, transitions, exceptional paths, callers and retirement of old code.
A commit still contains one reviewable idea; it may cover many files.

Internal function boundaries and redundant wrappers may change after checking
all callers and callback references. Preserve numeric dispatch behavior;
keep an old Rust entry point only where a real caller or identity contract
needs it. Use ordinary functions, enums and typed data where they simplify
the whole algorithm. Existing bank files can remain; a repository-wide
reorganization, universal state engine or new interpreter is not required.

Small preparatory changes, including temporary line-count increases, are
allowed when attached to a concrete replacement with named consumers and
an explicit deletion step. Explain counter increases in the commit. Judge
the completed replacement's net size, including every helper, table, wrapper
and source registration. Do not leave both implementations maintained.

## Baseline and diagnosis

At **ec45c573**, 72 tracked Rust files in sm/src contain **112,265 lines**.
Reaching 99,999 requires **12,266 net lines removed (10.9%)**; reaching
90,000 requires 22,265. Count physical lines with the existing progress
script, including comments, blanks, helpers and declarations. Removing
checks, moving code outside sm/src or compressing formatting is not progress
toward the intended goal.

These regions partition the baseline. Their size is search territory,
not a forecast of removable code. Bank names refer to sm/src/game/sm_XX.rs.

| Region | Banks / files | Lines |
| --- | --- | ---: |
| Rooms, menus, HUD, effects and cinematics | 80, 81, 82, 85, 87, 88, 89, 8F, 8B, 8D | 22,408 |
| Bosses and endgame | A4, A5, A6, A7, A9, AA, AD | 20,337 |
| Ordinary enemies and enemy projectiles | A2, A3, A8, B2, B3, waver, 86 | 20,308 |
| Samus and player projectiles | 90, 91, 92, 93, 94, 9B | 16,524 |
| Enemy engine and sprites | A0, B4 | 4,268 |
| PLMs | 84 | 3,490 |
| Shared gameplay declarations | constants, callback_tables, enemy_layouts, enemy_ram | 985 |
| Hardware, frontend, runtime and verification | Remaining sm/src files | 23,945 |
| **Total** | **72 files** | **112,265** |

Gameplay is **88,320 lines (78.7%)**. The first four regions contain
79,577 lines. Assess the complete regions, then prioritize actual savings
and available coverage; largest file does not necessarily mean best cut.

The previous large-family reset, 51c0e735..ec45c573, produced 42 commits:
24 reductions, one verifier change and 17 documentation-only commits.
Net Rust deletion was 4,286 lines; median reduction was 109 lines, and the
latest ten reduction commits removed 1,043 combined. PLAN grew 1,518 lines
to 28,306 lines / 1.86 MB. These are Git-history counts, not labor timings.
The loop was rewarding completed small edits and extensive reporting while
the overall replacement design remained unpriced.

Larger landed changes demonstrate the useful scale: shared enemy layouts
removed 1,696 lines and ordered word transfers removed 1,021. Those savings
are already in the baseline. Repeated clone scans and signature macros now
have diminishing measured returns. The next experiment must change how a
complete behavior is represented, rather than repeat those searches.

## Work order

1. [x] **Price a complete Samus movement/pose replacement.** Start with
   Samus_MovementHandler_Normal and its movement table in 90, then follow
   Samus_InputHandler, Samus_HandleTransitions and block-collision transitions
   in 91 into the movement/collision routines in 94. Include state entry,
   per-frame movement, pose selection and reset paths. Follow grapple and
   projectile interactions where the actual callers require them. This is
   wider than the parked 22-entry jump/movement draft described below.

   Compare two concrete designs: consolidated domain functions with explicit
   state-specific tails, and shared algorithms selected by ordinary typed
   movement profiles. Keep liquid/suit rules, collision calls, captured/live
   RAM reads and transition order explicit. Avoid a flag combination for
   every old function or a table that merely transcribes all old statements.
   Pilot the designs on related jumping/falling/ball states plus an exceptional
   path before expanding the better design to every eligible consumer.

   Finish this item with exact replacement spans, retained exceptions, complete
   formatted and compiled old/new counts, required checks and a decision to
   implement or reject. Target a replacement saving roughly 1,000+ lines;
   this is an experiment objective, not a predicted saving or a per-commit
   minimum. A pilot alone does not establish the full replacement's savings.
   A worthwhile measured candidate goes directly to item 3; otherwise continue
   discovery in item 2. Repeat that choice after each completed replacement.
   If it only yields another small extraction, record the reason in one short
   paragraph and advance to item 2. Do not spend an iteration landing its
   incidental small cleanups. If stuck for an hour, record findings and stop.

   Pilot decision: reject this representation and advance to item 2. Consolidated
   domain functions remove **77 formatted lines**; ordinary typed profiles remove
   **71**, including all helpers, declarations and callers. Both compile across
   all frontend targets. The pilots cover all 28 input states and the morph/spring
   ball entry, grounded movement, landing, bounce and equipment-refresh lifecycle.
   Jump/fall/spin, grapple, knockback and bank-94 collision paths retain their
   existing algorithms and phase boundaries. No full replacement or 1,000-line
   budget was established; no oracle execution or landed savings are claimed.
   Drafts and exact counts: TEMP sm-bindings-root-9c4daf2f/samus-cycle/REPORT.md.

2. [ ] **Compare the remaining large subsystem replacements.** Inspect in
   this order, using the same complete-replacement test as item 1:

   - Enemy/projectile lifecycles across A2/A3/A8/B2/B3/86: initialization,
     state selection, movement, collision, animation and reset together.
     Classify real differences before choosing shared algorithms and data.
   - Presentation sequences across 80/81/82/88/8B/8D: scene setup, ordered
     drawing/transfers, waits and transition decisions together. Existing
     copy/queue helpers are inputs to the design, not new savings.
   - Boss phase families in A4/A5/A6/A7/A9/AA/AD, following their projectiles
     into 86: prioritize a complete phase with obtainable oracle coverage.
     Unvisited endgame behavior has an explicit coverage dependency.

   Rank complete candidates by net deletion, implementation/check cost and
   coverage availability. Examine broader opportunities before spending time
   on lower-volume engine, PLM, frontend or hardware refinements. Small edits
   may serve a selected replacement; independent cleanup waits until item 4.
   Reopen a parked study only when the proposed representation or scope is
   materially different, and state that difference before starting.

3. [ ] **Implement the strongest measured replacements and recount.** Start
   a proven candidate as soon as it is ready; an exact budget for the entire
   remaining gap is not a prerequisite for useful implementation. At each
   completed replacement, record baseline -> result, net deletion including
   shared code, checks, and the remaining gap. Count overlapping spans once.
   Keep measured drafts separate from landed savings; the currently proven
   remaining budget is **zero**, not 12,266. Update it as drafts are completed.

   Keep one active implementation per overlapping subsystem. Use a stable
   source snapshot and checkout-local build cache for its checks. Design
   globally and land coherent stages; do not force every individual call site
   through a separate build/replay/commit cycle. Follow the validation strategy
   below, including its full-replay cadence. Continue choosing the strongest
   complete replacement until below 100,000; then reassess the path to 90,000.

4. [ ] **Refine the smaller implementation.** Resume local naming, ownership,
   unsafe removal and small duplication cleanup after the volume pass.
   Boot-to-credits coverage, remaining native Bang AI ownership, ROM bug-patch
   review and Map Rando remain later project milestones. Correctness defects
   and coverage needed by an active replacement take priority when encountered.

## Previously measured work

Use these results to avoid repeating discovery. Draft figures are historical,
may overlap landed work and are not a current reduction budget.

- Whole-function templates: the latest reconciliation retained 29 groups /
  71 functions for only 185 net lines; generic representation remains parked.
- Named repeated-algorithm extraction: 713 lines already landed. Earlier
  548/808-line estimates and the rejected 1,227-line draft overlap it.
- Hopping cycles: 31 entries / 549 old lines replaced with 259, saving 290;
  the selected Sidehopper/Beetom/Thin Hopping Blob subset is parked.
- Samus jump/movement subset: 22 entries, 152 net lines; parked. It did not
  redesign the joint movement/pose/transition pipeline in work-order item 1.
- Script execution: 22 selected loop spans contain only 578 gross lines and
  have differing cursor, suspension and live-slot contracts. A universal
  fetch/execute engine has no demonstrated large payoff.
- Promoted wrapping arithmetic: 620 sites grew 576 lines. Fractional-position
  kernels saved 29. Current-enemy accessors, terminal/indexed-loop rewrites,
  cinematic initializer rows and remaining generic arithmetic are parked.
- Residual micro-families (queues, palette/homing kernels, HUD/menu rows,
  bitwise word updates, typed ROM records) price in tens to low hundreds of
  lines. They do not independently justify a new volume-reduction workstream.
- Shared layouts, RAM/word/byte access, callback declarations, dispatch,
  exact aliases, graphics/word transfers, corpse tile traversal and audited
  unused-code removal are already harvested. Count only further net deletion.
- A revalidated unused-declaration draft removes 336 formatted lines: 587
  import bindings and five unreferenced Bang helpers. Static/compiler/release
  checks pass; hold it for item 4, outside the active volume budget. Local draft:
  TEMP sm-bindings-root-9c4daf2f/unused/.
- Renderer unification and PNG library substitution remain parked: previously
  assessed alternatives did not preserve the existing contracts.

When revisiting one of these exact families, consult its historical section
with `git show ec45c573:docs/PLAN.md` and search the relevant heading/symbol.
Temporary draft locations recorded there may no longer exist. Recheck current
source before using any old draft. Earlier "next" instructions are historical.

## Validation strategy (2026-10-20)

User direction: choose verification by the changed behavior and its callers,
not the size of the movie or a habit of running any% after every edit. This
section replaces earlier validation rules and historical per-commit recipes.

1. **Select before running.** Identify changed routines, relevant branches,
   callers, shared data and downstream effects. Choose the smallest scenario
   that exercises them. Documentation/comment/format-only edits need applicable
   static checks; removing demonstrably unused private code needs a reference
   search and compiler checks. Neither needs a gameplay replay. Frontend-only
   changes need frontend checks, since the headless movie does not call the UI.
2. **Focused behavior changes.** Run formatting, a release build and the shortest
   exercised oracle segment on the final source. Require explicit routine and
   changed-branch probes with `--require-hit`; a missing hit fails verification.
   Use `--frames` for a known endpoint, or `--after-hits N` to stop after every
   required probe first executes plus N follow-up frames. Choose N to observe
   the actual effects, including subsequent callbacks, frames, transitions or
   persistent writes. Require a probe at the later effect too when available.
   Entry hits alone do not prove branch coverage or correct behavior. Keep all
   normal oracle comparisons, rendering and audio advancement enabled.
3. **Skip only a justified prefix.** A compatible local checkpoint must precede
   affected setup and calls. Inspect earlier callers and shared-state effects
   before reusing it after an edit: runtime compatibility alone does not prove
   the changed build reaches that state. Otherwise start from boot, with a
   bounded endpoint when that covers the change. An uncovered routine needs
   another route or a focused component/oracle test of its real implementation;
   repeatedly running an unrelated green movie adds no coverage. Constructed
   tests must state their limits and do not establish movie execution. Choose
   arithmetic/aliasing cases for plausible failure modes rather than defaulting
   to large unrelated sweeps. If relevant context remains uncovered, report it
   and hold further cleanup of that behavior.
4. **Broader regression checks.** Initialization, shared RAM access, broadly used
   dispatch, runtime, hardware, audio, oracle and checkpoint changes need full
   boot replay before commit when their effects span the route or are uncertain.
   Run full any% and the broadest relevant route at subsystem-batch completion,
   before publishing an integration, and after at most five focused behavior
   commits or 30 minutes of accumulated behavior changes since the last full
   check. Static-only edits do not trigger that cadence. Build once and reuse
   it; skip shorter fixtures that are exact prefixes unless their endpoint
   assertion differs. Full runs check interactions and history; they still need
   changed-path checks when the movies miss the edited code.

Record selected scenarios, required hits, executed frame range and result in
the commit message, with the reason for a full replay or static-only check.
Use source-checked builds and `sh tools/progress.sh` as usual. No new test
registry, evidence store or coverage dashboard is needed. Commands and
checkpoint constraints are in [movies/README.md](../movies/README.md).

## The oracle

Latest full subsystem integration: af8c1745 passed any% **129712/0** and
Speed Booster **148406/0**, with normal rendering, audio and comparisons.
2941c9a9 subsequently passed the exercised HUD boot segment **48512/0**,
8,544 component cases and 8,064 oracle calls. The later ec45c573 snapshot
field sharing passed static compiler-expansion equivalence and release checks;
no later full replay is claimed here.

The any% input visits six rooms; Speed Booster visits 55 rooms and 53 enemy
definitions. Neither reaches credits. Kraid/Varia are reached by the extended
route; later bosses/endgame and all branches still need appropriate coverage.
The local Charge Beam route includes Morph Ball eye-beam inputs 28200..28550,
as used in movies/README.md. A green movie does not cover unexecuted behavior.
Existing RAM masks, patched-ROM behavior, hardware/audio limits and persistence
coverage still matter; inspect current cpu_infra.rs comparisons and PatchBugs
when a change touches them. Preserve checks and disclose their actual limits.

The Windows launcher checks source bytes embedded in the compiled library;
register new library source files in sm/src/verify_source.rs. Keep source and
inputs fixed through checks. It rejects stale artifacts and source changes.
Checkpoints must precede affected history; required probes and --after-hits
select exercised segments without disabling comparisons. Commands and caveats
are in [movies/README.md](../movies/README.md).

## Definition of done for any change

1. Applicable checks from the validation strategy pass on the final source.
   Documentation-only changes need static checks; code changes need formatting
   and appropriate compiler checks, with release builds for behavior changes.
2. Changed behavior executes in the selected checks, with no new failing frames.
   Static equivalence must establish unchanged operations; an unexercised green
   replay or source inspection alone does not validate a behavior rewrite.
3. Run `sh tools/progress.sh`; explain counter increases. Report complete
   replacement savings and remaining budget only when actually measured.
4. Recheck HEAD and other work before committing to main. Commit one coherent
   idea with the verify result. Update this plan's active item when status or
   direction changes; routine check details belong in the commit message.

## Workspace discipline

Keep this file an active plan, not a per-commit journal. Replace current status
in place. Historical results and stopped experiments remain in Git; consult
only the relevant section when needed. Do not add evidence stores, dashboards,
coverage registries or new process tooling.

Performance work targets the verification pipeline; readability takes priority
in maintained gameplay and hardware. Reuse the normal build and select checks
by changed behavior. Measure edit/build/result time before optimizing another
verifier operation; existing release builds are around 20 seconds and the
recent full replay pair about 4.4 minutes, dependent on source and load.

Coordinate heavy work with existing owners. Keep game/host artifacts local to
the checkout; seed registry dependencies only, not whole target directories.
Use CARGO_INCREMENTAL=0 for disposable snapshots. Remove disposable artifacts
only after checking they are no longer needed and resolving paths safely.
Preserve unrelated source changes and active build caches. Never commit ROMs,
movies, save states or other ROM-derived files.

## Progress

Baseline at ec45c573: raw g_ram offsets 30; extern C blocks 3; counted C ABI
exports 4; no_mangle 8; unsafe functions 5307; i32/c_int cast lines 8848;
Rust lines 112265. The working tree at this review also contains an unrelated
83-line shader reduction, so `sh tools/progress.sh` reports **112182**. That
uncommitted draft is excluded from the committed baseline and budget.

The Samus pilot decision changes documentation only. Both formatted pilots pass
all-target release compiler checks; no gameplay validation is claimed. Diff and
progress checks pass. Maintained Rust and its counters remain unchanged; the
active work is now item 2, beginning with enemy/projectile lifecycles.

Historical details before this reset remain in **ec45c573:docs/PLAN.md**.
