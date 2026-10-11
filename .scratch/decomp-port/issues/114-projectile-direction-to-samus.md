# Projectile direction toward Samus
Status: resolved
Blocked by: None
Triage: ready-for-agent

Translate native 82A5 direction instruction through existing angle math and
signed sine table. Preserve distinction between input projectile slot and global
destination slot, wrapping coordinate subtraction and pointer return. Verify
actual-C full WRAM/return, tests and cleanup.

## Completion evidence

Native opcode and wrapper translated through existing angle calculation and
signed sine table. Reads coordinates from argument slot while writing angle/
velocity to global projectile index. All 72 actual-C full-WRAM/return cases pass:
out/port/projectile-direction-samus/20261011T100732.971866Z/verdict.json.
Covers distinct source/destination slots, zero/axis directions, signed coordinate
boundaries and wrap. Full port suite and strict Clippy pass. Ordinary cleanup
zero; dedup reclaimed 9437184 bytes; all cases authenticate afterward. Free disk
268831170560 bytes. Retained binary/source/pinned ROM dependencies; no worktree
created. Offline scope only; live ownership separate. Branch decomp-port-integration.
