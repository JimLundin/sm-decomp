# Merge packed evidence pipeline into main

Status: resolved
Blocked by: 06
Triage: ready-for-agent
Type: task

User authorizes merging the completed storage/pipeline work into local main.
Main is the initial import and an ancestor of the committed integration branch;
the pipeline dependencies therefore travel with its committed history. Preserve
ongoing uncommitted gameplay edits. Verify the committed source in an isolated
checkout, fast-forward main, clean disposable checkout/builds and record evidence.

## Completion evidence

Fast-forwarded local `main` from `68de7f7` to `fec6d38`, including the
committed pipeline dependencies. This completion record is committed on
`decomp-port-integration` and also fast-forwarded into `main`. No remote is
configured. Ongoing unstaged gameplay changes retain their exact diff hash.

Verified the committed `fec6d38` tree in `/tmp/sm-storage-main-check`:
- `cargo test -p evidence --locked --offline --target-dir out/merge-check/target`:
  ten integration tests and doc tests pass.
- `cargo fmt -p evidence -- --check`, `git diff --check`, and focused
  `cargo clippy -p evidence --all-targets --locked --offline
  --target-dir out/merge-check/target -- -D warnings` pass.
- Python live tests: 16 pass with one existing skip; cleanup tests: three pass;
  evidence pipeline tests: five pass using the retained `sm-evidence` binary.

Cleanup completed after every writer exited:
- `python3 tools/port_cleanup.py --root out/merge-check --apply`
- `python3 tools/port_cleanup.py --root out/merge-check --deduplicate --apply`
Both report zero eligible bytes. Removed only disposable, unsealed
`out/merge-check` (273110073 bytes) and `tools/__pycache__` (162136 bytes),
unlinked the temporary dependency symlink, and removed the integrated
`/tmp/sm-storage-main-check` worktree. No evidence depends on that checkout.
Retained earlier sealed evidence/source snapshots stay in their original paths.
Remaining free space after build cleanup: 263625818112 bytes (263.63 GB).

Post-integration check with ongoing gameplay edits present: `main` and
`decomp-port-integration` both point to `35945eb`. `cargo test -p port -p evidence
--quiet`, strict Clippy for both crates/all targets and `git diff --check` pass.
No new isolated checkout or sealed run created. Shared Cargo target retained
for continued development; scoped cleanup utility reports zero eligible bytes.
Available disk: 263,667,990,528 bytes. Gameplay edits remain uncommitted.
