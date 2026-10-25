# Responsibility migration

The accepted destination and Windows migration policy are in [README](README.md).
Current work, checks and resource constraints follow [PLAN](../PLAN.md) and
[REVIEW](../REVIEW.md). This mapping describes responsibilities, not file moves
or a second work queue; the maintained Rust source is authoritative.

| Current responsibility | Destination | Removal condition |
| --- | --- | --- |
| `sm/src/game/`, `game_state.rs`, `ram_map.rs` | `sm_game`: owned Session and private gameplay modules | Complete producer/consumer cohorts use typed state; remaining RAM consumers have migrated. |
| `main.rs`, `host.rs`, `config.rs`, host portions of `rtl.rs` | `sm_app`: input, clock, lifecycle; `sm_runtime`: devices/storage | Gameplay consumes recorded values and returns owned records without host policy. |
| `platform/`, PPU/DMA drawing and effects | `sm_runtime`: our renderer/shaders; `sm_game`: Scene extraction | Complete drawing-to-pixels behavior is preserved; remaining game/oracle backend consumers are accounted for. |
| `spc_player.rs`, `native_audio.rs`, SNES audio/DSP | `sm_runtime`: our sequencer, voices, mixer/output | Cue order, instruments, pitch, envelopes, voice rules and echo have independent checks. |
| ROM tables, content interpretation, state formats | `sm_data`: validated definitions/records; `sm_tools`: offline importer/compiler | Prepared content and explicit schemas replace ROM layout as the public interface. |
| `verify.rs`, `verify_dll.rs`, `cpu_infra.rs`, `verify/`, comparison tools | Development oracle; target `sm_tools`: replay/scenarios/capture | Equivalent observations/coverage exist before retiring the working comparison path. |

For each replacement, identify its owner, callers, inputs/results, failure and
suspension paths, remaining legacy dependencies and their removal condition.
Reuse cohesive algorithms; use ordinary structs, enums, private modules and
direct calls. Temporary bridges disclose their dependencies. Keep SNES addresses
and register operations out of completed Session interfaces.

Keep whole-RAM comparisons while representations agree. When they diverge,
map both implementations into an explicit semantic observation at the same
Tick/phase, including numerical rules and ordered effects. Name exclusions and
uncovered paths. Old RAM snapshots need validated conversion or boot/replay;
self-determinism does not establish original-game fidelity.

Replacing a renderer does not retire hardware still used by the oracle. Before
switching reference backends, preserve required PC/instruction hooks, state
access, patches, snapshots and capture capabilities. The completed slice must
build/play from prepared content with its comparison backend unavailable;
headless gameplay/tools must build without device dependencies.

Map Rando seed/settings, stable room/asset identities, custom media and optional
mechanics constrain these boundaries now. Reordering must preserve identities;
options belong in their gameplay modules with explicit configuration. Vanilla
coverage does not validate enabled options. Current ROM tables and RAM adapters
do not establish integration readiness. ROM-derived assets remain local except
for the explicitly authorized encrypted CI inputs described in PLAN.
