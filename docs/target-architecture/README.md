# Super Metroid engine design

A modern Rust implementation built exclusively for Super Metroid, preserving its gameplay and content. This is a design pack, not an implemented engine or a claim of verified fidelity.

Open the [interactive report](report.html) for the simplified overview: Architecture, Execution, and Build plan. Select a package for its module contracts, or open the supporting notes for decisions, ownership, saves, and recovery. Everything required by the HTML is embedded; it works directly from disk.

The agreed architecture combines an owned deterministic `Session`, our GPU renderer and sound engine, and an application that coordinates validated content revisions, devices, and development tools. All five packages are our Rust code, organized into straightforward private modules. Small platform libraries such as `winit`, `wgpu`, and `cpal` provide device access. The completed target game runs without SNES emulation, recovered-C gameplay, or external engine/ECS modules.

Map Rando integration is a core requirement, reaffirmed by the user on 2026-10-23. The small modular core must support custom sprites/palettes, configurable room and door graphs, and explicit opt-in mechanics such as re-spin and other quality-of-life options. Vanilla remains the first fidelity profile. See the [integration contract](ARCHITECTURE.md#map-rando-integration); this is a target requirement, not a claim that the current migration already supports Map Rando.

The user confirmed a staged migration on 2026-10-22. Windows workers may keep the current Rust implementation's emulated backend and original-ROM comparisons while replacing responsibilities. Continue the current tasks and checks in [the work plan](../PLAN.md). Read [Migration](MIGRATION.md) before changing subsystem boundaries or choosing a backend; it explains Windows resource limits, comparison seams, and the existing reduction objective.

From the repository root in PowerShell, open the report with:

```powershell
Start-Process (Resolve-Path .\docs\target-architecture\report.html).Path
```

The report opens locally without a server or installed tooling. Read the documents below for implementation details.

| Document | Purpose |
| --- | --- |
| [Migration](MIGRATION.md) | Current implementation, Windows workflow, staged adoption, and replacement evidence |
| [Architecture](ARCHITECTURE.md) | Scope, alternatives, package graph, domain structure, rendering, audio, and authoring |
| [Contracts](CONTRACTS.md) | Ownership, public interfaces, Tick phases, queues, failures, and lifecycle rules |
| [Validation](VALIDATION.md) | Scenarios, evidence, performance measurements, and implementation gates |
| [Glossary](GLOSSARY.md) | Meanings of the project's domain terms |
| [Decision records](docs/adr/0001-owned-session.md) | The few decisions with substantial downstream cost |
| [References](REFERENCES.md) | Primary sources, historical counts, and limits of the evidence |
| [Report checks](REPORT-CHECKS.md) | Browser, keyboard, responsive, accessibility, and offline verification |

This handoff contains the specification and self-contained report. It adds no engine code, dependencies, build requirements, or new process tooling.

Decision records: [owned Session](docs/adr/0001-owned-session.md), [prepared revision cutovers](docs/adr/0002-revision-cutovers.md), [observable rules](docs/adr/0003-observable-rules.md), and [our Rust implementation](docs/adr/0004-owned-rust-implementation.md).

The user has accepted the architecture, purpose-built scope, preservation of gameplay/content, and ownership policy above. The simplified report retains its three-section presentation and reflects architecture revision 3 with the migration clarification. The completed target engine builds and plays from prepared content independently of the comparison backend. That milestone does not interrupt work on today's emulated runtime. The exact reference release, minimum desktop hardware, original slowdown policy, and numerical performance budgets remain evidence gates. The retired 30–60k runtime estimate is separate from the current project's below-100,000-line objective, which remains active.
