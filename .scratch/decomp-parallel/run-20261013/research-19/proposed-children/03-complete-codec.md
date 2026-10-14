# Implement the complete four-routine SRAM codec

Type: task
Status: proposed; coordinator review/publication required
Origin: exact research19 claim; see [Answer](../Answer.md) and [children.toml](../children.toml).
No writable ownership, supported native scope or gameplay acceptance is granted.

One owner implements SaveToSram818000, LoadFromSram818085,
PackMapToSave81834B and UnpackMapFromSave8182E4 in the exact ticket19 triplet:
`crates/port/src/persistence.rs`, `crates/port/tests/persistence.rs` and
`tools/port/persistence.toml`. Reuse canonical memory, ROM tables and reviewed
synchronous service ABI. Shared wiring is integrator-owned; preserve other
workers' changes. [Holding](../scope-holding.toml) is unsupported until shared21
and adapter admission provide genuine direct-save/direct-load evidence.

Meaningful controls: all three legal slots and4/5/6aliases; independent lower/
upper checksum pair fallback and forbidden mixed pairs; wrapping sum; all-zero
metadata versus valid zero payload; load save-buffer writes before validation;
invalid payload-only SRAM clear/live station-area zero without persistence or
player/map changes; copied player unnamed tail; all1628payload bytes and
953preserved compressed tail; sparse map holes/Ceres clear and extra two clear
bytes; surrounding WRAM/SRAM/checksum/signature/selected-slot preservation;
exactly one ordered save-service call. Unsupported slot3/zero count/bounds
controls refuse unchanged and never run unsafe or nonterminating C.

Initial unrun gate: `cargo test -p port --locked --offline --test persistence`
(60s). Conditional merged actual-C gate after adapter registration:
`cargo run -p pipeline --locked --offline --bin sm-pipeline -- family-verify --manifest tools/port/persistence.toml --output out/port/persistence --case-jobs 4`
(3600s). New files/targets are unimplemented. Require full WRAM/SRAM/raw-return/
order and retained source/input identities; then integrator all-affected checks,
both cleanup passes, worktree retirement and reauthentication. Caller/continuation
and original-ROM acceptance remain separate.
