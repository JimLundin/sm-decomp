# Generic enemy instruction handlers
Status: resolved
Blocked by: 06
Triage: ready-for-agent

Translate generic enemy instruction family with exact banked ROM pointer
semantics, byte-offset enemy records, timers, control flow and actual handlers.
Explore ProcessEnemyInstructions dependency closure; record unresolved bank
dispatch explicitly rather than silently returning defaults. Context pointer:
/tmp/sm-port-enemy-closure.md. Own enemy_instructions.rs and instructions.json.
Verify against actual compiled C handler bodies and independent expected states.

## Completion evidence

Implementation 32aef71 contains all 14 generic handlers and a bounded generic
interpreter. Actual pinned-C handler comparison: 1,344 full-WRAM/normalized
banked-pointer cases, out/port/instructions-agent-verify/20261010T200807.698971Z/verdict.json.
100 independent synthetic interpreter cases match actual ProcessEnemyInstructions
linked with its complete real C dispatch tree, retained at
out/port/interpreter-agent-verify/20261010T200808.874400Z/verdict.json.
Eight synthetic public-seam tests cover word/byte timer width, odd pointers,
packed VRAM queue, sleep/wait, unsupported bank dispatch and execution bounds.
The bounded interpreter is explicitly partial: bank-specific instructions return
an error and it is not installed as the whole live interpreter replacement.
Generic handlers may be installed independently with observed hit evidence.
