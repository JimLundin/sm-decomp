# Samus movement physics
Status: claimed
Blocked by: None
Triage: ready-for-agent

Translate a substantial bank 90 movement physics family: gravity/liquid state,
jump/wall/bomb/knockback initialization, running momentum, base acceleration,
horizontal speed and movement bookkeeping, pose alignment and related state.
Scope/dependencies: /tmp/sm-port-next-samus.md. Preserve original ROM table reads,
WRAM aliases and fixed-point widths. Differential verify actual upstream C and
independent worked numeric cases; unsafe malformed domains excluded before C.
Own crates/port/src/samus.rs, tests and tools/port/samus.json. Shared harness
extension dependency is coordinated separately. Collision callback/interpreter
dependencies remain explicit future work rather than returning defaults.
