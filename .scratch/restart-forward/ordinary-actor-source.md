# Prepared successor: complete Waver actor logic

Status: prepared source research; no claim, assignment or achieved gameplay stage.
Owner of this note: /root/waver_source. Integrator selects dispatch and publishes
progress. Prepared 2026-10-16 alongside player-motion25; persistence runtime
integration has queue priority. This supplements, and narrows one dependency in,
`.scratch/restart-verification/future-enemy-tracks.md`.

**Decision:** translate the complete four-definition Waver actor scope, retaining
the native enemy scheduler, generic instruction interpreter, terrain movement
and normal lifecycle. Enemy-core translation is not a prerequisite to this
scope. This establishes actor ownership only, never whole enemy lifecycle.

## Source identity and exact closure

All C references below are relative to
`/home/jim/sm-decomp-restart/restart/out/scroll/native/2c9c9a3cf51a059220afac1ee04727c956e45d067e10e17405bf3adbd268fc7f/source/src/`.

| Definition | Address / exact lines | Contract |
| --- | --- | --- |
| Waver_Instr_1 | A386E3, sm_a3.c:98–101 | Set current slot E=1; return input `const uint16 *jp` unchanged. Explicit `k` is ignored. |
| Waver_Init | A386ED, sm_a3.c:103–117 | Set horizontal velocity and animation state from parameter1 bit0; invoke local helper. Void. |
| Waver_Main | A3874C, sm_a3.c:119–143 | Horizontal move, conditional vertical sine move, phase and animation transitions. Void. |
| Waver_Func_1 | A387FE, sm_a3.c:145–154 | If F differs from C, install F-indexed animation and reset timers. Void. |

External leaves: `SignExtend8(uint16)->uint16`, sm_a0.c:3051–3054,
is `(a & 0x80) ? a | 0xff00 : a`, NOT a truncating i8 cast. In particular
`SignExtend8(0x8000)==0x8000`. Inline this exact expression in Rust as a
dependency; do not widen the claim to ownership of its global C entry.
Remaining leaves can stay typed native services:

| Service | Signature and C anchor | Required result/state |
| --- | --- | --- |
| Horizontal terrain | `uint8 Enemy_MoveRight_IgnoreSlopes(uint16 k,int32 amt)`, A0C6AB, sm_a0.c:3586–3627 | 0/1 collision result; all canonical WRAM changes, including x/subx and collision scratch. |
| Vertical terrain | `uint8 Enemy_MoveDown(uint16 k,int32 amt)`, A0C786, sm_a0.c:3629–3665 | 0/1 collision result; y/suby and collision scratch. |
| Sine | `uint16 SineMult8bit(uint16 angle,uint16 amplitude)`, A0B0C6, sm_a0.c:3068–3084 | Full u16 signed bit pattern, amplitude4; helper uses ROM A0B143. No float approximation. |

Call order matters: horizontal collision skips both sine and vertical services.
Otherwise sine receives low8(D),4; vertical amount is signed16(result)<<16.
Horizontal reversal uses overlapping bytes `(A>>8)|(B<<8)` narrowed to u16,
then the precise signed-byte negation in sm_a3.c:122–124. Vertical collision
sets D to low8(D+0x80); clear path adds2 as wrapping u16. Test `(D&0x7f)==56`
before processing E; both animation transitions may happen in one invocation.

## WRAM, ROM and scheduled boundaries

`cur_enemy_index` is WRAM 0xE54 (variables.h:767); record is
0xF78+index (enemy_types.h:4051). Native slots advance64 and are below0x800
(sm_a0.c:566–567). EnemyBase is48 bytes (ida_types.h:147–173);
Enemy_Waver is64 bytes (enemy_types.h:3591–3601).

| Record-relative offset | Field / use |
| --- | --- |
| 0x02/04,0x06/08 | x/subx,y/suby, consumed/updated by native terrain |
| 0x0a/0x0c | x_width,y_height, consumed by terrain |
| 0x18/1a/1c | timer,current_instruction,instruction_timer, helper writes |
| 0x30/32 | A/B packed horizontal velocity |
| 0x34/36 | C installed animation selector / D phase |
| 0x38/3a | E instruction event / F desired animation selector |
| 0x3c/3e | parameter1 read / parameter2 untouched |

Init does NOT initialize D, position, dimensions or arbitrary timers. Left init
leaves existing timers unchanged because F==C==0; right init installs selector1
and resets timers. Preserve untouched fields and neighboring slots.

ROM table `g_off_A386DB` (sm_a3.c:8,150) contains four little-endian words:
`86a7,86ad,86b3,86c7`. Direct pristine-ROM read at file0x1186db confirmed these.
Read runtime table from the Game's bound ROM, with source-valid selector0..3.
Do not silently assume a static table if the prepared ROM identity differs.
Initial list constant86a7 is ida_types.h:1997.

Pristine `/home/jim/sm-decomp/rom/sm.sfc` SHA256:
`12b77c4bc9c1832cee8881244659065ee1d84c70c3d29e6eaf92e6798cc2ca72`.
Read of file0x1186a7..0x1186e2 confirms lists86a7/86ad each have one frame and
812f sleep; lists86b3/86c7 each have four duration8 spritemap frames, then86e3
event instruction and812f. Event operands after86e3 are86c5 and86d9. Sprite
map pointers and rendering remain native assets; no new art, atlas or ROM
reconstruction is required. ROM data inspection is not live verification.

Native dispatch anchors are sm_a0.c:690–691 (`CallEnemyAi` init/main) and1307
(`CallEnemyInstr` instruction). Initialization calls AI at410; frame scheduler
calls selected AI at1764, then generic instructions at1775. Generic interpreter
sm_a0.c:3395–3417 passes `pc+1`, preserves bankA3 and consumes the returned
pointer. Hook these actual actor dispatch calls; retain actual-C originals for
reference mode. The helper is locally Rust-owned under init/main, with a direct
Fixture entry to check it. It is not a separate scheduled root.

Instruction ABI must encode the returned bankA3 ROM offset and reconstruct the
same pointer. It returns jp unchanged, NOT jp+1, zero or a void result. Capture
both explicit k and current index; source writes current index even when they
differ. If scheduled admission requires equality, keep the direct actual-C
Fixture capable of proving this distinction.

Movement services are synchronous private-WRAM round trips: flush pending Rust
writes, call actual native service once, copy ALL131072 bytes back, then resume
Rust from that state. Do not keep stale field snapshots across a service.
Terrain retains room/level/BTS state and slope/extension logic
(sm_a0.c:3686–3720). Spike reaction sm_a0.c:3427–3432 can call SpawnPLM; preserve
its native continuation and ordered effects. Parent/child context and reentry
guards must permit this exact service continuation without admitting unrelated
recursive actor callbacks. No fallback after owned service invocation or failure.

## Specific main reuse and missing work

Main snapshot observed:193cad941713b3539d23e08238a79e7fb09dde61.
`crates/port/src/enemy_ai.rs`: constants4–22, animation62–73,
main419–449, dispatch509–521 supply reusable actor arithmetic. Tests47–53
in `crates/port/tests/enemy_ai.rs` preserve unusual left-init sign extension.
Bounded search of `crates/port/src` found no Waver_Instr_1/86e3 implementation.
Add its three-line semantic contract from C.

Do not transplant the module's terrain implementation/Host whole: its Host only
models PLM allocation and its main calls Rust terrain and enemy::dispatch_rom.
Adapt actor calls to the three typed native services above. Source reuse gives
no restart verification or acceptance credit. Main helper rejects F>=4 before
checking F==C; C does not read ROM when F==C. Declare canonical F0..3 admission,
or preserve the C no-op domain for equal selectors; do not accidentally claim
all-u16 equivalence from that guard. Unknown/out-of-domain admission must fail
before side effects under the chosen policy.

## Disjoint implementation proposal and verification needs

Material worker owns only new `restart/waver/Cargo.toml`, `src/lib.rs`,
`tests/actual_c.rs` (or equivalent declared tests). Native worker owns only new
`restart/native-host/src/waver.c`, `waver_hooks.rs`, `waver_types.rs`,
`waver_prepare.rs`, and `examples/waver-host-control.rs`.
Integrator owns workspace/Cargo.lock, native lib/prepare/shim shared wiring,
Game/runtime_cli integration, shared effects and progress/index. Effect IDs are
allocated at actual dispatch;25 already reserves56..59. No ownership here of
player-motion, persistence, enemy-core, common instructions or terrain files.

Use meaningful actual-C Fixtures with full WRAM, scalar/pointer results and
ordered service/effect comparison:

1. Paired left/right init with nonzero sentinel D/timers and neighboring slot.
2. Horizontal obstruction reverses packed fractional velocity, flips direction
   and proves absence of sine/vertical service calls.
3. Open horizontal then vertical collision reverses low-byte phase; clear-path
   phase wrap and sine-positive/negative cases check signed16 amount handling.
4. Phase54->56 with E already1 proves transient special animation followed by
   reset; helper changed/unchanged selector checks timer behavior separately.
5. Actual instruction callback returns identical jp and sets E on current slot;
   chain instruction -> main consumes E and returns to normal animation.
6. Native CallEnemyAi/CallEnemyInstr composition shows positive Rust ownership,
   preserved generic instruction continuation, and zero actor fallback.
   A terrain spike continuation case is necessary if admitted service domain
   includes it; native parent guard correctness is not established by flat cases.

Admission/owned-error checks must include invalid slot, ROM identity/extent,
unsupported selector domain, service outside allowed entry and invalid scalar
or pointer result. These are future checks, not performed in this preparation.
Whole live lifecycle/contact/death/draw and original-ROM route parity remain
separate. A live replay is useful only when it naturally exercises Waver roots;
no fabricated root calls or arbitrary arrival replay count as live ownership.

Remaining implementation gaps: new instruction arm; three-service adapter and
native hook ABI/guards; shared registration; runtime installation; frozen-source
actual-C/composition/admission checks and any selected live evidence. No missing
actor C bodies or enemy-core translation blocker was found in this bounded scope.

## Source hashes and cleanup

### Bounded follow-up: sine and terrain callback closure

Keep sine as one typed native service for this scope; it uses only arguments,
ROM and local arithmetic in sm_a0.c:3068–3084, with no WRAM writes. Its128-byte
table at A0B143/file0x103143 is pinned by the pristine ROM hash above. Direct read:

```text
00 06 0c 12 19 1f 25 2b 31 38 3e 44 4a 50 56 5c
61 67 6d 73 78 7e 83 88 8e 93 98 9d a2 a7 ab b0
b5 b9 bd c1 c5 c9 cd d1 d4 d8 db de e1 e4 e7 ea
ec ee f1 f3 f4 f6 f8 f9 fb fc fd fe fe ff ff ff
ff ff ff ff fe fe fd fc fb f9 f8 f6 f4 f3 f1 ee
ec ea e7 e4 e1 de db d8 d4 d1 cd c9 c5 c1 bd b9
b5 b0 ab a7 a2 9d 98 93 8e 88 83 7e 78 73 6d 67
61 5c 56 50 4a 44 3e 38 31 2b 25 1f 19 12 0c 06
```

Concrete terrain PLM continuation: A0C2DA/file0x1042da has fifteen zero words,
then D094 at index0x0f. The following bytes are code, so this is NOT a proven
128-entry valid header table despite the source mask0x7f. For this finite spike
domain admit indexes0..0x0f; do not infer safety for higher arbitrary BTS values.
D094 header at84D094/file0x25094 is `d4 b3 53 cd`: setup84B3D4 and listCD53.
`PlmSetup_D094_EnemyBreakableBlock`, sm_84.c:1370–1374, only clears the target
level_data high nibble and returns0. `SpawnPLM`, sm_84.c:195–222, allocates the
slot and calls its setup synchronously; it does NOT run the PLM instruction
list. Thus this admitted continuation does not synchronously enter current Rust
item-instruction/player-motion callbacks. Allocation/full-slot paths and ordered
SpawnPLM effects3/4 still need composition verification. The recovered SpawnPLM
wrapper records those effects at196–200. Existing prepare.rs:263 only wraps
the distinct Ceres setupB9F1. Recheck current shared registrations at dispatch;
future hooks or broader terrain domains can change this conclusion.

This is enough to prepare the native bridge without a generic bankA3 interpreter
or whole enemy-core survey. Terrain extensions/slopes stay native; malformed
chains or undefined table indexes are outside this finite evidence proposal and
need explicit preflight/unsupported-domain policy before admission.

SHA256 sm_a3.c `fb8a23edb3353a970e343ced1b449b269979182b2267bae43d4ae5157ec871f6`;
sm_a0.c `a2cee5c286304e7d606357ac4960413e5fe3dfe585d468d0f41a654aa093e8d3`;
enemy_types.h `251af063d550e9e9ade1de9840df7617c8cb3648bbe241587a654b3ea3085f2d`;
ida_types.h `12974c5708740164692dac151104020b53e8316e7378521c974ddf602d1f28ff`;
variables.h `e2ce28d07ceff91129e93417099f33f20f161c37e95e745c575d0e30b3fd5585`;
main enemy_ai.rs `13fed8e12c622ff5303cb98e6f7434aad6d11a9cfc91251f83454e8aacf1ba91`.

Source-only reads and this one owned note. No builds, tests, game, capture,
claims, worktrees, source mutations or producers. Cleanup0 bytes; no generated
run directories. Available disk at inspection99048382464 bytes. Existing source,
ROM and retained evidence dependencies preserved. No translation stage published
because this is preparation only; integrator owns future assignment/publication.
