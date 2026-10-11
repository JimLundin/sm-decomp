# Room map and earthquake state
Status: resolved
Blocked by: None
Triage: ready-for-agent

Translate SetCollectedMap, SetZebesTimebombEvent, light/medium horizontal shake
and Escape4 shake setup. Preserve unaligned map word aliases, event scratch
writes, room-main variables and PLM setup. Require actual-C full WRAM coverage,
tests and scoped cleanup; distinguish offline translation from live ownership.

## Completion evidence

Five native callbacks translated, preserving map word aliases, event scratch
state, exact room variable writes and complete Escape4 hardcoded PLM setup.
All 240 actual-C full-WRAM cases pass:
out/port/room-map-quake-state/20261011T094901.870249Z/verdict.json.
Covers all eight areas, map word boundaries and free/full PLM tables. Independent
test checks area-seven aliased word and preserved adjacent bytes. Full port
suite and strict Clippy pass. Ordinary cleanup zero; dedup reclaimed 56623104
bytes; all cases authenticate afterward. Free disk 268072321024 bytes.
Retained source/binary/pinned ROM dependencies; no implementer worktree created.
Offline scope only. Branch decomp-port-integration.
