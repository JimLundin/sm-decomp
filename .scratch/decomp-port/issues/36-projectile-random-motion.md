# Enemy-projectile random-radius script motion
Status: resolved
Blocked by: 34
Triage: ready-for-agent

Implement bank86 opcode 81DF through public instruction and frame dispatch.
Preserve RNG rejection sampling order, byte truncation, signed direction flags,
wrapping coordinates and continuation pointer. Invalid/impossible masks must
fail explicitly outside the C oracle. Verify actual pinned C and clean runs.

## Completion evidence

Opcode 81DF now executes through instruction and scheduler dispatch. Preserves
the initial direction RNG call, separate X/Y rejection sequences, byte subtraction,
signed acceptance, wrapping positions and bank86 continuation. Impossible masks
are rejected before RNG/state writes; a bounded 65,536-sample loop reports an
explicit error if no value is reached rather than hanging.

Actual pinned C full-WRAM/raw-pointer comparison passes 384 cases (192 opcode,
192 full-frame) in
`out/port/projectile-random-motion/20261011T034043.433111Z/verdict.json`.
Six parameter profiles cover zero motion, rejection, sign and byte wrap. Independent
Rust tests assert a four-call RNG sequence with rejection and unchanged state on
an impossible mask. Full port suite, formatting and strict Clippy pass.
Offline fixture equivalence only; live execution remains unproven.

Cleanup: scoped `port_cleanup.py --apply` reclaimed zero bytes; two
`--deduplicate --apply` passes reclaimed 50,331,648 and 150,994,944 snapshot bytes.
No implementer worktree created. Retained fixture ROM at
`out/port/projectile-random-fixtures/random.sfc`, immutable verdicts and source
snapshots; final verdict authenticated after cleanup. 36 GiB disk space remains.
