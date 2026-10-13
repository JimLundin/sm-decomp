# Faster route to a playable readable Rust port

Status: proposal; read-only architecture audit completed
Owner: `/root/ridley_captures`
Date: 2026-10-13
Scope: existing source, retained evidence and coordinator observations; no new verification runs or shared source changes

## Recommendation

Keep the readable Rust already translated, but change the unit of delivery to a complete scheduled subsystem through one stable game host interface. Separately make the input/scheduler adapter reach a named gameplay milestone. Stop using raw input count or another verified helper family as the principal measure of progress toward a playable game.

This follows `docs/DECOMP_PIVOT.md`: temporary measured C gameplay is allowed, lag synchronization can be adapted, canonical RAM stays intact initially, and full ownership is the destination. It revises the current implementation loop rather than reversing the pivot. The initial offline/live pilot requirement has already been satisfied; it need not be repeated as a new bespoke foundation for every subsystem. No ADR directory exists.

The next experiment should answer two questions: can an existing complete Rust subsystem use the same interface in focused C/Rust comparisons and live execution, and can the unchanged C gameplay reach first Ceres under a causal input driver? Run these as independent workstreams. Defer another broad formal replay until that experiment is concrete; finish and clean already active runs.

## What the repository actually does

| Evidence | Consequence |
| --- | --- |
| `crates/port/src/lib.rs:7` exposes gameplay modules, mostly with string-based `dispatch*` entry points. Individual modules have scheduled-frame functions; there is no shared reset/input/frame/save lifecycle interface. | Translated code is callable, but the executable game still lives in C and generated per-family glue. A large exported inventory does not yet make a runnable Rust game module. |
| `ridley_subsystem.rs:8` keeps admission, movement, rendering, cutscenes and scripts private. `Host` at line 38 synchronously carries state to seven foreign operations, Mode7 and ordered effects. | This is already a deep module worth keeping. Whole subsystem ownership is practical without restructuring WRAM or translating the entire game at once. |
| `room_scroll.rs:46` needs synchronous VRAM flushing; `samus_projectile_frame.rs:32` needs HDMA and inherits projectile callbacks. Its default `NoCallbacks` adapter refuses those paths. | Several complete frames are waiting on a host adapter, not on more isolated arithmetic helpers. A stable shared host interface can admit existing behavior. |
| `live.rs:338` generates wrappers per family, renames native definitions and installs special projectile/Ridley templates. `family.rs:807` branches among ordinary, callback, power-bomb, PLM and Ridley adapters. | Every new behavior category can add manifest normalization, build glue, ownership counters, snapshots, authentication and live wiring. This spreads change across the pipeline rather than concentrating it in gameplay. |
| `inputs.rs:31` discovers modules by lexical words, then `inputs.rs:110` seals actual compiler dependencies. | Existing narrow source reuse is useful. Replacing it with a blanket dependency on every module would make reuse worse. Nested modules also need reliable dependency discovery rather than another global fallback. |
| `inputs.rs:130` seals `main.rs`, `inputs.rs`, and `builds.rs` into every family. `family.rs:591` seals batch, cleanup, metrics and other harness sources; `affected.rs:116` treats pipeline changes as shared. | An unrelated CLI, progress, cleanup or orchestration edit can invalidate all families. A stable gameplay interface alone will not fix verification amplification. Contract identity must be separated from orchestration identity. |
| `builds.rs:330` caches pinned C objects using the compiler's preprocessor closure. Generated C units are rebuilt, and family/live Rust wrappers are directly compiled by `rustc`. | The useful cache already exists. Priority is reusable candidate/host artifacts and narrower adapters, not another generic cache system. |
| `live.rs:1332` publishes four subjects sequentially. `packed.rs:400` authenticates the private store, imports under the shared store lock, then line 447 authenticates the shared store. Live verification also authenticates before and after cleanup. | Publication and repeated restoration/hash walks can dominate after gameplay has finished. Preserve final authentication, but avoid making shared publication mandatory for each development probe. |
| `headless.c:118` reads one authored video-frame pad for each `RtlRunFrame`. `DECOMP_PREFIX.md` records omitted polls and waits in that mapping; ticket 78 reaches 1,097 causal semantic updates through source frame 1,600. | Raw replay remains an inadequate gameplay target. Source-ordered polls and completed updates are already a stronger starting point; do not start a timing investigation from zero. |

The coordinator reports the completed 129,712-input native route ends in game state 8, room `$DF8D` in Ceres, rather than credits. This is a coordinator observation, not an independently authenticated completion claim. The retained r7 whole-Ridley probe ends at input 38,000 in state 8, room 57,525, with Ceres main/draw/script/getaway activity. Both observations reinforce the documented input problem. A long C/Rust parity pass can verify a translation while following an unintended route.

## Measured cost and limits

Retained v2 evidence provides useful scale without launching more work:

| Finished work | Recorded cost |
| --- | --- |
| Whole Ridley controlled family, 6,457 fixture pairs | `controlled/...105766-0/family.metrics.toml`: 565.45 seconds wall time |
| Ridley natural family, 670 pairs | `natural/...129925-0/family.metrics.toml`: 87.55 seconds wall time |
| One native C 129,712-input inventory replay | `inventory/...105188/replay.metrics.toml`: 282.16 seconds wall, 214.79 seconds user CPU |
| Controlled-family deduplication | `controlled-dedup.toml`: 19,371 snapshots checked; 1,667,891,200 bytes reclaimed |
| Current four-subject composed gate | Coordinator observation at approximately 26 minutes: gameplay took about 10 minutes under load; sequential inventory/import work was still running. Final result file was empty at inspection. |

The exact evidence root for the first four rows is `.scratch/decomp-port/ridley-hardware-recovery/`. Metrics for those historical runs have detailed stage sampling disabled; these totals cannot identify a precise dominant substage. Do not promise a speedup factor before measuring the experiment.

There is useful progress in the current bounded gate: 12 affected Offline families and 20,943 pairs passed according to the coordinator. These prove specific translation behavior, not any% completion. The same distinction must remain in the progress inventory.

## Proposed interface and module shape

Keep `Wram`, validated ROM data, and external projectile words canonical. Provide typed scheduled entry points for whole modules; retain string dispatch only inside the fixture compatibility adapter. The initial entry can remain `HandleProjectile`, with draw entries where the C scheduler actually separates them. Avoid one public method per translated helper.

A small execution module owns selection, invocation, canonical-state transfer and ownership reporting. One C ABI exposes that interface to the retained host. It receives a typed entry identifier and execution context, returns `Refused` before any publication, or executes Rust exactly once; an error after acceptance terminates. Subsystem code owns ordinary control flow and private dispatch. C fallback remains explicit at unowned foreign continuations, not an unreported Rust helper implementation.

Use one shared synchronous host implementation for the operations already required by the first consumers: terrain/PLM callbacks, HDMA allocation, VRAM flushes, register reads/writes and foreign gameplay calls. Keep distinct typed requests for hardware and unowned gameplay. The C adapter publishes latest WRAM and all external words before a request and refreshes them after it. Ordered MMIO cannot be deferred into an end-of-frame command list because later reads depend on earlier writes and latch state.

There are two justified adapters at this seam: the retained actual-C/hardware adapter for differential and live execution, and a recording adapter for deterministic interface controls. A future Rust hardware adapter is a later replacement at the same seam. Do not replace the hardware backend during this experiment.

Separate four contracts: gameplay module inputs, synchronous host transfer semantics, input/scheduler semantics, and retained evidence decoding. Freeze each contract's sources and source closure. A CLI/progress change must not invalidate gameplay results merely because `main.rs` changed; changing any source that actually implements selection, invocation, comparison or decoding must still invalidate its relevant results. Existing sealed evidence keeps its original reader and source snapshots; never rewrite old seals to fit the new contract.

## Concrete next experiment

1. **Input owner:** continue ticket 78's causal driver with unchanged pinned C gameplay. Reuse authenticated original captures where available. Extend from the verified 1,600-source-frame prefix to the first Ceres checkpoint, proving poll provenance and a named room/lifecycle transition. Preserve actual wait continuations, EOF behavior and the existing omitted-wait/RNG controls. Record the first divergence; introduce no corrective state injection. This establishes a useful gameplay trajectory before attributing its failures to translations.
2. **Gameplay/host owner:** route the already translated complete `samus_projectile_frame` closure through the stable execution/host interface. Supply its existing PLM/HDMA callbacks using retained native behavior. Reuse `projectile_flight`, `projectile_special`, `power_bomb`, rendering and trails as private dependencies. Do not rewrite their state model. Adapt the existing actual-C fixture adapter to invoke the same scheduled interface used live; make individual helpers diagnosis tools.
3. **Focused verification:** use the existing affected-family selector and sealed fixtures. During edits run only affected comparisons and one fresh C/Rust pair over the shortest interval that exercises the changed path. Use a private evidence store; retain failure evidence and clean finished intermediates. A fast probe has its own nonformal status. It cannot claim four-process acceptance or original-ROM parity.
4. **Milestone gate:** when source and interface stop changing, run interface/refusal/order controls, current-source affected Offline families, and one four-process replay through the agreed gameplay checkpoint. Require material state/effect execution, matching ownership, visible C/CPU fallback, and post-cleanup retained authentication. Publish integrated/live-exercised stages once. Schedule full any% only after the driver carries the intended route and a substantial subsystem increment is ready.
5. **Measure:** report edit-to-first-result wall time, affected pair count, rebuild time, replay time, private authentication time, publication time, bytes retained, and furthest gameplay checkpoint. Repeat an unrelated progress/CLI edit as a reuse control: unchanged gameplay and host contracts should remain reusable. This is the concrete test of the proposed architecture.

Completion of the experiment means existing whole gameplay is reachable through a reusable interface and the C trajectory makes meaningful Ceres progress. It does not mean Rust owns the whole game. The next milestones are a human-controllable boot/save/room session through that driver, then successive scheduled subsystem replacements along the route to credits. Assign complete subsystem source ownership, with one shared host/integration owner, as already required by `translation-progress.md`.

## Risks and decisions to preserve

- A universal host enum containing every helper would recreate the current shallow dispatch interface. Start with two existing consumers and hide internal calls inside modules.
- Whole module compilation can broaden Rust dependency seals. Keep explicit module source closures and validate compiler reads; do not import all of `port/lib.rs` into every narrow verification artifact.
- The whole-Ridley candidate presently admits any nonzero Rust root and calls first-two admitted snapshots material (`/tmp/sm-ridley-whole-live-source/.../live.rs:1248`). Timer-only or frozen calls could satisfy that generic gate. The observed r7 opcodes show real script activity, but future module gates need explicit material effects/state changes and honest absent domains. Boss and explosion roots are zero in r7; Ceres coverage cannot stand in for those domains.
- Restoring a late checkpoint during diagnosis is a fixture operation. Claiming a live milestone still requires ordinary reset without corrective state loads; checkpoints also need scheduler, hardware, external words and input position before resumable execution is trustworthy.
- Scheduler changes and gameplay changes must be isolated. Otherwise a mismatched input edge and a translation fault become inseparable again.
- Shared publication optimization must retain immutable object identity and final restored-byte authentication. First reduce its frequency by using development probes; redesign import only if measurements justify that additional work.

No verification runs, temporary binaries, source mutations, or worktrees were created for this audit. Only this Markdown report was written; there are no run-cleanup actions from this task.
