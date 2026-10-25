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

## Reference by task

| Need | Read |
| --- | --- |
| Map current responsibilities to destination packages | [Migration](MIGRATION.md) |
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
