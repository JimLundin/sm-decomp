# Adopting the owned Rust architecture

Accepted by the user on 2026-10-22: build a purpose-built Super Metroid game and engine in our own straightforward Rust, preserving the original gameplay and content. Small platform libraries such as `winit`, `wgpu`, and `cpal` are allowed. The five target packages are our code. The user also explicitly accepts the current emulated version as a practical development and comparison path for the workers on Windows.

[Architecture](ARCHITECTURE.md) and [Contracts](CONTRACTS.md) describe the destination. The [current work plan](../PLAN.md) still determines ongoing tasks and checks. This handoff adds a direction for responsibility replacement; it does not restart the work queue or make the new packages a prerequisite to useful current work.

## Current implementation and destination

This mapping was checked against [`main` at 498c8a4d](https://github.com/JimLundin/sm-decomp/tree/498c8a4dd9cd6268de1e30b8cc08f175a43b765e). The current game, runtime, and SNES emulation are maintained Rust, originally transpiled with c2rust. The earlier mixed C/Rust restart is archived. Current code is the behavior authority during migration; historical original-C experiments are separate evidence.

| Current responsibility | Destination | Migration boundary |
| --- | --- | --- |
| `sm/src/game/`, `game_state.rs`, `ram_map.rs` | `sm_game`: one owned Session with private player, collision, actor, room, progression, and sequence modules | Keep current WRAM compatibility for live consumers. Replace complete rules and their state access with typed domain state as callers and comparisons migrate. |
| `main.rs`, `host.rs`, `config.rs`, host portions of `rtl.rs` | `sm_app`: input/clock, effect delivery, lifecycle; `sm_runtime`: device/storage adapters | Move host policy out of gameplay. A direct Session call consumes recorded values and returns owned records. |
| `platform/`, PPU/DMA-dependent drawing and game effects | `sm_runtime`: our renderer and shaders; `sm_game`: Scene extraction | Replace the full path from game-facing drawing records to pixels, preserving effects. Keep emulation needed by the current path or oracle until its consumers move. |
| `spc_player.rs`, `native_audio.rs`, SNES audio/DSP behavior | `sm_runtime`: our sequencer, voices, mixer, and output transport | A device library replaces device access only. Preserve cue order, instruments, pitch, envelopes, voice rules, and echo with independent checks. |
| ROM-oriented tables, content interpretation, runtime state formats | `sm_data`: validated definitions and portable records; `sm_tools`: offline import/compiler | Move fixed content into validated packs and explicit schemas. Keep ROM-derived assets local under the repository's existing policy. |
| `verify.rs`, `verify_dll.rs`, `cpu_infra.rs`, `verify/`, `tools/`, built-in 65816 oracle | Development comparison path; target `sm_tools` replay/scenarios/reference capture | Keep the usable Windows verifier. Add semantic observations for new state before retiring checks that need the old layout. |

These are responsibility destinations, not a file-renaming plan. Existing cohesive Rust algorithms may be retained and adapted. Moving bank files into five crates does not by itself establish Session ownership or backend independence.

## Staged adoption

1. **Continue the active work.** Preserve worker changes, ongoing experiments, and the current verification path. The plan's below-100,000 maintained-Rust objective and 90,000-line stretch checkpoint remain active. The old 30–60k rewrite estimate in this design is a different, retired estimate.
2. **Choose a complete responsibility.** For the next subsystem candidate, identify its state owner, all callers, inputs/outputs, dependencies, and the oracle observations that cover it. Agree overlapping file/interface work with the existing owners before editing it. Finish the proposal when its full scope and checks are explicit in the current plan or change description; no new tracking system is needed.
3. **Implement through a bounded seam.** Prefer ordinary structs, enums, private modules, and direct calls. A temporary bridge may call the existing backend while the replacement matures. Record what it still depends on, who owns mutable state, and which replacement/check allows each dependency to go away. Keep SNES addresses and register operations out of the completed Session interface.
4. **Compare and integrate.** Use the current validation strategy for the changed behavior and its callers. Preserve whole-RAM checks while the representation is compatible; compare explicit semantic projections when state changes. Price the complete replacement, including retained old code, adapters, shaders, tooling, and dependencies. Adopt only the behavior and deletion claims that the checks establish.
5. **Prove an independent slice.** The new-engine slice completes its architecture gate when it builds and plays from prepared content with the comparison backend unavailable. This may be a separate target within the repository; a new repository or an immediate workspace split is not required. Headless gameplay/tools must also build without device dependencies.
6. **Retire the remaining backend uses.** Move reference execution outside the shipping game when equivalent observations and coverage are available. The final game owns its rules, pixels, and sound. Development can continue to use an emulator as an oracle without making it a shipping dependency.

The five-package dependency graph is the intended finished organization. Introduce packages when an actual responsibility can meet its contract; the handoff itself adds no code, dependencies, or skeleton framework.

Map Rando integration constrains these replacements now. Preserve an explicit path from imported seed/settings and custom media to validated content, room connections and a gameplay profile. Stable room/asset identities must survive reordering; optional mechanics belong in their gameplay modules with explicit configuration, not in emulator patches or scattered platform conditionals. Vanilla comparisons validate the disabled/default profile; intentional options need their own reference and interaction checks. Current numeric ROM tables and WRAM projections remain transitional and do not establish integration readiness. Continue complete responsibilities in PLAN order; this clarification does not insert unrelated mechanics into the visual-effects change.

## Comparison during migration

The current built-in oracle and its Windows host are useful infrastructure. Keep the normal rendering/audio advancement, mismatch checks, required probes, and source validation for checks that use them. Current checkpoint compatibility follows the existing verifier's rules.

Typed Session state will not have the same byte layout as SNES RAM. For a replacement, define a common observation record at a specified Tick/phase and map both implementations into it. Include the relevant numerical rules and ordered effects; name excluded fields and uncovered paths. A successful self-replay establishes determinism. Agreement with independent original-game observations supplies fidelity evidence only for the compared behavior. Old RAM snapshots need a validated conversion or a boot/replay start before they can initialize a different state model.

A replacement renderer does not retire a PPU, DMA, or CPU still used by the oracle. An external oracle is a later implementation choice, not a prerequisite to current work. Before switching, establish the instruction/PC hooks, state access, patches, snapshots, and capture capabilities required by the existing comparisons. Keep the working path until those observations are available.

The current plan records passing any% and Speed Booster comparisons, with explicit limitations; neither route reaches credits. Those results do not establish all branches or validate the proposed engine. Keep behavior fidelity, self-determinism, content coverage, backend independence, and platform execution results distinct.

## Windows workflow

Use the existing Windows commands in [movie instructions](../../movies/README.md) for the current verifier. Publish PowerShell commands and Windows paths when handing work to these workers. Do not assume Bash, WSL, Linux shared libraries, or a Linux GPU/audio path is available. A DLL host or C ABI at a development/device boundary does not mean gameplay is implemented in C.

The HTML report is self-contained and opens directly from disk in a browser. Reading it needs no server, Python, Node, or build. Repository-local documents provide the complete specification if browser execution is unavailable.

Use the measured compiler concurrency and memory limits in [PLAN.md](../PLAN.md#toolchain-and-windows-constraints); its current policy supersedes the earlier one-job limit. Reuse matching baseline builds/dependencies, keep output bounded, and retain evidence needed for adoption outside TEMP. Follow the existing durable `Local/sm-decomp-experiments/20261021` convention. Historical lost artifacts retain their recorded status; recreate and recheck a candidate before treating it as currently inspectable. Preserve the unrelated shader draft and other workers' source and build caches.

The recorded graphics experiment is blocked by Windows Defender ASR event 1121, rule `01443614-CD74-433A-B99E-2ECDC07BFC25`. Keep that execution parked under the current plan. Record blocked runs as blocked, and continue permitted source, compiler, or other checks. Preserve security settings and the block; a different OS result cannot be reported as a Windows runtime pass.

Record the commit/source snapshot, Rust compiler and target, enabled features, commands, scenario, and actual results for each candidate. State unrun checks and platform constraints alongside the result. Follow the existing Git concurrency rule: use isolated snapshots as needed, reconcile current HEAD before writing, let Git hold its normal short-lived locks, and preserve locks owned by other processes.

## What adopting this handoff means

Future subsystem and backend work follows this destination while existing tasks continue under the current plan. Each candidate makes its remaining legacy dependencies and next removal condition explicit. [Validation](VALIDATION.md) supplies the target architecture's completion evidence. Publishing the design is not an implementation result or evidence that every worker has read it.
