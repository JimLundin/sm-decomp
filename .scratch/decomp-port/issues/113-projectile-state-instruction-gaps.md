# Projectile state instruction gaps
Status: resolved
Blocked by: None
Triage: ready-for-agent

Translate 8CF6/8CFD preinstruction setters, 9475 collision bit, AF92/C8D0 Y
movement, B436 saved position and D1B6 Kago state. Verify native wrappers against
actual C full-WRAM/return across all slots and wrapping values; cleanup required.

## Completion evidence

Seven missing opcode paths and native wrappers translated. Preserves exact
preinstruction addresses, collision bit OR, wrapping Y increments, saved XY and
Kago RNG low-five-bit timer without advancing RNG. All 504 actual-C full-WRAM/
return cases pass across all projectile slots and boundary values:
out/port/projectile-state-instruction-gaps/20261011T100540.847127Z/verdict.json.
Full port suite, strict Clippy and diff check pass. Ordinary cleanup zero; dedup
reclaimed 127401984 bytes; all cases authenticate afterward. Free disk
268860596224 bytes. Retained binary/source/pinned ROM dependencies; no worktree
created. Offline scope only; live ownership remains separate. Branch
decomp-port-integration.
