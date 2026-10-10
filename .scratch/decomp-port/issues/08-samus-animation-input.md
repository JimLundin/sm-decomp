# Samus animation and input transitions
Status: resolved
Blocked by: 05
Triage: ready-for-agent

Translate substantial complete Samus animation delay/bytecode and speed-booster
state, liquid FX/splash/bubble routines, palette copying/loading and speed-boost
cancellation, pose-table input matching/cancel/momentum and input wrappers.
Read /tmp/sm-port-samus-closure.md for actual dependency closure. Preserve genuine
audio/physics dependencies, ROM tables and WRAM aliases. No fabricated collision
callbacks or default animation handlers. Differential actual-C evidence and
independent state-machine tests required. Own samus_control.rs and control.json.

## Completion evidence

Implementation b41fab8 merged and exposed publicly in 0479c7b. Expanded manifest
68d3c49 includes 46 public routines, 4,528 actual-C full-state cases. Exact agent
evidence /tmp/sm-control-commands/20261010T200433.602824Z/verdict.json exercises
all available animation opcode classes and booster stages. Five independent
synthetic-ROM tests pass, including loop/pose bytecode and booster frame timer.
Wrong echo-release speed candidate rejects at $0AC0. Fresh root family seals and
live coverage remain separate verification; collision/frozen input dependencies
are explicitly subsequent work.
