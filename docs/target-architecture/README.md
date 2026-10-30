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

## Current status (2026-10-30)

These notes record where the code stands; they do not change the design.

- `sm_data`, `sm_game`, `sm_runtime` and `sm_app` exist as workspace crates
  under `crates/`; `sm_tools` does not yet. Replay and import tooling is the
  verifier in `sm` and the Python scripts in `tools/`.
- CI requires `sm_game` and `sm_runtime` to depend only on `sm_data`, and
  `sm_data` to depend on nothing. All three forbid `unsafe`.
- Gameplay modules are moving from `sm/src/game/` into `sm_game` behind
  narrow host traits that `sm` implements. Moved code that still reads the
  RAM image is relocated, not owned: `tools/cycle_report.py` counts those
  files and sites.
- `sm_runtime` holds the shared scanline renderer, which the emulated PPU
  also uses, and Scene and message consumers. Audio is still in `sm`.
- `sm_app` is still the SDL2/OpenGL desktop built on `sm`, not the
  `winit`/`wgpu`/`cpal` runtime described here.
- No part of the game runs without the emulated backend, and no
  [validation gate](VALIDATION.md) has been run. [REVIEW](../REVIEW.md)
  holds the current milestone phase.

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

| Current responsibility | Destination | Removal condition | Status (2026-10-30) |
| --- | --- | --- | --- |
| `sm/src/game/`, `game_state.rs`, `ram_map.rs` | `sm_game`: owned Session and private gameplay modules | Complete producer/consumer cohorts use typed state; remaining RAM consumers have migrated. | Moving: `sm_game` has 363 files, 88 of them still on compat RAM or registers. |
| `main.rs`, `host.rs`, `config.rs`, host portions of `rtl.rs` | `sm_app`: input, clock, lifecycle; `sm_runtime`: devices/storage | Gameplay consumes recorded values and returns owned records without host policy. | `main.rs` and `config.rs` are in `crates/sm_app`; `host.rs` and `rtl.rs` remain in `sm`. |
| `platform/`, PPU/DMA drawing and effects | `sm_runtime`: our renderer/shaders; `sm_game`: Scene extraction | Complete drawing-to-pixels behavior is preserved; remaining game/oracle backend consumers are accounted for. | `platform/` is in `crates/sm_app`; the shared scanline core is in `sm_runtime`. |
| `spc_player.rs`, `native_audio.rs`, SNES audio/DSP | `sm_runtime`: our sequencer, voices, mixer/output | Cue order, instruments, pitch, envelopes, voice rules and echo have independent checks. | Still in `sm`; `sm_data` has a `Cue` record. |
| ROM tables, content interpretation, state formats | `sm_data`: validated definitions/records; `sm_tools`: offline importer/compiler | Prepared content and explicit schemas replace ROM layout as the public interface. | Records in `sm_data`, imported from the cartridge at load; no `sm_tools` yet. |
| `verify.rs`, `verify_dll.rs`, `cpu_infra.rs`, `verify/`, comparison tools | Development oracle; target `sm_tools`: replay/scenarios/capture | Equivalent observations/coverage exist before retiring the working comparison path. | Still in `sm` and `tools/`. |

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
