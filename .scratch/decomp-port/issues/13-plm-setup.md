# Native PLM setup and collision block families
Status: claimed
Blocked by: 11
Triage: ready-for-agent

Translate native SpawnPLM slot allocation/header reads and a substantial setup
dispatch family: quicksand/sandfalls, respawning shot/bomb/speed/crumble blocks,
extensions, scroll/touch, grapple and selected room block setup. Context:
/tmp/sm-port-plm-closure.md. Preserve full WRAM, PLM pointer/slot invariants,
mutable collision displacement and exact setup return flags. Unsupported setup
must be classified before mutation if the host chooses measured C execution;
never retry C after partial failed Rust execution. Own plm.rs/tests/plm.json.
