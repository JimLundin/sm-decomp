# Statue animation room setup
Status: resolved
Blocked by: None
Triage: ready-for-agent

Translate RoomSetup_RunStatueUnlockAnims with the four native animated tile
headers in exact allocation order. Verify partial/full slot occupancy against
actual C, then scoped cleanup and authentication. Offline evidence is separate
from animated allocator live coverage.

## Completion evidence

Native room setup now calls Rust animated tile allocation in Kraid/Phantoon/
Draygon/Ridley order. All 64 occupancy patterns match actual-C full WRAM:
out/port/statue-animation-room-setup/20261011T093242.601429Z/verdict.json.
Full port test suite passes, including independent order/full-table test;
strict Clippy passes. Offline only; allocator live requirement remains open.
Ordinary cleanup zero; dedup reclaimed 8519680 bytes; all 64 cases authenticate.
Free disk 269835788288 bytes. Retained sealed binaries, pinned source and ROM;
no implementer worktree created. Integration branch decomp-port-integration.
