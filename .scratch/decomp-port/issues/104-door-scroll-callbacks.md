# Door scroll callbacks
Status: resolved
Blocked by: None
Triage: ready-for-agent

Translate all native constant scroll-writing door callbacks, including UNUSED
callbacks, with exact byte writes and preserved surrounding WRAM. Keep recovered
names and actual indices even where names suggest a different index/value.
Require actual-C full-WRAM cases, cleanup and authenticated evidence.

## Completion evidence

All 78 native constant scroll callbacks are represented in readable Rust tables,
including both UNUSED callbacks. All 312 actual-C full-WRAM cases pass:
out/port/door-scroll-callbacks/20261011T094605.605516Z/verdict.json.
Full port tests and strict Clippy pass. Independent full-WRAM test verifies
recovered name/index discrepancies and byte-only writes with neighbors intact.
Ordinary cleanup zero; dedup reclaimed 94896128 bytes; all cases authenticate
afterward. Free disk 269884030976 bytes. Retained source, binary and pinned ROM
dependencies; no implementer worktree created. Offline scope only; movie
ownership and whole-game translation remain separate. Branch decomp-port-integration.
