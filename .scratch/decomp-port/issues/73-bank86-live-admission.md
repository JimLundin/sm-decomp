# Bank86 dispatcher evidence live admission
Status: open
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
