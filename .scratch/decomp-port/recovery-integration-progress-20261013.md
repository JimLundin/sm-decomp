# Current merged verification progress

Status: Offline cases passing; final cleanup and live gates running.

All twelve selected families passed actual compiled pinned C against the merged
Rust source at the retained bounded source freeze. Their individual verdicts are
indexed by `out/recovery-session/integrated/batch.toml`. This includes native
projectile initializers/frame/ownership, rendering/two contacts, trails, camera,
room, Samus projectile frame and controlled/captured bounded Ridley instructions.
The batch is still executing automatic deduplication; no final completion or
post-cleanup authentication is inferred from these partial results.

Independent trail live replay is running at
`out/recovery-session/trails-live/1791860457650339558-98848`, 9,000 inputs,
four C/mixed processes. Complete native projectile admission is tested separately
in the full 129,712-input composed gate at `out/recovery-session/composed-live`.
Samus internally invokes the translated trail scheduler; install trail and Samus
root wrappers in separate domains to retain accurate ownership/capture counters.
The first composed batch replay refused the active batch lock before generating
run output; the subsequent direct native live command is the active gate.

Both ordinary and dedup cleanup passed on the earlier failed room run, zero
bytes. Earlier camera frozen-source evidence authenticated after cleanup.
Broad Ridley controlled6457 evidence likewise authenticated after both cleanup
passes from exact frozen source. Stale worktree metadata was retired while small
source-only snapshots and all sealed artifacts/stores remain. Integration and
live stages stay unpublished until all final cleanup/authentication gates pass.
