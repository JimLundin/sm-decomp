# Evidence and implementation gates

All target-engine gates below are **not run**. Existing `sm-decomp` comparisons remain useful evidence for the behavior they exercise. A report that renders correctly is not evidence that the proposed engine is deterministic, faithful, fast, or complete. Implement the smallest useful slice at each gate and use its failures to revise the architecture before expanding content.

Use [Migration](MIGRATION.md) for current Windows work and intermediate replacements. These gates describe completion of the new architecture; they do not replace the current plan's work order, require immediate emulator removal, or invalidate its existing checks.

## Gate 0 · Pin the target

Select the original release/region and record its identity, input protocol, cadence, observation method, and reference-capture provenance. Establish native image/color interpretation and the sound comparison method. Explicitly resolve original slowdown and glitch scope; preserve established movement techniques in acceptance scenarios.

Choose minimum desktop hardware and intended operating systems. Define measurable CPU/GPU latency, sound-buffer, memory, and startup budgets from this profile. A rounded 60.1 Hz caption and a fast development machine are insufficient specifications.

Exit evidence: a versioned Reference profile, at least one independently captured trace/image/audio fixture, and a reproducible capture procedure. Without this, later “matches the original” claims have no stable meaning.

Map Rando integration is part of final acceptance, not established by a vanilla playthrough. Pin the upstream version/input contracts and demonstrate import of a real seed/settings set and custom sprite assets. Verify reordered room/door transitions, map and progression persistence, and custom sprite/palette rendering through the same runtime interfaces. Test supported opt-in rules such as re-spin disabled, enabled and in relevant combinations against the corresponding reference behavior. Use discriminating mutations for option selection and graph/asset resolution; require wrong-profile saves/replays to follow the documented compatibility policy. Keep ROM-derived fixtures local. These requirements shape the current architecture but do not block the staged vanilla refactors in PLAN.

## Gate 1 · Prove the real seams

Build the proposed package graph and a native one-room slice. Use the actual window/controller path, an owned Session, actual rendering, one scheduled sound, and headless input replay. Compile headless `sm_game`/`sm_tools` without window, GPU, or sound-device dependencies.

To finish this gate, build and run the slice from its workspace and prepared content with the comparison backend, archived repositories, recovered-C libraries, and emulator executables absent. Intermediate work may use the current backend and oracle until the new responsibilities are ready. Check that third-party runtime dependencies provide platform access and that our code owns gameplay, rendering, and sound generation. Reference capture and original-data import are development operations outside the completed slice's normal build and play.

Challenge the renderer with priority overlap, palettes, masks, color math, scanline effects, and an affine case early. Benchmark native rendering and presentation on the minimum target. If the focused GPU path is disproportionately complex or fails the budget, reconsider a CPU compositor with measured evidence; do not implement two complete engines.

Exercise the outbox with an audio queue deliberately held full and a save sink that accepts later. Verify no duplicated sound batch, no skipped effects, responsive event handling, and bounded memory. The save acknowledgment must remain distinct from enqueue success.

Exit evidence: same Tick inputs produce identical canonical gameplay state and semantic effect trace; display frequency changes do not alter either; the difficult image fixtures and initial audio timing pass; dependencies and resource ownership match the design.

## Gate 2 · Prove Samus and collision

Cover ordinary movement, running, jumping, aim/turn/posture transitions, Morph Ball, bombs and bomb jumps, wall jumps, damage/invulnerability, charge, missiles, beam combinations, speed boost/shinespark, grapple, slopes, liquids, edges, and high-speed contacts. Include known techniques such as mockball where the selected Reference profile requires them.

Use fixtures that expose one-Tick differences: simultaneous direction/fire input, landing while changing posture, firing on an Actor-spawn Tick, edge contacts, conflicting hits, and invulnerability expiry. Compare position, velocity, pose, hitboxes, timers, RNG, effects, and progression fields that can be observed reliably.

Exit evidence: behavior matches independently captured observations under documented tolerances, numeric policy is explicit, and the player state model represents ability combinations without scattered invalid combinations. A replay of our own outputs cannot satisfy the fidelity portion.

## Gate 3 · Prove rooms and a representative encounter

Build connected rooms with door scrolls, camera changes, a persistent pickup, resettable room objects, one shared enemy family, one multipart boss, and overlapping projectile/contact behavior. Enter, leave, die, reload, and return with changed progression.

Use this slice to refine our Actor store and query helpers. Evaluate total ordering, spawn visibility, cohesive boss state, Checkpoint restoration, and inspector work together; count generated helpers and ordering scaffolding as complexity too. Keep these implementations in our own Rust modules, consistent with the ownership decision.

Exit evidence: no stale handle resolves to a new Actor; simultaneous interactions have reference-correct order; transitions account for all carryover/reset state; Checkpoint → advance → restore → advance reproduces the same state/effects. Confirm or revise Session's private domain seams before expanding enemies and bosses.

## Gate 4 · Prove authoring and failure recovery

Compile representative Tiled/RON source with source-located diagnostics. Test duplicate IDs, broken door references, malformed collision, unsupported behavior kinds, invalid sequence targets, damaged packs, and resource-limit violations. Canonical builds from identical input must produce identical gameplay fingerprints.

Run invalid, media-only, and gameplay-changing reloads. The invalid candidate leaves the active revision untouched; media-only changes preserve gameplay state; gameplay changes restart the requested Scenario in a new epoch. Run a reload while old frames and voices are in flight, and measure peak memory until both retire.

Inject save failures before write, before replacement, and after replacement at the applicable acknowledgment point. Test unsupported versions, corrupt bytes, read-only/full storage, ordered writes to the same slot, and load/reset while a write is active. Validate the platform's replacement and crash-recovery guarantees separately from a mock worker.

Exercise surface resize to zero, occlusion, suspension/resume, device recreation, changed audio sample rate, queue saturation, bank retirement, and stale old-generation acknowledgments. Host recovery must not mutate progression or reuse invalid handles.

Exit evidence: live state and the previous valid save survive defined failure paths, all queues remain bounded, resource retirement is correct, and the Workbench cannot bypass Session ownership.

## Gate 5 · Cover the whole game

Inventory original rooms, enemies, bosses, items, progression events, menus, maps, sequences, endings, visual effects, and sound programs. Each entry has an implementation location, Scenario or playthrough coverage, reference provenance, and known deviations. “Uses the same shared behavior” does not remove the need to exercise content-specific parameters.

Combine focused scenarios with end-to-end playthroughs and progression routes, including sequence breaks accepted into the Reference profile. Test save slots, death/reload, endings, transitions, input remapping, and shipping packaging. Validate startup and decoded-memory footprint on the target hardware.

Exit evidence: reviewed content coverage and independent behavioral/visual/audio comparisons, with explicit deviations rather than an unsupported claim of perfect equivalence. Determinism, fidelity, performance, and completion have separate results.

## Verification strategy

Retain whole-RAM comparisons for the current compatible representation. When replacing it with typed domain state, define an explicit projection of both implementations into matching observations at the same Tick/phase: input, position/velocity, pose, timers, RNG, actor order, room/progression, and ordered effects as relevant. Record units, wrapping, excluded fields, and synchronization rules. A projection compares the fields it covers; it cannot claim byte-identical RAM or automatically import old checkpoints. Preserve the existing comparison path until the new one has adequate coverage.

Windows execution results are separate from compilation and from runs on another OS. Record compiler/target/features, scenario and source revision, commands, observed results, and unrun or blocked checks. Follow the existing resource and security constraints in Migration; a blocked executable is not a passing runtime result.

| Concern | Appropriate proof | What it does not prove |
| --- | --- | --- |
| Determinism | Canonical hashes/effects from equal starts and Tick inputs, across supported build targets | That the rules match the original |
| Original behavior | Independent reference traces and documented observation/tolerance | Full content coverage from one room |
| Visual fidelity | Native-resolution fixtures for overlapping effects, with defined color conversion | Gameplay order or sound behavior |
| Audio fidelity | Offline cue/sample comparisons at a fixed rate; focused live transport checks | Sample-exact recovery from device loss |
| Storage correctness | Real platform replacement/failure tests and schema migration fixtures | Identical guarantees on every filesystem |
| Bounded lifetimes | Stress reload/queues, retirement acknowledgments, memory measurements | Performance from static architecture diagrams |
| Content completeness | Coverage inventory plus routes/playthroughs and content-specific scenarios | Correctness merely because the pack compiles |

Tests cross the same Session and runtime interfaces as real callers. Use private numerical tests for collision primitives where they isolate difficult rules; avoid a mock interface per small function. A headless input source and native input source both produce the same value records, so they do not require a trait just to be different callers.

Checkpoint testing must include input history, RNG, pending actions, sequence cursors, allocator generations, and simulation order. An observation-only query, a different presentation rate, or enabling the inspector must not change state hashes. Release/debug arithmetic must agree.

## Measurements to collect

Record Tick/extraction CPU p50/p95/p99 and worst observed time; GPU submission/execution time; native draw/pass counts; end-to-end input latency; output packet allocation/copy cost; resident and peak two-revision memory; worker latency; audio underruns/overruns; queue high-water marks; and retirement latency. Measure worst-case encounters and effects, not only an idle room.

Numerical pass budgets are established from the minimum hardware profile at Gate 0. Queue capacities and catch-up defaults in [Contracts](CONTRACTS.md) are testable initial policies, not those performance budgets. Add gameplay threads, streaming, or pooling only when a measured failure identifies the need.

## Code-size accounting

Retire the old 30–60k runtime estimate as a target for this design. The current project's below-100,000-line goal and 90,000-line stretch checkpoint remain governed by its work plan. After the representative encounter, measure authored source and the distribution of content-specific work. Reducing complexity means reducing duplicated knowledge and improving locality; moving branches into a sequence file or adding opaque generated code does not automatically achieve that.

Count these categories separately and report their sum when discussing the whole rebuild:

- Custom runtime/host and game Rust, including adapters and sound/rendering implementations.
- Shaders and any other authored executable languages.
- Compiler, importers, Workbench, replay tools, and packaging support.
- Tests and reference tooling.
- Executable event sequences and generated executable code, with their source/generator counts distinguished.
- Imported/static content data and media, measured separately from executable source.
- Third-party library source and operating-system code, explicitly separate from authored code.

If “including the engine” means all upstream implementation too, report a separate reproducible vendored-dependency total from the pinned lockfile. Do not hide that code in a claimed from-scratch total or confuse dependency size with the amount we authored. Historical reconstruction counts in [References](REFERENCES.md) are context only.
