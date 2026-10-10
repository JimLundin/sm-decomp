# Samus movement physics
Status: resolved
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

## Completion evidence

Implementation f0793f1 merged in 0e6a121 and registered publicly in ef94568.
Fresh integration-source seal: out/port/sealed-samus/20261010T195837.084762Z/verdict.json,
27 routines and 5,184 full-WRAM/return comparisons against actual pinned C.
Six independent synthetic-ROM public Rust tests pass. Wrong jump-direction
candidate rejects at WRAM $0B36; evidence /tmp/sm-samus-reject.
Live exercising and capture coverage are recorded separately; collision and
animation handlers remain explicit subsequent tickets rather than placeholders.
