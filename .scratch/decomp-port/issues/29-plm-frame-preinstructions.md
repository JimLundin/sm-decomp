# PLM preinstruction frame ordering
Status: resolved
Blocked by: 28
Triage: ready-for-agent

Verify supported door and room preinstructions through actual PlmHandler_Async,
including deleted slots, timer underflow and same-frame link wakeups. Correct
frame discrepancies using independent Rust examples and full-WRAM C cases.
Preserve control-only boundaries and complete run cleanup before resolution.

## Completion evidence

Removed incorrect header-deletion skip: C decrements the instruction timer and
executes an expiring script even after the preinstruction deletes the slot.
Independent Rust regression covers instruction timers 0, 1 and 2. Added all
actual dispatcher no-ops, including hardcoded allocation default 8469.

Actual-C full-WRAM/raw-return comparison passes 756 cases, including 408 frame
cases (396 new) covering thirteen handlers and twenty no-ops, link wakeups,
timer underflow and deletion. Evidence:
out/port/plm-frame-preinstructions/20261010T225440.281820Z/verdict.json.
Full port suite, focused regression, fmt and Clippy pass. Offline only.

Cleanup --apply reclaimed 0 bytes; --deduplicate --apply reclaimed 309,067,776
snapshot bytes within out/port/plm-frame-preinstructions. Evidence authenticated
after cleanup; sealed assets retained, 39GB free. No implementer worktree created;
read-only exploration notes are at /tmp/sm-plm-frame-review.md. Synthetic fixture
ROM/manifest retained in out/port/plm-frame-fixtures as small reusable inputs.
