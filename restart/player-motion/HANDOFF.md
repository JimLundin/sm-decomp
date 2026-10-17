# Ticket25 source handoff

Owner: material, reused live agent `/root/material`.
Isolated worktree: `/home/jim/sm-decomp-restart-player-motion-rust`.
Branch: `restart/player-motion-rust`.
Baseline: `768779a56b186231c17116584f2b04f3e98b9df4`.
Writable ownership: only `restart/player-motion/`.

## Implemented source

The exact seven-definition Interface in the frozen ticket is implemented with
no dependencies and unsafe forbidden. Root owns workspace/lock/native/runtime
registration and all producer commands. The matrix API was agreed with native
before its control implementation: `Entry` and `Output` are Copy/Clone/Debug/Eq;
`BodyFixture` is Clone/Debug with public name:String, entry:Entry, wram:Vec<u8>.

Pinned recovered source revision: `578f90b3cc49557bb70060ad033bb90b8cf8ac50`.
Authoritative definitions read from the retained scroll-native source under
`/home/jim/sm-decomp-restart/restart/out/scroll/native/2c9c9a3cf51a059220afac1ee04727c956e45d067e10e17405bf3adbd268fc7f/source/src/`:

- `sm_90.c`: roots1201/1215, NoColl1451/1456, feedback5911–5940.
- `sm_a0.c`: checker2813–2947.
- `variables.h`: canonical aliases; movement0A1F is byte, other owned fields words.

The existing main checker/control implementation was inspected as permitted
reuse context. This module additionally preserves the original C promoted
left/up exact-contact comparison, distinct from wrapped separation zero.
The upstream notice is retained in `LICENSE.upstream.txt`.

## Shared body matrix

`body_fixtures()` returns192 valid original-C cases in stable order:

- 75 checker cases: gates, empty/ordered/duplicate/frozen/property lists, last
  pool slot/last list terminator, four directions, fraction relations, positive
  and negative separation, wrap/carry/borrow, promoted-zero and abs8000.
- 87 DownFeedback cases:28 movement indexes ×3 collision values, low-byte5
  preservation, unreachable invalid index, collision overriding low-byte5.
- 3 UpFeedback cases,13 NoColl cases,14 composed roots (10air,4enemy contacts).

Base geometry is width16/height16, player40/40/radii8, area0, movement2 and
disabled list. Native terrain owns its own finite reaction matrix and admission.
Root air cases preserve the original sign chain: terrain receives the incoming
root amount for either direction. Direct helper cases do not require terrain.
No invalid native fixture is included: the sole invalid movement byte is in
DownFeedback with collision0 and low-byte5, so its table is unreachable.

## Source checks and remaining gates

Source-only `rustfmt --edition 2024 restart/player-motion/src/lib.rs` and
`git diff --check` completed. No Cargo, test, native, game or capture producer
was run in this worktree. The unit suite and192 actual-C pairs are pending,
not verification evidence. Tests cover whole-WRAM changes, list admission,
canonical aliases, independent feedback outputs, fractional arithmetic,
enemy bypass, immutable unavailable-service refusal and accepted-service errors.

After integration, root's focused command is
`cargo test --manifest-path restart/Cargo.toml --locked --offline -p sm-player-motion`.
Native supplies its exact control CLI and terrain/transport cases separately.
Root must run actual-C full-WRAM/raw-return/ordered-service comparisons,
Clippy/build, fatal/refusal controls and any separately frozen live run before
publishing the corresponding stages. No player-route, arbitrary terrain,
original-ROM or gameplay acceptance is claimed here.

No private outputs, active producers or disposable intermediates exist;
local cleanup0 bytes. Root owns stage publication, shared checks, both cleanup
passes, retained-source authentication and eventual worker retirement.
