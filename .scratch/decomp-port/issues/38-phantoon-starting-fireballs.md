# Phantoon starting fireball lifecycle
Status: resolved
Blocked by: 34
Triage: ready-for-agent

Translate radial starting-fireball initialization 993A and fall/pause/bounce
preinstructions 9981/99BF/9A01/9A44. Preserve byte sine multiplication, area
positions, timer underflow, frame parity, collision ordering and script changes.
Verify allocation and complete scheduler transitions against actual pinned C.

## Completion evidence

Implemented radial initializer 993A, falling 9981, pause 99BF, bounce 9A01 and
terminal no-op 9A44 through public allocation/preinstruction/frame dispatch.
Radial coordinates retain byte-width sine products; landing preserves low
property bits, pause honors underflow/frame parity, and vertical bounce returns
before horizontal collision until the third hit.

Actual pinned bank86 full-WRAM/raw-return comparisons pass 449 cases (64
allocation, 192 preinstruction, 192 frame, one sleep control) in
`out/port/phantoon-starting-fireballs/20261011T034514.859854Z/verdict.json`.
Independent Rust tests cover landing, eight-frame pause/parity launch and third
bounce termination. Full port suite, formatting and strict Clippy pass.
Offline fixture equivalence only; full Phantoon boss/live ownership remains open.

Initial run rejected a script control copied from a synthetic ROM manifest while
using the original ROM. Replaced with actual-ROM sleep control and reran fully;
retained failure evidence.

Cleanup: scoped `port_cleanup.py --apply` reclaimed zero bytes;
`--deduplicate --apply` reclaimed 80,871,424 snapshot bytes. Rejected run also
cleaned (zero bytes). No implementer worktree created; immutable ROM/source/run
dependencies retained, final verdict authenticated after cleanup. 36 GiB free.
