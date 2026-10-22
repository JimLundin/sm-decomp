# Movies and owned state

Source investigation: 2026-10-22, production at `43a4a1c9`. The local
player-projectile candidate is inspected separately and remains unadopted.
This note proposes a small extension of the existing comparison path; it
implements no importer, observation format or new verification framework.

## Answer

Yes: we can automatically translate reference state into owned state **once
the mapping is defined and validated**. We already have scoped examples. The
movies currently imported by this project contain controller inputs, however,
not a memory image for every frame. Running those inputs through the oracle
produces the reference states. We should distinguish three operations:

| Operation | Purpose | Current position |
| --- | --- | --- |
| Movie inputs to replay inputs | Drive both implementations through the same actions | Clean-start LSMV/BK2 to SNES controller words exists; target Session input admission still needs its mapping |
| Reference state to observations | Compare the new state model with the original at an agreed point | Whole-RAM/hardware comparisons exist; target semantic projections are specified but not implemented |
| Reference checkpoint to owned starting state | Resume the new engine without replaying the prefix | Scoped codecs exist; a complete target Session converter does not |

These operations follow the existing importer, verifier and accepted migration
contract.[^inputs][^runner][^migration] A smaller observation record can validate
selected behavior without containing everything necessary to resume execution.
The target validation document explicitly separates projection from checkpoint
import.[^validation]

## What the existing files contain

`tools/movie_inputs.py` reads an LSMV/BK2 input log and writes one hexadecimal
SNES controller word per frame. Its LSMV path rejects savestates, RAM/SRAM
initialization, alternate controller settings, resets and subframes; the BK2
path rejects `Core.bin`, `SaveRAM.bin`, active reset/power and extra controllers.
The movie manifest supplies recording identities, frame counts and hashes;
it is not a state trace.[^inputs][^manifest]

The verifier supplies each word to `RtlRunFrame`, advances audio and observes
the resulting game. The comparison runner captures `before`, runs the CPU
branch, captures `theirs`, restores `before`, runs the Rust branch and captures
`mine`. Its snapshots include RAM/SRAM and selected hardware/audio data.
They are temporary comparison objects, with historical partial restore rules,
rather than portable owned-engine checkpoints.[^runner][^snapshots]

Local `.smvchk` verifier checkpoints are a different format. They serialize
RAM, explicitly selected `Game` fields, recorder state, hardware, both PPUs,
audio and SRAM. They preserve a completed-frame continuation, verify ROM,
input-prefix, runtime and audio compatibility, and test capture/load round
tripping. They omit allocation pointers and padding and retain the newly
initialized host's allocations. This loader has no mechanism that automatically
discovers a future typed field.[^checkpoint][^load]

The existing console replay trace reports selected room/player/progression
fields. It neither contains complete state nor constitutes the target
semantic comparison format. Existing probes count explicit hits and locate
them in movie input frames; they do not infer state-machine transitions or
compare semantic records.[^trace][^probes]

## Concrete translations already present

The local candidate's `codec.rs` imports ten owned `Shot` records from the
legacy parallel arrays and writes their fields back. For slot `i`, it reads
X at `$0B64 + 2i`, Y at `$0B78 + 2i`, separate fractional positions, velocity,
kind, damage, animation cursor/timer, radii and motion. Keeping fractions,
timers and cursor alongside position is essential for continuation. Its
mapping comes from the actual layout, not an inference from a screenshot or
position trace.[^codec][^ram]

The same codec maps preinstruction `$AF68` to `Motion::Missile`, `$AFE5` to
`Motion::SuperMissile` and `$B169` to `Motion::Inactive`; unknown words remain
`Motion::Legacy(word)` and encode unchanged. The candidate's pending-aim codec
also retains unknown marker bytes through `Capture::Restored`. A conversion
must not silently assign an unknown or aliased value to the nearest plausible
named state.[^codec][^aim]

Production `launch_motion_ram` provides another example: four high/low word
pairs at `$0DAA..$0DB9` become signed 32-bit displacement history. Its launch
consumer deliberately rereads after a vertical-velocity write, because a
noncanonical projectile index can alias that history. Correct translation
includes ownership and observation order, not just field names.[^motion]

The local session-pass codec illustrates why context matters: continuation
tag `1` means `Gameplay::EnemyMessage` or `Door::Vblank`, depending on the
separately supplied `door` context. The byte alone does not identify the typed
state. These scoped codecs are useful implementation starting points; the
local pass is not the complete target `Session`.[^continuation][^migration]

Constructed candidate checks publish and reopen the owner around selected
phases, comparing RAM, APU, DMA and callback observations against legacy
execution. That establishes those checked transfers, not movie execution or
frontend/checkpoint integration.[^candidatechecks][^state]

## What can and cannot be automatic

After a domain defines its exact encoding, the reads, enum decoding, input
conversion, projections and compatibility checks can run automatically.
Replaying a boot prefix can also construct the new engine's own state,
including history that a small observation omits. The accepted architecture
already permits boot/replay when old checkpoints lack a validated converter.
[^migration][^contracts]

A generic RAM dump or a finite trace does not determine a unique new state
machine: different internal histories can have the same observed position,
pose and timers. The converter must explicitly account for input history,
RNG, pending actions, sequence cursors, actor references, simulation order and
allocator generations required by the destination. New generations/order
counters need a defined reconstruction and reference-fixup policy, or must
be produced by replay. Guessing them because visible fields match does not
establish a valid starting state.[^contracts][^validation]

ROM-bank cursors, numeric function selectors and room/actor definitions must
map to the validated content revision's stable identities and sequence
positions. Host allocation addresses cannot become persistent IDs. The current
candidate intentionally still uses encoded cursor/sprite wrappers; that is a
compatibility representation, not the finished content-ID conversion.
[^codec][^contracts][^migration]

Timing also needs a contract. Today an input word is installed in the emulated
controller before a comparison frame, while `ReadJoypadInputs` computes held
and newly pressed values using previous sampled input. Target replay should
admit equivalent values at the same logical latch. A movie frame must not be
assumed to equal one player update through freezes, suspension, menus or input
polling. Preserve the original cadence and history rather than recomputing
edges from only the selected gameplay frames.[^latch][^contracts]

## Smallest useful next steps

1. **Keep replaying existing inputs.** Define the target input admission and
   frame/Tick relationship for the first owned slice, including previous input
   and the current reset/start policy. Run from boot or a supported Scenario
   until exact checkpoint conversion is justified.[^latch][^contracts]
2. **Add one comparison seam when the owner is ready.** Extend the current
   verifier with a domain observation at a specified existing phase or frame
   boundary. Decode the CPU state and observe the owned state into the same
   record: for projectiles, include slot/order, motion, positions/fractions,
   speed, kind/damage, animation state, counts and relevant ordered effects.
   Keep units, wrapping, content identity and exclusions explicit. Existing
   entry/branch/effect probes select exercised movie segments; they are not
   themselves this observation mechanism. Preserve current RAM/render/audio
   comparisons while the representation supports them. Capture independent CPU
   observations before normalization copies Rust values into its snapshot;
   copied continuation fields need separate checks, not a fidelity claim from
   their normalized equality.[^migration][^validation][^probes][^normalization]
3. **Validate scoped imports before complete imports.** Reuse the candidate
   codecs while preserving the exact current checked byte layout. Cover
   canonical, unknown-marker, alias, suspension and partial-failure cases;
   retain unknown words where the compatibility model supports them and
   explicitly reject unsupported destination states. Check decode/encode on
   represented bytes, then compare advance from imported state against advance
   through the real legacy implementation.
   A codec round trip alone proves neither transitions nor owner completeness.
   Use existing component checks and verifier checkpoints; add no registry or
   parallel capture framework.[^candidatechecks][^motion][^plan]
4. **Integrate ownership transfers as the lifecycle closes.** Export before
   compatibility writers, import the actual selected state after reset/load,
   CPU/native callbacks, snapshot restore, replay patches and oracle adoption.
   Import the normalized selected image where required: some continuation bytes
   are copied from the Rust branch into the oracle image. Keep only one mutable
   authority, and invalidate it on partial load failure.[^state][^normalization]
5. **Offer full checkpoint conversion only when complete.** Declare the source
   format/ROM/content and destination schema/fingerprint, supported capture
   boundary, all required state and explicit unsupported cases. Compare direct
   imported continuation with boot/replay continuation over relevant future
   effects, including restore and pending actions. Continue using boot/replay
   starts for states outside that validated scope.[^load][^contracts][^validation]

Extend the existing verifier with **movie inputs -> synchronized execution ->
reference/owned observations** as each responsibility becomes owned. Add
**checkpoint -> complete owned state** only for a separately validated capture
scope. Semantic observations alone do not establish a unique full importer or
justify retiring the old comparison.[^migration][^plan]

[^inputs]: [Movie importer](../tools/movie_inputs.py#L16), especially `load`, `parse_lsmv` and `parse_bk2`; [movie commands](../movies/README.md).
[^manifest]: [Published recording manifest](../movies/manifest.toml#L1).
[^runner]: [Replay execution](../sm/src/verify.rs#L252); [paired execution](../sm/src/cpu_infra.rs#L1085).
[^snapshots]: [Snapshot fields](../sm/src/cpu_infra.rs#L53); [capture and partial restore](../sm/src/cpu_infra.rs#L752).
[^checkpoint]: [Explicit machine serialization](../sm/src/verify/checkpoint.rs#L322); [capture preconditions](../sm/src/verify/checkpoint.rs#L390).
[^load]: [Checkpoint save metadata](../sm/src/verify/checkpoint.rs#L414); [load and round trip](../sm/src/verify/checkpoint.rs#L448).
[^trace]: [Existing replay observations](../sm/src/verify.rs#L23).
[^probes]: [Probe implementation](../sm/src/verify_probe.rs); [selection and follow-up commands](../movies/README.md#L103).
[^ram]: [Projectile layout](../sm/src/ram_map.rs#L556).
[^codec]: Local unadopted [projectile codec](</C:/Users/Jim.Lundin/AppData/Local/sm-decomp-experiments/20261022/player-projectiles/runtime-impact-working/codec.rs:13>) and [owned records](</C:/Users/Jim.Lundin/AppData/Local/sm-decomp-experiments/20261022/player-projectiles/runtime-impact-working/engine.rs:5>).
[^aim]: Local unadopted [pending-aim codec](</C:/Users/Jim.Lundin/AppData/Local/sm-decomp-experiments/20261022/player-projectiles/runtime-impact-working/placement_adapter.rs:6>).
[^motion]: [Launch-history codec and ordered reread](../sm/src/game/launch_motion_ram.rs#L17).
[^continuation]: Local unadopted [context-dependent continuation codec](</C:/Users/Jim.Lundin/AppData/Local/sm-decomp-experiments/20261022/player-projectiles/runtime-impact-working/encoded_session_pass.rs:8>).
[^candidatechecks]: Local constructed [phase-transfer comparison](</C:/Users/Jim.Lundin/AppData/Local/sm-decomp-experiments/20261022/player-projectiles/runtime-impact-working/runtime_session_checks.rs:502>).
[^latch]: [Controller installation](../sm/src/cpu_infra.rs#L1249); [held/pressed sampling](../sm/src/game/sm_80.rs#L839).
[^normalization]: [Snapshot normalization](../sm/src/cpu_infra.rs#L362), applied during [comparison](../sm/src/cpu_infra.rs#L649); [selected-image restore](../sm/src/cpu_infra.rs#L1214).
[^state]: [State migration: authority, external writers, reset/load, checkpoint and oracle adoption](ENGINE_STATE_MIGRATION.md).
[^migration]: [Accepted comparison/migration policy](target-architecture/MIGRATION.md#comparison-during-migration).
[^contracts]: [Target input, ownership, checkpoint and compatibility contracts](target-architecture/CONTRACTS.md#public-gameplay-interface), especially Tick phases, input admission and saves/checkpoints/replays.
[^validation]: [Target verification strategy and checkpoint completeness](target-architecture/VALIDATION.md#verification-strategy).
[^plan]: [Active work order and validation strategy](PLAN.md#validation-strategy-2026-10-21): exercised checks, no new framework/evidence stores, ROM-derived artifacts stay local.
