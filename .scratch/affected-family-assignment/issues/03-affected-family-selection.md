# Deepen affected Offline family selection

Status: claimed
Blocked by: 02
Triage: ready-for-agent
Type: task

## Required work

Implement a Rust selection module and read-only explanation through the same
seam consumed by batch. Include changed paths, selected/excluded families,
dependency matches and uncertainty reasons. Handle tracked/untracked edits,
renames/deletions, manifests, Fixture/ROM assets and current module discovery
namespace. Validate planning metadata used for exclusions; incomplete closure or
unknown contracts remain affected. Avoid full unrelated artifact authentication
as the selection mechanism. Preserve actual current-source authentication for
reuse/admission. Existing selection breadth must no longer be forced solely by
every ordinary port-source or family-manifest edit.

Prove small actual-C/Rust families exercise only affected verification work;
assert skipped records/artifact counts and reused verdicts, not accumulated batch
membership alone. Do not alter fixture execution or evidence formats to achieve
selection savings.

## Comments

Claimed by the selection implementer on branch `affected-selection` in
`/tmp/sm-affected-selection`, based on `f2ffe4a`.

May proceed alongside ticket04 once the agreed shared result types are fixed.
Claimed by selection implementer on `affected-selection` at
`/tmp/sm-affected-selection`; integration owner retains the authoritative status.

## Implementation checkpoint

`affected.rs` owns read-only typed reports and current conservative Rust module
discovery. `batch plan` returns those reports without creating outputs, locks or
metrics; changed-since verification consumes the same selected-family result.
Untracked/NUL paths, removed namespace members, transitive imports, family
manifests and captured/ROM inputs are accounted for. Dynamic or missing inputs
remain explicit family uncertainty. Current-source authentication is unchanged.

Eight focused real-Git selection controls passed with
`cargo test -p pipeline --locked --offline --test affected --target-dir /tmp/sm-assignment-target`.
Focused formatting and `git diff --check` passed. Strict Clippy exposes existing
fixture_execution/metrics lint failures outside this change; Clippy passed with
those existing lint kinds allowed. The integration owner supplies `mod affected`
in main.rs and performs actual-C fixture verification after the source freeze.

No verification run directories or retained evidence were generated: test-only
temporary Git repositories were removed by their guards. Cleanup reclaimed no
run artifacts; shared `/tmp/sm-assignment-target` is retained for the integration
owner's checks. Available disk space was 216,895,090,688 bytes. This ticket remains
claimed pending actual-C integration, final cleanup and implementer worktree
removal by the integration owner.
