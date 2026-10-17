# Focused verification

Thirteen dependency-free tests cross the public three-root interface with
complete authored WRAM inputs and an ordered synchronous Host probe. They
exercise all 13 routines and check:

- Right/left setup order, masking direction high bits, immediate movement,
  raw FE math return admission, old completion-bit preservation, both BG2
  offsets using layer 1, and wrapping previous-block adjustments.
- Downward setup's transient first movement, unchanged Samus position at
  counter zero and preservation of child writes outside the four restored
  vertical scalar fields, including high-WRAM state.
- Upward correction resetting the counter, nested temporary subtraction and
  restoration, and subsequent upward setup retaining counter one. A layer2Y
  service deliberately overwrites layer1Y to verify the saved source local;
  destination and previous-block arithmetic wrap.
- All four directions' fractional carry/borrow, full 32-bit position overflow,
  layer-coordinate wrapping, previous integer position updates and preservation
  of previous fractional fields. Following still executes with an old 8000 bit.
- Upward direct BG-shadow updates at counter 4 and the child switch at 5,
  including separate layer1/layer2 coordinate use and wrapping sums.
- Right's two distinct terminal children observing 63 then 64, child-owned
  final counter changes and selective layer1-only destination snap. Distinct
  child-written layer2/BG sentinels make a broader snap observable.
- Completion boundaries around right/left counter 63 and up/down counter 56;
  downward's inequality, no-motion terminal path and FFFF→0 wrap. Counter
  locals remain authoritative across child writes in every moving direction.
- Downward's second terminal child sees counter 57 and owns subsequent writes.
- Setup discards a private movement's completion byte instead of snapping or
  setting the completion flag; only the following root consumes that byte.
- Both truncated and oversized WRAM reject before observations or mutation.
  All four service failures and passive observation failures propagate as
  errors; execution does not resume or return refusal after ownership begins.

The probe checks service order, WRAM extent, authored intermediate values,
balanced nonrecursive routine nesting and zero results for void routines.
It does not implement the foreign math/scroll algorithms or establish actual-C
or hardware agreement. Real parent/child transaction admission and fatal-control
tests belong to the native host; actual route comparison belongs to the
integrator. The existing 9,380-step route reaches two rightward transitions;
left/down/up remain focused branch tests unless separately exercised live.
No original-ROM equivalence is claimed here.

The first isolated checks passed with one Cargo job and the private target:

```sh
cargo test --manifest-path restart/door-scrolling/Cargo.toml --offline -j1 --target-dir restart/door-scrolling/target
cargo clippy --manifest-path restart/door-scrolling/Cargo.toml --offline -j1 --target-dir restart/door-scrolling/target --all-targets -- -D warnings
cargo fmt --manifest-path restart/door-scrolling/Cargo.toml -- --check
```

Raw stdout/stderr are retained in the ignored `checks/` directory. The temporary
standalone `[workspace]` declaration supports isolated worker checks; the
integrator removes it when registering the crate in the shared workspace.
The integrator owns retaining those logs/source and retiring the worker target
and worktree after integration under the project's run-cleanup rules.
