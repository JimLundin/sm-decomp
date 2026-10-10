# Generic enemy mechanics
Status: resolved
Blocked by: None
Triage: ready-for-agent

Translate substantial generic enemy geometry/movement/angle/offscreen mechanics
from bank A0. Record byte-offset record domain and original ROM dependencies,
preserve wrapping positions and aliases, and compare full WRAM and returns with
actual pinned C. Scope/dependencies: /tmp/sm-port-next-enemy.md.
Own crates/port/src/enemy.rs, tests and tools/port/enemy.json. Do not conceal
untranslated instruction handlers or collision callbacks with defaults.

## Completion evidence

Implementation 795aff1 merged in 7b7ca39, public module registered ef94568.
Fresh seal: out/port/sealed-enemy/20261010T195837.090810Z/verdict.json, 32 routines
and 2,452 actual pinned-C full-WRAM/return fixtures. Nine independent Rust tests
pass, including promoted vertical index rejection before unchecked C execution,
fixed-point sine fractions and SNES byte-divisor angle quirks. Full slope and
bank-specific enemy instruction dispatch remain subsequent explicit work.
