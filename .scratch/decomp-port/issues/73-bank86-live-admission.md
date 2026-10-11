# Bank86 dispatcher evidence live admission
Status: resolved
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

## Whole-movie exercise follow-up

Late bank86 movement call sites mostly belong to Ridley/Mother Brain paths,
so attempted the complete129712input movie instead of repeated guessed short
prefixes. Added bounded frame-proportional replay timeout(max240s,frames/40)
while retaining240s build limits; final live-tool suite9passes/1optional skip.
Fresh385-case dispatcher evidence after ticket75source settled:
`out/port/projectile-movement-live-leaves/20261011T060317.623936Z/verdict.json`.
Cleanup0ordinary+92,143,616snapshotbytes; evidence reauthenticated.

Whole-movie live run:
`out/port/projectile-movement-live/20261011T060355.160063Z/verdict.json`.
Reference C process SIGSEGV at43188, before any mixed process. Ticket76records
deterministic null-call diagnosis and reset sentinel. No new Rust ownership
claim; ticket73stillclaimed pending runtime repair/exercising replay.
Cleanup/retained diagnostic authentication details are in ticket76.


## Corrected reset replay in progress

After ticket77 Rust source settled, fresh385-case leaf evidence:
`out/port/projectile-movement-live-leaves/20261011T061240.572959Z/verdict.json`.
Cleanup apply0bytes/deduplicate92,143,616bytes; retained dispatcher evidence
authenticated against current source. 25GiB free; snapshots/binaries retained.

Corrected headless host mirrors pinned comparison runtime soft-reset entry.
Full129712-frame C/mixed run:
`out/port/projectile-movement-live/20261011T061431.667466Z`.
Run remains active; no live admission or completion claim yet.


## Completion evidence

Full129712-frame replay verdict:
`out/port/projectile-movement-live/20261011T061431.667466Z/verdict.json`,
status mixed-C-Rust-integration/live_exercisedtrue/state_effects_matchtrue.
Two C and two mixed fresh processes match full RAM/hardware/effects inventories
and progress, with zero CPUfallback. Each mixed process executes native
MoveEprojWithVelocityX11times and Y13times, with corresponding C counts0.
Combined MoveEprojWithVelocity has0calls in every process; its live exercise
remains unproven. Broad allocator/scheduler admission remains excluded.
Original-ROM parity is explicitly not-tested; raw movie timing is ticket78.

Source-stable predecessor385cases authenticated. Live binary, retained host/header
seals, transformed sources and all four artifact inventories authenticate.
Ticket76/commit33ef42c fixes the C-only host reset; ticket77/commitb6a874c
adds offline drop transition coverage. Final Rust tests/fmt/Clippy and live
tool suite10passes/1optional skip pass.

Cleanup commands apply0bytes; deduplicate10,864,418,956snapshotbytes.
Additional byte-identical completed-process ROM snapshots reclaimed
3,523,215,360bytes; identical C hardwarelogs reclaimed4,967,833,433bytes,
with hashes checked before sharing. Total19,355,467,749bytes reclaimed.
Automatic target/archive cleanup finished.20GiBfree. Retain exact binary,
host/header, transformed sources, verdict/predecessors and authenticated
inventories; no implementerworktree exists. Post-cleanup inventory checks
cover all four processes.
