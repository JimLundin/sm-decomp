# Extend shared native-root provenance to audio and UI dispatch/state/assets

Type: task
Status: proposed; coordinator review/publication required
Origin: research 18 exact claim, attempt 1; see [Answer](../Answer.md) and [manifest](../children.toml).
Admission: `reviewed-infrastructure-scope-required`. No ownership or gameplay acceptance is granted.

This supplements existing ticket 21. The shared producer/reader already has bank86 schema-2 diagnostics; add tagged audio/UI domains without changing historical seals. Parent reconciles exact shared ownership, source epoch and mutation/drill policy before publishing one infrastructure implementation scope.

The audio root has a specific static gap: HandleSoundEffects's indexed kSfxHandlers call is absent from its current inventory indirect edges. The existing lexical reachability walk cannot use callee observations as proof of the root. Review and bind exact source/table/selected-handler edges alongside measured entry/return. HandleSoundEffects, MainPauseRoutine, EquipmentScreenMain, FileSelectMenu, FileSelectMap and GameState_2_GameOptionsMenu have addressless inventory entries; preserve their definition identity until explicitly reviewed native entry provenance exists.

Audio observations include music timer/entry/pointer/upload, channel/state/cursor/request/delay/downtime and actual port/clock/backend effects. UI observations include screen/item/category and object allocation/global-index/prehandler/opcode selections, pause yield/resume, map/resources and save/controller polling. The new root producer must observe material naturally obtained input and reject zero/truncated/forged caller evidence.

## Exact proposed files and ownership

- `crates/pipeline/src/native_exercise.rs`
- `crates/pipeline/src/native_exercise_model.rs`
- `crates/pipeline/src/native_exercise_host.c.in`
- `crates/pipeline/src/translation.rs`
- `crates/pipeline/src/main.rs`
- `crates/pipeline/tests/native_exercise.rs`

Additional shared review candidates: `crates/pipeline/src/source_inventory.rs`, `crates/pipeline/tests/assignment_queue.rs`.

The coordinator owns public lib/state/ROM/SRAM wiring, shared pipeline/ABI/catalog registration, main Git/index, tickets and inventory. Isolated workers are not alone in the codebase; they must retain other workers' changes and use only returned exact writable files/branch/worktree/baseline. These proposals authorize no shared edit.

## Dependencies

- 21-shared-producer-review
- 11-held-source-completion-and-frozen-source-gates
- current-inventory-producer-reader-equivalence

Parent must replace dependency descriptions with exact completed local task paths, current input/dependency evidence hashes and supported observed-root receipts before native admission. Historical bounded leaves remain historical until current-source authentication.

## Proposed verification commands

All new targets/manifests/adapters here are unimplemented, and all commands are unrun. Review must register real adapter support and require nonzero executed tests/verification receipts before accepting these gates.

Submission, at most 60 seconds:

```sh
cargo test -p pipeline --locked --offline --test native_exercise
```

Merged gate, at most 3600 seconds:

```sh
cargo test -p pipeline --locked --offline --test native_exercise --test assignment_queue --bin sm-pipeline
```

Exact four-subject material Live replay and independent original-ROM argv/timeouts are a required infrastructure/research deliverable before implementation scope publication. No invented current live flag or adapter string is admitted. Native scope argv must not override source/checkout/root/changed-since; the integrator separately records the exact baseline and all-affected selection on frozen source with workers 4.

## Acceptance and controls

- Reuse one additive tagged Rust producer/reader through 21; retain historical bank86 schema2 diagnostic behavior and seals.
- Direct actual-C root entry/return/yield/resume with full address or exact addressless symbol/source/definition; no leaf/callee-to-caller promotion.
- Authenticate audio channel/state/downtime/music entry/upload selectors and UI screen/category/item/object/preinstruction/opcode/persistence/poll selections.
- Review missing HandleSoundEffects indexed edge and source-bound table-handler reachability additively; retain original lexical inventory provenance.
- Seal complete original/generated source, host/templates, producer/readers/build/binary/argv, ROM/native-patch/movie and nonzero observations.
- Reject forged roots, zero/truncated execution, wrong selectors/definitions/inventory/source/build/binary/ROM/movie/backend and stale receipts.

## Limitations and cleanup

- Supplement to21 only; no second engine or authority initialization.
- Proposed new module/test/manifest/adapter is unimplemented; all new gate argv are unrun and require separately reviewed adapter registration.
- No current supported exercise, native Scope admission, verified gameplay or original-ROM verdict is asserted.

After any future run, wait for writers, run both scoped project cleanup passes, preserve source/readers/store/private-input dependencies, retire integrated worktrees, and reauthenticate retained receipts. Record measured reclaimed/free bytes including zero. This research created no experiment or run; [cleanup](../cleanup.toml) records that result.

