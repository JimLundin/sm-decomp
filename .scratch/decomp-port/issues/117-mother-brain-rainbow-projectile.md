# Mother Brain rainbow projectile
Status: resolved
Blocked by: None
Triage: ready-for-agent

Translate C80A rainbow-beam initializer and C814 tracking preinstruction.
Preserve enemy-one position aliases and zero graphics/velocities on allocation.
Require actual-C full-WRAM allocation and direct callback cases, tests, cleanup.

## Completion evidence

C80A initialization and C814 tracking translated. Nine allocation cases and 54
tracking cases match actual pinned C full WRAM. Port tests and strict Clippy pass.
Fixed full-C harness duplicate allocator module/type identity. Current seals:
out/port/mother-brain-rainbow-allocation/20261011T101910.386981Z/verdict.json
out/port/mother-brain-rainbow-tracking/20261011T101911.560655Z/verdict.json
Both authenticate after ordinary cleanup (zero bytes) and deduplication
(1,179,648 and 13,762,560 bytes). Earlier failed allocation and successful tracking
runs cleaned: zero ordinary bytes; 13,762,560 deduplicated bytes total.
Retained binaries, source snapshots, ROM and case inventories required by seals.
Available disk 268 GB. Offline equivalence only; live ownership remains separate.
