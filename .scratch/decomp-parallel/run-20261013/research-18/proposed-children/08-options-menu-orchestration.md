# Own complete options-screen root and controller/special settings transitions

Type: task
Status: proposed; coordinator review/publication required
Origin: research 18 exact claim, attempt 1; see [Answer](../Answer.md) and [manifest](../children.toml).
Admission: `translation-scope-v1-after-supported-exercise-and-adapter`. No ownership or gameplay acceptance is granted.

The 29 routines include GameState_2_GameOptionsMenu and13 screen states, while EnableNMI remains hardware service ownership. The root selects its state handler, executes OptionsMenuFunc1, draws real options objects, and conditionally appends VRAM work in that order.

Own five item table entries/three unique handlers, three special entries/two handlers and nine controller entries/three handlers. Preserve canonical seven eproj_F bindings, uniqueness swaps/defaults, Japanese and special toggles, scroll/dissolve state and file/start transitions. Import any changed child/global options state through the actual aliases.

DMA/decompression/SRAM/audio/NMI/controller calls must use accepted synchronous services. File-menu transitions change state but do not own those later roots. Explicitly review pinned zero placeholders/initializer defects; avoid undefined-behavior fixture input and declare original-ROM differences before live acceptance.

## Exact proposed files and ownership

- `crates/port/src/options_menu.rs`
- `crates/port/tests/options_menu.rs`
- `tools/port/options-menu.toml`

The coordinator owns public lib/state/ROM/SRAM wiring, shared pipeline/ABI/catalog registration, main Git/index, tickets and inventory. Isolated workers are not alone in the codebase; they must retain other workers' changes and use only returned exact writable files/branch/worktree/baseline. These proposals authorize no shared edit.

## Exact routine boundaries

| Routine | Pinned source | Entry | Inherited definition SHA-256 |
| --- | --- | --- | --- |
| `GameOptionsMenuFunc_0` | [src/sm_82.c:4471](../../../../../deps/snesrev-sm/src/sm_82.c:4471) | 0x82EBDB | `7a8466bddacf9c32834622bb763c821459fd3abb53c377a4f62e87c964990f08` |
| `GameOptionsMenuItemFunc_0` | [src/sm_82.c:4576](../../../../../deps/snesrev-sm/src/sm_82.c:4576) | 0x82EDB1 | `11167f1e3ec19fb31ddc7b0ff6e8a899474d598f9d26464b770c866321b689b7` |
| `GameOptionsMenuItemFunc_2_ToggleJapanese` | [src/sm_82.c:4586](../../../../../deps/snesrev-sm/src/sm_82.c:4586) | 0x82EDDA | `da4ef06a2654fa2008fed3a6aa9471ee42e300ef013562f73fc6c4aa5503bca5` |
| `GameOptionsMenuItemFunc_4` | [src/sm_82.c:4606](../../../../../deps/snesrev-sm/src/sm_82.c:4606) | 0x82EE55 | `68a22ad5ab6d83ef326653c2a772797c3066cca63bbba929cc1e9b63d31cadee` |
| `GameOptionsMenuSpecialSettings_0` | [src/sm_82.c:4760](../../../../../deps/snesrev-sm/src/sm_82.c:4760) | 0x82F08E | `0fdfc05ee67b8c6f103f3da2b1cb153563c1b07fa94c3dbc7f5e8075d2faad1f` |
| `GameOptionsMenuSpecialSettings_2` | [src/sm_82.c:4769](../../../../../deps/snesrev-sm/src/sm_82.c:4769) | 0x82F0B2 | `3ad22e6438642b16ae75f3811de0f3e808af2ba6999c56ee7d129f397e00cb34` |
| `GameOptionsMenu_1_LoadingOptionsScreen` | [src/sm_82.c:4485](../../../../../deps/snesrev-sm/src/sm_82.c:4485) | 0x82EC11 | `f93f3e763e15237b0799859f115b019e89de4d86a679605d369ffb4edf4f582f` |
| `GameOptionsMenu_2_FadeInOptionsScreen` | [src/sm_82.c:4524](../../../../../deps/snesrev-sm/src/sm_82.c:4524) | 0x82ECE4 | `8ef75ab8cbe694569a78ba118007ec5d615324edd545444ef715d13991961f9d` |
| `GameOptionsMenu_3_OptionsScreen` | [src/sm_82.c:4558](../../../../../deps/snesrev-sm/src/sm_82.c:4558) | addressless | `cf910c9452cd7bfc4d5b7abde22144548789d9d1d1cdead69c40ef90a1b00eac` |
| `GameOptionsMenu_4_StartGame` | [src/sm_82.c:4635](../../../../../deps/snesrev-sm/src/sm_82.c:4635) | 0x82EEB4 | `38b511d98016b0cb5515a0dc658a3f4d0d85e675e77b2605037633768b7fc431` |
| `GameOptionsMenu_5_DissolveOutScreen` | [src/sm_82.c:4658](../../../../../deps/snesrev-sm/src/sm_82.c:4658) | 0x82EF18 | `59f8706c60427923b5dcda65337466391027ada3b4a58617d294fc24369e0fef` |
| `GameOptionsMenu_6_DissolveInScreen` | [src/sm_82.c:4715](../../../../../deps/snesrev-sm/src/sm_82.c:4715) | 0x82EFDB | `17ccb9e7f411605f85037b10854504879276f92e854f2f99d39e37937a7fda41` |
| `GameOptionsMenu_7_ControllerSettings` | [src/sm_82.c:4805](../../../../../deps/snesrev-sm/src/sm_82.c:4805) | addressless | `b1163900ddc8a71ae7aeb43e65dfff069b5218711f2c095e680ae1272a34e54c` |
| `GameOptionsMenu_8_SpecialSettings` | [src/sm_82.c:4739](../../../../../deps/snesrev-sm/src/sm_82.c:4739) | addressless | `af898063a8f6bf11204d7edbe5437ea2ddc66762a095743237635ea4c118632d` |
| `GameOptionsMenu_9_ScrollControllerSettingsDown` | [src/sm_82.c:4876](../../../../../deps/snesrev-sm/src/sm_82.c:4876) | 0x82F271 | `4fd3ff15740f634f425ed0f6910941efcbf3d0ebdc9d880da1bd6860a5024661` |
| `GameOptionsMenu_A_ScrollControllerSettingsUp` | [src/sm_82.c:4882](../../../../../deps/snesrev-sm/src/sm_82.c:4882) | 0x82F285 | `7d4a5d56d7a5863bb1947581b974bafdbcfe8063e2f2b0fe1bc7d1f0d2411a0c` |
| `GameOptionsMenu_B_TransitionBackToFileSelect` | [src/sm_82.c:4613](../../../../../deps/snesrev-sm/src/sm_82.c:4613) | 0x82EE6A | `814967f1d0a9d48cb691bb301f1f798e578ae8b4e10bcdcb005b5695e2ca7b97` |
| `GameOptionsMenu_C_FadeOutOptionsScreenToStart` | [src/sm_82.c:4625](../../../../../deps/snesrev-sm/src/sm_82.c:4625) | 0x82EE92 | `a5189f860ef9df6c06b62aafef18e7c4fb424b9c9dc746b3286fd61ce6e30035` |
| `GameState_2_GameOptionsMenu` | [src/sm_82.c:4462](../../../../../deps/snesrev-sm/src/sm_82.c:4462) | addressless | `6ed50e575a5fdbd3df2f97744733e510e9556b45db9c927470441541e90b7b16` |
| `LoadControllerOptionsFromControllerBindings` | [src/sm_82.c:4995](../../../../../deps/snesrev-sm/src/sm_82.c:4995) | 0x82F4DC | `416089dfc835e6432fd791dd5509142315fe3cf63e057750c88c110b0f37a778` |
| `OptionsMenuControllerFunc_0` | [src/sm_82.c:5084](../../../../../deps/snesrev-sm/src/sm_82.c:5084) | 0x82F6B9 | `36a365502e89008afae217bd0722346356455d7692b43ea4c06179243fbe4ff7` |
| `OptionsMenuControllerFunc_7` | [src/sm_82.c:4869](../../../../../deps/snesrev-sm/src/sm_82.c:4869) | 0x82F25D | `aa4094eadab606376affb01a026b66bd4c5da7c195cdfce46984ef85f97208ad` |
| `OptionsMenuControllerFunc_8` | [src/sm_82.c:4855](../../../../../deps/snesrev-sm/src/sm_82.c:4855) | 0x82F224 | `42d04ec27161dac11807ed0420fa895edfbe61fc9e6a15d8f6e61fa3eb35a7e3` |
| `OptionsMenuFunc4` | [src/sm_82.c:4592](../../../../../deps/snesrev-sm/src/sm_82.c:4592) | 0x82EDED | `8251af105a482482d3da4bd89cfec012704e6f86f7ead1d563ad090105e39529` |
| `OptionsMenuFunc5` | [src/sm_82.c:4544](../../../../../deps/snesrev-sm/src/sm_82.c:4544) | 0x82ED28 | `1a190d1a6756aeb2b5b4d9fe97957c2f3d0c916ed5c2317f02bbcc1deed04675` |
| `OptionsMenuFunc6` | [src/sm_82.c:5055](../../../../../deps/snesrev-sm/src/sm_82.c:5055) | addressless | `cf8514d733223abe2b5d6ef11c039c085735c5bfebe2ca08756f9c798d229f0e` |
| `OptionsMenuFunc7` | [src/sm_82.c:4782](../../../../../deps/snesrev-sm/src/sm_82.c:4782) | addressless | `693397718c688d2baaa13d52195b1d835288a7efdf97a8b1432797124dd8463f` |
| `OptionsMenuFunc8` | [src/sm_82.c:5037](../../../../../deps/snesrev-sm/src/sm_82.c:5037) | addressless | `f6a7b1c29a65aa59f583f0e085bba1b4aa6f84fedac1f280597114614b736f21` |
| `OptionsMenu_AddToVramQueue` | [src/sm_82.c:4533](../../../../../deps/snesrev-sm/src/sm_82.c:4533) | 0x82ECFF | `1a0d4a4d514d71070bcbbd2a2cbc3947c2e3d9e21c9038d7ae3d54110c110d88` |

## Dependencies

- 18-exercise-contract-supplement
- 18-ui-transaction-adapter
- 18-options-menu-objects
- 18-audio-service-and-audio03
- 12-controller-binding-poll-contract
- 19-save-options-transaction
- 22-23-NMI-DMA-decompression-contract

Parent must replace dependency descriptions with exact completed local task paths, current input/dependency evidence hashes and supported observed-root receipts before native admission. Historical bounded leaves remain historical until current-source authentication.

## Proposed verification commands

All new targets/manifests/adapters here are unimplemented, and all commands are unrun. Review must register real adapter support and require nonzero executed tests/verification receipts before accepting these gates.

Submission, at most 60 seconds:

```sh
cargo test -p port --locked --offline --test options_menu --test options_menu_objects --test audio
```

Merged gate, at most 3600 seconds:

```sh
cargo run -p pipeline --locked --offline --bin sm-pipeline -- family-verify --manifest tools/port/options-menu.toml --adapter options-menu --output out/port/options-menu --case-jobs 4
```

Exact four-subject material Live replay and independent original-ROM argv/timeouts are a required infrastructure/research deliverable before implementation scope publication. No invented current live flag or adapter string is admitted. Native scope argv must not override source/checkout/root/changed-since; the integrator separately records the exact baseline and all-affected selection on frozen source with workers 4.

## Acceptance and controls

- 29 routines own real GameState 2 root and all 13 screen states, item/special/controller selections and object-update/draw/VRAM queue order.
- Seven canonical eproj_F binding slots, uniqueness swap/default mapping, Japanese/special toggles, scroll/dissolve and file/start state changes.
- Actual MMIO/DMA/decompression/NMI/SRAM/audio/controller services preserve effects and failures; EnableNMI is external hardware ownership.
- Pinned zero placeholders/initializer defects are explicitly reviewed domains and original differences; avoid undefined-behavior fixtures.
- Exact current-source actual-C full WRAM/raw return/ordered effect parity; independent semantic negative controls.
- Supported directly observed complete root/selected-handler/assets provenance with source/build/binary/ROM/movie/readers sealed.
- Whole-request preflight and canonical-state commit; unsupported domain refuses unchanged and fallback is visible; first/late service failure never retries native root.
- Four independent material C0/C1/mixed0/mixed1 subjects, actual playback/menu resources and separate original-ROM route evidence.
- Integrator alone wires shared lib/state/ROM/callback/ABI/catalog, freezes merged source, verifies all affected families and publishes independent stages.
- Both scoped cleanup passes, retained source/readers/store/private inputs, retired implementer worktree and post-cleanup authentication before completion.

## Limitations and cleanup

- File/file-map game-state roots and full pause orchestration remain successor scopes; shared ABI writes belong to integrator.
- Proposed new module/test/manifest/adapter is unimplemented; all new gate argv are unrun and require separately reviewed adapter registration.
- No current supported exercise, native Scope admission, verified gameplay or original-ROM verdict is asserted.

After any future run, wait for writers, run both scoped project cleanup passes, preserve source/readers/store/private-input dependencies, retire integrated worktrees, and reauthenticate retained receipts. Record measured reclaimed/free bytes including zero. This research created no experiment or run; [cleanup](../cleanup.toml) records that result.

