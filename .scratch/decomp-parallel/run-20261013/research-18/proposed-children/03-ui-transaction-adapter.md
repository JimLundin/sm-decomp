# Specify complete equipment/options UI actual-C and exclusive live adapters

Type: research
Status: proposed; coordinator review/publication required
Origin: research 18 exact claim, attempt 1; see [Answer](../Answer.md) and [manifest](../children.toml).
Admission: `reviewed-infrastructure-scope-after-hardware-and-provenance`. No ownership or gameplay acceptance is granted.

Select four concrete adapter domains for equipment rendering, EquipmentScreenMain/categories, options objects and GameState 2 options orchestration. These share a versioned canonical full-state fixture/live contract, not a new synthetic menu root. File/map/pause restoration are later domains and remain out of this proposed implementation.

Import the full WRAM image including options/eproj aliases and player equipment/reserve/controller words. Reuse reviewed NMI/hardware service state for PPU/DMA/reverse DMA/VRAM/CGRAM/OAM, plus exact SRAM/audio/backend/coroutine state. MMIO, DMA, decompression, save and polling service calls are synchronous and fail without native retry after the first effect.

DrawSpritemap clips under its pinned per-record/off-screen logic and OAM capacity behavior; DrawMenuSpritemap wraps its OAM cursor and processes all records. Gameplay scopes own these real routines, so the adapter supplies ROM/OAM state and retains effects instead of replacing drawing with a broad service. Preflight every consumed descriptor/script/opcode/prehandler/resource extent before public effects. Preserve real VBlank/ScreenOn/controller schedule and prohibit unsupported continuation or slot skipping.

Deliver exact adapter registration, supported manifests, material input windows, C0/C1/mixed0/mixed1 argv and independent original-ROM argv. No current CLI invocation for these adapter names has been run.

## Exact proposed files and ownership

- `crates/pipeline/src/menu_adapter.rs`
- `crates/pipeline/src/menu_model.rs`
- `crates/pipeline/src/menu_host.c.in`
- `crates/pipeline/src/menu_abi.rs.in`
- `crates/pipeline/tests/menu_adapter.rs`

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
- 22-full-state-MMIO-DMA-service
- 12-controller-poll-contract
- 19-SRAM-services
- 18-audio-transaction-adapter

Parent must replace dependency descriptions with exact completed local task paths, current input/dependency evidence hashes and supported observed-root receipts before native admission. Historical bounded leaves remain historical until current-source authentication.

## Proposed verification commands

All new targets/manifests/adapters here are unimplemented, and all commands are unrun. Review must register real adapter support and require nonzero executed tests/verification receipts before accepting these gates.

Submission, at most 60 seconds:

```sh
cargo test -p pipeline --locked --offline --test menu_adapter
```

Merged gate, at most 3600 seconds:

```sh
cargo test -p pipeline --locked --offline --test menu_adapter --test callback --test family --test live
```

Exact four-subject material Live replay and independent original-ROM argv/timeouts are a required infrastructure/research deliverable before implementation scope publication. No invented current live flag or adapter string is admitted. Native scope argv must not override source/checkout/root/changed-since; the integrator separately records the exact baseline and all-affected selection on frozen source with workers 4.

## Acceptance and controls

- Invoke actual category/root/object/screen/draw signatures with full canonical128 KiB WRAM, PPU/DMA/OAM/VRAM/CGRAM/SRAM/audio and coroutine/host state.
- Preflight every selected descriptor/script/prehandler/opcode/sprite/palette/wireframe/decompressed range and queue/continuation before effects.
- Real DrawSpritemap and DrawMenuSpritemap effects preserve respective clipping versus OAM wrapping; no canned draw service substitutes them.
- Preserve actual controller polls, word player bindings/bitsets/reserve modes, SRAM transactions and upload/NMI scheduling.
- Exclusive complete root ownership/refusal before effects, nonretrying service failure and four independent material subjects with visible fallback.
- Publish exact supported adapters and Live/original-ROM argv plus semantic selector/asset/state/poll/save/refusal/failure controls before native scope admission.

## Limitations and cleanup

- Exact new/shared files and any controller/persistence ABI extensions need separately reviewed ownership; file/map successors are excluded.
- Proposed new module/test/manifest/adapter is unimplemented; all new gate argv are unrun and require separately reviewed adapter registration.
- No current supported exercise, native Scope admission, verified gameplay or original-ROM verdict is asserted.

After any future run, wait for writers, run both scoped project cleanup passes, preserve source/readers/store/private-input dependencies, retire integrated worktrees, and reauthenticate retained receipts. Record measured reclaimed/free bytes including zero. This research created no experiment or run; [cleanup](../cleanup.toml) records that result.

