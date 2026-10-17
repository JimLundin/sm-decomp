# Native game boundary

`sm-native-host` builds and loads the pinned recovered C independently of the
legacy pipeline. Cargo builds require neither the ROM nor extracted C. Native
preparation requires the pinned local archive and a GNU-compatible `cc`; `CC`
may name an alternative executable (not a shell command).

`prepare(archive, output)` validates the archive, extracts only C sources,
headers and `LICENSE.txt`, applies exact-count observational patches and builds
the source inventory described in restart baseline research. It preserves the
upstream MIT and Opus notices. Quote-only include lookup is intentional:
upstream `src/features.h` must not shadow glibc's `<features.h>`.

The resulting directory contains `libsm_native.so`, `library.json`, compiler
output and all retained source. Its manifest identifies pristine and patched
source, every patch anchor, compiler binary/version/arguments, recipe and final
library. Existing artifacts are accepted only when recipe, source set and
library hashes still match. Native compilation has a 60-second deadline and
terminates the compiler's process group on timeout. Failed build directories
are retained for diagnosis.

`Game::boot(library, rom)` checks pristine ROM identity and the prepared library,
then performs ordinary cold reset with zero SRAM. Only one Game can ever boot
per process. Its library remains loaded until process exit, and Game cannot be
sent or shared between threads. Both native fatal errors and hung C calls need
the caller's supervised process boundary; a Rust method cannot safely interrupt
an arbitrary C stack. Upstream saves use `saves/sm.srm` relative to process CWD;
run the child in its private output directory with a `saves` subdirectory.

`host_step` delivers the native 12-bit pad mask, runs one recovered scheduler
iteration, draws pixels and drains the audio queue once. `pixels` contains
256×240 packed `0x00RRGGBB` values; `audio` contains 534 interleaved stereo
frames at 32 kHz. This matches the actual upstream player's fixed block and
implies a presentation cadence of `32000/534` steps per second. It is not an
original-hardware timing or audio-equivalence claim. No asynchronous audio
consumer may advance C state.

The returned counters observe actual controller calls, completed inner updates,
NMI calls, yielded explicit waits, native scheduler entries, C-to-CPU gateway
entries and executed fallback opcodes. Requested buttons and effective buttons
remain separate because upstream clears opposite directions. The initial logo
uses 150 host steps without a controller poll or completed main-loop update.
`c_gameplay_calls` counts scheduler entries, not every internal C function.
The host explicitly enables rendering because the upstream GUI normally owns
that flag and the bundled allocator/reset functions leave it uninitialized.

The only behavior compatibility fix resumes RESET when the recovered native
game requests its `0xffff` soft-reset sentinel; the same fix exists in upstream's
comparison driver. The pristine ROM is patched by upstream initialization and
again when installing its bug-fix state. `patched_rom_sha256(&mut self)` reports
the current exact ROM view on demand. This runtime is not an unmodified-ROM
oracle. It exposes read-only WRAM for diagnosis and has no arbitrary restore or
state-injection API.

The `native-smoke` example is a bounded ordinary-boot and controller observation check,
not a Ceres route. Run from a private directory containing `saves/`; pass archive,
ROM and preparation-output paths. The shared CLI owns gameplay scenarios,
presentation, child deadlines and retained run observations.

The ignored `prepared_integrity` test deliberately corrupts a private copy of a
real prepared library, then a retained source file, and requires both cache hits
to fail validation. Run with `SM_NATIVE_ARCHIVE` naming the pinned archive and
`SM_NATIVE_PREPARED` naming the complete prepared directory:

```sh
cargo test -p sm-native-host --test prepared_integrity -- --ignored
```

## Projectile pilot boundary

The four scheduled entries are `FireUnchargedBeam`, `HandleProjectile`,
`DrawPlayerExplosions2` and `DrawBombAndProjectileExplosions`, in that numeric
order. Reviewed source patches preserve the original function body behind a
wrapper. Without a hook the original C executes; candidate Rust changes do not
change the prepared C library or its authenticated identity.

`Game::set_projectile_hook` owns a stable boxed closure until unregistering it.
Its arguments are the entry, a private complete WRAM copy, an owned ROM snapshot
and `ProjectileServices`. ROM snapshots refresh when the actual upstream patch
generation changes. A handled invocation commits WRAM. Refusal requires both
unchanged private WRAM and zero foreign service requests. Error, panic or invalid
refusal terminates the native process without retrying C. No unwind crosses C.

`ProjectileServices::spawn_plm(ram, header)` is the one foreign service. The
bridge copies WRAM into actual C, invokes `SpawnPLM(uint16)` synchronously and
copies WRAM back before returning its actual carry/overflow byte. It counts the
request before foreign execution. There are no Rust references into C-mutated
state. Headers below `$8000`, unknown active collision contexts and quicksand
setup `$B4C4` with vertical collision are rejected. The pinned `sm_94.c` binds
`cur_coll_amt32` only within the helpers at lines 957/968 and explicitly clears
it before return at 961/972; non-null state at these projectile or host-step
boundaries is unsupported, rather than a serializable stack address.

Ownership counters separately report original C entries, Rust attempts,
completions/refusals and service requests. `drawn_oam_entries` counts actual
four-byte OAM records emitted by the two draw operations while an ordinary
beam or explosion slot was active; empty draw calls do not count as material
drawing. These ownership counters are excluded from behavior comparison.

## Room scrolling boundary

`Game::set_scroll_hook` admits `DisplayViewablePartOfRoom`,
`UpdateScrollVarsUpdateMap` and `CalculateLayer2PosAndScrollsWhenScrolling` in
that numeric order. Its closure receives the entry, a private complete WRAM
copy and `ScrollServices`. Both consumers use the same copied-WRAM transaction:
handled commits, unchanged refusal permits C, and failures or refusal after a
foreign request terminate without C retry. Native dispatch rejects nested
callbacks across consumers as well as within one consumer.

`ScrollServices::flush_vram(ram)` executes the actual
`NMI_ProcessVramWriteQueue` synchronously with the current private WRAM and
copies back its queue tail and cleared low enable bytes before returning. The
function also consumes horizontal and vertical scrolling DMA when the ordinary
queue tail is zero. Empty flushes still write hardware registers. Mode7 upload
early returns remain gameplay behavior; display still makes its 17 flush calls.

`ScrollCounters` separates ownership, Rust foreign requests/completions and
actual completed hardware work. `flush_requests`, `flush_completions`,
`columns_flushed`, `rows_flushed` and `entry_vram_dma_bytes` count synchronous
work inside each scheduled root. The two-dimensional arrays contain BG1/BG2
counts. `total_flush_requests`, `total_flush_completions`, `consumed_columns`,
`consumed_rows` and `vram_dma_bytes` include all queue flushes, including ordinary
NMI after a scroll callback has returned. Consumed enables are counted only
after native flush completion clears them; DMA bytes count real writes to VRAM
ports `$18/$19` in `dma_transferByte`. They do not infer transfers from callback
counts or associate later NMI work with a particular earlier entry.

The `scroll-host-control` example checks unchanged-refusal parity and fatal
dirty refusal, panic, foreign-request refusal and wrong-buffer-request refusal.
Its `service` mode injects a bounded queue plus all four row/column descriptors,
requires synchronous copied-back clears (preserving high enable bytes), nine
real DMA requests and exactly 18 VRAM bytes between flush entry/return events.
These isolated controls deliberately alter gameplay and cannot establish
gameplay acceptance. Run each in a private process/directory with a deadline.

## Ceres door boundary

`Game::set_ceres_door_hook` schedules thirteen typed `CeresDoorEntry` values:
Init, Main, Instr1 through Instr8, SetTangible, SetInvisible, SolidSetup. The
first twelve own the E23F actor family, parameters 0–6; SolidSetup owns BA48's
four-block setup at 84:B9F1. Exact dispatcher-case patches in `sm_a0.c` and
`sm_84.c` preserve the original function bodies. Shared Steam4/5 instructions
enter this boundary only for E23F. Internal calls such as Instr5→Instr2 and
Main→Steam5/Instr2 remain private and never manufacture scheduled entries.

The callback receives entry, enemy/PLM byte index, canonical A6 instruction
offset (zero for Init/Main/SolidSetup), copied WRAM, generation-tracked ROM
snapshot and `CeresDoorServices`. `CeresDoorOutcome::Handled` carries exactly
one matching `CeresDoorOutput`: Done, InstructionOffset(u16), or SetupStatus(u8).
The adapter normalizes native ROM pointers and encoded C return pointers;
process pointers never enter the gameplay API. Instruction offsets must be
even and at least $8000. Typed output validation precedes committing WRAM.
Refusal/error/panic semantics reuse `Transaction`, with the same fatal handling
of dirty refusal or refusal after any foreign request. Native dispatch rejects
cross-consumer reentry with projectile and scrolling callbacks.

`CeresDoorServices::spawn_eproj_with_room_gfx(ram, header, parameter, point)` is
the sole foreign gameplay service, scoped to the Main invocation, header E509
and parameters 3/12. `point` is `[x, y]`; the bridge sets the actual external
`eproj_spawn_pt`, executes C synchronously, and copies WRAM back. Requests are
counted before native scope validation and completions after copied-back state.
Wrong-allocation requests never pass their pointer to C but still prohibit
refusal. The C escape call site and Rust service share the same actual spawn
observer, including the external point. No general projectile service is added.

`CeresDoorCounters` reports thirteen-element C/Rust attempt/completion/refusal
arrays, foreign requests/completions, proximity fallthrough returns for each of
32 enemy slots, and completed solid setups whose four actual block types are
$8000. These counts are separate from equivalent behavior. Route verification
must combine them with per-actor script/property changes and completed crossings.
Generic enemy scheduling, timed sprite records, Goto, rendering/collision,
PLM allocation/deletion, NMI queue consumption and room loading remain native.

`ceres-door-host-control` invokes exact native dispatcher fixtures without any
gameplay steps. Its off/refuse modes check all thirteen entries, canonical
instruction returns, shared-steam bypass, internal-call counts and the four
solid blocks. Fatal controls cover dirty refusal, panic, wrong output type,
cross-consumer reentry and refusal after valid/invalid/wrong-allocation service
requests. Escape-off and service modes check the same actual spawn events and
point; service also checks actual E509 allocation and coordinate copyback.
They deliberately inject state through diagnostic symbol bindings and establish
boundary behavior only. Pass archive, ROM, preparation-output and mode, using a
private process/directory with a deadline. Mode `prepare` only prepares the C
package and performs no boot or fixture invocation.

## Room-transition scheduler interface

`Game::set_room_transition_hook` admits three `RoomTransitionEntry` values:
HitDoorBlock(0), LoadingNextRoom(1), LoadingNextRoomAsync(2). Only the state9/10/11
slots in `kGameStateFuncs` dispatch gameplay hooks. Internal calls to state10
remain private. `RoomTransitionOutcome::Handled(u8)` commits WRAM and returns
the exact coroutine byte to the existing scheduler, including any nonzero
pending value. Refused requires unchanged WRAM, no requested foreign operation
and no emitted private observation. Errors, panic or invalid refusal terminate
without a C retry. No transaction survives a pending root return or NMI.

The fixed service catalog and IDs are represented by `RoomTransitionSyncService`
(0–47), the exact-width methods `palette_fade`(48), `has_queued_music`(49),
`call_samus_code`(50), and `RoomTransitionCoroutineService::WaitForNmi`(51) /
`PlmHandler`(52). Source/data conversion happens in the narrow adapter: addresses
are 24-bit bus values, decompression destinations are u16 WRAM byte offsets,
VRAM destinations are word addresses and transfer sizes are bytes. Native
validation rejects unexpected high bits and unused arguments. No process pointer
crosses the gameplay interface. `room_transition_types.rs` and the matching
`room_transition_prepare.rs` catalog specify each exact named C operation.

Each operation receives the complete parent scratch allocation, copies it into
canonical WRAM, executes synchronously and copies it back before returning the
actual scalar or coroutine byte. Direct C call expressions inside the selected
27 routines use the same operation observer. Internal calls within a foreign
operation remain part of that operation. Shared background helpers outside an
active scheduled transition execute ordinary C without transition observations.

A named transition operation may enter an existing projectile, scrolling or
Ceres-door hook. The child owns a separate copied transaction, sees preceding
parent writes and commits before parent copyback. Sequential children are legal;
overlapping sibling/recursive children, recursive transition roots, nested
transition operations and child entry outside an active named parent operation
are fatal. Children retain their existing admitted services. Hook registration
cannot change during any active transition. These rules hold for C and Rust
parents; existing child hooks are never bypassed.

`RoomTransitionRoutine` names exactly27 routines: three roots, private dispatcher,
fifteen phases, eight background commands. `RoomTransitionObservation` emits
typed Enter/Return with canonical operands/results. Passive C wrappers and Rust
observations share a fixed stack and call-graph validator. This records private
state10 calls and background8/E→2 calls without manufacturing scheduled hooks.
Background offsets may be odd. A routine return does not commit state or cause
gameplay execution. Root completion requires a balanced matching root observation
whose result equals the returned raw coroutine byte.

`RoomTransitionCounters` is Copy/Clone and separates scheduled C/Rust ownership,
private C/Rust routine entry/completion, actual named service requests/completions,
Rust-only foreign requests/completions, coroutine service pending results and
Rust child callback attempts/handled completions by service×consumer. Child
columns are projectile, scrolling, Ceres door. C child execution still emits the
same child effects but does not fabricate Rust callback counts. `root_pending`
counts nonzero returns; `root_resumed` counts a nonzero canonical state1 on entry.

The `room-transition-host-control` example injects a retained WRAM fixture and
invokes the scheduled adapters directly without gameplay host steps. Off/refuse
exercise private state9→state10→state11, while pending-off/pending-refuse exercise
the real NMI return and its resume after an injected IRQ-completion bit. Separate
controls check raw127 pending propagation, synchronous native pending copyback,
actual Rust child composition, dirty/observed refusal, panic, malformed service
arguments/observation, wrong scratch allocation and recursive/sibling rejection.
These controls are deliberate fixtures, not gameplay acceptance. Run each in a
private process/directory with a deadline; `prepare` mode only prepares the C
package. The fixture path is required for non-prepare modes.

## Observation format

`observation_bytes()` is a passive post-step capture, not a restore format.
Version `SMNOBS02` is currently 543,791 bytes. Numeric fields use little-endian
64-bit values; fixed scalar arrays use their existing little-endian element
width on the admitted Linux target. Struct padding and process pointers are
never emitted. The inventory and exact field order live in `pilot.c` and are
part of the authenticated native recipe:

- Host-step/poll/completed-update/NMI/wait ordinals and requested/effective pads;
  native frame and installed bug-fix counters, run mode and collision context.
- External enemy-projectile spawn point as two explicit u64 coordinates;
  SRAM; CPU registers/flags; SNES timing, register/latch and input state; both
  PPUs' VRAM, CGRAM, OAM, registers and drawing state; every DMA channel/timer.
- Recovered SPC player variables, channels and RAM; both DSPs; the hardware
  APU/SPC, timers and ports; all 16 queued audio writes and queue indices.
- Actual 256×240 framebuffer and this step's 534 stereo PCM frames.

Compare `Game::wram()` separately: it contains every canonical WRAM byte. The
capture excludes upstream recorder-log storage and debug histories, which are
not active gameplay state in this fixed no-replay/no-debug pilot. Pristine and
patched ROM identities remain separate manifest/run inputs. This is a declared
native observation contract, not original-ROM causal or timing equivalence.

`effects()` contains source-ordered 16-byte records of four little-endian u32s:
`kind, a, b, c`. Each host step starts a fresh sequence. Storage grows from
64 KiB to a hard limit of 64 MiB, preserving every record; allocation failure or
the hard limit is fatal. Steps above 4 MiB report their complete event counts
by kind, and retained capacity is reused on following steps.

| Kind | Meaning | Values |
| --- | --- | --- |
| 1 | Actual `WriteReg` request | register, byte, 0 |
| 2 | Actual `RtlApuWrite` request | register, byte, 0 |
| 3 | Actual `SpawnPLM` entry | header, current block, 0 |
| 4 | Actual `SpawnPLM` return | return byte, 0, 0 |
| 5 / 6 | Scheduled projectile entry / return | entry ID, 0, 0 |
| 7 / 8 | Actual `ReadReg` request / result | register, 0 or result byte, 0 |
| 9 | Poll/update/NMI/wait observation | event 0/1/2/3, ordinal low/high u32 |
| 10 / 11 | Scheduled scrolling entry / return | entry ID, 0, 0 |
| 12 | Actual VRAM queue flush entry | context, queue tail, enable mask |
| 13 | Actual VRAM queue flush return | context, actual VRAM DMA bytes low/high u32 |
| 14 | Scheduled Ceres door entry | entry ID, identity/index, instruction offset |
| 15 | Scheduled Ceres door return | entry ID, identity/index, typed return |
| 16 / 17 | Actual Ceres E509 spawn entry / return | header, parameter, packed point |
| 18 | Scheduled transition entry | entry ID, phase word, state1 byte |
| 19 | Scheduled transition return | entry ID, raw coroutine byte, state1 byte |
| 20 / 21 | Private transition routine enter / return | routine ID, operand/result, 0 |
| 22 | Named transition operation entry | service ID, argument a, argument b |
| 23 | Named transition operation remaining argument | service ID, argument c, 0 |
| 24 | Named transition operation return | service ID, actual scalar result, 0 |
| 25 / 26 | Transition operation child enter / return | service ID, consumer ID, child entry ID |

Flush context is 0 outside a scrolling root, otherwise the scrolling entry ID
plus one. Enable mask bits 0/1 identify BG1/BG2 columns; bits 2/3 identify rows.
Door identity/index packs E23F (actor) or BA48 (setup) in the high half and the
byte index in the low half. Typed return packs kind 0/1/2 (Done/InstructionOffset/
SetupStatus) in the high half and value in the low half. The spawn point packs
x in the low half and y in the high half. These events are identical for C and
Rust ownership. Existing effect IDs retain their meanings; observation version
02 adds the previously unobserved external spawn point.

The trace observes native register requests and their results. It does not claim
to enumerate every internal emulated bus cycle; final device state and output
are compared as well. Hardware operations remain synchronous.

The `pilot-host-control` example replays supplied native input through the same
Game and real scheduled wrappers. Its `off` and `refuse` modes should produce
identical behavior hashes with distinct ownership counts. `dirty-refuse`,
`panic` and `service-refuse` must terminate. `service-clear` and `service-set`
exercise actual harmless-setup PLM headers `$B633`/`$B637`, verify copied-back
allocation and ordered enter/return effects, and expect returns 0/1. These
isolated service controls deliberately alter state and grant no gameplay
acceptance. Run each in its own process and private directory with a deadline.

`escape-update` is a separate bounded synthetic fixture mode of that same
example (`LIBRARY ROM UNUSED_INPUT_PATH escape-update`). It uses private native
symbols only inside the diagnostic to install a fixed air room with a solid
wall and one ordinary rightward beam. Four Mode7 matrices cover identity,
signed fractional products, truncation and wrapping with M7D deliberately
different from M7A. Actual patched-ROM instruction and trail records are used.
Every invocation compares all 131072 WRAM bytes and ordered effects at the
actual `HandleProjectile` return, including the native Rust transaction. Each
case requires a first-tick allocation of both trail sides, flight, instruction
progression, solid collision, impact explosion and deletion within 64 updates.
No host step or draw entry runs. These fixtures establish bounded update-return
agreement, not live-from-reset active-shot or undefined Mode7 rendering parity.

`ProjectileCounters` separately records entry-time Ceres high-bit context and
Rust completions. A refusal belongs to `ceres_high_bit_refused` only for Fire
or a draw root, with the high bit set and every other inventory/slot admission
guard clear at entry. All remaining refusals belong to `other_refused`.
Refused calls increment entry context once; C fallback does not double count.
These ownership/context counters are excluded from compared effects and state.

## Directional door scrolling

`Game::set_door_scrolling_hook` intercepts the actual function bodies before their
first mutation: FixDoorsMovingUp(0), ScrollingSetup(1), FollowTransition(2). Each
callback receives exactly 128 KiB of private WRAM and `DoorScrollingServices`,
with synchronous `HookOutcome` semantics. There is no ROM borrow or coroutine
result. Refusal requires unchanged state, no operation requests and no routine
observations. Errors, panic and invalid refusal terminate without retrying C.

The four methods execute the existing C operations with full WRAM copyback:
`calculate_layer2_x`(0), `calculate_layer2_y`(1), `calculate_blocks`(2), and
`update_scroll_vars`(3). The first two preserve their actual byte results.
Service3 calls the existing scroll entry1 wrapper. Coordinator services11/18
admit only directional roots0/1 respectively. A separate fixed coordinator slot
keeps nested scroll work attributed to the directional parent; the established
coordinator service×three-consumer matrices retain their shape. Math services
admit no children. Sibling and recursive callbacks are rejected.

`DoorScrollingRoutine` and `DoorScrollingObservation` expose a fixed thirteen
routine catalog. The passive observer has no WRAM pointer and performs no
synchronization or gameplay work. It validates a bounded, balanced call stack
and preserves the counter supplied at each entry/return. Void results are zero;
movement results retain their exact byte. The actual IRQ callers still own the
completion-bit guard. FollowTransition itself remains callable with that bit
already set, matching its source body.

Effects27/28 record root begin/end `(entry, direction, counter | flags<<16)`;
29/30 record routine enter/return `(routine, result, counter)`;31/32 record named
service begin/return `(service, result, counter)`;33/34 record nested scrolling
begin/end `(service, scroll_entry, counter)`. Entry results are zero. Snapshots
35/36 record before/after `(L1X|L1Y<<16, L2X|L2Y<<16, BG1HOFS|BG1VOFS<<16)`;
37/38 record before/after `(BG2HOFS|BG2VOFS<<16, destX|destY<<16, flags)`.
Root order is27,35,37 …36,38,28. Existing effects and observation format remain
unchanged. Ownership counters remain outside behavior effects.

`door-scrolling-host-control ARCHIVE ROM OUTPUT FIXTURE_WRAM MODE` uses retained
WRAM and exact native function calls with zero host steps. `off` and `refuse`
cover all roots, directions and eleven counter values with actual C scrolling.
`services`, `child` and `coordinator` check exact returns and copied state across
one, two or three participants. `c-right` and `c-vertical` inject a Rust scroll
child into actual C movement to test saved locals, the separate counter63/64
calls, four-field restoration and selective layer1 snapping. Twelve fatal modes
cover invalid refusal, panic/error, unbalanced observations and invalid nesting.
These are boundary controls, not live-route or original-ROM acceptance.

## Normal camera

`Game::set_camera_hook` intercepts the actual `MainScrollingRoutine` body before
mutation. Its callback receives complete private WRAM and `CameraObserver`, with
no ROM or foreign services. The root runs independently of the current state
word: a latched state8 gameplay invocation can have published state9 before
reaching this call. Camera is standalone; recursive entry and nesting in either
direction with all five existing consumers are fatal.

`CameraObservation` records Enter `{ routine, operand }` and Return `{ routine }`
for the thirteen `CameraRoutine` values. Only CallFinishedHook carries an operand,
the low16-bit target9589. All returns are void. C and Rust observations share a
closed bounded call stack. The observer never reads, writes or synchronizes WRAM.
A pure candidate can calculate once, retain events until complete admission,
emit them in source order and return Handled for a single full-state commit.
Refusal after an observation or private mutation, callback errors and panic all
terminate without retrying C. Shared C distance/constraint helpers outside the
camera root do not manufacture camera observations or ownership counts.

Effects39/40 are root begin/end `(game_state, room_ptr, slow_grabble_scrolling_flag)`.
Effects41/42 are routine enter/return `(routine, operand, 0)`; return operand is0.
Effects43/44 are before/after `(layer1_x, layer1_y, samus_x)` and45/46 are
before/after `(samus_y, samus_previous_x, samus_previous_y)`. Each coordinate packs
integer in the low16 bits and fraction in the high16 bits. Root order is
39,43,45 …44,46,40. Existing effects, counters and SMNOBS02 remain unchanged.
`CameraCounters` separates four scalar root ownership counts from four arrays
of thirteen C/Rust routine entries/completions.

`camera-host-control ARCHIVE ROM OUTPUT FIXTURE_WRAM MODE` runs direct native
fixtures with zero host steps. Off/refuse compare forty-four valid C fixtures,
covering all thirteen routines and both state8/state9 body calls. Dimensions,
actual scroll accesses, hook targets and camera table indices are bounded;
unsupported synthetic C inputs are never used as reference fixtures. Observer
mode proves passive events cannot synchronize private mutation and successful
return commits that state. Fatal modes cover invalid refusal/observation,
panic/error, recursive registration and both directions of cross-consumer entry.
These establish the native interface, not candidate gameplay or live acceptance.
