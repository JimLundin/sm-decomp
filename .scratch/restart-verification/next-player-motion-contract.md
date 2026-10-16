# Proposed first player-motion contract: vertical movement and solid-enemy contact

Status: prepared source proposal; root chooses and freezes the final contract.
No implementation claim, source admission or whole-frame ownership is created.
This is independent of the SRAM codec and active energy work.

Select a smaller complete boundary than the 28-mode normal movement dispatcher:
**Samus_MoveUp / Samus_MoveDown**, including solid-enemy clipping, fixed-point
position/displacement writes and collision-to-pose feedback. Keep native terrain
motion as one explicit synchronous dependency. This is a useful first kernel:
standing, running, jumping and falling callers already converge on these roots,
without taking input/pose selection, gravity/speed calculation, horizontal
movement, speed-boost cancellation or every normal movement mode.

## Authority and exact owned definitions

Recovered C: revision `578f90b3cc49557bb70060ad033bb90b8cf8ac50`, retained source
directory `/home/jim/sm-decomp-restart/restart/out/scroll/native/2c9c9a3cf51a059220afac1ee04727c956e45d067e10e17405bf3adbd268fc7f/source/src/`.
Inspected source SHA256: `sm_90.c` =
`38a661917219e6647826d25ea0bc45039c36bd54924f8cf968facb3f463389fa`;
`sm_a0.c` = `a2cee5c286304e7d606357ac4960413e5fe3dfe585d468d0f41a654aa093e8d3`;
`sm_94.c` = `fee1e46c9cd18a576766fa401840176c8e4c120a57527b8929010134919860c0`.
Main Rust reuse baseline is the supplied `193cad9`; reuse does not grant restart
acceptance. `collision.rs:423,492,1223` and `samus_control.rs:539` contain the
corresponding checker, directional composition and pose feedback; `samus.rs`
contains the NoColl fixed-point helpers.

| Definition | Recovered source | Role |
| --- | --- | --- |
| `Samus_MoveUp` | `sm_90.c:1201`, 90:93EC | Complete upward root |
| `Samus_MoveDown` | `sm_90.c:1215`, 90:9440 | Complete downward root |
| `Samus_CheckSolidEnemyColl` | `sm_a0.c:2813`, A0:A8F0 | Ordered interactive-list contact; native compound return |
| `Samus_MoveUp_NoColl` | `sm_90.c:1456`, 90:988D | Subtract allowed displacement; publish signed projectile-init displacement |
| `Samus_MoveDown_NoColl` | `sm_90.c:1451`, 90:9871 | Add allowed displacement; publish projectile-init displacement |
| `Samus_MoveUp_SetPoseCalcInput` | `sm_90.c:5911`, 90:E606 | Collision feedback 4 or 0 |
| `Samus_MoveDown_SetPoseCalcInput` | `sm_90.c:5930`, addressless definition | Collision/falling table feedback; preserve low-byte-5 continuation |

Local width-correct arithmetic (`abs16`, `AddToHiLo`, `SetHiLo`, `INT16_SHL16`,
byte writes) is part of this closure. Do not fabricate an address for the last
definition. The checker may support directions 0..3 as a directly controlled
Fixture boundary; only vertical composed roots earn this scope's ownership.

## Canonical state and assets

Carry the complete **131072-byte canonical WRAM** into and out of every root and
service. Root-owned local reads/writes are explicitly bounded:

- Samus geometry: words X/XS/Y/YS/X-radius/Y-radius at 0AF6/0AF8/0AFA/0AFC/0AFE/0B00;
  collision direction word0B02; movement type is **byte0A1F**, sharing its word
  with facing byte0A1E. Preserve that alias.
- List gate word17A6; interactive indexes are 32 words at17EC..182B, immediately
  followed by four collision-index words182C..1833. Require a FFFF terminator
  within the 32-word list for enabled checking. Preserve duplicates/order;
  do not sort or interpret the gate as the scan length.
- Enemy slots are byte offsets 0000..07C0, step0040, relative to WRAM0F78.
  The checker reads record offsets+02 X,+06 Y,+0A width,+0C height,+0E properties,
  +26 frozen timer. Frozen or property8000 makes the enemy eligible; enemy_ptr
  is not this routine's eligibility test. Reject misaligned/out-of-pool entries.
- Checker output index word18A6, per-direction collision index182C+2*direction;
  exact contact clears **Samus Y subposition0AFC even for horizontal checker
  Fixtures**. Collision word0DD0 distinguishes native FFFF enemy contact from
  terrain's 0001. Pose feedback word0DC6; movement displacement words0DB2/0DB4
  (up high/low) and0DB6/0DB8 (down high/low).
- `kSamus_MoveDown_SetPoseCalcInput_Tab0[28]` and `Tab1[28]`, `sm_90.c:5918–5929`,
  are two fixed 28-byte tables; Tab0 is annotated90:E61B. Preserve the exact
  arrays; Tab1 has no separately declared address here. Kernel-local ROM reads
  are unnecessary if these pinned tables are encoded as validated constants.
  ROM-byte equivalence remains a separate original-ROM check.

Typed terrain dependency reads/writes additional room/collision/PLM state.
Pass the pinned full ROM and WRAM rather than projecting only player geometry.
Its assets include level words at10002 with allocation0x3200 words, BTS bytes
at16402 with allocation0x3200 bytes, room width07A5, current block0DC4,
frame parity05B6, slope tables and block-reaction selectors, area-specific PLM
tables and door data. Their supported extents belong to the terrain service's
contract; this proposal does not claim a fresh bank94 asset closure.

## Native service and admission

Only composed-root service:
`TerrainMoveVertical { displacement: i32 } -> { applied_displacement: i32 }`,
executing the real `Samus_MoveDown_NoSolidColl`, `sm_94.c:1078`, **94:9763**.
It mutates position/collision/quicksand/slope state and can invoke real SpawnPLM,
door and spike reactions. Keep all these continuations synchronous on the latest
state. Record ordered service invocation/return, all native host words and full
copyback before the Rust root publishes displacement and pose feedback. Preserve
even/odd frame traversal order, not just final position. Do not stub PLM work or
infer whole terrain ownership from this call.

Proposed support is the seven listed definitions, valid full state/ROM, movement
type0..27 whenever the down-feedback table is reachable, a valid enabled list,
and a terrain continuation admitted by the integrator's native service bridge.
Direct `MoveUp` amount excludes i32::MIN because C negates it; direction and
amount signs follow the original callers, including unusual direct Fixture signs.
MoveUp writes direction2 before checking; MoveDown writes3. Horizontal checker
Fixtures honor the supplied direction&3, without silently changing it.

Preflight must be immutable: validate list/fields and the actual terrain-service
continuation before the root writes direction or collision. Unsupported input
refuses unchanged; an error after accepted service execution is fatal, with no
C retry. No native service bridge for this exact entry currently forms part of
this proposal: **root must freeze its selector, nested-call/copyback policy and
registration before implementation is dispatched**. If that is unavailable,
the smaller complete checker plus feedback/NoColl Offline family can be prepared
first; composed roots remain held, with no invented live acceptance.

## Meaningful actual-C Fixture distinctions

Compare full WRAM, raw/normalized scalar returns and the checker's native
`{collision, amt}` return separately; preserve collisionFFFF versus0001. Use
independent expectations and material state/ordering mutations:

1. Disabled list gate leaves index state untouched; enabled empty list resets
   checker index0. Frozen/property8000 eligibility, ignored ordinary enemy,
   duplicate indexes, first-contact order, last valid slot, 32-slot termination,
   invalid/missing terminator unchanged refusal.
2. All four checker directions; fraction equal/less/greater, carry/borrow at
   position0000/FFFF, abs16 boundary8000, just touching versus one-pixel overlap,
   negative separation skipped. Zero separation clears Y subposition and returns
   enemy collisionFFFF with amount0; positive separation returns integer16.16.
3. Solid-enemy hit takes no terrain service; no hit calls it once. Upward sign
   reversal, allowed signed displacement and high/low projectile writes; fractional
   position wrap. Spy service sees direction/collision already written, then its
   full copyback is visible before pose feedback. A material service modifies
   collision/position and an unrelated legitimate PLM state field.
4. Terrain return no collision, collision0001 and quicksand collision; zero amount
   with preexisting quicksand; even/odd traversal order. Native terrain reactions
   need positive/mid-service-error and ordered continuation controls in the service
   contract, rather than a matching arithmetic mock alone.
5. Down feedback covers all28 movement-type table entries, collision/no collision,
   low-byte5 with nonzero high byte preserved, and byte0A1E facing unchanged.
   Up feedback4/0. Unsupported movement type and unsupported service refuse
   before local writes; accepted service errors terminate without replay in C.

Initial owned Rust tests and bounded actual-C Fixture pairs can precede route
arrival; no build or gate was run here. Root supplies exact frozen Cargo/fixture
commands using the existing restart verification path. No new framework is needed.

## Ownership, ordering and remaining reach

Proposed exclusive implementation files are
`restart/player-motion/Cargo.toml`, `src/{lib,memory,vertical,solid_enemy,feedback,tests}.rs`.
Native-host hooks/prepare/types, workspace registration and scheduler wiring
remain one integrator's separately frozen ownership. No writes to player input,
item acquisition, SRAM, existing projectiles or route-policy files are granted.

Natural caller order stays native: beta90:E722/E725 ->
`RunSamusMovementHandler`90:E94B -> normal movement selector90:A337 -> selected
mode -> vertical root. A337 labels both the recovered 28-entry handler table
region and `funcs.h`'s normal handler identity; it is not evidence for all28
modes. This kernel does not own alfa, beta, gamma, minimap, hit/palette, input,
gravity, speed echoes or horizontal movement. Controller/pose/room/enemy state
arrives from their existing owners in the original order.

The existing route establishes no direct-root movement ownership receipt in this
research. Ordinary gameplay plausibly reaches vertical roots, but exact calls,
amounts, solid-enemy branches, PLM reactions and original-ROM parity remain to
be measured. Neither a future boss nor SRAM/reopen is required to prepare the
source and Offline family. Live promotion waits independently supported direct
exercise of these roots and the admitted terrain continuation; energy/item/camera
acceptance cannot substitute for that evidence.

Read-only research except this note; no claims/build/game/test/capture/processes
or evidence outputs were created. Run cleanup is not applicable; reclaimed0bytes.
