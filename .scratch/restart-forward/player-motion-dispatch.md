# Player vertical motion: prepared implementation packet

Status: prepared successor; integrator assigns source baseline/worktrees.
Prepared by session 01a145c7-f377-7b90-83ee-4139073f7d79 on 2026-10-16.
No gameplay implementation or acceptance is claimed by this packet.

This makes the retained [seven-definition proposal](../restart-verification/next-player-motion-contract.md)
concrete at the native Interface. Its exact canonical fields, fixed-point rules,
solid-enemy list rules and feedback tables remain authoritative source findings;
read them once rather than commissioning another player survey. The additional
[terrain findings](terrain-dependency.md) cover the previously missing foreign
continuation. The recovered C pin remains 578f90b3cc49557bb70060ad033bb90b8cf8ac50.
Restart adapter inspection was at 67409e3d939fc2ecbee15b23cd15c3d5afb53af6;
5363d46c70483f13de322a913977be4d797bac16 subsequently claimed SRAM implementation.
Neither commit is silently prescribed as the future merged candidate baseline.

## Scheduling and useful outcome

The integrator confirmed no overlapping player assignment. Current source lanes
are energy route22 and SRAM codec24 (preparation23 completed). Put this packet in
the prepared queue. When either source slot frees, the Rust worker can implement
this module while the other existing source scope continues. Its native adapter
is support for the same scope. Do not delay SRAM, replay energy again, or wait
for a new geographical milestone to translate player motion.

The delivered behavior is the complete upward/downward displacement operation:
solid-enemy clipping, fixed-point displacement/position updates and collision
feedback to pose calculation. Terrain collision remains explicitly native.
It does not grant normal-movement dispatcher, terrain engine, input, gravity,
horizontal movement, enemy lifecycle or whole-player ownership.

## Module Interface to freeze

Create `sm-player-motion` in `restart/player-motion/`. Keep its dependency on
native-host indirect through one small synchronous trait, like item-instructions.
Use canonical 131072-byte WRAM, the exact local constants from recovered C and
signed 32-bit displacement. No pointer into native memory crosses this Interface.

Suggested declaration, to be frozen together by the two implementers before
parallel edits (names are new, not existing callable functions):

```rust,ignore
pub enum Entry {
    MoveUp(i32), MoveDown(i32), CheckSolidEnemy(i32),
    MoveUpNoColl(i32), MoveDownNoColl(i32), UpFeedback, DownFeedback,
}
pub enum Output { Done, EnemyCollision { collision: u16, amount: i32 } }
pub trait Terrain {
    // Immutable capability/domain check; no native call, mutation, or observation.
    fn available(&self) -> bool;
    fn move_vertical(&mut self, ram: &mut [u8], amount: i32) -> Result<i32, String>;
}
pub fn execute(
    entry: Entry, ram: &mut [u8], terrain: &mut impl Terrain,
) -> Result<Option<Output>, String>;
```

`None` means unchanged refusal before any request; `Some` means handled. Error or
panic after acceptance is an owned failure. The two composed roots return Done;
only the checker returns its native compound result. The remaining five entries
are direct Fixture access to the same implementation called internally, not five
additional scheduled hooks. A private helper may replace a public enum variant
if actual-C tests still cross the actual implementation Interface.

Preflight all locally owned invariants from the retained proposal before writes:
WRAM size, bounded enabled enemy list and aligned slot indexes, reachable feedback
table index, signed-negation limitation, and available terrain capability for a
possible no-enemy-hit path. Preserve byte aliases and all unchanged bytes. Do not
call the native service merely to find out whether it is supported.

The trait's native implementation owns valid terrain-state admission. Distinguish
normal source-valid canonical game state from arbitrary injected Fixture bytes.
For first actual-C cases use the finite valid terrain layouts documented below
and in the terrain note; invalid native array indexes and extension cycles are
host refusal controls and must never be sent to C. This packet does not demand
reimplementing bank94 collision as a second preflight engine. If the implementer
chooses a narrower terrain domain, record the actual immutable guard and report
its mixed-mode refusals. A bounded fixture pass does not prove arbitrary terrain
or uninterrupted full-route ownership. Original gameplay already delegates this
solver to native C; this new adapter must preserve, not broaden, that contract.

## Exact scheduled interception

Use `camera_prepare.rs:35`'s existing body-wrapper technique, not linker wrapping
or a caller catalog. Match each full definition once in pinned `src/sm_90.c`:

- `void Samus_MoveUp(int32 amt)` at 90:93EC.
- `void Samus_MoveDown(int32 amt)` at 90:9440.

Retain its original body under a distinct private name and export the original
name as the wrapper. All same-file callers and existing dispatch then cross the
wrapper. Root Begin chooses Rust or original C, and root Done observes successful
return. A private original-body name must never itself redispatch the root.
The unchanged compiled C root is the actual-C reference; the Rust root calls the
same native terrain implementation through the service below. Keep helper body
references callable for direct Fixture comparison without adding a second game
scheduler. Patches must have exact one-match validation and retained before/after
records, using existing `replace_once`/`structural_end` support.

Two native scheduled selectors: Up=0, Down=1. These are local to the new module.
The callback carries `(context, selector:u32, amount:i32)` and returns existing
status semantics: 1 handled, 0 unchanged refusal, negative fatal. No invented
ROM address for the addressless Down feedback definition.

## One native terrain call

Provide module-local service0 for the real `Samus_MoveDown_NoSolidColl`94:9763.
A concrete proposed ABI is:

```c
int sm_native_player_motion_terrain(uint8_t *private_wram,
                                   int32_t amount, int32_t *applied_amount);
```

Return 0 on success; negative on invalid adapter context/error. Signed returned
movement is written separately so negative valid displacement cannot be mistaken
for an error status. The native callback wrapper validates selector, active root,
callback ownership, buffer and output pointers, no service already active, and
no incompatible parent/sibling registration. Rust also validates that the buffer
is this invocation's exact Transaction allocation.

Sequence:

1. Mark a foreign request before entry, including requests that fail. Validate
   adapter context before copying state. Require `cur_coll_amt32 == NULL`.
2. Set service-active, publish all private WRAM to `g_ram`, record service Begin
   with selector/amount and call the original terrain routine exactly once.
3. Keep every block reaction and SpawnPLM setup on that stack, including the
   quicksand setup that mutates the current collision amount through the external
   pointer. Do not route SpawnPLM through the projectile adapter: `pilot.c:153`
   intentionally rejects the pointer context this service requires.
4. Require the pointer NULL again after normal return. Record real result and
   service End, copy all native WRAM back, clear service-active, and return the
   applied displacement. Root then performs its actual feedback/displacement
   operations in original source order and commits its final private WRAM.
5. An accepted service error, failed observer, panic or escaped effect terminates
   the invocation. There is no replay in C or roll-back-to-refusal shortcut.

Existing `Transaction` in `native-host/src/transaction.rs:16` provides initial
copy and final handled/refused checks. Existing `item_instruction_hooks.rs:54`
and `item_instruction.c:189` demonstrate request-before-call, invocation-buffer
identity, synchronous full-WRAM publication/copyback and native error handling.
Use that pattern. A new persistence transaction covering SRAM is unrelated;
player motion still uses the existing WRAM transaction.

Do not store the collision pointer, clone a Game, retain a mutable native-memory
borrow across Rust, reset external failure/effect state after the service, or
allow a Rust callback to recursively enter collision. Existing host singleton,
owned hook boxes, panic containment and ROM preparation remain in force.

For the bounded terrain setups, SpawnPLM performs immediate native setup and
returns; it does not call the item instruction scheduler. Door reaction schedules
state9 rather than executing the room-transition engine. Therefore these cases
need no permissive nested item/room registration exception. Confirm that fact
in the retained prepared C; any future terrain continuation reaching another
scheduled Rust module requires a named parent-child contract and fresh controls.

## Disjoint writable files

| Owner | Exact new files / responsibility |
| --- | --- |
| Rust worker | `restart/player-motion/Cargo.toml`; `src/lib.rs`, `memory.rs`, `vertical.rs`, `solid_enemy.rs`, `feedback.rs`, `fixtures.rs`, `tests.rs`; `README.md`, `HANDOFF.md`, upstream license notice if code is copied |
| Native worker | `restart/native-host/src/player_motion.c`, `player_motion_hooks.rs`, `player_motion_types.rs`, `player_motion_prepare.rs`; `restart/native-host/examples/player-motion-host-control.rs`; one native handoff note under the assigned scratch scope |
| Integrator | All existing shared files, workspace/lockfile/index, `native-host/src/lib.rs`, `prepare.rs`, `observe.h`, existing consumer guards, runtime installation/options and result serialization; execution, publication and cleanup |

Rust fixtures are defined once behind `fixture-support`; native control imports
them rather than maintaining a competing matrix. The native worker's exact new
files can compile after root's small registration patch; no worker writes shared
registration just to make its isolated tree standalone. Exact worktree, branch,
baseline, source routine list and stage publication are set by root at assignment.

Shared registration checklist is small and concrete:

- `restart/Cargo.toml` member/dependency and native-host dev-dependency on the
  fixture feature; existing lockfile regenerated only if needed.
- `native-host/lib.rs` new modules/types, library symbols, optional owned
  registration/setter/counters and unregister-on-drop, matching existing consumers.
- `prepare.rs` include new C/types/hooks/preparation in recipe identity, invoke
  instrumentation, write/compile the adapter; `observe.h` declarations.
- Include player-active in existing consumer entry/registration guards. Initial
  player entry rejects active projectile, scroll, Ceres door, room transition,
  door scrolling, camera, item and newly admitted persistence callbacks. Audit
  both directions, coordinating with concurrent SRAM registration once on root.
- Install the trait bridge through runtime selection. Allocate any effect tags
  from the then-current shared namespace; local selector0 needs no shared enum.
  Preserve existing CSV and artifact fields; place new observations separately.

## Focused verification, then independently earned live extent

Run once on the exact integrated candidate using root's shared producer budget.
This is a proposed finite plan, not a claim of passing checks or exact case count.

1. `cargo test --manifest-path restart/Cargo.toml --locked --offline -p sm-player-motion`
   checks the distinct arithmetic/list/feedback cases already enumerated in the
   retained proposal. Include every down-feedback table entry through compact
   table cases, not a separate executable per table cell.
2. `cargo run --manifest-path restart/Cargo.toml --locked --offline -p sm-native-host
   --example player-motion-host-control -- <frozen control arguments>` invokes
   actual C and Rust through the same Game/native library. The native worker
   defines exact CLI arguments in its handoff; no placeholder command is called.
   Compare full WRAM, raw checker result, ordered real terrain/PLM events and
   meaningful unchanged-state sentinels. Use fresh child processes where the
   native singleton requires isolation; one compiled helper serves all cases.
3. Retain compact adapter controls: no-request unchanged refusal; changed-private
   refusal fatal; request-then-refusal fatal; post-request error/panic fatal;
   wrong active context/buffer; collision reentry rejected before it corrupts the
   outer pointer. Exercise a real quicksand pointer modification and a native
   slot-full return, not just matching arithmetic mocks.
4. Root may use one existing successful fixed-input route for live comparison
   when both roots are wired. The current 24832-input retained route is a possible
   anchor, not a promise that every branch is reached. Observe positive root
   completions, service use and actual branch reach; preserve previous artifact
   compatibility. Never rerun each previous milestone separately. Missing player
   cases remain actual-C-only rather than blocking unrelated source integration.

Terrain distinctions to combine into a compact matrix: zero/no-scan versus air,
floor/ceiling fractions, one square and one nonsquare slope, parity and a bounded
extension, fresh/invincible spike and harmless vertical spike-air, door target
versus non-room target, quicksand amount mutation/carry/full pool, crumble/speed
block state. See exact ROM triples in terrain note. Unsupported source states
stay separate refusal controls; do not invoke undefined native behavior for a
spurious differential pair. No broad original-family sweep, new emulator,
per-run full inventory scan or full-ROM replay gate is added here.

Publish translating at actual claim; translated/verified/integrated separately
with exact evidence, then live-exercised only for reached routines. Count native
terrain calls explicitly. Cleanup follows the normal two passes after writers
stop and preserves reference snapshots/cache dependencies; root retires the
integrated worker trees. This preparation itself generated only Markdown, no
run/build/capture/worktree, so cleanup is zero bytes.

## Keep the recurring cost bounded

The existing transaction copies 128 KiB; vertical roots may run several times
per update. Start with that proven ownership mechanism and measure its actual
candidate cost before changing it. Full WRAM comparison belongs in finite pairs;
live instrumentation should use counters and small ordered scalar records, not
retain a 128-KiB snapshot for every movement call. Reuse one prepared native
library/reference identity across compatible pairs and do not rebuild it per
case. These are implementation choices, not new performance acceptance gates.
