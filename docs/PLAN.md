# Plan

Goal: a compact Super Metroid game engine with descriptive gameplay routines,
playing from boot to credits. SNES emulation remains the current video/audio
backend and verification oracle; it is not the long-term engine architecture.
Map Rando support comes after vanilla.

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

The research-led structural/variant audit has now covered same-file and cross-file
function bodies, callback rows and compound blocks within large routines. It
selected the opcode-grid consolidation below, but established no new 500-line
net candidate. [REDUCTION_RESEARCH.md](REDUCTION_RESEARCH.md) supplies the method
background; the bounded follow-up findings are in item 2. Do not repeat these
detector passes without a materially new scope. Detector hits are not savings.

The requested parallel experiment on the original 1311d3b4 transpilation is
complete: combined tool-assisted replacements remove **49,687 lines
(278,936 -> 229,249)** after formatting, including all costs and overlap.
Optimized compilation, structural correspondence and focused routine checks
pass; no full-game/SNES replay is claimed. The compared current snapshot
already shares these implementations, so this result supplies no new main
deletion budget and does not establish a benefit from restarting. Details and
isolated patches are in the research note's original-transpilation experiment.

Latest user clarification: code-volume reduction is the immediate priority.
Focus on repeated code patterns; broad system redesign is too costly unless
it directly removes substantial repetition. Investigate efficient methods in
parallel and price the complete repeated group before expanding implementation.
Re-architecture, shared lifecycle engines and declarative behavior sequences
are justified when they replace substantial implementation. Price complete
families, including their replacement costs, before runtime checks. Previous
module-move experiments do not test the payoff of changing the algorithms or
their representation. Do not use their small savings as a lower-bound claim.

User clarification: the long-term structure is descriptive gameplay modules
and routines, with **bank-based source organization removed**. Keep bank
addresses as ROM/dispatch data. Domain ownership and code-volume reduction
are separate outcomes to measure; a file move alone supplies no deletion.

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
the whole algorithm. Migrate one complete responsibility at a time, redirect
real callers, and remove its old bank implementation. Remaining banks are
temporary homes for unmigrated responsibilities. A shared engine must serve
concrete repeated behavior and reduce the complete maintained implementation;
an interpreter or data table is neither automatically disallowed nor a saving
by itself. Keep meaningful gameplay names and avoid permanent bank facades.

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

## Domain experiments (2026-10-20)

Five independent compiled experiments start from 7d1af86e, whose maintained
Rust matches ec45c573. Counts include all new modules, imports, callers and
source registrations, after formatting. These are alternative drafts, not
landed savings; the three Samus alternatives overlap and cannot be added.

| Experiment | Complete measured scope | Whole sm/src | Net change |
| --- | --- | ---: | ---: |
| Samus motion/pose modules | 3,167 moved lines; input, movement, jump setup and pose progression | 112,237 | -28 |
| Typed Samus profiles | 14 air/ball routines and macros, including ground/landing/reset | 112,195 | -70 |
| Samus frame phases | 2,355 moved lines; input/frame/pose routing and real callers | 112,226 | -39 |
| Roach lifecycle | 33 functions plus two forwarding rows; all 17 phases | 112,236 | -29 |
| Menus, sprites and save data | Entire sm_81 removed; 120 ordinary routines moved | 112,306 | +41 |

Relocation controls grow by 65 lines for motion/pose, 42 for frame phases and
23 for Roach. Consolidation recovers that overhead but produces small net
reductions. No experiment establishes a 1,000-line replacement or a route
covering the 12,266-line gap. These results reject that forecast for the tested
designs; they do not prove that larger reductions elsewhere are impossible.

The useful architectural result is explicit ownership. Prefer ordinary named
domain routines; the profile pilot concentrates several exceptional branches
for only 70 lines. Samus needs separate prepare_frame and finish_frame seams:
enemy/projectile work and a possible message-box suspension occur between
them. Retain out-of-band grapple, demo, X-ray and cinematic entry points where
real callers need them. Roach has a true initialize/update interface. Menus
has three update entries, with private state handlers. The menu experiment
retires a complete bank; the Samus experiments still depend on unmigrated
physics/collision code and do not retire all Samus banks.

All drafts pass formatting and core compiler checks. The motion/pose,
frame-phase and bank-removal drafts each pass clean, source-checked optimized
any% **129712/0** and Speed Booster **148406/0** replays. Motion/pose's seven
required probes cover normal input/movement/pose, both spin branches and both
grounded-ball branches; frame phases have five exercised normal-control probes.
Demo, X-ray, grapple, Spring Ball and unusual transition coverage remains
limited to the stated static checks where those paths were not observed.

The bank-removal draft preserves all 120 ordinary function bodies under its
explicit name/visibility mapping and passes frontend-library metadata checks.
Its probes exercise menus, sprites and SRAM loading/saving; map selection has
zero hits, correctly failing that requested coverage requirement. The clean
full replays and source correspondence do not establish map-branch execution.
Fresh frontend Cargo dependency build scripts hit Windows OS error 5; the
gameplay checks use the existing optimized core/DLL verifier instead.

Roach passes **14,172 full-RAM comparisons** between actual optimized original/
replacement cores, including 21 persistent 600-update sequences. This is
constructed Rust differential coverage, not SNES-oracle or movie execution.
Typed profiles pass an exercised Speed Booster segment **46360/0**, covering
all five motion entries plus 120 follow-up frames; Spring Ball landing and
other exceptional branches remain unproven. None of the routes reaches credits.

Runnable draft roots under TEMP, in table order: sm-samus-minimal-clean-20261020,
sm-samus-profiles-20261020, sm-samus-common-20261020, sm-roach-domain-1cb6fb50,
and sm-domain-menu-7d1af86e/candidate. The menu experiment's parent directory
contains REPORT.md and trial.patch; the patch applies to main without changing
the unrelated shader work. Experiments remain outside maintained source.

Decision: extend domain ownership across coherent subsystems for readability,
while pricing actual algorithm deletion separately. Resolve each draft's
remaining coverage and dependency surface before landing it. Keep these
experiments parked as volume workstreams; use their domain design when the
relevant subsystem is migrated. Continue the independent large-reduction
comparison below without extrapolating savings from these pilots.

These domain moves remain parked during the immediate volume pass. Preserve
their useful phase/dependency findings when a replacement needs them, but
prioritize deleting repeated implementation over adopting relocation controls.

Four subsequent experiments at 7617b682 change the representation itself.
Their formatted net savings include replacement code, callers and source
registration; only moving platforms is adopted below.

| Replacement | Net deletion | Validation and disposition |
| --- | ---: | --- |
| Moving platforms and shutters: shared triggers, travel, waits and growth stages | 192 | 16,414 full-RAM comparisons; final integration below |
| Mother Brain: 124 typed encounter steps, shared delay gates and continuations | 318 | 2,208 comparisons cover 69 steps; park pending remaining coverage and independent module costs |
| Crawlers and Snail: shared surface movement and native lifecycle routing | 144 | 6,528 full-RAM comparisons; park before movie integration |
| Configuration/input: native collections and shared option parsing | 165 | Frontend-library compilation only; park before runtime validation |

Component comparisons use actual optimized original/replacement Rust, not a
SNES oracle. Mother Brain remains a child of sm_a9, so its saving does not price
complete bank removal; 55 steps and real encounter history remain untested.
The experiments establish useful local deletion, not a forecast for the gap.
Price complete repeated algorithms before spending on broad runtime checks;
continue discovery beyond these scopes instead of multiplying their savings.
Drafts and reports remain under TEMP: sm-mechanism-lifecycle-7617b682,
sm-motherbrain-sequence-7617b682, sm-native-crawlers-7617b682 and
sm-config-engine-7617b682. Unlanded drafts contribute nothing to the budget.

## Work order

1. [x] **Test Samus domain alternatives.** The concrete domain experiments
   above reject the proposed large volume payoff and establish useful named
   interfaces. Keep measured scope and unvisited behavior explicit. Subsequent
   candidates need complete formatted/compiled replacement counts, including
   retained exceptions and caller costs, before implementation expands. If a
   candidate produces only a small extraction, record its result and advance
   discovery; if stuck for an hour, record findings and stop.

   Pilot decision: reject this representation and advance to item 2. Consolidated
   domain functions remove **77 formatted lines**; ordinary typed profiles remove
   **71**, including all helpers, declarations and callers. Both compile across
   all frontend targets. The pilots cover all 28 input states and the morph/spring
   ball entry, grounded movement, landing, bounce and equipment-refresh lifecycle.
   Jump/fall/spin, grapple, knockback and bank-94 collision paths retain their
   existing algorithms and phase boundaries. No full replacement or 1,000-line
   budget was established; no oracle execution or landed savings are claimed.
   Drafts and exact counts: TEMP sm-bindings-root-9c4daf2f/samus-cycle/REPORT.md.

2. [ ] **Compare the remaining large subsystem replacements.** The research-led
   follow-up uses existing AST/Pygments/Copydetect tools on 2,393 function and
   callback bodies, normalizing names and numbers. Sparse comparison took 8.6s
   and included same-file and cross-file near-miss islands. A separate 53.8s
   pass covers 2,995 compound fragments inside 362 large routines, excluding
   ancestor overlaps and already-studied function pairs. Neither establishes a
   new 500-line net family; the strongest complete draft is the adopted 184-line
   opcode grid. These are bounded findings, not proof that all sharing is done.
   Reports: TEMP sm-near-miss-functions-f57e4abf,
   sm-inner-variants-c98e8439 and sm-gameplay-variant-audit-f57e4abf.
   Require a materially new named algorithm
   or scope before repeating discovery. The regions and prior results below
   guide coverage, not boundaries that candidate replacements must preserve:

   Enemy/projectile preflight found no credible large replacement. Complete
   Metalee/Skree bodies occupy 252 lines; B2 plus laser/claw bodies occupy about
   996 before retaining distinct wall, ninja and walking behavior. Rio, linked
   flyers, crawlers, pipe bugs, hoppers and ordinary projectiles retain distinct
   lifecycle contracts after existing sharing. No draft or saving is claimed.

   Presentation preflight likewise found no candidate to price across title/
   cinematic exits, file-map and pause flows, Power Bomb/Crystal Flash, suit
   pickups and door presentation. Shared geometry, drawing, transfer and beam
   phases are already extracted; remaining ordered state effects differ.
   A broad scene profile would transcribe arbitrary callbacks and statements.
   Both bounded studies advance discovery without claiming exhaustive limits.

   Boss preflight found no credible roughly 1,000-line replacement across the
   examined Torizo, Crocomire, Ridley, Spore Spawn and Mother Brain phases and
   their projectile consumers. Existing algorithms are already shared; the
   remaining phase contracts differ. This is bounded evidence, not exhaustion.

   The intro narrative-page replacement qualifies for item 3: `intro_pages`
   owns page markers/input, four scene setups, both crossfade directions,
   Japanese continuations and the handoff to Ceres. The separate Japanese
   accumulator correction landed first; its helper is retired into the module.
   Engine primitives and scene sprite motion remain in their existing homes.
   Continue discovery from the remaining responsibilities, using these bounded
   preflights and measured drafts instead of repeating the same searches.

   Complete hopping-enemy repricing at b64a0470 includes initialization,
   routing, animation, movement and contact handling for Sidehopper, Beetom
   and Thin Hopping Blobs. Relocation costs 26 lines; the full replacement
   removes only **44 net lines**, including all imports and registrations.
   It formats and compiles across all targets, with no new runtime coverage.
   Park this volume candidate and advance discovery. The earlier covered
   Sidehopper/Beetom sharing is already in the baseline, so its historical
   projection cannot be counted again. Draft: TEMP
   sm-bindings-root-9c4daf2f/hopping-lifecycle/REPORT.md.

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
   may serve a selected replacement. Complete repeated-pattern cohorts can
   proceed through item 3; isolated naming/unsafe cleanup waits until item 4.
   Reopen a parked study only when the proposed representation or scope is
   materially different, and state that difference before starting.

3. [ ] **Implement the strongest measured replacements and recount.** Start
   a proven candidate as soon as it is ready; an exact budget for the entire
   remaining gap is not a prerequisite for useful implementation. At each
   completed replacement, record baseline -> result, net deletion including
   shared code, checks, and the remaining gap. Count overlapping spans once.
   Keep measured drafts separate from landed savings. No selected large
   replacement yet funds the **8,891-line** gap; parked small drafts are not
   an aggregate reduction forecast. Update the budget as candidates qualify.

   Complete local-binding cohort: **207 bindings across 31 files**, placing
   declarations at their first value or binding an exhaustive branch result,
   removes **192 lines (112,261 -> 112,069)** with no new helpers. Scalar/raw
   pointer types, calculations, branch exits and effect order are preserved.
   Source reconstruction and independent scope/macro review, headless compiler,
   normal release and compiled-source checks pass. Static-only validation;
   no new movie result is claimed. This is useful landed repetition cleanup,
   but does not establish the desired multi-thousand-line replacement.

   Complete intro-page ownership removes **233 lines (112,069 -> 111,836)**:
   605 bank lines retired, a 367-line descriptive module, and five import/source
   registration lines. All 47 numeric callback entries retain their addresses.
   English/Japanese routes exercise the page-to-Ceres transition; focused oracle
   cases cover slot exhaustion and idle/delayed input. Full any% and Speed
   Booster pass; final unused-import retirement has static/compiler validation.
   This is a completed family replacement, not a forecast of the remaining gap.
   Routine results are in its commit.

   Enemy callback references now use their owning modules directly, retiring
   1,020 duplicate private imports. The complete cohort removes **159 lines
   (111,836 -> 111,677)**, including macro support and table wrapping at the
   existing 100-column width. Source reconstruction and compiler IR establish
   unchanged routing and operations; release and all-target checks pass.
   This removes declaration repetition, without changing gameplay ownership.

   Ordered named WRAM stores share the existing word macro across 97 runs /
   312 stores, removing **263 lines (111,677 -> 111,414)** including its helper
   and documentation. Independent reconstruction and whole-library compiler
   expansion establish identical executable operations and store order. Normal
   release and compiled-source checks pass; this is static-only validation.

   Complete PNG texture loading removes **1,119 lines (111,414 -> 110,295)**:
   a shared filter/unpack/pass loop, chunk parser and buffered input replace
   repeated decoding paths; the exact legacy inflater remains maintained.
   Counts include both modules, imports, caller and source registrations; no
   dependencies are added. All 385 frozen cases match through decoding and file
   loading (770 comparisons), including unusual formats and malformed-stream
   quirks. Independent input-operation review, normal release and compiled-source
   checks pass. This frontend-only change needs no gameplay replay.

   Complete beam-firing ownership removes **65 lines (110,295 -> 110,230)**:
   charge input, reservation, uncharged/charged/hyper firing and flare reset
   share a 191-line module, with the old entry points retired. All callers,
   imports and source registrations are included. Focused SNES comparisons
   cover exceptional firing and subsequent projectile calls; full any% and
   Speed Booster pass on the clean release. Hyper coverage is constructed,
   not an endgame movie. This domain migration does not fund the remaining gap.

   Complete moving-platform/shutter ownership removes **192 lines
   (110,230 -> 110,038)**: 650 old lines become a 448-line module plus ten
   caller/registration lines. Named phases share triggers, travel, waits and
   timed growth while preserving carrying, pushing and shot differences.
   All 16,414 constructed full-RAM comparisons pass, including persistent
   updates and wrapping boundaries. Full integration and observed branch
   limits are below. This retires about 30% of the old family, not a full bank.

   The complete byte-partial-product cohort removes **71 lines
   (110,038 -> 109,967)**: six decomposed products become native multiplication,
   and two duplicate sine-scaling helpers become one with eight callers.
   Explicit magnitude/sign, selected product byte and low-amplitude truncation
   preserve the original arithmetic. Independent actual-function/RAM checks,
   normal release and full any%/Speed Booster integration pass. This is a
   completed repeated algorithm, with all helper/import/caller costs included.

   INI section/value decoding removes **65 lines (109,967 -> 109,902)**:
   section lookup, boolean suffix validation and keyboard/gamepad group lookup
   share their repeated algorithms. All 354,060 frontend cases pass against
   the original decoder, including normal-library file loading, invalid values,
   aliasing outputs and the accepted control-byte booleans. Release, formatting
   and all-target checks pass. This frontend-only change needs no movie replay.

   Complete countdown-callback sharing removes **198 lines
   (109,902 -> 109,704)** across 97 callbacks in eight banks, including the
   shared declaration helper. Exact setup, capture, expiry, effects and tails
   remain explicit; callback names and routes are unchanged. Independent
   compiler expansion matches all 97 bodies after bound-local renaming;
   outside selected spans and macro resolution remain unchanged. Formatting,
   normal release and compiled-source checks pass. This is static equivalence,
   with no new movie result. Lower unsafe/cast counters reflect shared source
   declarations, not removal of unsafe runtime operations.

   Complete beam/missile flight ownership removes **148 lines
   (109,704 -> 109,556)**, including the 331-line module and every caller/import.
   Shared direction routing, trail timing, acceleration and launch axes retire
   28 bank entry points while preserving all 35 numeric dispatch addresses.
   All 51,791 focused SNES comparisons pass, including 180 sustained-flight
   scenarios and inherited launch motion. All 15 requested component probes
   execute; focused movies and clean full integration pass as recorded below.
   The remaining projectile instruction, drawing and firing owners are separate.

   Complete callback entry-capture sharing removes **482 lines
   (109,556 -> 109,074)** across 754 callbacks / 67 groups, including seven
   shared support lines. Independent source reconstruction and compiler
   expansion preserve every selected callback and all 7,006 current named
   definitions exactly, with no binding renames. Formatting, normal release
   and compiled-source checks pass. This is static equivalence, with no new
   movie result. Lower unsafe/cast counters reflect shared declarations;
   no unsafe runtime operations disappear. Continue the structural/variant
   audit rather than repeating the completed prefix/signature census.

   Complete CPU/SPC opcode-grid consolidation removes **184 lines
   (109,074 -> 108,890)** by decoding repeated operation/address combinations.
   All 512 opcode selections independently preserve helper arguments, order
   and fallback behavior; all 134 existing helpers remain unchanged. The 8,192
   compiled selector-boundary traces pass, as do release/source checks and full
   any%/Speed Booster replays. The two added unsafe functions are shared
   selectors; all their costs are included. These checks preserve existing
   arithmetic/bus operations without claiming every opcode executes in movies.

   Keep one active implementation per overlapping subsystem. Use a stable
   source snapshot and checkout-local build cache for its checks. Design
   globally and land coherent stages; do not force every individual call site
   through a separate build/replay/commit cycle. Follow the validation strategy
   below, including its full-replay cadence. Continue choosing the strongest
   complete replacement until below 100,000; then reassess the path to 90,000.

4. [ ] **Refine the smaller implementation.** Resume isolated local naming,
   unsafe removal and small duplication cleanup after the volume pass. Whole
   domain migrations follow the user clarification above and can precede it.
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
- Hopping cycles: the historical 290-line projection overlaps the covered
  Sidehopper/Beetom sharing landed in 7a9e0122. The remaining Blob animation,
  Beetom descent and release changes are unlanded; the complete current
  lifecycle draft above saves 44 lines, not 290.
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
- A chained VRAM-write cursor stores each argument immediately, preserving
  later expressions' access to earlier writes. Across 43 retained-pointer
  entries / 42 functions it grows **63 formatted lines**, including its helper.
  Reject this representation; wrapped/mixed writers and the consumer remain
  outside its measured scope. No compilation or runtime claim. Draft/report:
  TEMP sm-bindings-root-9c4daf2f/vram-cursors/REPORT.md.
- Shared layouts, RAM/word/byte access, callback declarations, dispatch,
  exact aliases, graphics/word transfers, corpse tile traversal and audited
  unused-code removal are already harvested. Count only further net deletion.
- A revalidated unused-declaration draft removes 336 formatted lines: 587
  import bindings and five unreferenced Bang helpers. Static/compiler/release
  checks pass; hold it for item 4, outside the active volume budget. Local draft:
  TEMP sm-bindings-root-9c4daf2f/unused/.
- Grapple angular sweep: a 126-line draft passes 6,574 original/replacement
  comparisons and 6,574 oracle cases plus 254 follow-up calls. It remains
  unlanded and parked as a standalone fragment; no movie execution is claimed.
- Renderer unification remains parked: assessed alternatives did not preserve
  the existing contracts. PNG inflater substitution is rejected by concrete
  EOF/reserved-code counterexamples; the native decoder replacement above
  retains that inflater and removes only the surrounding repeated algorithms.
- Single-use local inlining: 318 order-safe scalar candidates across 38 files
  grow by 86 formatted lines with ordinary inference, or 181 with explicit
  type constraints. Reject both; no compilation or behavior claim. Branch
  prefix/middle sharing across 72 files exposes only 72 gross repeated lines;
  residual audio families have no credible 150-line net budget. These bounded
  studies do not justify repeating the same scans. TEMP reports/drafts:
  sm-single-use-8d654c29, sm-branch-prefixes-8d654c29,
  sm-audio-repetition-8d654c29 and sm-dead-initializers-8d654c29.

Further syntax preflight at b64a0470 found no substantial unpriced cohort:
844 non-shader direct wrapping assignments occupy 963 lines; 329 adjacent
computed-result/store pairs occupy 844 gross lines before replacement costs.
Pointer, timer, null-guard and string-cast envelopes provide no credible
thousand-line replacement. These bounds do not exhaust algorithmic sharing.

Further complete-cohort pricing at 7617b682 excludes these repeats:

- Index-only Eproj/PLM/HDMA slot methods: 958 replacements, 253 locals and
  86 methods across nine callers add **668 formatted lines**. This differs
  from accessor-declaration drafts but also fails the volume test.
- Field-aware word arithmetic: 135 expressions across 16 banks remove only
  **39 net lines**, including helper/documentation costs. Park this notation.
- PPU kernel sharing is already landed. The surveyed 1,417 gross function
  lines include existing shared kernels and required renderer differences;
  they are not removable duplication.
- Remaining callback signatures expose 313 gross header lines and at most
  **255 net lines** after optimistic minimum grouping costs, before real
  formatting and ordering costs. This cohort has no large volume payoff.
- Multibox collision bodies occupy 219 lines and normal counterparts 117;
  traversal, count widths, edge rules, gating and effect order differ.
  Residual comparable PLM/scrolling tile loops total roughly 60 lines after
  existing tile sharing. Neither preflight establishes a large replacement.

These studies changed no maintained source and ran no runtime checks.
Drafts/reports: TEMP sm-slot-index-price-7617b682,
sm-word-arithmetic-7617b682, sm-ppu-kernels-7617b682 and
sm-signature-headers-7617b682. Continue discovery beyond these bounded
cohorts; they do not exhaust possible algorithmic reductions.

Countdown pricing selected the landed callback form over runtime stage enums
(-11 boss/presentation, +34 ordinary-enemy lines) and operation-level gates
(-173 before any additional caller scope). The bounded census found 374
countdown-containing functions / 9,364 gross lines, not removable duplication.
Snapshot capture/restore mapping removes only 84 lines including its macros;
park this uncompiled draft, with no runtime claim. Reports: TEMP
sm-timed-transitions-63b0ffe4, sm-enemy-programs-63b0ffe4 and
sm-snapshot-map-price-f698eb50.

The complete Torizo ROM-tuple projectile initialization draft covers four
variants and removes only 42 lines at c98e8439, including the shared facing/origin
selection. Park it without builds or runtime claims. Whole-initializer and
macro-aware audits found no credible 500-line group. Draft/report: TEMP
sm-eproj-init-study-c98e8439. A separate word-sign predicate census finds 1,512
predicates on 1,655 occupied lines: only 143 continuation lines before any
replacement costs. This does not justify another global arithmetic rewrite.

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

Latest full integration: opcode grids over c98e8439 pass any% **129712/0** and
Speed Booster **148406/0**, with normal rendering, audio and comparisons.
Independent complete selector correspondence and compiled boundary traces
cover all opcode values; these are not claims of every opcode's movie execution.
Earlier projectile flight over c1ebc4ca passes
any% **129712/0** and Speed Booster **148406/0**, with normal rendering, audio
and comparisons. Its 51,791 focused SNES comparisons exercise all 20 migrated
numeric entries, subsequent flight and launch inheritance. Probed any%
**11113/0** and Speed Booster **10934/0** cover the selected demo/Ceres branches
plus 120 follow-up frames. Exotic beam/super-missile paths have constructed
coverage; no endgame execution is claimed. Earlier byte-product checks have
independent actual-function coverage, without proven changed HDMA execution.
Earlier platform probes cover timed initialization, triggering,
closing and stage advancement. Required opening/carry and non-timed movement/
activation probes have zero hits: those coverage checks fail despite zero
differing frames. Those paths have constructed old/new Rust comparisons,
not movie/SNES coverage; hold further cleanup until that context is covered.
Beam firing's 20,420 focused SNES comparisons cover 5,780 input/
firing cases and 14,640 subsequent projectile calls. The earlier intro-page
checks include Japanese **7570/0** and 233 marker/inactive-input oracle cases.

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
Rust lines 112265. The unrelated 83-line shader draft is excluded from the
committed baseline and budget.

After the Japanese correction, local bindings, intro pages, enemy callback
imports, ordered stores, PNG texture loading, beam firing, moving platforms,
byte-partial-product sharing, INI decoding, countdown callbacks, projectile
flight, callback entry captures and opcode grids, maintained Rust is **108,890**,
leaving **8,891** to reach 99,999. Counters:
raw offsets 30; extern C blocks 3; counted C ABI exports 4; no_mangle 8;
unsafe functions 4431; cast lines 8107. The unrelated 83-line shader draft
remains excluded; working-tree counts are not the committed baseline.
Item 2's function/compound-body variant audits are complete for their stated
scope. The remaining gap has no established replacement budget. The hopping
and initializer drafts remain parked; repeating the completed scans or adding
their gross matching spans does not supply that budget.

Historical details before this reset remain in **ec45c573:docs/PLAN.md**.
