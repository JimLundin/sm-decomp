# Treadmill door setup
Status: resolved
Blocked by: None
Triage: ready-for-agent

Translate both Wrecked Ship treadmill door callbacks, including their animated
tile allocation and complete native hardcoded PLM setup. Verify against actual C
with partial/full allocation tables and tile mutations. Cleanup and authenticate
before completion; report offline scope separately from movie ownership.

## Completion evidence

Both callbacks translated through Rust animated allocation and existing PLM
setup B04A, including the exact 56-tile clear. All 54 current-source actual-C
full-WRAM cases pass:
out/port/treadmill-door-setup/20261011T094615.237598Z/verdict.json.
Covers low/high/full occupancy in both tables and widths 16/32/255. Full port
suite and strict Clippy pass; independent test verifies headers, exact cleared
range and preserved neighboring tiles. Offline only; no movie ownership claim.

Initial 54-case oracle 20261011T094458.273092Z was cleaned: ordinary zero,
dedup 11403264 bytes. Its source seal is historical after door-scroll additions.
Refreshed run ordinary cleanup zero; dedup 11403264 bytes; all cases authenticate
afterward. Free disk 269869608960 bytes. Retained sealed binary, pinned source
and ROM dependencies; no implementer worktree created. Branch decomp-port-integration.
