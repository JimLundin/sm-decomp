# Focused verification

Eighteen dependency-free tests invoke the public root with complete authored
WRAM and a passive observer. Successful cases verify nonrecursive balanced
routine observations, the eighteen-event bound and preservation of every byte
outside the fourteen published words. Source-derived cases cover:

- Positive fractional movement and one-pixel distance addition, ideal
  overshoot fraction reset, exact nested X-before-Y order, and all four
  previous-position words.
- Negative movement with fractional borrow; wrapping 32-bit position
  differences and the exactly-half-range signed integer direction decision.
- Every even and odd index into both camera tables, with facing, knockback,
  movement and acceleration XOR branches, including adjacent byte fields.
- Fraction-only motion admitting an unused invalid table index, preserving
  ideals and selecting the two autoscrollers.
- Slow unsigned windows at both thresholds, wrapping three-pixel movement,
  preservation of fractions/distances and the signed-high X/Y distinctions.
- Frozen autoscrollers alongside active normal following, distance updates
  and tail publication.
- The Spore Spawn clamp below, at and above 464, preserved Y fraction and
  exact finish dispatcher operand/nesting.
- Same-frame X movement selecting the Y scroll cell, exact direction-one
  selection of the down-scroller, all four directional
  trigger constraints, blue versus green downward margin, and automatic
  correction away from current/neighbor red cells in both axes.
- Unknown finish hooks, accessed invalid table indices, invalid room extents
  and late scroll failures leaving the entire caller WRAM and observer empty.
  One case resets a private X fraction before failure; another completes an
  X room-maximum clamp before Y accesses an unsupported row.
- Valid dimension extremes, unused invalid scroll/table values on frozen
  stationary axes, observation failures before commit, invalid memory extent,
  and execution independent of visible game state.

The first full run passed sixteen tests and failed an authored trigger fixture
whose expected movement overlooked the source's clamp at the current 256-pixel
boundary. The fixture was moved away from that boundary to exercise directional
correction directly; no gameplay implementation changed. The second full run
passed all seventeen tests. A subsequent refinement made the late-Y refusal
fixture reach a successful X clamp first; that focused test passed again.
A final missing Y-direction selector case was added and the full eighteen-test
suite passed. The original failed log is retained alongside the passing logs.

Worker commands used one Cargo job and the private target:

```sh
cargo test --manifest-path restart/camera/Cargo.toml --offline -j1 --target-dir restart/camera/target
cargo test --manifest-path restart/camera/Cargo.toml --offline -j1 --target-dir restart/camera/target unsupported_accesses_refuse_atomically_even_after_private_fraction_reset
cargo clippy --manifest-path restart/camera/Cargo.toml --offline -j1 --target-dir restart/camera/target --all-targets -- -D warnings
cargo fmt --manifest-path restart/camera/Cargo.toml -- --check
```

Strict Clippy and formatting checks passed. Raw stdout/stderr are retained in
the ignored `checks/` directory. The temporary standalone `[workspace]` is for
worker isolation; the integrator removes it when registering the crate.

These authored probes establish the implementation's focused behavior and
atomic admission contract. They do not establish actual-C or original-ROM
agreement. Native transaction/refusal/fatal controls belong to the host worker;
the combined 9,380-step reference/candidate comparison belongs to the integrator.
Ordinary camera following, material X/Y changes and state-9 tail calls must be
measured there. Slow-grapple, Spore Spawn, knockback and all table variants
remain focused coverage unless separately exercised live.

The integrator owns source/log retention and retirement of this isolated
target/worktree under the run-cleanup rules. No replay, native build or shared
target was used by this gameplay worker.
