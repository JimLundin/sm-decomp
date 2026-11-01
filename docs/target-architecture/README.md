# Accepted engine architecture

The destination is a purpose-built Super Metroid engine in our own Rust:
one deterministic, owned `Session`, our renderer and sound engine, and small
platform libraries (`winit`, `wgpu`, `cpal`) for devices. The finished game builds
and plays from prepared content without SNES emulation, recovered-C gameplay or
an external engine/ECS. This is the accepted design, not a claim of completion.

Map Rando integration requires custom sprites/palettes, configurable room/door
graphs and explicit opt-in re-spin/QoL mechanics. Keep content, topology and
options separate from platform and ROM/RAM compatibility. Vanilla is the first
fidelity profile; intentional options need their own interaction checks.

## Implementation evidence

For package migration counts, backend dependencies and validation status, read
the dated [architecture snapshot](../PROGRESS_REPORT.md#architecture-snapshot).
[REVIEW](../REVIEW.md) owns milestone direction; this document owns the destination.

## Migration policy

Windows workers may retain the current Rust game's emulated backend and built-in
oracle while replacing responsibilities. [PLAN](../PLAN.md) and
[REVIEW](../REVIEW.md) govern current tasks, verification, resources and platform
restrictions. Do not remove a working comparison path before replacement evidence
exists. Backend independence is a completed-slice/final-runtime criterion.

Migrate a complete responsibility and its callers, with one state owner and
explicit inputs/results. Preserve RAM compatibility until consumers move;
[the state migration contract](../ENGINE_STATE_MIGRATION.md) covers external
writers, reset/load, oracle adoption and suspended execution. Introduce packages
when real responsibilities meet their contracts; file moves or crate skeletons
alone do not achieve ownership. Count retained code, adapters and dependencies
in the full replacement cost.

## Responsibility migration

The source is authoritative; these destinations describe responsibilities, not a
second work queue. Each replacement names its callers, inputs/results, failure
and suspension paths, retained dependencies and their removal condition.

| Current responsibility | Destination | Removal condition |
| --- | --- | --- |
| `sm/src/game/`, `game_state.rs`, `ram_map.rs` | `sm_game`: owned Session and private gameplay modules | Complete producer/consumer cohorts use typed state; remaining RAM consumers have migrated. |
| `main.rs`, `host.rs`, `config.rs`, host portions of `rtl.rs` | `sm_app`: input, clock, lifecycle; `sm_runtime`: devices/storage | Gameplay consumes recorded values and returns owned records without host policy. |
| `platform/`, PPU/DMA drawing and effects | `sm_runtime`: our renderer/shaders; `sm_game`: Scene extraction | Complete drawing-to-pixels behavior is preserved; remaining game/oracle backend consumers are accounted for. |
| `spc_player.rs`, `native_audio.rs`, SNES audio/DSP | `sm_runtime`: our sequencer, voices, mixer/output | Cue order, instruments, pitch, envelopes, voice rules and echo have independent checks. |
| ROM tables, content interpretation, state formats | `sm_data`: validated definitions/records; `sm_tools`: offline importer/compiler | Prepared content and explicit schemas replace ROM layout as the public interface. |
| `verify.rs`, `verify_dll.rs`, `cpu_infra.rs`, `verify/`, comparison tools | Development oracle; target `sm_tools`: replay/scenarios/capture | Equivalent observations/coverage exist before retiring the working comparison path. |

Preserve whole-RAM comparisons while representations agree; otherwise compare
explicit semantic observations at the same Tick/phase, with named exclusions.
Old snapshots require validated conversion or boot/replay. Self-determinism does
not establish original-game fidelity.

Before replacing a reference backend, preserve its PC/instruction hooks, state
access, patches, snapshots and captures. Headless gameplay/tools must build
without device dependencies; completed runtime slices play from prepared content
with the comparison backend unavailable.

## Reference by task

| Need | Read |
| --- | --- |
| Map current responsibilities to destination packages | [Migration](#responsibility-migration) |
| Package graph, domain modules, Map Rando and media | [Architecture](ARCHITECTURE.md) |
| Interfaces, Tick phases, ownership and recovery | [Contracts](CONTRACTS.md) |
| Prove a completed independent engine slice | [Validation](VALIDATION.md) |
| Domain terminology | [Glossary](GLOSSARY.md) |
| Source provenance and historical size comparisons | [References](REFERENCES.md) |

Accepted decisions: [owned Session](docs/adr/0001-owned-session.md),
[revision cutovers](docs/adr/0002-revision-cutovers.md),
[observable rules](docs/adr/0003-observable-rules.md), and
[our Rust implementation](docs/adr/0004-owned-rust-implementation.md).
The exact reference release, minimum desktop hardware, original slowdown policy
and numerical performance budgets remain evidence gates. Current size goals
live in PLAN/REVIEW; the old 30-60k estimate is retired.
