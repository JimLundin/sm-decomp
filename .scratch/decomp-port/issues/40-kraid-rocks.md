# Kraid rock projectile spawning and flight
Status: resolved
Blocked by: 34
Triage: ready-for-agent

Translate 9CA3/9CD8/9D0C initializers and 9D56/9D89/9DA5 preinstructions.
Preserve current-enemy aliases, RNG complement offset, sound queue, collision
short circuit and acceleration wrapping through allocation and frame dispatch.
Compare actual pinned C and finish scoped cleanup.

## Completion evidence

Implemented spit, ceiling and rising-rock initializers, including both rising-rock
definitions. Preserve current-enemy source coordinates, RNG bitwise complement
offset without advancing RNG, SFX3 1E, collision short circuit, threshold X
velocity wrap, fourteen-bit ceiling acceleration and graphics-clear callback.
Public allocation, preinstruction and scheduler dispatch own these paths.

Actual pinned C full-WRAM/raw-return comparisons pass 513 cases (128 allocation,
192 preinstruction, 192 scheduled frames and one sleep control) in
`out/port/kraid-rocks/20261011T034828.170182Z/verdict.json`.
Independent Rust tests verify positive-threshold velocity wrap and horizontal
collision skipping vertical motion/acceleration. Full port suite, formatting and
strict Clippy pass. Offline fixture equivalence only; live Kraid ownership remains
unproven and Mini-Kraid paths are still open.

Cleanup: scoped `port_cleanup.py --apply` reclaimed zero bytes;
`--deduplicate --apply` reclaimed 161,611,776 snapshot bytes. No implementer
worktree created; immutable ROM/source/run evidence retained and final verdict
authenticated after cleanup. 36 GiB free.
