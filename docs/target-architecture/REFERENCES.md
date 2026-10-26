# Sources and evidence limits

Sources checked during this design session, 2026-10-22. Links to `latest` and `master` can change; pin dependency versions and capture source provenance when implementing. Architectural recommendations are judgments, not findings established by these libraries' documentation.

| Primary source | What it supports |
| --- | --- |
| [wgpu Surface](https://docs.rs/wgpu/latest/wgpu/struct.Surface.html) | Surface configuration/acquisition constraints, including nonzero dimensions and surface-image lifetimes |
| [wgpu CurrentSurfaceTexture](https://docs.rs/wgpu/latest/wgpu/enum.CurrentSurfaceTexture.html) | Current acquisition outcomes; map these into project-level recovery outcomes rather than copying an older `SurfaceError` interface |
| [winit ApplicationHandler](https://docs.rs/winit/latest/winit/application/trait.ApplicationHandler.html) | Event-loop lifecycle, suspension/resume, redraw integration; the observed documentation was winit 0.30.13 |
| [CPAL](https://docs.rs/cpal/latest/cpal/) | Audio-device/callback integration; it does not itself specify Super Metroid's sequencer or mixer |
| [hecs](https://docs.rs/hecs/latest/hecs/) | Historical private-ECS alternative; excluded by the later decision to own the implementation and allow platform dependencies only |
| [Tiled custom-property documentation](https://raw.githubusercontent.com/mapeditor/tiled/master/docs/manual/custom-properties.rst) | Typed properties, classes/enums, inheritance, and editor export support for the proposed authoring workflow |

The current `wgpu` acquisition documentation differs from older examples using `SurfaceError`. This design therefore uses stable project concepts such as skip, reconfigure, and recreate. None of the pseudocode in this pack is a claim to compile against an unspecified latest release.

## Project baseline for the migration handoff

The current project was inspected at [`JimLundin/sm-decomp` commit 498c8a4d](https://github.com/JimLundin/sm-decomp/tree/498c8a4dd9cd6268de1e30b8cc08f175a43b765e). Its [README](https://github.com/JimLundin/sm-decomp/blob/498c8a4dd9cd6268de1e30b8cc08f175a43b765e/README.md) describes maintained Rust gameplay and emulated hardware after a one-time c2rust translation. The earlier mixed C/Rust restart is archived. Its [plan](https://github.com/JimLundin/sm-decomp/blob/498c8a4dd9cd6268de1e30b8cc08f175a43b765e/docs/PLAN.md) records the active reduction objective, Windows resource/security constraints, and observed comparisons with their coverage limits.

[Migration](README.md#responsibility-migration) maps those responsibilities to the accepted destination. This source inspection did not rerun engine builds or comparisons.

## Historical source-size observations

These measurements were collected earlier in this conversation and retained as context. They describe reconstruction artifacts and different counting units, not original Nintendo source size or a rewrite budget.

| Observation | Scope and pinned source |
| --- | --- |
| 76,500 C lines | Nonblank, noncomment lines across 32 game-bank files in [`snesrev/sm` at 578f90b](https://github.com/snesrev/sm/tree/578f90b3cc49557bb70060ad033bb90b8cf8ac50) |
| 106,080 C/header lines | Full `src` scope including headers, platform, and emulation code, excluding `third_party`, at the same revision |
| 132,534 assembly instructions | Recognized 65C816 instruction lines in bank files, excluding labels/data, in [`sm_disassembly` at 7af131e](https://github.com/InsaneFirebat/sm_disassembly/tree/7af131ebc328359e852e923603163a329c7aa7af) |

Assembly instructions and C source lines are not interchangeable units. Reconstruction often preserves original control structure and hardware-facing work that a rebuild may express differently. Conversely, a rebuild adds its own compiler, tooling, diagnostics, lifecycle work, shaders, and tests.

These counts establish neither bloat nor a minimum. They do not justify dividing by a Rust language multiplier. The previous 30–60k runtime estimate has not been validated and is retired as a design constraint.
