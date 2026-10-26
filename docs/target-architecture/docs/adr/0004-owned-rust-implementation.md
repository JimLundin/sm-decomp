---
status: accepted
---
# Write and own the game and engine in Rust

After reviewing `sm-decomp` and its recovered-C restart, the user explicitly chose a standalone implementation with our own gameplay, scheduling, collision, rendering, sound generation, content handling, and persistence. Write these in straightforward Rust, with our own GPU shaders where required; use ordinary structs, enums, functions, and private modules rather than adopting an external game engine, ECS, emulator, or recovered-game runtime.

Small platform libraries such as `winit`, `wgpu`, and `cpal` are explicitly allowed for window, GPU, and audio-device access. They do not supply the game's rendering rules, sequencer, mixer, or gameplay. The five packages in the design are our own workspace code. Original source, emulators, and the old project may inform optional reference capture and offline content import, but are not linked, loaded, or called by the new game and are not prerequisites for its normal build or play from prepared content.

This provides control over the implementation and removes the old runtime's ownership constraints. It also makes rendering, sound fidelity, numerical behavior, and complete gameplay coverage our implementation work; existing comparison results guide that work without validating the new engine automatically. The external-ECS alternative is retained only as historical design context.

On 2026-10-22 the user clarified that Windows workers may retain the current Rust implementation's emulated backend and built-in comparisons throughout staged migration. The earlier C-backed restart is archived. Runtime independence remains the destination and the completion criterion for a new-engine slice; it is not a demand to remove a working backend before replacement evidence exists. [Migration](../../README.md#responsibility-migration) defines adoption and current-work constraints.
