# Generic enemy instruction handlers
Status: claimed
Blocked by: 06
Triage: ready-for-agent

Translate generic enemy instruction family with exact banked ROM pointer
semantics, byte-offset enemy records, timers, control flow and actual handlers.
Explore ProcessEnemyInstructions dependency closure; record unresolved bank
dispatch explicitly rather than silently returning defaults. Context pointer:
/tmp/sm-port-enemy-closure.md. Own enemy_instructions.rs and instructions.json.
Verify against actual compiled C handler bodies and independent expected states.
