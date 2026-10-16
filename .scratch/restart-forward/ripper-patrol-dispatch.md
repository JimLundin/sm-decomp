# Ripper initialization and patrol: prepared successor after Waver

Status: prepared source packet; root alone selects implementation admission.
Prepared directly by01a145c7-f377-7b90-83ee-4139073f7d79, without workers or producers.
Current source inspected: restart35f3bb98a191a8b7098b377e7c9dd287f9accc23;
HEAD advanced toed98027729fb985525fe4b5ce714b1eb209c2e5d during preparation.
Waver27's source/finite terrain design is a reuse candidate; its pending controls
are not silently treated as completed acceptance. Root freezes current source.

## Coherent scope and native ownership

Select the complete initialization/horizontal-patrol operation for both Ripper
variants. This is a patrol-family scope, not whole actor or lifecycle ownership.
Four definitions in pinned recovered C578f90b:

| Entry | Address | Definition in sm_a2.c | Operation |
| --- | --- | --- | --- |
| JetInit | A2E318 | 3215–3233 | Speed/direction/animation initialization |
| JetMain | A2E353 | 3235–3254 | Horizontal collision, reversal and animation reset |
| RipperInit | A2E49F | 3273–3291 | Ordinary variant initialization |
| RipperMain | A2E4DA | 3293–3312 | Ordinary variant patrol/reversal |

Source root:
`/home/jim/sm-decomp-restart/restart/out/scroll/native/2c9c9a3cf51a059220afac1ee04727c956e45d067e10e17405bf3adbd268fc7f/source/src/`.
sm_a2.c SHA256 d62422845f0fee20c929ed7a635a0eb397d376a6b2d62b9c0aab21e3c18a315e.

All four signatures are `void(void)`; they obtain the current enemy slot from
canonical WRAM. No local dependency bodies or actor-specific instruction callback
are called by these roots. Both Main roots call exactly one temporary native
service: `uint8 Enemy_MoveRight_IgnoreSlopes(uint16 k,int32 amt)` atA0C6AB
(sm_a0.c:3586). Native result is0/1 and full WRAM copyback is required.

`JetPowerRipper_Shot`A2E3A9 at3260–3271 and `Ripper_Func_1`A2E3A0 at3256–3258
remain native. Shot calls NormalEnemyShotAi then chooses a frozen spritemap;
Func_1 calls Enemy_NormalFrozenAI_A2. Contact, frozen timing, shot damage/death,
active-list/AI selection, generic instructions, rendering and drops remain native.
Do not list these bodies as translated or live-owned by this scope. This explicit
cut avoids bringing an unresearched normal-shot service into a patrol assignment.

## Exact data contract and arithmetic

Current slot: wordE54, aligned0x40 and below0800. Enemy record starts0F78+slot.
Enemy_Ripper and Enemy_JetPowerRipper share EnemyBase plus six words and two
parameters (`enemy_types.h:1189–1199,2711–2721`; accessors3992/4033).

| Record offset | Field |
| --- | --- |
| 02/04,06/08 | X/subX,Y/subY; native terrain input/output |
| 0A/0C | X/Y radii; native terrain input |
| 18/1A/1C | timer, instruction pointer, instruction timer |
| 34/36 | C low16 fraction, D high16 integer of horizontal amount |
| 38 | E byte offset into the selected linear-speed table row |
| 3C/3E | speed parameter1 / direction parameter2 |

Init sets instruction, E, D and C only. It leaves both timers, positions,
A/B/F and unrelated fields untouched. Direction is parameter2 **nonzero**, not
bit0. `uint16 v2=8*parameter1` truncates to16 bits; E=v2 and table word index=v2>>1.
Nonzero parameter2 reads row words0/1 into D/C; zero reads words2/3. Preserve
wrapping aliases if admitted: parameter2001 produces the same row as0001.

Main sends `((D as u32)<<16 | C as u32) as i32` to native movement. No collision
means no actor writes after the service. On collision, read current D after full
copyback: nonnegative D selects row words2/3, negative D words0/1. Set instruction
timer1 and general timer0 only on collision. Do not implement reversal by simply
negating current speed: C explicitly reloads the stored row, so mismatched but
admitted current velocity/table values distinguish the behavior.

| Variant | Init parameter2==0 / !=0 | Collision from D>=0 / D<0 |
| --- | --- | --- |
| Jet | E2E0 / E2F4 | E2E0 / E2F4 |
| Ordinary | E48B / E477 | E48B / E477 |

These are bankA2 instruction offsets; Main's signed constants encode those same
u16 values. D==0 takes the nonnegative branch. Row0 remains zero speed after a
collision but still resets animation/timers according to that branch.

## Finite ROM domain, confirmed directly

`sm_rtl.h:171` maps kCommonEnemySpeeds_Linear toA28187; the next named quadratic
table startsA2838F. Seal the contiguous520-byte sliceA28187..A2838E as **65
supported rows of four u16 words**. This is an explicit finite input domain,
not a claim about arbitrary C pointer reads beyond it.

Pristine ROM12b77c4bc9c1832cee8881244659065ee1d84c70c3d29e6eaf92e6798cc2ca72;
LoROM file slice110187 of length520 SHA256:
`b8c7ce12514576f554227bdf8a4e22d00bee41cd4b7b3aeea5dac2a594ecbcfb`.
Direct xxd/dd reads, no game execution or ROM mutation:

| Row | Four native little-endian words D+,C+,D-,C- |
| --- | --- |
| 0 | 0000,0000,0000,0000 |
| 1 | 0000,1000,FFFF,F000 |
| 2 | 0000,2000,FFFF,E000 |
| 63 | 0003,F000,FFFC,1000 |
| 64 | 0004,0000,FFFC,0000 |

Bind that exact slice in the prepared runtime ROM at admission, or explicitly
bind its runtime bytes with the selected native identity. Do not equate pristine
ROM inspection with prepared-ROM/runtime acceptance. Proposed Init admission:
`parameter1.wrapping_mul(8) <= 0x0200`, retaining its source wrap aliases.
Proposed Main admission: E<=0200 and E divisible8; this intentionally describes
initialized row states rather than all arbitrary u16 pointer offsets. Validate
all potentially required row words before invoking the native service; reject
unsupported row/ROM unchanged before writes. Unsupported rows are outside this
scope, not asserted to be invalid in every possible C context.

The four20-byte instruction lists were also inspected at file1162E0/1162F4 and
116477/11648B: four duration/spritemap pairs (8,7,8,7), then generic80ED loop to
the same list. Their execution and sprite maps remain native. No new pointer
return ABI, actor-specific instruction body, art or asset reconstruction needed.

Main reuse at193cad9: `crates/port/src/enemy_ai.rs:271–341,473–474` supplies both
init/main arithmetic. Adapt selected logic, not its old Host or Rust terrain.
Its `parameter<=63`/index<=252 guard excludes row64 and source wrap aliases;
**do not copy that guard as if it were native semantics**. Its comment ending
the linear table at8387 omits the directly observed row64 before838F. Reuse does
not inherit old verification. This packet's domain and independent expectations
must cover the64th endpoint and at least one wrapping alias.

## Reuse Waver's movement pattern with correct slot state

New pure Rust `sm-ripper` in restart/ripper/, with Entry repr(u32)
JetInit=0,JetMain=1,RipperInit=2,RipperMain=3; address(), ALL and TryFrom.
Proposed Interface:

```rust,ignore
pub trait Services {
    fn available(&self) -> bool;
    fn move_horizontal(&mut self, ram: &mut [u8], slot: u16, amount: i32)
        -> Result<u8, String>;
}
pub fn execute(entry: Entry, ram: &mut [u8], rom: &[u8], services: &mut impl Services)
    -> Result<Option<()>, String>;
```

None is unchanged refusal; Some(()) handled. All malformed inputs/domain checks
precede writes. Main checks availability before movement, validates native result
0/1, and propagates service failure fatally with no C retry. Init needs no service.

The inspected Waver bridge uses active-root/issued-buffer/generation/poison
checks and a finite terrain predicate (`native-host/src/waver.c:29–74`). Its
velocity comes from A/B offsets30/32, whereas Ripper uses C/D34/36. Do not call
sm_native_waver_service or sm_native_waver_available from a Ripper root, and do
not borrow Waver's three-service completion state. Reuse that reviewed pattern
in a dedicated one-service adapter with the correct amount fields.

The finite terrain domain can initially retain Waver's conservative bounds:
room width/height1..255, total cells<=3200, radii1..16, movement within+/-16px,
inside-room scan rectangle, byte-row bound, zero/one-hop valid extensions,
spike BTS masked index<16 and exact D094/setupB3D4 table. Its extra vertical
margin may remain conservative although Ripper has no vertical step. Reuse its
facts and finite fixtures, not an assumed pass of still-running27 gates. A shared
parameterized helper is an integrator choice after27's source freezes; creating
a new generic enemy engine is unnecessary for this scope.

MoveHorizontal is synchronous full-WRAM publish/call/copyback. Native motion
runs once; its spike PLM setup allocates/changes the block synchronously with
effects3/4 and does not execute a PLM instruction list. Preserve the existing
nonreentry/pointer/error constraints from27. Out-of-domain canonical runtime
states may retain counted native fallback; malformed diagnostic inputs must not
be sent to original C. Exact admitted vs refused counts remain explicit.

## Integration edits and disjoint work

| Owner | Exact proposed writable files |
| --- | --- |
| Rust worker | New restart/ripper/Cargo.toml; src/lib.rs,memory.rs,patrol.rs,fixtures.rs,tests.rs; README.md,HANDOFF.md and copied-source license |
| Native worker | New native-host/src/ripper.c,ripper_hooks.rs,ripper_types.rs,ripper_prepare.rs; examples/ripper-host-control.rs; scoped handoff |
| Root | Existing Cargo/lockfile, Game/lib, prepare/observe declarations, sibling guards, runtime installer/selection/results, shared effect IDs, index, all producers/publication/cleanup |

Intercept the four exact `CallEnemyAi` cases in sm_a0.c:659–663, leaving the
interleaved JetPowerRipper_Shot case unchanged. No bank-wide dispatcher change.
Use local service0, dedicated four-root counters and Begin/End observations.
Allocate shared effects against current27 namespace (60..63 reserved by27);
this packet reserves none. No new instruction pointer/output encoding needed.

Waver preparation already wraps Enemy_MoveRight_IgnoreSlopes for passive service
observations. New preparation must preserve that wrapper and make inactive
actor observations no-ops. One checked option wraps the existing public function
with a Ripper observation layer while preserving Waver's layer and actual body;
root selects the exact reviewed patch. Never overwrite/remove27's instrumentation
or count a Ripper call as a Waver request. Verify both registrations coexist,
mutual active guards prohibit nested actor callbacks, and one movement occurs.
Workers own only new files; root coordinates this shared generated-source detail.

## Compact verification and independently earned reach

One shared fixture-support list, root-owned locked/offline commands and a frozen
native recipe. Required distinctions (group cases; no inflated quota):

1. Both variants/directions, parameter2=8000 proving nonzero semantics; rows0,1,
   63,64 and wrapping alias2001. Init sentinel timers/A/B/F and neighboring slot.
2. Each Main moves through air, preserves timers/instruction after no collision,
   and matches complete native WRAM including terrain scratch and fractions.
3. Left/right obstruction reloads correct stored speed and animation; include
   fractional carry/borrow, D==0, and current speed different from stored row.
4. Init->Main->collision->Main chain through actual CallEnemyAi: state is inherited,
   not reseeded to success. Actor root counters positive; generic list continuation
   stays native and is not claimed Rust-owned.
5. One relevant real spike/PLM allocate/full-pool comparison if that domain is
   admitted. Reuse27's fixture layout and facts while exercising the new root.
6. New adapter unavailable/dirty refusal, wrong issued buffer/generation and
   post-request error controls; invalid collision scalar is fatal. Compact
   coexistence check with inactive Waver observation proves one native movement
   and unchanged27 counters. Expand only on a concrete changed interaction.

No full replay of accepted player/persistence/item matrices, no arbitrary ROM
capture, no repeated22navigation experiment. Native wrapping changes get focused
composition coverage; unchanged table/body receipts are not new acceptance for
Ripper. Source integration can precede natural Ripper route reach. Publish four
source stages only when earned, live-exercised only on actually reached roots.
Shot/frozen/contact/death/render and original-ROM parity remain explicit native
or unverified dependencies. Root records exact source, checks and cleanup before
retiring future worker worktrees.

## Preparation completion

Read primary C, current restart Waver interfaces and selected main reuse; direct
finite ROM reads only. No worker was spawned, no source/claim/worktree/build/
test/game/capture created. Cleanup0bytes, no run utility applicable. Available
disk at inspection97345433600bytes. Source notes preserve original production
fences, caches and sealed evidence. The scope is ready for root's ordinary
baseline/interface/assignment freeze; no missing patrol C body or enemy-core
rewrite was found.
