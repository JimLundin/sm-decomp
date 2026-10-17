# Directional door scrolling

Dependency-free Rust implementation of the complete 13-routine directional
door-scrolling boundary in pinned recovered C bank 80. The three synchronous
roots are upward correction, scrolling setup and IRQ transition following.
The four setup routines, four movement routines and two bookkeeping routines
remain immediate private calls. The crate consumes exactly 128 KiB of canonical
WRAM, owns no persistent state outside that WRAM and makes no second full-RAM
copy. No ROM input is needed.

`run(entry, ram, host)` returns `Outcome::Handled` after a completed invocation.
Movement completion is a private byte consumed by the following root; it is
not a coroutine result or an ownership outcome. `Refused` is retained in the
frozen ownership ABI, but every typed root and all four masked directions are
implemented. Invalid WRAM extent errors before observations or mutation.
Later service/observation errors propagate as owned failures without fallback.

The native IRQ caller owns its completion-bit guard and scheduler placement.
The following root itself runs even when bit 8000 is already set. Setup also
runs with an old completion bit and leaves it unchanged; the coordinator clears
it later. Coordinator-owned Samus rebasing, downward BG1 vertical adjustment,
IRQ selection and room loading remain outside this crate.

## Services and observations

`Host` exposes four synchronous services, in the frozen numeric order:

1. `calculate_layer2_x` returns the actual `u8` service result.
2. `calculate_layer2_y` returns the actual `u8` service result.
3. `calculate_blocks` returns `()`.
4. `update_scroll_vars` returns `()` and calls the installed room-scrolling
   entry 1 through the native adapter.

Setups ignore the two raw math bytes as the recovered source does. Every
service receives preceding WRAM writes and completes full private-WRAM
copyback before execution continues. Scalar locals survive those calls. The
native adapter owns the bounded coordinator → directional → room-scrolling
composition; this crate introduces no scheduler or general hook manager.

The separate `observe` method receives passive `Enter { routine, counter }`
and `Return { routine, result, counter }` events. `Routine` is the frozen
13-value `repr(u32)` enum. Counter is the actual WRAM 0925 word at each event;
void routine results are zero and movement results retain their actual byte.
Observations do not synchronize RAM or dispatch gameplay. Successful calls
have balanced, nonrecursive nesting, including immediate setup movement.

## Source-sensitive behavior

All coordinate, block and counter arithmetic wraps at 16 bits. Samus movement
adds/subtracts the complete 16.16 position/speed pairs with 32-bit wrapping;
previous integer positions are updated before calling the scrolling child.
The saved old counter determines the later increment even if a child writes
the canonical counter. A second terminal child may then change that final
counter again, and that result is preserved.

- Right makes separate terminal scrolling calls at counters 63 and 64; left
  makes one call. Both return completion only for old counter 63.
- Down counter zero temporarily subtracts 15 from both layer Y positions.
  Nonzero counters below 57 move, while larger counters skip movement. Its
  completion predicate is the wrapped new counter being at least 57, so a
  wrap from FFFF to zero does not complete or call a child.
- Upward correction resets the counter to zero, executes the temporary
  minus-16 preparation, and advances it to one. Later upward setup preserves
  that counter. Nonzero upward movement at counters 1 through 4 updates the
  four BG shadows directly; counter 5 and later call the child. Completion
  tests old counter 56.
- Both vertical preparations restore only saved layer2Y, layer1Y, BG2VOFS
  and BG1VOFS, in that order. Other child changes, including blocks, transfer
  descriptors and queues, remain committed.
- Both BG2 offsets are calculated against layer 1 during setup. Upward setup
  saves layer1Y across the layer2Y service and adjusts destinationY by 32.
- Completed following snaps only layer1X/Y to the current destination and
  ORs completion bit 8000. It preserves layer2 positions and BG shadows and
  makes no extra scrolling call.

## Source identity

Recovered C revision `578f90b3cc49557bb70060ad033bb90b8cf8ac50`, archive SHA-256
`69c78831351112887575f852dadf0281dd5c23cef890b07a268ae6cf60aa9193`.
The inspected source is retained by the integrator under
`restart/out/scroll/native/2c9c9a3cf51a059220afac1ee04727c956e45d067e10e17405bf3adbd268fc7f/source/src/`.

| Source | Responsibility | SHA-256 |
| --- | --- | --- |
| `sm_80.c:2290` | All 13 routines through line 2486 | `171d73158e0e7323118b4bb142d4e7c79f3d74cdc4129a667aa8873d7eb785b7` |
| `variables.h` | Canonical WRAM fields | `e2ce28d07ceff91129e93417099f33f20f161c37e95e745c575d0e30b3fd5585` |

The accepted ABI is the integrator's
`.scratch/restart-pipeline/door-scrolling-abi.md`. Source research is retained
in main's `.scratch/restart-collaboration/next-door-scrolling-scope.md`.
The upstream license is retained in `LICENSE.upstream.txt`.
