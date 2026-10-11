# Phantoon destroyable fireballs and contracting orbits
Status: resolved
Blocked by: 38
Triage: ready-for-agent

Translate all four 9824 initialization modes and 9A45/9A94/9ADA/9B29/9B41
preinstructions. Preserve radial byte products, X-before-Y short circuit,
delay/sound order and parity-sensitive contraction through scheduled frames.
Compare pinned actual C full WRAM and complete run cleanup.

## Completion evidence

Implemented four destroyable-fireball initializer modes plus expanding orbits,
delayed fall/sound, boss-triggered orbit delay and parity-sensitive contraction.
Shared radial byte-product helper also serves the existing starting initializer.
Preserves X-before-Y short circuit, angle byte wrapping, delay underflow,
sound queue order and final script transitions through public frame dispatch.

Actual pinned C evidence
`out/port/phantoon-orbits/20261011T034658.290488Z/verdict.json`
passes 809 full-WRAM/raw-return cases: 168 allocation, 320 preinstruction,
320 full-frame and one sleep control. Parameters cover all four valid modes;
states cover offscreen centers, radius wrap, delay boundaries and parity.
Independent Rust tests assert skipped Y writes and delayed odd-frame contraction.
Full port suite, formatting and strict Clippy pass. Live execution/full Phantoon
boss ownership remain unproven.

Cleanup: scoped `port_cleanup.py --apply` reclaimed zero bytes;
`--deduplicate --apply` reclaimed 165,281,792 snapshot bytes. No implementer
worktree created; immutable ROM/source/run evidence retained and final verdict
authenticated after cleanup. 36 GiB free.
