# Native pirate host callers
Status: resolved
Blocked by: 85
Triage: ready-for-agent

Translate actual B2EF2A/EF5D laser and B2F564 claw instruction callers, preserving
currentenemy/sourceROM offsets, point/r22/rect writes and explicitwithgfxhost
allocation. ActualC fullWRAM/rawreturns/hoststate and bankB2ROMptr normalization,
independentchecks/fullchecks/cleanup required. No wholeenemyAI/live claim.

## Completion evidence

Implemented native bankB2 EF2A/EF5D laser callers and F564 claw caller through
explicit ProjectileHost. Source enemy is cur_enemy_index, independent of unused k.
Laser point/r22 and claw rectangle persist on allocation refusal; left header read
precedes refusal and parameter is overwritten as in C. Gfx allocation uses source
enemy tile/palette OR. Claw consumes three pinned B2 ROM operands and returns jp+3.
Malformed low-half/out-of-extent claw pointers fail before host writes.

Optional per-routine pointer_bank supports84/86/B2 while preserving old defaults.
Regression checks mixed pointer returns, null/integer pointers and rejects a pointer
from the wrong actual bank and invalid config. Independent Rust checks cover
fullslots context/parameters/returns and successful gfx/claw wrap allocation.

Authenticated actual-C evidence: `out/port/pirate-host-callers/20261011T071437.042816Z/verdict.json`
577 cases (192 per B2 caller plus existing86 control), complete WRAM, raw normalized
returns and eight explicit host words match. Covers all32 canonical source enemies,
differing unused argument, fullslot refusals, signed coordinate wrapping, six real
claw script operand sites, gfx aliases, parameter and audio quota/disabled states.
This is caller-level offline evidence; no whole enemy AI or original-ROM live claim.

`cargo test -p port --quiet`, `cargo fmt --all -- --check`,
`cargo clippy -p port --all-targets -- -D warnings` passed.
`PYTHONPATH=tools python3 -m unittest test_port_plm_instructions test_port_family -q`
passed12 tests. Temp-directory regression evidence is automatically removed.

Cleanup: `python3 tools/port_cleanup.py --root out/port/pirate-host-callers/20261011T071437.042816Z --apply`
reclaimed0 bytes; same utility `--deduplicate --apply` reclaimed178,388,992 snapshot
bytes. Retained evidence reauthenticated with verify_plm_instruction_evidence.
Remaining disk6.9GiB. No worktree created. Source/harness frozen pending root
integration review; no implementer commit made.
