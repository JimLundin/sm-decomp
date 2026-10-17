# Room-transition coordinator

Dependency-free Rust implementation of the 27-routine door-transition
coordinator in pinned recovered C bank 82: three scheduled game-state roots,
their private dispatcher, all fifteen phases and all eight background commands.
The complete 128-KiB WRAM is supplied by the existing native transaction. This
crate owns no persistent state outside that WRAM and makes no second full-RAM
copy.

`run(entry, ram, rom, host)` returns `Outcome::Handled(u8)` or unchanged
`Refused`. The handled byte is the exact native coroutine result: zero completes
this invocation, nonzero commits and yields it. It is separate from ownership.
The entry order is HitDoorBlock, LoadingNextRoom, LoadingNextRoomAsync.
The native game-state table selects the entry; the visible game-state word
must not replace the outer scheduler's persisted state-0 latch on resume.

The fifteen phase words remain E17D, E19F, E29E, E2DB, E2F7, E310, E353, E36E,
E38E, E3C0, E4A9, E659, E664, E6A2 and E737. Their ordinary next phases run on
later scheduled invocations. Source-defined private calls remain immediate:
HandleElevator may call Wait48Frames, and either may call state 10. These
private calls emit routine observations without creating another scheduled
callback or transaction.

## Continuations and services

The loading phase uses WRAM byte 0612 exactly as the recovered coroutine:

- Position 0 executes room initialization and the background stream once.
- Position 1 resumes its outstanding NMI wait even if IRQ has meanwhile set
  the completion bit. A successful resumed wait leaves position 1 published
  until a later suspension or the final coroutine end. The loop then rechecks
  IRQ completion; it does not substitute a fixed frame count.
- After scrolling, palette entry 196, BG3 HDMA setup and HDMA enable precede
  the PLM operation. Position 2 resumes that PLM operation directly, preserving
  any pending byte and skipping previous initialization/HDMA work.
- Only completed PLMs permit X alignment, phase E659 and position reset to 0.
  Only the later completed fade-in restores game state 8.

`Host` has the frozen 48-variant `SyncService` enum, raw-byte palette/music
queries, `call_samus_code` returning `u16`, and the two raw-byte coroutine
operations WaitForNmi/PlmHandler. These are 53 distinct named native operations;
their scalar widths and ordered synchronous copyback are part of the contract.
Decompression uses a 24-bit source address and a `u16` WRAM byte destination.
Immediate VRAM copy uses a VRAM word destination, 24-bit bus source and byte
size. The generic room/actor loaders, hardware operations, directional IRQ
scrolling, fades, sound/music, PLM scheduler and rendering remain named services.

Each service sees all preceding WRAM writes and publishes its resulting WRAM
before the coordinator proceeds. The native adapter permits the frozen
transition → named service → existing projectile/scroll/Ceres-door child path,
including that child's already admitted service. The child finishes before
parent copyback. Pending root returns retain no Rust borrow or active native
marker across NMI. This crate does not implement a new scheduler or child-hook
manager.

The eight background commands remain Rust control flow. They return canonical
bank-8F offsets, which may be odd after seven-byte operands. Opcodes index the
source's eight-command table by `opcode >> 1`; values outside that table are
owned errors. Commands 8/E make their source-defined private calls to transfer
command 2. Graphics decompression/copies and the three tilemap-clear bodies
remain named synchronous services.

`Host::observe` is separate from gameplay services. Its fixed `repr(u32)`
27-value `Routine` enum and `Observation::{Enter,Return}` mirror actual C
routine calls, including private nesting. Dispatcher operands preserve the
phase word; background operands/results preserve input and next offsets.
Scalar pending results are widened to `u16` without Boolean normalization.
Observation does not synchronize RAM or create gameplay invocations.

Before any observation, the entry validates memory extents, phase admission and
the selected loading continuation. Unknown phases refuse without effects;
invalid extents/continuations error. A later malformed command or failed service
is an owned error and cannot fall back to C. Runtime data returned by room
services is validated when used. The background stream is bounded by the
number of advancing commands that fit in its source bank.

## Source identity

Recovered C revision `578f90b3cc49557bb70060ad033bb90b8cf8ac50`, archive SHA-256
`69c78831351112887575f852dadf0281dd5c23cef890b07a268ae6cf60aa9193`.
The exact inspected source is retained in the integrator under
`restart/out/ceres-door/native/8cf2945825f87c13bb3fc6cb240d1e59b3bb3b76a95c847624cad9978e62c0ad/source/src/`.

| Source | Responsibility | SHA-256 |
| --- | --- | --- |
| `sm_82.c:3770` | Three roots, dispatcher, fifteen phases and eight background commands through line 4183 | `20ae568cc21260a577e441bfcd25ac3fdc33e00fce3eb5c82fa85c7f9ccabebe` |
| `types.h:83` | Exact coroutine macros and byte return type | `02d6acb15eb207f36fa03ebab65bf622521657f2168db0a415a55c7f9598a0c0` |
| `variables.h` | Canonical state locations | `e2ce28d07ceff91129e93417099f33f20f161c37e95e745c575d0e30b3fd5585` |
| `ida_types.h:983` | Room-state layout and transfer constants | `12974c5708740164692dac151104020b53e8316e7378521c974ddf602d1f28ff` |

Source and ABI research remain under the integrator's ticket 09 and
`.scratch/restart-pipeline/room-transition-abi.md`. The upstream license is
retained in `LICENSE.upstream.txt`. See `VERIFICATION.md` for checks and limits.
