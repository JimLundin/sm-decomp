# Enemy death explosion to pickup transition
Status: resolved
Blocked by: 75
Triage: ready-for-agent

Translate EEAF instruction using native RandomDropRoutine, preserving the original
slot-zero exclusion, pickup script table, lifetime/timer/properties and fallback
empty script. Verify real-ROM opcode calls and scheduler transitions against C.
Pickup collection and host-coordinate drop allocation remain separate frontiers.

## Verification required

Actual-C full WRAM/raw return cases, independent Rust example, full crate checks,
cleanup and authenticated retained evidence before resolution. Offline scope.

## Implementation

EEAF now selects native weighted drops, excludes projectile slot zero only after
consuming RNG, copies real EF04 pickup scripts, sets lifetime 400/timer1/preinstr
EFE0 and clears only property bit4000. No-drop and slot-zero paths install ECA3,
EFDF/timer1/properties3000. Native scheduler recognizes EFDF as inert.
Independent Rust test verifies slot-zero RNG consumption and both transition
branches. Full port tests, formatting check and strict Clippy pass settled source.

## Diagnostic evidence

Initial run `out/port/enemy-drop-transition/20261011T061219.170323Z` rejected
because direct/frame fixtures reused names and the harness shares case directories.
Frame names now have distinct prefixes; no source correction was required.
Both cleanup apply/deduplicate commands succeeded, reclaiming zero bytes.
Diagnostic evidence retained.

## Completion evidence

2,434 actual-C cases (1,793 opcode calls, 641 same-frame scheduler transitions)
match full WRAM and raw returns at
`out/port/enemy-drop-transition/20261011T061313.175289Z/verdict.json`.
The retained verdict authenticated after cleanup using
verify_plm_instruction_evidence. `port_cleanup.py --root <run> --apply` and
`--deduplicate --apply` succeeded: zero unsealed bytes, 319,160,320 snapshot
bytes reclaimed. Both completed runs cleaned; 25 GiB disk free. No implementer
worktree or disposable build remains. Full crate tests/fmt/strict Clippy pass.

Pickup collection preinstruction EFE0 remains unsupported; verification covers
creation of pickup state and immediate script processing, not subsequent pickup
collision/collection or live gameplay ownership.
