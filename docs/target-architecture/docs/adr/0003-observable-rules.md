---
status: accepted
---
# Preserve observable rules without copying the original runtime layout

The rebuild's acceptance target is Super Metroid's gameplay and content, including established movement techniques and order-sensitive interactions. Express those rules in typed Rust domain state, explicit numerical operations, and a reference-calibrated serial schedule; use original disassembly and reconstruction projects as evidence rather than the runtime's module graph or memory layout. This rejects general rigid-body physics and automatic cleanup of unusual movement behavior, while avoiding a CPU, bus, or register emulator as the game's architecture.

Hardware-related effects that are observable still need a deliberate representation and a test, including priority/color math, sound behavior, actor limits, and any slowdown accepted into the reference profile.
