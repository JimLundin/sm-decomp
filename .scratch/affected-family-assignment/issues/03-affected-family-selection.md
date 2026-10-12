# Deepen affected Offline family selection

Status: open
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

May proceed alongside ticket04 once the agreed shared result types are fixed.
