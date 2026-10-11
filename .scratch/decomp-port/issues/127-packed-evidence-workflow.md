# Adopt packed evidence in continued decomp workflow
Status: resolved
Blocked by: None
Triage: ready-for-agent

Use merged packed evidence defaults for future live validation. Update decomp
workflow instructions and cleanup retention guidance; verify pipeline controls.
Preserve separate actual-C family checks and historical evidence.

## Completion evidence

Confirmed merged live runner defaults to packed and batch commands inherit it.
Updated docs/DECOMP_PORT.md with family verification, packed live capture,
cleanup, authentication and extraction guidance. Updated
docs/agents/run-cleanup.md to retain shared objects, pointers and reader.
Five pipeline controls, 16 live tests (one existing skip), three cleanup tests
and git diff --check pass. Tests remove their temporary runs; no durable live
run created. Shared target cleanup apply/deduplicate reports zero eligible bytes;
retain existing shared build and evidence dependencies. Available disk
263,685,861,376 bytes. Prior real packed/plain verification is recorded in
trace-storage-study/issues/06-pipeline-integration.md.
