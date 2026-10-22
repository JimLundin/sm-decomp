# Moving gameplay state out of RAM

Source audit: 2026-10-22, through `15589b48`. This follows
[PLAN.md](PLAN.md) and builds on [RAM_OWNERSHIP.md](RAM_OWNERSHIP.md).
No runtime experiment or state promotion is claimed by this note.

## Recommendation

Do not add an always-live `Game::launch_motion` or `Game::message_handoff`
beside the still-writable compatibility bytes. The current engine repeatedly
replaces those bytes, sometimes inside one Rust gameplay update. A second field
would become stale or overwrite the state selected by the oracle.

Use LaunchMotion to test explicit **authority transfer**: decode an imported
state image, let typed behavior own its state, encode before relinquishing it,
and reconstruct after an external writer returns. This is a useful migration
experiment, not yet complete persistent ownership. The next substantial closure
should be the player-projectile lifecycle: slots, launch history, flight,
reflection, deletion and their contact outcomes. Its implementation must make
the projected bytes unavailable as an independent mutable source while the
typed owner is active.[^launch][^flight][^reflection]

The audit below identifies known bulk and physical writers. It does **not** prove
that every native indexed access is disjoint from a promoted field. `word!`, raw
RAM pointers and other generic indexed operations can reach outside a nominal
array while remaining inside `g_ram`; preserving those behaviors is a separate
closure obligation.[^word] Two bus-write hooks plus named-field migration would
therefore be insufficient grounds to claim full ownership.

## What an authoritative model would mean

During a typed-owned execution interval, transitions read and update that model.
They do not reread a competing RAM copy to discover the current state. The byte
image is an encoding for the oracle, persistence and legacy adapters.
During a RAM-owned interval, CPU/native compatibility code can mutate the bytes;
the previous typed value cannot be used until an explicit import rebuilds it.
Moving ownership between these intervals is compatible with saving/loading an
engine; leaving two mutable authorities alive is not.

Current `launch_motion_ram` decodes all four records on each operation, invokes
the pure model and writes the result. It deliberately reloads after the vertical
projectile-speed write because an off-table index can alias launch history.
The pure behavior is already useful, but RAM remains the authority.[^launch]
`MessageHandoff` is likewise reconstructed from a continuation byte at PLM entry.
The script slot and queued message intentionally remain live inputs.[^handoff]

The [shot-to-room reaction experiment](RAM_OWNERSHIP.md#synchronous-shot-to-room-reaction-protocol)
exercises a concrete mixed-ownership interval. Its projectile owner publishes
before the actual mechanism allocator/setup executes, then imports before flight
and animation resume. Room geometry and mechanism slots remain byte-owned. The
prepared mechanism retains its captured block association even if setup changes
the current block or selected projectile. Persistent room ownership must migrate
these setup writers and their later consumers together; the pure reaction protocol
alone does not transfer authority over the room or mechanism pool.

The [mechanism-pool follow-up](RAM_OWNERSHIP.md#owned-mechanism-allocation-and-selected-setup)
now holds projectile and mechanism state in one typed world during creation,
selected setup, release and reuse. It publishes at definition failures and setup
callbacks, then imports callback mutations before continuing. The signed hit-kind
read can alias a cleared mechanism association; its explicit compatibility read
must observe that pending write. This is a concrete reason to preserve authority
transfers beyond nominal field boundaries. The room grid and full PLM scheduler
remain outside this ownership interval; no persistent runtime promotion follows
from the component result alone.

The [integrated setup checks](RAM_OWNERSHIP.md#integrated-mechanism-setup-and-terrain)
now place selected setup policies on the existing pool instead of calling the
raw setup bodies. Candidate terrain and projectile consumers observe their effects.
The full candidate subsequently shares collision, room-entry and hardcoded
creation (`7314fe2f`). This closes selected creation/setup intervals, not all
mechanism authority: the PLM interpreter, external consumers, suspension and
reset/load imports still need migration. Use that richer schema as the
implementation to extend; the independent coordinator composition supplies
additional checks rather than a second owner.

## One RAM allocation serves both execution branches

`Game::new` initializes the large allocation in place. `SnesInit` passes the
address of `game.g_ram` to `snes_init`, which retains it as `Snes::ram`.
CPU memory, the Rust implementation, snapshots and save states consequently
observe the same live allocation, not independently owned CPU/Rust RAM.[^game]
[^init]

The normal paired-frame execution is:[^compare]

1. Capture `before` from the live RAM/hardware.
2. Run the emulated CPU frame and draw it; capture `theirs`.
3. Restore `before` into that same RAM allocation.
4. Run the Rust frame and draw it; capture `mine`.
5. Normalize the documented comparison exceptions and compare snapshots.
6. Restore the completed `theirs` snapshot into live RAM, including on success. These steps describe main RAM; native player, physical
   audio and queue state have separate selection rules in this same runner.
   An integration must preserve each owner's selection rather than adopting
   every CPU-side value universally.

`MakeSnapshot` stores RAM, SRAM and selected hardware/audio state, but no future
gameplay field in `Game`. `RestoreSnapshot` similarly copies snapshot RAM into
`snes.ram` and restores its historical partial hardware selection.[^snapshot]
Adding a field to `Game` alone therefore neither snapshots nor rewinds it.

The headless failure path also adopts the completed oracle frame. Interactive
failure recovery first restores `before`, may loop for debugging, or reruns the
CPU frame after saving a bug snapshot. These are additional authority transfers;
success-path-only integration would preserve neither recovery nor retries.
`RM_MINE` bypasses paired snapshots; `RM_THEIRS` runs CPU-only. Both remain
supported paths for any state importer/exporter.[^compare]

### Comparison exceptions matter to the proposed owner

`normalize_snapshot_ram` copies `$0611..$0616` from `mine` into `theirs` before
comparison, along with other documented ranges. Thus the Rust continuation byte
at `$0613` is not independently validated against a CPU equivalent. The modified
`theirs` image is later restored, carrying the Rust continuation forward.
LaunchMotion's `$0DAA..$0DB9` records and external `$0DA8` fraction are not in
that exception list.[^masks]

Do not decode the unnormalized CPU snapshot and then keep that model while
restoring normalized bytes. Import the actual selected image. Do not copy a
typed value from `mine` merely because comparison passed: masked fields, failure
recovery and explicit branch selection define which state becomes live.

## Known replacement and mutation paths

| Path | Existing RAM effect | Required ownership treatment |
| --- | --- | --- |
| `Game::new` | Initializes all RAM to zero | Construct the model explicitly from that initial state; update the zero-initialization validity argument for added fields |
| `snes_loadRom` / `snes_reset(hard=true)` | Hard reset clears all 128 KiB | Invalidate/rebuild any promoted state after the clear |
| `RtlReset` | Hard reset, continuation initialization, optional SRAM/recorder reset | Complete the typed reset before returning to gameplay or saving |
| `Vector_RESET_Async` | Clears first 8 KiB, later first 64 KiB, during Rust execution | These are within-frame state imports/resets, not merely frontend initialization |
| `Samus_Initialize` | Clears `$0A02..$0E0B`, then sets Samus defaults | Reset LaunchMotion alongside the workspace; preserve independent absolute-Y initialization |
| Paired-frame `RestoreSnapshot` | Copies a complete selected 128 KiB image | Rebuild the promoted value after each before/theirs restoration and recovery restoration |
| `StateRecorder_Load`, base snapshot | `snes_saveload(Load)` replaces hardware and all RAM | Import after the RAM image is loaded; do not wait for the next ordinary frame |
| `StateRecorder_Load`, final snapshot | Same mechanism, distinct non-replay branch | Same import, including saved suspended execution |
| Replay with absent/SRAM-only base | Calls `RtlReset(2)` instead of loading full RAM | Use the reset import; an 8 KiB SRAM payload is not a RAM snapshot |
| Verifier checkpoint `machine(Load)` | Deserializes `game.g_ram` directly before other state | Import this distinct path; frontend load hooks do not run here |
| Replay patch command | Writes arbitrary bytes to `g_ram`, possibly wrapping at 128 KiB | Revoke/import an affected cohort before the next consumer; batch must preserve ordered patches |
| `RunAsmCode` | Runs CPU opcodes and DMA inside a Rust call | Export before CPU ownership, import after CPU returns |
| CPU writes / DMA / WRAM data port | Mutate `snes.ram` through physical accessors | These can run outside normal full CPU frames; a transferred interval must include them |
| Native generic indexed access | Can alias an owned range from another nominal array | Audit/migrate the complete access path, or transfer authority around it; bus hooks do not intercept it |

Sources for the table: allocation/ROM/reset,[^game][^init][^reset] Rust workspace
reset,[^rustreset] snapshot comparison,[^snapshot][^compare] frontend snapshot
loading,[^load] checkpoint loading,[^checkpoint] replay patches,[^patches]
inline CPU execution,[^asm] and physical/native access.[^bus][^word]

### Frontend save/replay paths

`snes_saveload` transfers CPU, APU, DMA, PPU, cartridge and SNES fields, then the
complete 128 KiB RAM block and RAM port address. Its serializer does not receive
`Game` and therefore cannot discover a new model automatically.[^load]

For replay, `StateRecorder_Load` either loads the full base snapshot or resets
with an optional SRAM-only base. For a normal state load, it reads the final
snapshot. Both full-image branches call `FinishLoadSnesState`, which restores
render routing, adjusts a legacy return address and invokes
`RtlSynchronizeWholeState`. However normal `SnesInit` registers **no** sync-all
callback; that existing hook alone is not a state-model integration.[^load]
[^sync]

`StateRecorder_Save` and `StateRecorder_ClearKeyLog` both call `snes_saveload(Save)`;
the latter creates a replay base snapshot. `RtlSaveSnapshot` adds music handling
around the save. Every byte-oriented save needs the current owner's encoding
before it captures RAM. An immediate mirror policy can satisfy this, but only
if projected bytes are never an independent writer during typed ownership.
[^save]

`RtlReadSram` loads cartridge SRAM and records an SRAM-only replay base; it does
not load LaunchMotion or the PLM continuation. In-game `LoadFromSram` restores
the progression/save region and copied player save data, not these two cohorts.
The subsequent gameplay initialization/reset remains the relevant transition.
[^sram]

### Verifier checkpoints are a separate format

`checkpoint::machine` serializes RAM and explicitly listed `Game` fields, both
PPUs, hardware, audio, recorder state and SRAM. It excludes the oracle scratch
snapshots because they are overwritten on the next paired frame. The capture
precondition is a successful idle verifier frame, not an arbitrary gameplay
callback. `load` validates metadata/checksum, runs `machine(Load)`, updates ROM
patches, and checks a capture round trip.[^checkpoint]

No new typed field is included implicitly. Prefer reconstructing a model from
the canonical serialized image after loading, rather than storing two separately
mutable versions of the same fact in the checkpoint. If state ceases to be fully
encoded in RAM, the format must deliberately include it. Compatibility hashes
include checkpoint/oracle/runtime source; update the actual serialization and
compatibility coverage together.[^checkpoint]

Deserialize/import sequencing must also specify partial failure. These loaders
mutate fields in order; a later error is not a transactional rollback. Import
after the relevant RAM bytes have actually been replaced, or invalidate the
typed value until the caller discards/reconstructs the game. Do not silently
leave a usable pre-load model beside partially replaced bytes.[^checkpoint]
[^transfer]

### CPU and physical writers can occur inside a Rust frame

`Bang_Main` still calls `Call(parameter_1 | $A30000)`, which invokes `RunAsmCode`.
That function runs CPU opcodes and DMA before returning to the Rust caller.
A decode performed only at ordinary frame entry misses this route.[^asm]

`snes_write` writes low-RAM mirrors and banks `$7E/$7F`; `snes_writeBBus` writes
through the WRAM port `$2180`. DMA uses both accessors. `DrawFrameToPpu` advances
scanline/HDMA events after the gameplay body and can invoke the Rust IRQ handler.
An end-of-gameplay import is therefore not equivalent to importing after all
external mutations at the end of the selected execution interval.[^bus][^draw]

These are necessary integration points, not a complete exclusion proof. Existing
native pointer and off-table array writes bypass them. The previously preserved
launch case `k=$01BC` writes projectile Y speed at `$0DAC`, changing the following
horizontal inherited-speed read. A persistent field that ignored this native
write would change defined behavior even with perfect snapshot integration.
[^launch][^word]

## Why the two candidate promotions differ

LaunchMotion already has a pure representation and eight migrated producer sites,
one ordinary clear, and a launch consumer. It is a useful authority-transfer
probe because its 16-byte encoding is complete and not comparison-masked.
Its externally sampled absolute-Y fraction intentionally remains a separate
lifetime. Still, generalized native accesses can alias the 16 bytes, and
projectile initialization itself has preserved alias cases.[^launch]

MessageHandoff is not the sole owner of `$0613`. `StartGameplay_Async` uses the
same byte for multiple startup stages, and `Kraid_UnpauseHook_IsSinking` uses it
for an unpause continuation. `RunOneFrameOfGameInner` clears the nested
continuation bytes in order on a new logical frame. It retains them when the
outer frame resumes after a yield. A permanent two-variant MessageHandoff field
would erase these mode-specific meanings and the invalid-byte behavior.
[^handoff][^continuations]

If promoted later, the continuation owner must include the active session/call
context and raw-import handling, or keep each operation's typed continuation
within its scoped lifetime. A PLM message model cannot simply appropriate the
shared byte because its common values happen to be zero and one.

## Mechanism continuation boundary

The [interpreter observations](RAM_OWNERSHIP.md#mechanism-consumers-and-suspended-execution)
exercise 29 actual scheduler invocations through controlled callbacks. A yield
retains the message queue and selected mechanism; resume consumes their live
values without repeating slot admission, pre-action or timer decrement. Completed
display clears the queue and reacquires the selected program and ROM allocation.
These fields belong to the suspended operation's state-transfer contract, not
just the two-variant `MessageHandoff` value.

The owner must retain the session mode, outer gameplay/door continuation and
mechanism pass together. Pool activation is independent: clearing the active
header while suspended does not cancel the continuation. A generic reset of
all slot fields would also lose the stale associations and cursors retained by
`ClearPLMs`. Existing native callbacks require explicit export/execute/import
until their identified encounter, projectile and presentation consumers migrate.

The component experiment copies the yielded RAM image into a fresh `Game` and
resumes it. It does not exercise either frontend snapshot path or the verifier
checkpoint. Nor does ordinary SRAM loading restore this continuation: its main
copy is `0xd7c0..0xde1c`, followed by player and map data; the normal mechanism
arrays lie outside that range and the instruction timers begin at its exclusive
end. See [LoadFromSram](../sm/src/game/sm_81.rs#L100). Whole-image restore and
save-game restore need distinct transition semantics. Generic indexed aliases
remain an independent obligation.

The [owned-pass experiment](RAM_OWNERSHIP.md#owned-mechanism-pass) subsequently
compares 45 scheduler invocations with the actual interpreter. Typed pass state
survives four native invocations without intermediate decoding; its encoded
provider consumes/reimports the owner at legacy callbacks and broad accesses.
This establishes a scoped pass owner. It does not promote the shared continuation
byte to a permanent standalone field, migrate the outer gameplay/door context,
or validate frontend save/checkpoint integration.

The subsequent [nested-session comparisons](RAM_OWNERSHIP.md#mechanisms-inside-session-and-dialogue-continuations)
execute the actual frame/gameplay/door and dialogue control flow around that pass.
The latest script composition passes 40 nested cases, including RAM-image resume
into fresh `Game` values. Selected mode, in-flight frame dispatch, caller phase,
mechanism continuation and active dialogue have distinct lifetimes. Changing the
mode alone retains the suspended dispatch; an explicit frame-continuation write
can redirect it. A changed request also does not replace an already active dialogue,
and completion clears the live request. Preserve all of these relationships on
import rather than reconstructing a pending call solely from the selected mode.

The child animation/input continuation is controlled in this experiment. The
actual frontend snapshot/checkpoint paths, startup/Kraid interpretations and
complete generic-write closure remain open. These component comparisons include
continuation bytes without oracle normalization; a movie whose normalization
masks them would not independently prove the same contract.

## Concrete next implementation cohort

1. Use a LaunchMotion authority-transfer experiment to implement and compare
   image import/export, before/theirs rewind, failed-frame adoption, reset,
   save/checkpoint load, replay patches and nested CPU execution. Keep results
   distinct from a claim of persistent gameplay ownership.
2. Close the player-projectile lifecycle around a typed slot owner. Include
   allocation/reuse, launch, instruction/flight update, reflection, collision,
   trails/extensions and retirement. Give contacts explicit ordered outcomes;
   keep pinned identities distinct from live executor selectors.
3. Replace native raw/indexed accesses to that cohort, including supported
   off-table semantics, or put the remaining legacy operation behind an explicit
   RAM-owned interval. Do not keep a usable typed owner across that interval.
4. Only then retain authoritative state in `Game`/the engine across transitions.
   Import serialized/external states at the audited transfer points and encode
   when observers need the compatibility image. Remove per-method decoding from
   the closed owner. Its behavior must run from typed state without consulting RAM.

The smallest defensible complete promotion is the smallest **closed execution
cohort**, not the smallest struct. Current evidence supports the transfer probe
and this larger lifecycle direction; it does not yet establish that LaunchMotion
alone is closed to every legacy native writer. No source-volume saving is inferred.

## Discriminating checks for the promotion

- Start typed/RAM state deliberately different; verify each explicit import
  selects the restored image and each export selects the actual owner.
- Compare before -> CPU -> rewind -> Rust -> selected-frame adoption, including
  headless failure and interactive retry. Check the model as well as encoded RAM.
- Save and reload after a yielded PLM message; include startup and Kraid uses of
  the shared continuation byte. Do not treat their masked oracle bytes as an
  independent proof of the continuation implementation.
- Load through both frontend snapshot branches and the independent verifier
  checkpoint path; include reset/SRAM-only replay setup and partial-load failure.
- Apply a replay patch and a nested CPU/DMA write to owned bytes, then consume
  them before the next ordinary frame. Add the known native off-table alias case.
- Demonstrate a sequence of typed transitions with no intermediate RAM decode.
  This distinguishes an authoritative owner from another cached wrapper.

These are proposed checks of actual code at its interfaces, not completed
experiments or a request for a new testing registry.

## Follow-up experiment

The [authority-transfer experiment](RAM_OWNERSHIP.md#persistent-authority-transfer-before-the-full-projectile-replacement)
subsequently passed 60 comparisons and 14 discriminating checks on frozen source
`15589b48`. It exercises selected real writer/restore paths and typed execution
without intermediate decoding. Its controlled frame schedule, locally represented
private snapshot types and checkpoint-RAM-only scope are explicit in the result.
The audit above remains a source study; full production integration, generic
native-write closure and whole-projectile ownership are still unproven.

## Sources

[^launch]: [LaunchMotion](../sm/src/game/launch_motion.rs), [temporary byte adapter](../sm/src/game/launch_motion_ram.rs), and [launch handoff audit](RAM_OWNERSHIP.md#launch-motion-handoff-closed-source-trace).
[^flight]: [Player projectile executor](../sm/src/game/sm_90.rs), symbols `HandleProjectile`, `ClearProjectile`, `ResetProjectileData`; [flight algorithms](../sm/src/game/projectile_flight.rs).
[^reflection]: [ProjectileReflection](../sm/src/game/sm_90.rs), [beam firing](../sm/src/game/beam_firing.rs).
[^word]: [RAM access contracts and pointer creation](../sm/src/ram_map.rs#L85).
[^handoff]: [MessageHandoff](../sm/src/game/item_pickups.rs#L104), [PlmHandler_Async](../sm/src/game/sm_84.rs#L151).
[^game]: [Game allocation and new](../sm/src/game_state.rs), symbols `Game`, `Game::new`.
[^init]: [SnesInit](../sm/src/cpu_infra.rs#L890), [snes_init](../sm/src/snes/snes.rs#L23), [ROM-load hard reset](../sm/src/snes/snes_other.rs#L153).
[^reset]: [snes_reset](../sm/src/snes/snes.rs#L46), [RtlReset](../sm/src/rtl.rs#L219).
[^rustreset]: [Vector_RESET_Async](../sm/src/game/sm_80.rs#L187), [Samus_Initialize](../sm/src/game/sm_91.rs#L1260).
[^snapshot]: [Snapshot type](../sm/src/cpu_infra.rs#L54), [MakeSnapshot and RestoreSnapshot](../sm/src/cpu_infra.rs#L731).
[^compare]: [RunOneFrameOfGame_Both and RtlRunFrameCompare](../sm/src/cpu_infra.rs#L1038).
[^masks]: [normalize_snapshot_ram](../sm/src/cpu_infra.rs#L349).
[^load]: [snes_saveload](../sm/src/snes/snes.rs#L35), [StateRecorder_Load](../sm/src/rtl.rs#L235), [FinishLoadSnesState](../sm/src/rtl.rs#L188).
[^sync]: [RtlSetupEmuCallbacks and RtlSynchronizeWholeState](../sm/src/rtl.rs#L135), [default callback registration](../sm/src/cpu_infra.rs#L900).
[^save]: [StateRecorder_Save and StateRecorder_ClearKeyLog](../sm/src/rtl.rs#L373), [RtlSaveSnapshot](../sm/src/rtl.rs#L598).
[^sram]: [RtlReadSram](../sm/src/rtl.rs#L938), [LoadFromSram](../sm/src/game/sm_81.rs#L106).
[^checkpoint]: [Checkpoint compatibility, machine, capture and load](../sm/src/verify/checkpoint.rs).
[^transfer]: [SnapshotIo block/load ordering](../sm/src/snapshot_io.rs#L23), [snapshot_region and diagnostics](../sm/src/rtl.rs#L159).
[^patches]: [StateRecorder_ReadNextReplayState and RtlRunFrame](../sm/src/rtl.rs#L473).
[^asm]: [RunAsmCode](../sm/src/cpu_infra.rs#L822), [Bang_Main CPU call](../sm/src/game/sm_a3.rs#L2371).
[^bus]: [WRAM port and CPU bus access](../sm/src/snes/snes.rs#L135), [snes_write](../sm/src/snes/snes.rs#L256), [DMA transfer](../sm/src/snes/dma.rs#L261).
[^draw]: [DrawFrameToPpu](../sm/src/cpu_infra.rs#L965).
[^continuations]: [StartGameplay_Async](../sm/src/game/sm_80.rs#L1572), [Kraid_UnpauseHook_IsSinking](../sm/src/game/sm_a7.rs#L799), [RunOneFrameOfGameInner](../sm/src/game/sm_82.rs#L681).
