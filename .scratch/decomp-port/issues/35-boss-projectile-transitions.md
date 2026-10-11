# Boss projectile spawning and terrain transitions
Status: resolved
Blocked by: 34
Triage: ready-for-agent

Translate Crocomire projectile/bridge initialization, delayed fireball initialization
and collision preinstruction. Stationary explosion initialization needs an explicit
host-state seam: upstream `eproj_spawn_pt` is a C global outside canonical WRAM,
so a guessed WRAM alias cannot represent it. Keep that dependency open.
Integrate through public spawn and preinstruction dispatch. Compare actual pinned
C full WRAM, including timer/velocity wrapping, global aliases and sound queues.
Keep malformed inputs outside the C oracle. Complete scoped run cleanup.

## Completion evidence

Implemented native initializers 9023 (Crocomire projectile), 9286 (bridge debris)
and 934D (delayed fireball), public allocation dispatch, preinstructions 92BA/9392
and full-frame dispatch. Preserve current-enemy source positions, unsigned
parameter table bounds, fixed-point movement, fourteen-bit acceleration wrapping,
eight-frame activation delay, impact script/timer/velocity changes and SFX2 2B.

Actual pinned bank86 comparison:
`out/port/boss-projectile-transitions/20261011T033828.720067Z/verdict.json`
passes 354 full-WRAM/raw-return cases: 96 allocations, 128 preinstructions,
128 scheduler frames and two existing instruction controls. Two new independent
Rust tests verify delay/impact/sound and acceleration wrapping. Complete port
suite, formatting and strict Clippy pass; targeted sound assertions also pass.
Offline equivalence is proven only in this fixture domain; live ownership remains
unproven. Explosion spawn coordinates and further boss behavior remain open.

Cleanup in the selected run root: `port_cleanup.py --apply` reclaimed zero bytes;
`--deduplicate --apply` reclaimed 285,474,816 snapshot bytes. No implementer
worktree was created. Immutable artifacts/source snapshots retained; final verdict
authenticated after cleanup; 36 GiB free. Earlier valid runs retained but superseded
by the final scheduler-expanded evidence.

## Host-context obligation closed

Ticket80 implements the missing stationary fireball explosion initializer947F
(definition9660) through explicit ProjectileHost coordinates outside WRAM.
No guessed WRAM alias is used. Commit eb9f1ab also adds dust/drop host initializers
and actual-C comparison of all external globals. Current final769-case evidence:
`out/port/projectile-host-context/20261011T063835.827427Z/verdict.json`.
FullWRAM/hoststate/return comparison, fullcrate checks, cleanup and authentication
passed as recorded in ticket80. Historical boss-transition evidence remains
retained; its source hashes differ after subsequent expansions. This closes the
initializer obligation only; live context propagation/ownership remains open.
