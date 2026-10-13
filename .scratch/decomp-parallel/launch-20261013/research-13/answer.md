# Accepted NMI hardware lifecycle research

Status: answered; contract extensions required before gameplay promotion
Blocked by: None for research; implementation depends on held integration11 and reviewed extensions
Triage: ready-for-agent

Root: `01a1356e-a8dd-7a80-88b0-1239a6a9b876`.
Worker: `01a13571-7282-7660-99e1-227a72a2c4ff` (`/root/launch_owners`).
Research claim: `(ticket-5b357039d4b20673c7d62a43260abada8f7f4f65d7c26999c08138037691dc12, 785e3c24a0e08cd0ab9815084aade2973c955bfcdbf96b48d00a1862d3771771, 1, decomp-launch-20261013)`.
Baseline: `40a8489856f36d21925188823e33b520f74d4cb4`.
This Answer and [child proposals](children.toml) grant no new source ownership or gameplay stage.

## Answer

Select one complete invocation of **Vector_NMI ($809583)**, including both its
accepted branch (`waiting_for_nmi != 0`) and missed branch. Translate its ten
direct/transitive hardware helpers together: eleven routines, all in native
bank80. The scheduled body consumes prepared graphics queues and shadow
registers; it does not run the gameplay producers of those queues. Native
MMIO/DMA/PPU remain explicit synchronous hardware services. This is a coherent
complete scheduled call boundary within the 1–32-routine contract.

Source is the pinned recovered-C revision
`578f90b3cc49557bb70060ad033bb90b8cf8ac50`, declared by
[tools/dependencies.toml](../../../../tools/dependencies.toml).
[source-inventory.toml](../../../../tools/port/source-inventory.toml), SHA-256
`7842e9035e7360eef10a93a96e6abc1cca601888b8423ea92fb554bbb15abb25`,
provides the exact definition/dependency records; the selected records are in
[routines.json](routines.json). Their common `sm_80.c` SHA-256 is
`ab9abc1042ea876c51de90d00632d1af13e9a765c77cf371950ddcdc783130c7`.
The complete inspected input seals are [inputs.sha256](inputs.sha256).

| Native routine | Address | Source line | Responsibility |
| --- | --- | --- | --- |
| Vector_NMI | 809583 | 1052 | Accepted/missed branch, ordered children, HDMA table pointers, counters |
| NmiUpdatePalettesAndOam | 80933A | 907 | DMA0 OAM + DMA1 CGRAM, combined MDMAEN=3 |
| NmiTransferSamusToVram | 809376 | 921 | Optional top/bottom bank92 tile descriptors, DMA1 |
| NmiProcessAnimtilesVramTransfers | 809416 | 955 | Descending six-slot bank87 tile DMA0; clear serviced pointers |
| NmiUpdateIoRegisters | 8091EE | 839 | Shadow CPU/PPU registers, latched low/high pairs, gameplay mirrors |
| NMI_ProcessMode7Queue | 808BBA | 436 | Consume/reset Mode7 queue |
| NMI_ProcessMode7QueueInner | 808BD3 | 444 | Nine-byte VRAM or seven-byte CGRAM entries, DMA1 |
| NMI_ProcessVramWriteQueue | 808C83 | 478 | Seven-byte VRAM entries, terminator, both scroll DMA helpers |
| Nmi_ProcessHorizScrollingDma | 808CD8 | 499 | BG1 then BG2 columns; unwrapped then optional wrapped halves |
| Nmi_ProcessVertScrollingDma | 808DAC | 557 | BG1 then BG2 rows; unwrapped then optional wrapped halves |
| NMI_ProcessVramReadQueue | 808EA2 | 615 | Dummy VRAM read, reverse DMA, byte-tail reset |

Primary source: [sm_80.c](../../../../deps/snesrev-sm/src/sm_80.c).
No indirect function-table dispatch occurs inside this eleven-routine closure.

## Exact execution and state contract

For an accepted invocation preserve this order: palette/OAM → Samus tiles →
animated tiles → register update → six HDMA table-pointer writes → conditional
Mode7 queue → VRAM write/column/row queues → VRAM read queue → final HDMAEN →
clear waiting/missed state → increment accepted-byte/accepted-word counters →
increment including-lag counter. A missed invocation emits no hardware service;
its byte missed count wraps, updates the maximum under C's promoted comparison,
and increments only the including-lag word. C's accepted branch does not clear
`nmi_frames_missed_max`.

The register updater tests `(reg_BGMODE & 7) == 7 || (reg_BGMODE_fake & 7) == 7`
for M7 register pairs, while the root tests exact byte equality to7 for queue
processing. Preserve this distinction. Preserve `WriteRegWord` as two ordered
byte writes and `ReadRegWord` as two ordered reads, low then high; PPU address,
scroll, matrix, OAM/CGRAM and VRAM latches make collapsing these unsafe.
[sm_80.c:839](../../../../deps/snesrev-sm/src/sm_80.c#L839),
[sm_rtl.c:546](../../../../deps/snesrev-sm/src/sm_rtl.c#L546).

Use full 131,072-byte WRAM and exact byte/word aliases from
[variables.h](../../../../deps/snesrev-sm/src/variables.h): shadow registers
around0051–0085; waiting/counters05B4–05BB; Samus copy flags/pointers071D–0721;
HDMA channels18B4, bank slots18C0 and table pointers18D8; animation flag1EF1,
IDs1EF5, sources1F25, sizes1F31, destinations1F3D; graphics queues00D0/02D0/0340
and tails0330/0334/0360; OAM0370, paletteC000; column/row descriptors0956–098C
and tilemap sourcesC8C8–CA94. Register updates also change gameplay mirrors and
the high byte of Ceres HDMA state07EB.

Preflight the entire invocation before the first hardware request: queue entry
alignment, terminator space, count bounds, six HDMA slots/channel encodings,
Samus descriptor extents, transfer addresses and permitted banks, and remaining
WRAM aliases. Native `DAS=0` semantics must be preserved (a large transfer),
not interpreted as an empty transfer. Mode7 flag bits choose real DMA modes,
source bank/address and VMAIN; reject unsupported hardware destinations before
requesting anything. VRAM write destinations have a signed-bit VMAIN choice;
VRAM read tails use only the low byte. Packed layouts come from
[ida_types.h:1054](../../../../deps/snesrev-sm/src/ida_types.h#L1054) and
[ida_types.h:1272](../../../../deps/snesrev-sm/src/ida_types.h#L1272).

## Dependencies and ownership

| Dependency | Concrete review / required evidence |
| --- | --- |
| WriteReg / ReadReg / their word forms | Rust sequences bytes; host executes actual pinned native primitives synchronously. Bind full pre/post state and effect order, not only request count. `sm_rtl.c:525–555`. |
| snes_write / snes_read / DMA | MDMAEN runs DMA to completion before returning; HDMAEN arms channels. Capture DMA channels/timers/busy state, CPU NMI/IRQ pending flags, hardware NMI/IRQ enable/timing and WRAM writes from reverse DMA. `snes.c:289–378`, `dma.c`, `dma.h`. |
| PPU views/open bus | Capture active `ppu` discriminator, both `snes_ppu` and `my_ppu` views, relevant VRAM/CGRAM/OAM and register latches, plus open bus and B-bus behavior. `snes.h:30`, `ppu.h`, `dma.c:309`. Do not substitute the older Ridley20-byte PPU snapshot for this broader state. |
| RomPtr_92 | Decode the seven-byte bank92 descriptor exactly, then bind both descriptor and source tile-data ranges to original/native ROM identities. `sm_rtl.h:70`, `ida_types.h:615`. |
| ADDR16_OF_RAM, gVramWriteEntry, LOBYTE/HIBYTE, sign16 | Fixed canonical WRAM offsets, packed little-endian layout and byte-only writes; independently test bank edges, odd addresses and alias boundaries. `variables.h`, `ida_types.h:2554`, `types.h`. |
| Prepared animation/room/Samus/HDMA queues | Producers remain their existing owners. Consuming their outputs does not assign animation interpreters, scrolling gameplay, player pose or HDMA allocation to this child. Existing holds96/97 and held game-host source remain fenced. |
| NMI scheduling/controller continuation | External caller boundary must prove enable/disable/setup behavior and native polling order; ticket12 owns the known extra-poll correction. Neither elapsed frames nor a helper counter proves this scheduled root. |
| IRQ/APU | NMI register writes affect IRQ setup; whole `Vector_IRQ` dispatch/door scrolling and audio acknowledgment/upload remain explicit outside services. Require unchanged APU state for supported NMI transfer domains and reject DMA targeting APU/unsupported registers before effects. |

The gameplay implementer owns only these proposed new files:
`crates/port/src/hardware_lifecycle.rs`,
`crates/port/tests/hardware_lifecycle.rs`,
`tools/port/hardware-lifecycle.toml`. It must not modify held `game_host.rs`,
its tests/manifests, `lib.rs`, `memory.rs`, pipeline registration or frozen
Ridley source. The sole integrator supplies shared wiring and lifecycle adapter.

The new narrow hardware trait can expose byte `ReadRegister` and
`WriteRegister` requests with typed replies. Do not route a complete native
`Vector_NMI` back through the host: that would retain the proposed root's
gameplay ownership in C. Keep native DMA/PPU execution explicit. Use the same
execution-once rules already established by
[game_host.rs](../../../../crates/port/src/game_host.rs): private WRAM until
completion, full latest WRAM before/after every service, no-request refusal
leaves all caller state untouched, and any first-service failure or later error
is owned fatal failure with no native retry. Count attempts before the service;
a failed host may already have effects.

## Existing evidence and material gaps

The held shared-host implementation has31 Rust tests and unchanged Samus/room
consumers. The two actual-C verdicts establish7 projectile pairs and3 room
pairs with ordered PLM/HDMA and17 VRAM flushes:

- [projectile verdict](../../../../out/port/ceres-continuation/game-host-projectile/1791873394532541-258459-0/verdict.toml), SHA-256 `18f355479a34ef10be6196e2c1507f6ded34d522693f068feb1313078492abad`.
- [room verdict](../../../../out/port/ceres-continuation/game-host-room-scroll/1791873418836128-259326-0/verdict.toml), SHA-256 `ddb49cbd79a7249b45745044f9f6c95a7e98671fa74e0b313a7f81bf8c869d1d`.
- [implementation observation](../../../../.scratch/ceres-continuation/host-implementation.toml), SHA-256 `7b4f118b21b969bd02abc16192cbc3caccb1c118d5cef6a5d39e3751afeaffeb`.

Both verdicts explicitly state `live_exercised=false` and
`original_rom_parity="not-tested"`. Their callback log demonstrates reusable
ordered hardware observation, not complete NMI state. The current callback
fixture format loads only WRAM; it has no general pre/post NMI hardware image.
[callback_host.c.in](../../../../crates/pipeline/src/callback_host.c.in),
[family_model.rs](../../../../crates/pipeline/src/family_model.rs).

The controller2240-frame diagnostic establishes1736 causal completions, with a
later mismatch at ordinal7741/frame8508; it is not a source-authenticated
per-routine NMI exercise receipt. This research found no currently supported
NMI exercise receipt. [controller handoff](../../../../.scratch/ceres-continuation/controller-handoff-20261013.md).

The native scope reader [translation.rs:400](../../../../crates/pipeline/src/translation.rs#L400)
accepts only schema2 bank86 projectile diagnostics. It authenticates deterministic
bank86 instrumentation, `projectiles.tsv`, pinned host/build/binary and replay
commands; every non-ID address is constructed as860000 | low16. Relabeling an
NMI counter would therefore be invalid. Preserve this reader unchanged for
existing receipts and add the reviewed extension before producing native scope.
No executable `translation-scope-v1` has been fabricated here.

## Original scheduling and reset limitations

Recovered `RunOneFrameOfGame` calls `Vector_NMI` unconditionally after its
coroutine step. The pinned CPU runtime patches out original NMI controller call
$8095E1 and APU acknowledgment reads. The causal host supplies controller input
after each C NMI. These are explicit adaptations, not original-ROM scheduling
proof. [sm_82.c:702](../../../../deps/snesrev-sm/src/sm_82.c#L702),
[sm_cpu_infra.c:871](../../../../deps/snesrev-sm/src/sm_cpu_infra.c#L871),
[prefix_host.c:69](../../../../tools/decomp/prefix_host.c#L69).

The recovered hardware loop requests `cpu->nmiWanted` while its `nmiEnabled`
check is commented out; its NMITIMEN setter also records a TODO for enabling
NMI during VBlank. An original-ROM gate must observe native NMI edges/polls and
enabled/disabled intervals in an independent original emulator, including the
ticket12 setup interval; equality to this recovered-C runtime alone cannot
prove those semantics. [snes.c:130](../../../../deps/snesrev-sm/src/snes/snes.c#L130),
[snes.c:303](../../../../deps/snesrev-sm/src/snes/snes.c#L303).

Reset is a subsequent complete boundary, not a missing helper of this root.
`Vector_RESET_Async ($80841C)` reaches CPU/PPU initialization, logo coroutine,
APU upload, WRAM clear, OAM clear, NMI enable and SRAM verification/load/write.
Cold reset uses coroutine positions1/2; soft reset position3 skips logo and
initial MEMSEL/8KiB clear. Keep SRAM and hardware state rather than power-cycling
for the soft-reset sentinel. The resolved
[ticket76](../../../../.scratch/decomp-port/issues/76-reference-soft-reset-crash.md)
establishes its crash repair, not full original reset parity. `GameState_0_Reset_Async`
only selects the opening state and is not an equivalent hardware root.
[sm_80.c:153](../../../../deps/snesrev-sm/src/sm_80.c#L153),
[sm_82.c:803](../../../../deps/snesrev-sm/src/sm_82.c#L803).

Whole reset therefore follows reviewed persistence19, audio18, logo/coroutine
and original hardware dependencies. Whole IRQ follows its14-way table and
door follow-up dependency. Keep these consumers blocked where they actually
need those services; completing this NMI child alone cannot resolve06–08.

## Concrete children and verification

The [child proposals](children.toml) are review inputs, not admitted scope files.
Root assigns official ticket numbers and blockers before dispatch.

1. **13-exercise-contract**: additive Rust native-root exercise producer/reader,
   full24-bit identities and deterministic instrumentation/build authentication.
   Produce bounded diagnostic receipts with `accepted=false`, measured direct
   Vector_NMI entry/return, accepted/missed branch counts, per-helper material
   counts, first/last frame/event and hardware-domain observations. Preserve
   bank86 schema2 behavior. Reject wrong banks/symbols, callee-derived roots,
   changed sources/instrumentation/ROM/movie/binary/build, invented counters,
   incomplete/truncated receipts and redirected commands. Submission60s;
   integration3600s including source-bound actual-C producer control.
2. **13-hardware-adapter**: integrator-owned additive Rust full hardware-image,
   callback, actual-C and live boundary support. Store byte-order MMIO trace,
   both PPU views/discriminator, DMA/IRQ/NMI/open-bus state and WRAM effects in
   sealed versioned binary artifacts with readers. Preserve existing host
   receipts unchanged. Submission60s; integration3600s including actual DMA0/1,
   latch/read/reverse-DMA and refusal/failure controls. No gameplay root is
   claimed by this infrastructure acceptance.
3. **13-nmi-implementation**: after11 and both reviewed extensions, generate and
   validate native `translation-scope-v1` with these eleven routines, exact
   source/exercise hashes, every external review and the dedicated3-file envelope.
   Implement the complete preflight/execution body; integrator wires it. Submit
   `cargo test -p port --locked --offline --test hardware_lifecycle --test game_host --test samus_projectile_frame --test room_scroll`
   with60s timeout. Freeze merged source; run
   `cargo run -p pipeline --locked --offline --bin sm-pipeline -- family-verify --manifest tools/port/hardware-lifecycle.toml --output out/port/hardware-lifecycle --case-jobs 4`
   with3600s timeout and authenticate its returned verdict after cleanup.

The actual-C matrix must include accepted/missed and counter wrap, all shadow
register/pair writes, both M7 discriminators and entry types, top/bottom/both
Samus parts, six-slot descending animation and disabled/empty cases, channel
table writes, horizontal/vertical BG1/BG2 wrapping, reverse DMA and dummy reads,
terminators/odd/one-byte tail behavior, overlapping WRAM/ROM source ranges and
size-zero native semantics. Include malformed preflight unchanged-state controls,
wrong-response kinds, failure on the first/late MMIO request, and no native retry.
Differential expectations come from compiled native C; independent tests assert
ordering and state effects rather than mirroring implementation branches.

Merged verification uses `batch plan --changed-since BASE` to identify every
Affected Offline family, then `batch verify` for that exact selection, including
the unchanged projectile/room actual-C matrices and required material137/trail/
camera regressions when source dependencies make them affected. Do not rerun
unrelated full-game gates solely because research concluded.

Proposed live wiring must first admit the hardware adapter; then the existing
`live-verify --collision-verdict NMI_VERDICT --frames 9000 --live-jobs 4` gate
(3600s) runs four independent subjects from ordinary reset with nonzero exclusive
Rust root/accepted/material DMA hits, full state/ordered hardware parity and
zero CPU fallback. This argv is conditional on the additive adapter explicitly
supporting that option; current `game-host` wiring cannot execute the new root.
Additional input domains are needed when nine-thousand inputs do not exercise
Mode7, animation, reverse-DMA or other branches; unobserved branches retain only
offline coverage and are never credited as live.

The original gate remains `decomp-prefix --profile compact --frames 8600`,
bounded60s for an initial reviewed experiment, then the existing ticket12
corrected/pinned poll-continuation gate and an independent original-ROM hardware
comparison under the3600s integration budget. The current8600 result is a
retained failure. Do not advertise Ceres escape until ordinary reset reaches
observed32→33→34 with exact native polls; the smaller accepted-NMI body gate is
not a substitute for this milestone.

Publish `translating` only at approved gameplay dispatch, and independently
`translated`, `verified`, `integrated`, `live-exercised` with exact source,
domain, evidence and cleanup. Contract-extension completion alone adds no
gameplay stage. Root adds the generated children to13 and to consumers06–08,
14/15/18/19/20 only for the specific hardware contract each needs.

## Research verification and retention

This worker performed source/receipt reads and hashes only. No Cargo build,
fixture, live run, emulator experiment or disposable worktree was created.
The Answer relies on existing sealed evidence without executing its recursive
admission readers; pointers and limitations remain intact. Authored research
artifacts are retained in this directory. No run cleanup is required for this
no-run research; reclaimed bytes0. Exact final filesystem measurements and
artifact hashes are in [handoff.md](handoff.md) and `artifacts.sha256`.
