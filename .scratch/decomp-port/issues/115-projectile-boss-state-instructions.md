# Projectile boss state instructions
Status: resolved
Blocked by: None
Triage: ready-for-agent

Translate CAEE purple breath state clear and B3B8 Kraid state conditional branch.
Preserve extra enemy RAM aliases and native pointer return. Verify actual-C
full WRAM/return, tests and scoped cleanup; live ownership remains separate.

## Completion evidence

Both native opcodes and wrappers translated. Exact Mother Brain RAM alias at
786A and Kraid extra-enemy sign-bit branch preserved. All 72 actual-C full-WRAM/
return cases pass:
out/port/projectile-boss-state-instructions/20261011T100918.324693Z/verdict.json.
Full port suite, strict Clippy and diff check pass. Ordinary cleanup zero;
dedup/authentication results recorded below. No worktree created; retained
binary/source/pinned ROM dependencies. Offline only; live ownership separate.
Integration branch decomp-port-integration.

Dedup reclaimed 18874368 bytes; all 72 cases authenticate after cleanup.
Free disk 268811522048 bytes.
