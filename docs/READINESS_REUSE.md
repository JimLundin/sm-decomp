# Verified evidence reuse

`just readiness-reuse --predecessor out/runs/<id>` allocates a new evidence run,
verifies setup, and explains which accepted checks have unchanged dependencies.
This command only plans reuse. Readiness still requires the complete gate registry.

Each parent run calls `record_dependency_policy(run)` before validation. The
versioned policy preserves exact source path/digest/mode inventories, hardware,
compiler, input/reset/profile and build settings. Reference synchronization and
fallback parity omit only function/jump-table config and recompilation source;
all hardware, runtime, reference, subject, observer, comparator, movie and build
inputs remain dependencies. Unknown files force fresh checks. Documentation
is irrelevant only in known documentation locations and when executable inputs
remain unchanged; a document referenced by executable/build text is a dependency.
Check/lint reuse requires every executable/build input to match.

`plan_reuse(run, predecessor)` verifies a complete accepted origin and returns
fresh/reuse decisions. `apply_reuse(run, predecessor, check)` rechecks the origin
and artifacts, imports one allowed report, records the predecessor and unchanged
dependency inventory, and guards linked evidence against mutation through run end.
Generated parity, changed-entry coverage and focused instruction diagnosis always
require new candidate evidence. Runtime or hardware changes invalidate fallback
and independent candidate synchronization. Missing or uncertain proof reruns checks.

Accepted origins need all twelve mandatory gates in the shared versioned contract,
exact domain and completed scopes, preserved source, sealed owned evidence and a
verified current or historical acceptance pointer. `seal_gate_evidence(run)` binds
reports and all owned evidence immediately before finalization; callers must not
add or modify evidence after sealing. The acceptance publisher then verifies the
terminal run with `verify_accepted(owner)` before atomically publishing the pointer
from `accepted_pointer(owner, verified)`. Reuse additionally requires its current
or retained-history pointer. A standalone pass does not establish accepted lineage.

Fallback reuse independently checks full 18,000-frame CPU/clocks, complete WRAM
and framebuffer evidence, empty compilation, input/executable identities and the
successful replay command. Independent synchronization revalidates capture/source/
build/observer identities and recomputes the exact checkpoint comparison. Check/lint
requires the recorded successful `just` command and digest-bound stdout/stderr.

Checkpoint reuse uses exact candidate identities and complete cold trace/state
validation. `cache_reuse_decision` is a precheck; restoration always invokes the
paired checkpoint API. Different candidate identities rebuild cold because
cross-candidate equivalent history is unproven. The original oracle remains
separate: unchanged verified original-only attestation may be reused while the
current candidate synchronization reruns.
