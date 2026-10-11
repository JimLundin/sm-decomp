# Authenticate and return packed inventory in one traversal

Status: resolved
Blocked by: 02
Triage: ready-for-agent
Type: task

Add a verified-inventory operation that reads run/provenance, restores and hashes
each member once, and checks file/byte counts against the run record. Use it in
the inventory CLI and after capture's commit/reopen. Keep independent precommit
closure verification and Python staged-file equality before pointer publication.

Current capture restores members in put_run, reopened verify_run, and tree_inventory.
Inventory authentication does verify_run plus tree_inventory. Eliminate the latter
duplicate traversal without trusting stored member hashes in place of restored bytes.

## Acceptance

- Frozen inventory/member hashes remain identical across reader versions.
- Missing/corrupt/truncated packs, incorrect run counts and changed staged files fail.
- Old reader binaries/pointers remain usable; new reader is sealed per run.
- Compare capture sealing and authentication on frozen short and long subjects.
- Keep evidence authenticatable after cleanup.

## Completion evidence

Rust verified_inventory authenticates restored bytes and counts in one traversal; capture/inventory use it. Eleven real evidence tests cover existing frozen formats, corrupt/truncated packs and remapped imports. Serial/parallel observations remain exactly equal; final optimized reader authenticates retained evidence.

Integration evidence and actual measurements: [implementation.md](../implementation.md).
Final serial/parallel 45000 integration passed and reauthenticated after cleanup.
Scoped cleanup and deduplication finished (zero additional bytes on final passes);
prior scoped snapshots reclaimed 1768292352 bytes and disposable builds reclaimed
843648190 bytes. Shared evidence dependencies preserved; final free bytes
256302972928. Integration branch: `decomp-pipeline-performance`.
