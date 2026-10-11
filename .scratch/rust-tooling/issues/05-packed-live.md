# Native packed live orchestration

Status: resolved
Blocked by: 01, 03, 04

Replace live observer generation, authenticated predecessor admission, four fresh
subject processes, private capture stores, shared publication and inspection with
Rust. Preserve all ordered effects, state snapshots and failure diagnostics.
Frozen authentication is implemented; source-current live admission remains
separate and must preserve the old gate's exclusivity/nonzero execution checks.

## Completion evidence

Final four9000-frame typed packed run passed and authenticated after cleanup;
see [completion.md](../evidence/completion.md). This supersedes pending notes below.

Native direct system replay and batch system/audio replay passed four independent
1000-frame subjects, with identical complete state/effects and exclusive actual
Rust execution. Batch report is `out/port/rust-migration/native-live-batch/batch.json`;
direct verdict is `out/port/rust-migration/native-live-direct/1791733093942375279-2/verdict.json`.
Native family and PLM ABI controls cover bad extents, aliases and contained panics.
Remaining adversarial live controls and final cleanup are pending.

## Typed evidence migration follow-up

2026-10-11 utilities section replaced producer JSON metadata with the typed
`evidence::Provenance`, `Pointer`, `Inventory` and receipt structures. New
completed run payloads explicitly use version 2 and canonical TOML provenance;
observation tree identities remain independent of producer metadata. Capture,
private-store publication, retained-reader authentication and extraction use
`evidence.toml` / `sm-evidence-v2`. Unsupported legacy plain/v1 readers are not
new-code adapters; original sealed readers remain retained for old evidence.

Native evidence library/CLI controls passed (`/tmp/sm-native-data-tests.log`).
Packed/observation pipeline tests now exercise current v2 real stores and typed
TOML protocol, with changed pointer/reader/binary and incomplete inventory
controls. Final integrated pipeline runs and cleanup remain with orchestrator.

Integrated typed controls subsequently passed in the orchestrator's
`/tmp/sm-root-migration-controls-3.log`: four retained observation authentication
controls, four private captures/shared publication with failed producer/timeout
retention, and routine admission refusal. This confirms the new native producer
and sealed-reader protocols work together. Source-current live replay/cleanup
remain orchestrator completion work.

2026-10-11 family section migrated generated predecessor admission, live counters,
four-subject orchestration and verdicts to typed records plus version-2 TOML pointers.
Exclusive C/Rust replacement ownership, positive dispatcher execution, full
observation equality and current source authentication remain mandatory. Dispatcher
bank/signature controls and counter double-execution/duplicate/zero-execution
controls are native unit tests. Current-source integrated live replay and cleanup
remain orchestrator-owned; earlier JSON evidence does not admit changing sources.
