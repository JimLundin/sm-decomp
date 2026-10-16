# Waver actor: second prepared successor

Status: prepared source assignment; root selects after persistence runtime packet.
No implementation claim, source changes or new gameplay stage. Exact source,
ROM bytes and case distinctions: [ordinary-actor-source.md](ordinary-actor-source.md).

The complete four-definition actor can run under the existing native enemy
scheduler. It does not wait for enemy-core translation. Own Init A386ED, Main
A3874C, event instruction A386E3 and local animation helper A387FE. Generic
instruction timing/dispatch, collision terrain, contact/damage, draw/death/drop
and active-list iteration stay native. Keep these limitations in stage reports.

## Ready interface

New pure Rust `sm-waver` crate under `restart/waver/`. One execute Interface
accepts typed Entry, current canonical WRAM, pinned runtime ROM and a small
synchronous Services implementation. Four entries are Init, Main, Instruction
(explicit k and bankA3 jp offset) and SelectAnimation (direct Fixture entry).
Only Instruction returns Continue(same jp offset); others return Done. Main's
local calls invoke the same Rust animation helper without artificial scheduled
callbacks. Use Result<Option<Output>,String>: None unchanged refusal, error fatal
under the native invocation contract.

Current slot comes from canonical E54 and must be aligned64 below0800. Preserve
Instruction's source distinction: explicit k is ignored for the E-field write,
which uses current slot. The actual instruction continuation is jp, not jp+1.
An interface can admit aligned bankA3 upper-half offsets for real scripts while
explicitly excluding synthetic invalid pointers; no native pointer enters Rust.
Expose full128KiB WRAM and validate available ROM/table extents before writes.
Canonical F0..3 is the straightforward initial domain. If direct helper fixtures
include equal F==C outside0..3, preserve the no-op before indexing rather than
claiming C equivalence for a stricter unreported guard. Init can overwrite F,
so do not reject its irrelevant old F without declaring that narrower domain.

Three typed services, available before Main mutates state:

| Native service | Call signature / return | Context |
| --- | --- | --- |
| MoveHorizontal | Enemy_MoveRight_IgnoreSlopes(k:u16, amount:i32) -> u8 collision | Main only, active current slot |
| Sine | SineMult8bit(angle:u16, amplitude:u16) -> u16 signed bits | Main only; angle low8(D), amplitude4 |
| MoveVertical | Enemy_MoveDown(k:u16, amount:i32) -> u8 collision | Main only, active current slot |

A proposed Rust trait has `available()` and `move_horizontal`, `sine`,
`move_vertical` methods returning Result; movement methods take the active
private WRAM slice. Sine itself has no WRAM writes, but observes call order and
returns the exact native signed bit pattern. Native adapter can use one local
service enum and a scalar result out parameter so valid signed values and
adapter errors cannot be conflated. Allocate global event IDs at root dispatch;
none are reserved by preparation.

Use existing Game-owned hook/Transaction pattern, not main's old generic
string-symbol dispatcher. Each request is counted before entry; movement copies
all private WRAM into native, executes once, copies all state back and only then
resumes Rust. Keep no stale borrowed record or cached mutable state over a native
call. Unavailable services/refusal are decided before writes; errors after a
service are fatal with no C retry.

## Actual actor interception and registration

Exact existing dispatcher cases are identified in source note:
CallEnemyAi at sm_a0.c:690–691 for Init/Main and CallEnemyInstr at1307 for the
event instruction. Have new waver_prepare.rs replace those exact cases once with
new typed adapter entrypoints. Preserve original C functions as the native
reference; adapter C mode invokes them directly, without redispatch loops.
No generic bankA3 interpreter, normal-enemy scheduler replacement or root gameplay
index edit belongs to a worker. Direct helper comparison invokes its original
C body and the same Rust helper through a diagnostic entry, not another runtime
scheduler slot.

New native entry captures selected Entry, current slot, explicit k and jp offset;
C instruction wrapper reconstructs the unchanged pointer after handled output.
Encode/validate output before commit. Actor entry/service/observer must be
nonreentrant, with appropriate mutual active guards against current consumers.
Root rechecks those guards against player25 and persistence runtime source at
actual integration. A raw observation callback must not reenter the Rust FnMut
registration or mutate canonical state.

Finite enemy-terrain PLM continuation is already mapped: spike/BTS index15 uses
A0C2DA -> header84D094 -> setup84B3D4. Setup clears a level-data high nibble and
returns; SpawnPLM allocates/setup only, not instruction scheduling. Hence this
finite continuation needs real synchronous PLM effects but no permission to
enter current item/player callbacks. Table has only16 defined words, even though
source masks BTS with7F. Do not call C at arbitrary masked indexes or fabricate
an enemy-table validator from that mask. Use source-valid native game states and
explicit finite fixture layouts; malformed extensions/terrain selectors are
outside admitted tests. Enlarging the terrain domain requires a named concrete
case, not a whole enemy-core rewrite.

## Exact ownership and source reuse

| Owner | Writable source |
| --- | --- |
| Actor worker | Only new restart/waver/Cargo.toml, src/lib.rs, memory.rs, actor.rs, fixtures.rs, tests.rs, README.md and HANDOFF.md; license notice for copied upstream work |
| Native worker | Only new restart/native-host/src/waver.c, waver_hooks.rs, waver_types.rs, waver_prepare.rs and examples/waver-host-control.rs, plus scoped handoff |
| Integrator | Existing workspace/lockfile, native-host lib/prepare/observe declarations and sibling guards, runtime installer/selection/results, shared effect IDs, all execution/publication/cleanup |

One actor family is one active scope; its native adapter is support, not a second
independent scope. Respect the existing two-active-source limit. Root may reuse
current available material/native workers once their exact prior files are
submitted and new worktrees assigned. This preparation opens no worker pool.

Main enemy_ai.rs has Init/Main/helper arithmetic and the specific nonstandard
SignExtend8 init test; Waver_Instr_1 is missing in the searched implementation.
Keep SignExtend8(8000)==8000. Reuse selected actor expressions, not old terrain
implementation, Host or acceptance. The native Sine/terrain services permit
this module to remain independent of unfinished shared enemy code.

## Focused checks and remaining live extent

One fixture definition list behind fixture-support feeds Rust tests and the
actual-C control. Complete current/neighboring WRAM comparison and ordered native
requests matter; actor-owned position alone can hide lost collision scratch.
Compact meaningful cases from source note are:

- Left/right Init with sentinel D/timers; unchanged and changed animation helper.
- Horizontal collision, fractional packed reversal and skipped sine/vertical.
- Open horizontal plus vertical hit/clear; sine positive/negative and phase wrap.
- Phase54->56 with E=1, proving both animation changes in source order.
- Event instruction same jp/current-slot versus explicit-k behavior, followed
  by Main consuming the event and returning normal animation.
- Actual CallEnemyAi/CallEnemyInstr path, and native spike/PLM allocation/full
  pool if that domain is admitted. Test invalid adapters with unchanged refusal
  or exact fatal ownership, never undefined C memory accesses.

One compiled control serves a bounded set of isolated processes under root's
normal budget. Root freezes exact commands at assignment, using cargo test/run
--locked --offline and the existing native library preparation. Shared guards
justify only affected compact composition controls. Do not rerun the complete
player, item or SRAM matrices or rebuild native separately for every fixture.

Source and actual-C acceptance can proceed before a live Waver encounter.
A future fixed-input scenario earns live ownership only by naturally reaching
these roots; synthetic direct invocations remain Fixture evidence. No whole
lifecycle/contact/death/draw or original-ROM parity claim follows. Publish the
four translated/verified/integrated routines independently and only reached
live routines when their actual evidence exists. No new artwork/assets needed.

Preparation reads only primary source and finite ROM slices. No code, build,
test/game/capture, claim or worktree created; cleanup0bytes. Existing pinned C,
ROM, main reuse and retained evidence stay untouched.
