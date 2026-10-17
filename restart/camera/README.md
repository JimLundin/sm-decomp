# Normal-camera frame

Dependency-free Rust implementation of the complete thirteen-routine
`MainScrollingRoutine` closure. The native root is synchronous and pure WRAM;
there are no ROM inputs, foreign services, hardware effects or coroutine waits.
The native caller owns scheduling. The root executes independently of visible
game state, including the state-8 invocation's tail after collision publishes
state 9.

`run(ram, observer)` returns `Outcome::Handled` or atomic `Refused`.
The canonical input extent is exactly 128 KiB. Invalid extent is an error;
unsupported room, table, hook or accessed scroll inputs refuse without changing
any caller bytes or emitting observations. Room dimensions must each be 1–255
and their product at most 256. Scroll lookups are checked when actually accessed.
The only admitted finish hooks are zero and 9589. Camera table indices use
`index >> 1`, accepting odd values through 7; an invalid unused index on an
integer-stationary axis does not cause refusal.

## Calculate once, commit once

The implementation adapts main's existing bounded camera frame, selected room
constraints and Samus distance helper. It imports no old dispatch machinery.
It copies the same thirty input words and 256 scroll bytes into a compact
316-byte `State`, computes the complete frame once, and publishes fourteen
output words only after success. This replaces the old preparation's full
WRAM-sized buffer and avoids calling both `supports` and `dispatch_rom`.

The fourteen outputs are layer1 X/Y integer/fraction, ideal X/Y, previous Samus
X/Y integer/fraction, and calculated X/Y distance integer/fraction. Every other
caller byte is preserved. Calculations and admission use the same private
state, so the Y constraints see X's same-frame result. A constraint may change
private values before a later unsupported lookup; that entire partial result
is discarded on refusal.

`Observer::observe` receives passive `Enter { routine, operand }` and
`Return { routine }` events with the frozen thirteen-value `repr(u32)` Routine
enum. All routines return void. Operand is the low 16-bit target for
`CallFinishedHook`, and zero otherwise. The calculation buffers at most
eighteen events in source call order. After complete admission it emits them,
then commits output words. There are no services or hardware effects requiring
interleaving. An observer error leaves outputs uncommitted and propagates as
an owned error; the host must not retry C after observation begins.

## Preserved source behavior

Normal order is distance X → camera X → distance Y → camera Y → optional
finish hook → all four previous Samus words. Distances use the sign of the
16-bit integer difference to select a complete wrapping 32-bit pair difference,
then add one pixel. Integer equality selects autoscrolling even when only the
fraction changed. X follows the exact facing-byte/knockback/movement-byte/
acceleration XOR and the four-entry left/right tables.

Slow-grapple mode uses unsigned windows and three-pixel integer moves, keeping
fractions and existing distance words. Signed-high Samus X skips both manual
axes; signed-high Y skips only manual Y. Both autoscrollers still run afterward.
The frozen flag guards autoscrollers alone; it does not skip manual camera
following, distance calculations, the finish hook or previous-position
publication. The Spore Spawn finish hook raises layer1 integer Y to 464 when
it is at most 464 and preserves its fraction.

The six bounded room constraints retain the source's current-cell versus
neighbor-cell decisions, 8-bit multiplication, wrapping coordinates, integer
distance corrections, ideal overshoot fraction resets, and the extra 31-pixel
vertical margin for a non-blue scroll cell. The source-specific admission
bounds do not imply support for arbitrary recovered-C memory inputs.

## Source and reuse identity

Recovered C revision `578f90b3cc49557bb70060ad033bb90b8cf8ac50`, archive SHA-256
`69c78831351112887575f852dadf0281dd5c23cef890b07a268ae6cf60aa9193`.
The inspected source is retained by the integrator under
`restart/out/scroll/native/2c9c9a3cf51a059220afac1ee04727c956e45d067e10e17405bf3adbd268fc7f/source/src/`.

| Source | Responsibility | SHA-256 |
| --- | --- | --- |
| `sm_90.c:1262` | Root, finish dispatcher/hook, manual axes and distance routines through line 1381 | `38a661917219e6647826d25ea0bc45039c36bd54924f8cf968facb3f463389fa` |
| `sm_80.c:1889` | Six auto/trigger constraints through line 2095 | `171d73158e0e7323118b4bb142d4e7c79f3d74cdc4129a667aa8873d7eb785b7` |

Reused main-source algorithms were identified by the released scope packet at
checkout `193cad941713b3539d23e08238a79e7fb09dde61`; their file hashes were
checked again during this implementation:

| Main source | Adapted code | SHA-256 |
| --- | --- | --- |
| `crates/port/src/camera_scroll.rs` | Main frame, manual axes, finish hook and input/output contract | `5700c005e1306ee61771b6c7e603621f0ec15eb9568c5aa3c215dae17bd789f3` |
| `crates/port/src/room.rs:142` | Bounded scroll access and six constraints through line 305 | `e7b13c17ac668f8aba50a21aedfcd4d4dcf93f5e1cd2d4c610557cb56774ba72` |
| `crates/port/src/samus.rs:237` | Distance arithmetic through line 250 | `4f7e14ecbdf724305094d5924ec931b04dca422e8bcc3108fd5ff13531271234` |

The integrator's accepted ABI is `.scratch/restart-pipeline/camera-abi.md`.
Main's released source packet is
`.scratch/restart-collaboration/next-camera-scope.md`. The upstream license
is retained in `LICENSE.upstream.txt`. Verification limits are recorded in
`VERIFICATION.md`.
