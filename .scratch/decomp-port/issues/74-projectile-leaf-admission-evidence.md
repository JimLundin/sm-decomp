# Projectile movement leaf live admission evidence
Status: resolved
Blocked by: None
Triage: ready-for-agent

Seal fresh actual-C full-WRAM MoveEprojWithVelocity/X/Y evidence for ticket73
live admission. Preserve synthetic opcode control but do not register it live.
Verify cleanup/authentication; live execution counts determine exercised scope.

## Offline verification evidence

385 actual-C full-WRAM/raw-return cases pass: 128 each of full/X/Y movement
plus synthetic sleep control excluded from live registration. Verdict:
`out/port/projectile-movement-live-leaves/20261011T055520.868093Z/verdict.json`.
Full port tests, fmt and strict Clippy pass. Admission tooling tests:9 pass,
one optional test skipped. Live comparison underway under ticket73.

Cleanup applied/deduplicated offline run:0ordinary bytes,92,143,616snapshot
bytes reclaimed. Offline verdict authenticated post-cleanup. Immutable source/
ROM snapshots retained; no worktree created;31GiB available.

## Completion evidence

Fresh offline contract sealed and authenticated for ticket73. Tool commit
7b37fcb consumes it.9000/18000 live prefixes match all eight process state/effects
inventories but execute zero movement leaves; no live ownership admission is
claimed. Ticket73 remains claimed for an exercising replay. Offline deliverable
is complete; cleanup/authentication recorded above and live cleanup in ticket73.
