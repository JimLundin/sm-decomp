# Specify full APU transaction, audio fixture/live adapter and original acknowledgment domain

Type: research
Status: proposed; coordinator review/publication required
Origin: research 18 exact claim, attempt 1; see [Answer](../Answer.md) and [manifest](../children.toml).
Admission: `reviewed-infrastructure-scope-after-host-closure`. No ownership or gameplay acceptance is granted.

The pinned C handlers 1/2/3 differ from original ROM: state 1 advances without acknowledgment and sets delay 6; state 2 omits hardware zero; state 3 immediately resets/sends. Original state 1 and3 compare the acknowledgment port, rewrite on mismatch, and wait; state 1 uses delay 2, and state 2 writes zero to hardware. The 12 changed bytes in rom-observations.json corroborate the patch declarations, not original timing.

Do not change the pinned oracle silently. Specify a versioned pinned-C actual-C domain plus a separately reviewed original-ROM acknowledgment domain. Resolve native APU reads (current asserted branch), real SPC/native backend clock/sample progression, original bank-upload/sentinel waits, queue coalescing/suppression and reset/load state before promising original playback.

APU_UploadBank delegates only when g_use_my_apu_code is true. Full selected upload chunks need preflight through zero-length termination and 16-bit target wrap. RtlApuUpload resets the pending queue and mutates SPC/DSP/player state; RtlApuWrite's pending last-value storage and runningWhichVersion2 suppression must remain independently observed from ordered root writes.

HdmaObjectHandler calls music before game state; RunOneFrameOfGame calls SFX after completed inner dispatch, and message coroutines call both around waits. Adapter signatures and root ownership must preserve those separate positions. The future audio-frame adapter name is not implemented today. Deliver exact supported family and four-subject Live/original-ROM argv, timeouts and semantic controls before gameplay scope publication.

## Exact proposed files and ownership

- `crates/pipeline/src/audio_frame_adapter.rs`
- `crates/pipeline/src/audio_frame_model.rs`
- `crates/pipeline/src/audio_frame_host.c.in`
- `crates/pipeline/src/audio_frame_abi.rs.in`
- `crates/pipeline/tests/audio_frame_adapter.rs`

Shared hook candidates for separately reviewed integrator ownership:

- `crates/pipeline/src/hardware_lifecycle_adapter.rs`
- `crates/pipeline/src/hardware_lifecycle_model.rs`
- `crates/pipeline/src/family.rs`
- `crates/pipeline/src/family_model.rs`
- `crates/pipeline/src/callback.rs`
- `crates/pipeline/src/live.rs`
- `crates/pipeline/src/live_model.rs`
- `crates/pipeline/src/main.rs`
- `tools/port/batch.toml`

The coordinator owns public lib/state/ROM/SRAM wiring, shared pipeline/ABI/catalog registration, main Git/index, tickets and inventory. Isolated workers are not alone in the codebase; they must retain other workers' changes and use only returned exact writable files/branch/worktree/baseline. These proposals authorize no shared edit.

## Dependencies

- 11-held-host-completion
- 18-exercise-contract-supplement
- 22-hardware-full-state-and-service-failure-contract
- 19-reset-and-load-state-contract
- 12-controller-and-original-ROM-timing

Parent must replace dependency descriptions with exact completed local task paths, current input/dependency evidence hashes and supported observed-root receipts before native admission. Historical bounded leaves remain historical until current-source authentication.

## Proposed verification commands

All new targets/manifests/adapters here are unimplemented, and all commands are unrun. Review must register real adapter support and require nonzero executed tests/verification receipts before accepting these gates.

Submission, at most 60 seconds:

```sh
cargo test -p pipeline --locked --offline --test audio_frame_adapter
```

Merged gate, at most 3600 seconds:

```sh
cargo test -p pipeline --locked --offline --test audio_frame_adapter --test callback --test family --test live
```

Exact four-subject material Live replay and independent original-ROM argv/timeouts are a required infrastructure/research deliverable before implementation scope publication. No invented current live flag or adapter string is admitted. Native scope argv must not override source/checkout/root/changed-since; the integrator separately records the exact baseline and all-affected selection on frozen source with workers 4.

## Acceptance and controls

- Exact nine real C signatures; independent HandleMusicQueue and HandleSoundEffects scheduling, no adjacent synthetic replacement.
- Complete WRAM and selected APU/SPC/DSP/backend/pending 16-entry queue/cycle/sample state; ordered reads/writes/uploads plus coalescing/suppression/reset effects.
- Preflight entire selected pointer/chunk stream through sentinel and 64 KiB wrap before effects; synchronous service copyback and first/late failure no retry.
- Explicit pinned-C domain and separate original-ROM ack domain with read-compare/rewrite, delay 2, clear-port and state 3 wait semantics; no unreviewed oracle change.
- Review native APU read assertion, clock/sample driver, original upload sentinel/waits, reset/SRAM-load restoration and backend support without editing owned vendored source in this scope.
- Publish exact supported family adapter registration and material Live/original-ROM argv, natural root windows, nonzero ownership and independent semantic controls before gameplay promotion.

## Limitations and cleanup

- Research must freeze exact files/ABI/backend domain after 22/19 review; these candidates grant no source writes.
- Proposed new module/test/manifest/adapter is unimplemented; all new gate argv are unrun and require separately reviewed adapter registration.
- No current supported exercise, native Scope admission, verified gameplay or original-ROM verdict is asserted.

After any future run, wait for writers, run both scoped project cleanup passes, preserve source/readers/store/private-input dependencies, retire integrated worktrees, and reauthenticate retained receipts. Record measured reclaimed/free bytes including zero. This research created no experiment or run; [cleanup](../cleanup.toml) records that result.

