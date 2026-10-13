# Own complete pinned-C music queue and three-channel SFX service invocation

Type: task
Status: proposed; coordinator review/publication required
Origin: research 18 exact claim, attempt 1; see [Answer](../Answer.md) and [manifest](../children.toml).
Admission: `translation-scope-v1-after-supported-exercise-and-adapter`. No ownership or gameplay acceptance is granted.

The scope owns complete real music and SFX service invocations for the declared admitted domain. Every active SFX channel is considered in ascending order; invalid selectors/cursors or a missing upload asset refuse the whole root before any write. Keep audio.rs/system.rs as accepted dependency modules; do not copy queue behavior into a new state layout.

Preserve unsigned music timer and byte delay wrap, even music cursor units, low-byte bank pointer offset and cur_music_track width, bank-upload early return, ordered APUI00/01/02/03 writes, downtime low-byte-only update and exact ResetSoundQueues footprint. Pinned handler behavior is the oracle for this scope. Original acknowledgment remains an explicit separate backend/source comparison.

The original caller schedule is an integrator obligation, as documented in Answer.md. A test-only combined audio wrapper is not provenance of those game/message roots. Material live playback requires real clock/sample progress, not only headless writes.

## Exact proposed files and ownership

- `crates/port/src/audio_frame.rs`
- `crates/port/tests/audio_frame.rs`
- `tools/port/audio-frame.toml`

The coordinator owns public lib/state/ROM/SRAM wiring, shared pipeline/ABI/catalog registration, main Git/index, tickets and inventory. Isolated workers are not alone in the codebase; they must retain other workers' changes and use only returned exact writable files/branch/worktree/baseline. These proposals authorize no shared edit.

## Exact routine boundaries

| Routine | Pinned source | Entry | Inherited definition SHA-256 |
| --- | --- | --- | --- |
| `APU_UploadBank` | [src/sm_80.c:16](../../../../../deps/snesrev-sm/src/sm_80.c:16) | 0x808028 | `32aec38ebfa1bf4b44005479c7527547a5d64adebdaa22b149af89caa7b8f47b` |
| `HandleMusicQueue` | [src/sm_80.c:646](../../../../../deps/snesrev-sm/src/sm_80.c:646) | 0x808F0C | `f635d911c2c92f687fc62c3eee96617f10f92fadb01d8e9700101980ebeaab2f` |
| `HandleSoundEffects` | [src/sm_82.c:738](../../../../../deps/snesrev-sm/src/sm_82.c:738) | addressless | `455dfd8579639a95fe3e4e08833b94813f64576dfdfd57df11d0cd418783fc6d` |
| `ResetSoundQueues` | [src/sm_82.c:791](../../../../../deps/snesrev-sm/src/sm_82.c:791) | 0x828A9A | `53a4c43e98176d230cb1a163909992d28ccb5f96d511573b49a9fbe432503f2b` |
| `SfxHandlers_0_SendToApu` | [src/sm_82.c:753](../../../../../deps/snesrev-sm/src/sm_82.c:753) | 0x828A2C | `4965c0cb4c9cd1ca226aca43f4a88f8f071e8e7e4533a9f4c149d2630027a8bb` |
| `SfxHandlers_1_WaitForAck` | [src/sm_82.c:764](../../../../../deps/snesrev-sm/src/sm_82.c:764) | 0x828A55 | `4198f7478a20df511fdca15887ba96e3f073b00af5855b8a8b634fe51d680316` |
| `SfxHandlers_2_ClearRequest` | [src/sm_82.c:771](../../../../../deps/snesrev-sm/src/sm_82.c:771) | 0x828A6C | `e1ae710260a7e16bc486bfb4985a311b01b4a10147225c0e19f335909db90574` |
| `SfxHandlers_3_WaitForAck` | [src/sm_82.c:779](../../../../../deps/snesrev-sm/src/sm_82.c:779) | 0x828A7C | `6df227d31f1218c16d6790cf04f6cd98d61f38ca437d7798a013e47643fb9195` |
| `SfxHandlers_4_Reset` | [src/sm_82.c:786](../../../../../deps/snesrev-sm/src/sm_82.c:786) | 0x828A90 | `37a4f8f7d6f1d4debe9e1242d93d79fba378d4a19c7f1b7ac10753d0a49bb45a` |

## Dependencies

- 18-exercise-contract-supplement
- 18-audio-transaction-adapter
- current-source-verified-audio03-system02
- 13-hardware-reset-inputs
- 19-load-state-if-route-loads

Parent must replace dependency descriptions with exact completed local task paths, current input/dependency evidence hashes and supported observed-root receipts before native admission. Historical bounded leaves remain historical until current-source authentication.

## Proposed verification commands

All new targets/manifests/adapters here are unimplemented, and all commands are unrun. Review must register real adapter support and require nonzero executed tests/verification receipts before accepting these gates.

Submission, at most 60 seconds:

```sh
cargo test -p port --locked --offline --test audio_frame --test audio --test system
```

Merged gate, at most 3600 seconds:

```sh
cargo run -p pipeline --locked --offline --bin sm-pipeline -- family-verify --manifest tools/port/audio-frame.toml --adapter audio-frame --output out/port/audio-frame --case-jobs 4
```

Exact four-subject material Live replay and independent original-ROM argv/timeouts are a required infrastructure/research deliverable before implementation scope publication. No invented current live flag or adapter string is admitted. Native scope argv must not override source/checkout/root/changed-since; the integrator separately records the exact baseline and all-affected selection on frozen source with workers 4.

## Acceptance and controls

- Nine exact routines and separate real caller schedules preserve music16-bit postdecrement, byte-offset pointer selection, bank-upload early return and regular track writes.
- Three ascending channels, exact five states, ring cursor widths, byte delays, low-byte downtime writes and nine-byte ResetSoundQueues footprint.
- Full selected bank upload service and g_use_my_apu_code gate are explicit; pinned-C handlers 1/2/3 preserve modified semantics.
- Exact current-source actual-C full WRAM/raw return/ordered effect parity; independent semantic negative controls.
- Supported directly observed complete root/selected-handler/assets provenance with source/build/binary/ROM/movie/readers sealed.
- Whole-request preflight and canonical-state commit; unsupported domain refuses unchanged and fallback is visible; first/late service failure never retries native root.
- Four independent material C0/C1/mixed0/mixed1 subjects, actual playback/menu resources and separate original-ROM route evidence.
- Integrator alone wires shared lib/state/ROM/callback/ABI/catalog, freezes merged source, verifies all affected families and publishes independent stages.
- Both scoped cleanup passes, retained source/readers/store/private inputs, retired implementer worktree and post-cleanup authentication before completion.

## Limitations and cleanup

- Original acknowledgment remains a separate reviewed domain, not a silent change inside pinned-C oracle.
- Proposed new module/test/manifest/adapter is unimplemented; all new gate argv are unrun and require separately reviewed adapter registration.
- No current supported exercise, native Scope admission, verified gameplay or original-ROM verdict is asserted.

After any future run, wait for writers, run both scoped project cleanup passes, preserve source/readers/store/private-input dependencies, retire integrated worktrees, and reauthenticate retained receipts. Record measured reclaimed/free bytes including zero. This research created no experiment or run; [cleanup](../cleanup.toml) records that result.

