# Generic enemy mechanics
Status: claimed
Blocked by: None
Triage: ready-for-agent

Translate substantial generic enemy geometry/movement/angle/offscreen mechanics
from bank A0. Record byte-offset record domain and original ROM dependencies,
preserve wrapping positions and aliases, and compare full WRAM and returns with
actual pinned C. Scope/dependencies: /tmp/sm-port-next-enemy.md.
Own crates/port/src/enemy.rs, tests and tools/port/enemy.json. Do not conceal
untranslated instruction handlers or collision callbacks with defaults.
