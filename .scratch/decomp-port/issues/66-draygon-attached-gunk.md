# Draygon attached gunk
Status: resolved
Blocked by: 65
Triage: ready-for-agent

Port 8D99 attach opcode and 8DCA follow/release scheduling. Preserve power-bomb
ordering, contact short circuit, timer wrapping and signed speed-divisor clamp.
Verify actual C full WRAM and cleanup. Allocation/live ownership remain open.

## Completion evidence

Implemented 8D99 attachment and 8DCA follow/release. Preserves power-bomb
collision first, signed divisor limit, properties/timer resets, contact short
circuit, wrapping attachment timer and signed divisor decrement clamp.
Independent Rust test checks contact release skips position/timer writes.

1,281 actual pinned C full-WRAM/raw-return cases pass: 897 Draygon helper/
scheduling regressions plus 128 attachment opcodes, 128 follow/release
preinstructions and 128 scheduler frames. Evidence:
`out/port/draygon-attached-gunk/20261011T043251.095224Z/verdict.json`.
Rebuild with `tools/port_draygon_attached_fixtures.py`. Full port tests, fmt
and strict Clippy pass. Allocation and live gameplay ownership remain open.

Cleanup applied/deduplicated `out/port/draygon-attached-gunk`: zero ordinary
bytes, 243,007,488 snapshot bytes reclaimed. Accepted evidence authenticated
after cleanup; immutable source/ROM snapshots retained. No implementer worktree
created. 32 GiB available disk space.
