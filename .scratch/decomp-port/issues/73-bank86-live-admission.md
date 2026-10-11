# Bank86 dispatcher evidence live admission
Status: claimed
Blocked by: None
Triage: ready-for-agent

Recent bank86 offline coverage needs live tool support before runtime ownership
can be claimed. Extend tools/port_live.py with dispatcher-verdict authentication
through verify_plm_instruction_evidence. Exclude synthetic EprojInstruction and
EprojPreinstruction from live owning-unit discovery; they only exist in the
offline shim. Initially admit actual leaf MoveEprojWithVelocity/X/Y symbols.

## Admission constraints

Current port_live.py family admission uses verify_family_evidence, incompatible
with dispatcher verdict seals. tools/port/batch.json has no bank86 family and
port_batch.py replay only admits family/callback entries. Actual allocator and
scheduler can reject unsupported ROM states after writes; do not blindly route
these through the fatal generic ABI. Broader admission requires support
selection before committing WRAM, following positioned PLM admission semantics.

## Verification required

Authenticated dispatcher evidence, source-stable live wrapper build, independent
C/mixed process replay, per-symbol Rust hit counts, complete RAM/effects parity,
cleanup and retained authenticated evidence. Existing gameplay replay does not
exercise these new modules merely because offline cases pass.

## Investigation evidence

Read-only parallel agent inspected tools/port_live.py (family admission and
generate_family_abi), tools/port_batch.py replay_command, tools/port/batch.json,
and tools/port_plm_instructions.py bank86 shim. No current command consumes the
bank86 dispatcher seal for live admission. No new live claims made.

## Implementation and partial verification

Added repeatable `--dispatcher-verdict` admission via authenticated dispatcher
seals, exact bank86/module/source identity, exclusion of offline-only synthetic
symbols and an explicit three-leaf allowlist with ABI signature checks.
Scheduler/allocator symbols require future support selection before mutation.
The ABI imports the gameplay dependencies; selected dispatcher symbols and
per-process dispatcher counters are retained. Dispatcher admission requires
nonzero Rust dispatcher hits, even when another family runs.

Nine live-tool tests pass (one optional compiled gameplay control skipped).
New controls check synthetic filtering, unsafe scheduler rejection, real module
ABI compilation, fractional movement and invalid-slot rejection without writes.

Leaf-only 9,000-frame and 18,000-frame replays each completed four independent
C/mixed processes with matching RAM/hardware/effects artifacts:
`out/port/projectile-movement-live/20261011T055613.057872Z/verdict.json`
`out/port/projectile-movement-live/20261011T055731.062689Z/verdict.json`.
All three leaf hit counts were zero, so both correctly reject admission and
retain `state_effects_match: true`, `live_exercised: false`. Ticket remains
claimed: an exercising replay or captured gameplay trigger is still needed.
The final optional `dispatcher_symbols` verdict field was added while the
second process had already loaded its tool; both replay verdicts predate that
metadata-only addition. Final tool unit tests passed; no claim of a sealed
final-tool replay is made. Current live verdict format seals binary and
artifacts but has no harness-source seal.

Cleanup applied and deduplicated `out/port/projectile-movement-live`: zero
bytes reclaimed by utility (rejected runs are excluded from snapshot sharing).
Both target trees were automatically removed by the tool. Removed four
identified unsealed static-link archives; their byte count was not recorded.
Retained live binaries, transformed sources and parity observations. Checked
binary SHA256 and every inventoried artifact in all eight processes after
cleanup; all match. Dispatcher predecessor evidence authenticated (385 cases).
No implementer worktree created. 26 GiB available.
