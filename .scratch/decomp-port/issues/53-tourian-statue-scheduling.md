# Tourian statue effect scheduling
Status: resolved
Blocked by: 52
Triage: ready-for-agent

Port particle/soul/splash/statue preinstructions, splash and decoration
initialization, and particle/tail spawn opcodes. Compare actual C full WRAM,
including water crossings, wrapping motion, allocation exhaustion and same-frame
scheduling. Offline coverage; live gameplay ownership remains open.

## Completion evidence

Implemented five preinstructions (B977/B982/B9FD/BA37/BA42), four initializers
(B87A/B93E/B951/B964), and particle/tail spawn opcodes B7EA/B818. Checked-in
`tools/port_tourian_fixtures.py` rebuilds fixtures; future Tourian fixture work
uses this entry point. Independent Rust tests distinguish soul bit-eight removal
from particle high-byte removal and check HDMA scrolling/finished flag retention.

538 actual-C full-WRAM/raw-return cases pass (364 preinstruction, 93 scheduler,
72 allocation, nine opcode/control) at
`out/port/tourian-statue-scheduling/20261011T041114.735057Z/verdict.json`.
Full port tests, formatting and strict Clippy pass. Offline evidence only;
these paths have not been exercised in live gameplay.

Cleanup: `port_cleanup.py --root out/port/tourian-statue-scheduling --apply`
reclaimed zero bytes; `--deduplicate --apply` reclaimed 78,643,200 snapshot
bytes. The initial rejected fixture-address run is retained and cleaned too.
No implementer worktree created. Accepted evidence authenticated after cleanup;
immutable source/ROM dependencies retained. 35 GiB free.
