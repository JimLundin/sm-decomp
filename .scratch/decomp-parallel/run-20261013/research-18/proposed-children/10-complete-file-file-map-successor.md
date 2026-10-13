# Close separate file selection/copy-clear and file-map subsystems with SRAM/controller ownership

Type: research
Status: proposed; coordinator review/publication required
Origin: research 18 exact claim, attempt 1; see [Answer](../Answer.md) and [manifest](../children.toml).
Admission: `research-before-native-implementation-scopes`. No ownership or gameplay acceptance is granted.

FileSelectMenu has 34 table entries / 29 unique handlers; FileSelectMap has 23/20. Full closures include draw/map/persistence/initialization dependencies and cannot be admitted from handler counts alone. Preserve addressless root identity and the observed menu_index/table selections.

Copy and clear operate on 1628 bytes plus four checksum positions. Main menu records selected SRAM slot/inverse and invokes load/new-save/map mirrors; preserve two SFX calls on the B branch and draw/persist ordering. Shared SRAM transaction, selected slot and new-save player/controller/map initialization belong to 19, not duplicate menu persistence.

Held78/controller 12 owns the original VBlank/ScreenOn wait/poll placement reached by file states 0/16. Follow its retained evidence and future accepted service contract. Never change prefix_host/scheduler or assume one poll per frame to complete this scope.

Research separate file selection/copy-clear and file-map material roots with all tile/sprite/map/decompression assets and exact failure semantics. Produce coherent1–32 routine native children only when complete observed closure/accepted services/current seals exist; the two triplets and family names are provisional.

## Exact proposed files and ownership

- `crates/port/src/file_menu.rs`
- `crates/port/tests/file_menu.rs`
- `tools/port/file-menu.toml`
- `crates/port/src/file_map_menu.rs`
- `crates/port/tests/file_map_menu.rs`
- `tools/port/file-map-menu.toml`

The coordinator owns public lib/state/ROM/SRAM wiring, shared pipeline/ABI/catalog registration, main Git/index, tickets and inventory. Isolated workers are not alone in the codebase; they must retain other workers' changes and use only returned exact writable files/branch/worktree/baseline. These proposals authorize no shared edit.

## Dependencies

- 19-full-SRAM-save-load-new-save-contract
- 12-held-controller-VBlank-ScreenOn-polls
- 22-23-full-map-render-NMI-state
- 18-audio-service
- 18-pause-equipment-render-for-shared-draw-leaf

Parent must replace dependency descriptions with exact completed local task paths, current input/dependency evidence hashes and supported observed-root receipts before native admission. Historical bounded leaves remain historical until current-source authentication.

## Proposed verification commands

All new targets/manifests/adapters here are unimplemented, and all commands are unrun. Review must register real adapter support and require nonzero executed tests/verification receipts before accepting these gates.

Submission, at most 60 seconds:

```sh
cargo test -p port --locked --offline --test file_menu --test file_map_menu
```

Merged gate, at most 3600 seconds:

```sh
cargo run -p pipeline --locked --offline --bin sm-pipeline -- family-verify --manifest tools/port/file-menu.toml --adapter file-menu --output out/port/file-menu --case-jobs 4
```

```sh
cargo run -p pipeline --locked --offline --bin sm-pipeline -- family-verify --manifest tools/port/file-map-menu.toml --adapter file-map-menu --output out/port/file-map-menu --case-jobs 4
```

Exact four-subject material Live replay and independent original-ROM argv/timeouts are a required infrastructure/research deliverable before implementation scope publication. No invented current live flag or adapter string is admitted. Native scope argv must not override source/checkout/root/changed-since; the integrator separately records the exact baseline and all-affected selection on frozen source with workers 4.

## Acceptance and controls

- Follow full FileSelectMenu/FileSelectMap roots and every selected handler/draw/persistence/map/new-save dependency; partition coherent1–32 routines.
- Preserve1628 byte copy/clear, four checksum locations, selected slot/inverse, draw/persist order, new-save player/controller/map reset and two B-branch SFX writes.
- Bind exact menu/sprite/tile/map/decompressed resources and original VBlank/ScreenOn poll count/position already owned by 12/78.
- Publish direct-root/material/coroutine input evidence, exact shared adapters/files, real 60 s / 3600 s gates and separate original-ROM route argv before implementation.

## Limitations and cleanup

- Full closures exceed unique-handler counts; these provisional triplets and gate names grant no ownership or existing adapter support.
- Proposed new module/test/manifest/adapter is unimplemented; all new gate argv are unrun and require separately reviewed adapter registration.
- No current supported exercise, native Scope admission, verified gameplay or original-ROM verdict is asserted.

After any future run, wait for writers, run both scoped project cleanup passes, preserve source/readers/store/private-input dependencies, retire integrated worktrees, and reauthenticate retained receipts. Record measured reclaimed/free bytes including zero. This research created no experiment or run; [cleanup](../cleanup.toml) records that result.

