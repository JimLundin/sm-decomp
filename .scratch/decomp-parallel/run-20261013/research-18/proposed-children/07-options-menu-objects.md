# Own complete admitted options-object interpreter, allocation and real drawing

Type: task
Status: proposed; coordinator review/publication required
Origin: research 18 exact claim, attempt 1; see [Answer](../Answer.md) and [manifest](../children.toml).
Admission: `translation-scope-v1-after-supported-exercise-and-continuation-review`. No ownership or gameplay acceptance is granted.

The 29 routines include actual DrawSpritemap and the complete admitted interpreter/allocation/prehandler/drawing invocation. Eight even ids 0..14 allocate highest free and iterate descending. A prehandler can change global optionsmenu_index; later timer/continuation operations must use the actual resulting global index. All options arrays alias canonical eproj memory 1A8F..1B3D.

The 154 observed script/descriptor bytes at 82F442..F4DB include the six-byte descriptors F4B8/F4C4/F4CA/F4D0/F4D6. Seal every consumed duration/opcode/operand/prehandler/sprite range for actual observed object domains. Preserve deletion and byte-pointer continuation and actual drawing capacity/off-screen rules.

OptionsInstr_8C79 installs pre-pointer 828C81, absent from CallOptionsPreInstr. Static native bytes show an RTS at that location, but no actual original continuation execution was observed. sub_82F404 is also unregistered. Review the source/ROM domain explicitly; an admitted domain must reject unsupported continuations before effects, or first obtain a reviewed implementation and oracle declaration. Never skip an unsupported active object or invent a no-op to produce apparent parity.

## Exact proposed files and ownership

- `crates/port/src/options_menu_objects.rs`
- `crates/port/tests/options_menu_objects.rs`
- `tools/port/options-menu-objects.toml`

The coordinator owns public lib/state/ROM/SRAM wiring, shared pipeline/ABI/catalog registration, main Git/index, tickets and inventory. Isolated workers are not alone in the codebase; they must retain other workers' changes and use only returned exact writable files/branch/worktree/baseline. These proposals authorize no shared edit.

## Exact routine boundaries

| Routine | Pinned source | Entry | Inherited definition SHA-256 |
| --- | --- | --- | --- |
| `CallOptionsEntryFunc` | [src/sm_82.c:897](../../../../../deps/snesrev-sm/src/sm_82.c:897) | addressless | `d5a40ddc34c97c49cbf5092b0db0af5c29643788fa07289d4ca5654828c7a5fe` |
| `CallOptionsInstr` | [src/sm_82.c:956](../../../../../deps/snesrev-sm/src/sm_82.c:956) | addressless | `0f813ad4fd61b8b0dd182b131574d146c798c172e7877ef24b1229a44c539fc6` |
| `CallOptionsPreInstr` | [src/sm_82.c:944](../../../../../deps/snesrev-sm/src/sm_82.c:944) | addressless | `93d941ba476cc3b0b3cda73b2eaa608ef38572dc048f7983335d4f62a40c113b` |
| `CreateOptionsMenuObject_` | [src/sm_82.c:909](../../../../../deps/snesrev-sm/src/sm_82.c:909) | addressless | `9a88ebe9e28f179e091dde8c2811c545b0dc6f8c324fd5d7f0721ea8e8382a9a` |
| `DeleteAllOptionsMenuObjects_` | [src/sm_82.c:889](../../../../../deps/snesrev-sm/src/sm_82.c:889) | addressless | `cd6d7641f038d2eaf07d29f1ebc1d4ca7e695cfdea7162257f530b5a18017833` |
| `DrawOptionsMenuSpritemaps` | [src/sm_82.c:1030](../../../../../deps/snesrev-sm/src/sm_82.c:1030) | 0x828CA1 | `d82b7785c4d6071f0573949d75ac6e3bed0a4f9e941bb71e2e2acfe94e277888` |
| `DrawSpritemap` | [src/sm_81.c:144](../../../../../deps/snesrev-sm/src/sm_81.c:144) | 0x81879F | `037f27ace841ed04fca1dbf849a969b7a9b74aeee74a9b27aa37b029d8d64c2e` |
| `OptionsInstr_8C64` | [src/sm_82.c:998](../../../../../deps/snesrev-sm/src/sm_82.c:998) | 0x828C64 | `424ae4a0c65f4ae633ddb235b178c15c9c900c37a8317278d6a5f580f51fbf9e` |
| `OptionsInstr_8C79` | [src/sm_82.c:1008](../../../../../deps/snesrev-sm/src/sm_82.c:1008) | 0x828C79 | `e1949ebc97a7dde824000f1014dcfaaa92150fc3e1282fc2aafa71cfea231c85` |
| `OptionsInstr_8C89` | [src/sm_82.c:1017](../../../../../deps/snesrev-sm/src/sm_82.c:1017) | 0x828C89 | `475eea1381547904eadbc3a4d7a95ce8a58bc20a705b8c3be5c7e41ef2f09521` |
| `OptionsInstr_8C93` | [src/sm_82.c:1025](../../../../../deps/snesrev-sm/src/sm_82.c:1025) | 0x828C93 | `cdbbaad2463c652842ef69e7a1800c9fad9fc282cbfa73470360677066d00229` |
| `OptionsInstr_Destroy` | [src/sm_82.c:991](../../../../../deps/snesrev-sm/src/sm_82.c:991) | 0x828C5A | `3dbb689ab7c0e9a6201ec670febe3037624e2a3bf7a90c36c36af019f8c7f06d` |
| `OptionsInstr_Goto` | [src/sm_82.c:1013](../../../../../deps/snesrev-sm/src/sm_82.c:1013) | 0x828C82 | `b98a55f1dd23940d044e3788b36bf1dff8994e2b816918388f8381ee7a54066d` |
| `OptionsInstr_SetPreInstr` | [src/sm_82.c:1003](../../../../../deps/snesrev-sm/src/sm_82.c:1003) | 0x828C6E | `f7079de2f7e5b5e72c8f48a740bdd2794ac6059f03eae30f84713d57ec3d54f6` |
| `OptionsMenuFunc1` | [src/sm_82.c:934](../../../../../deps/snesrev-sm/src/sm_82.c:934) | 0x828C11 | `3fb0b5a653b81c205ded3c99338be668e9290fd8d716dff021a883426733483b` |
| `OptionsMenuFunc2` | [src/sm_82.c:969](../../../../../deps/snesrev-sm/src/sm_82.c:969) | 0x828C2B | `8dc27887d2bd41c1a8ac293dc8cef28518a52473088641d8bbfdb1ec717bedad` |
| `OptionsPreInstr_F2A9` | [src/sm_82.c:4894](../../../../../deps/snesrev-sm/src/sm_82.c:4894) | 0x82F2A9 | `85be7472b64dcc3f3a0e92f43e8a5bba7cf5c63ca6065091b8a3595e450d83ad` |
| `OptionsPreInstr_F376` | [src/sm_82.c:4939](../../../../../deps/snesrev-sm/src/sm_82.c:4939) | 0x82F376 | `3bcba3a6d47a5c5f3c3b3ae4d2edfdf35fdaa54e07ed9eeaa0bd04f2858e4a8c` |
| `OptionsPreInstr_F3A0` | [src/sm_82.c:4947](../../../../../deps/snesrev-sm/src/sm_82.c:4947) | 0x82F3A0 | `3c2625ff4c2004d3d86842c524bd57a9e20799ed400f7efa934067d7b21e4897` |
| `OptionsPreInstr_F3E2` | [src/sm_82.c:4965](../../../../../deps/snesrev-sm/src/sm_82.c:4965) | 0x82F3E2 | `e1ff0fbac6196467d18543d47acfc67211b62e019f298fa925c5b190f73a0848` |
| `OptionsPreInstr_F42C` | [src/sm_82.c:4988](../../../../../deps/snesrev-sm/src/sm_82.c:4988) | 0x82F42C | `b4b03a42efdcaf90d333f2b9165d16e155c109a3a794a998c3881ab912eb0c00` |
| `OptionsPreInstr_nullsub_57` | [src/sm_82.c:930](../../../../../deps/snesrev-sm/src/sm_82.c:930) | 0x828C10 | `bb2bddbe296ed61204fdc11266da81fd1df4d4ca3c521262c3fb12605dc760ab` |
| `sub_82F296` | [src/sm_82.c:4888](../../../../../deps/snesrev-sm/src/sm_82.c:4888) | 0x82F296 | `f7240c8d376ed45555295bb46fdd6e4b4d84c95fbb24b2033f7e92ea819ee0b7` |
| `sub_82F34B` | [src/sm_82.c:4915](../../../../../deps/snesrev-sm/src/sm_82.c:4915) | 0x82F34B | `ec043723997060f09fc1b7f5bd3d7154683e7006f718c0cfdd02373305913ee8` |
| `sub_82F353` | [src/sm_82.c:4919](../../../../../deps/snesrev-sm/src/sm_82.c:4919) | 0x82F353 | `7b3e2322f52c25a7a7fee99af2a653f153638b0093650684ab183e2aacb2f09f` |
| `sub_82F35B` | [src/sm_82.c:4923](../../../../../deps/snesrev-sm/src/sm_82.c:4923) | 0x82F35B | `efcafd362088f42a8d6054d1c2b8b26052de5c44d769f4ee822bb7b8463a4428` |
| `sub_82F363` | [src/sm_82.c:4927](../../../../../deps/snesrev-sm/src/sm_82.c:4927) | 0x82F363 | `62e06e1fad30cd0f6a65b004d8e17b90be6a18aeeb0f6a3bfe5faeacd40f7055` |
| `sub_82F369` | [src/sm_82.c:4932](../../../../../deps/snesrev-sm/src/sm_82.c:4932) | 0x82F369 | `b08631594dba74333317398f38bd956ee727520a8c16c7532b6bfdb3b5b92f54` |
| `sub_82F419` | [src/sm_82.c:4981](../../../../../deps/snesrev-sm/src/sm_82.c:4981) | 0x82F419 | `634e804349c8891dbe9ff8aa5fb3f08b6af77ef84ff9dfddb0ce0d71c3a14f6e` |

## Dependencies

- 18-exercise-contract-supplement
- 18-ui-transaction-adapter
- reviewed8C81-F404-continuation-domain
- current-canonical-WRAM-ROM-render-state

Parent must replace dependency descriptions with exact completed local task paths, current input/dependency evidence hashes and supported observed-root receipts before native admission. Historical bounded leaves remain historical until current-source authentication.

## Proposed verification commands

All new targets/manifests/adapters here are unimplemented, and all commands are unrun. Review must register real adapter support and require nonzero executed tests/verification receipts before accepting these gates.

Submission, at most 60 seconds:

```sh
cargo test -p port --locked --offline --test options_menu_objects
```

Merged gate, at most 3600 seconds:

```sh
cargo run -p pipeline --locked --offline --bin sm-pipeline -- family-verify --manifest tools/port/options-menu-objects.toml --adapter options-menu-objects --output out/port/options-menu-objects --case-jobs 4
```

Exact four-subject material Live replay and independent original-ROM argv/timeouts are a required infrastructure/research deliverable before implementation scope publication. No invented current live flag or adapter string is admitted. Native scope argv must not override source/checkout/root/changed-since; the integrator separately records the exact baseline and all-affected selection on frozen source with workers 4.

## Acceptance and controls

- 29 routines own highest-free even-slot allocation, descending eight-slot iteration,6 entries/6 prehandlers/7 opcodes and DrawSpritemap.
- Use actual options aliases1A8F..1B3D overlapping eproj state, global optionsmenu_index mutation and timer/byte-pointer/deletion continuation.
- Seal descriptor/script154 bytes at 82F442..F4DB plus complete naturally selected drawing/prehandler assets before effects.
- Explicit refusal of unsupported8C81/F404 continuation before effects until reviewed source/ROM contract; do not silently install guessed no-op.
- Exact current-source actual-C full WRAM/raw return/ordered effect parity; independent semantic negative controls.
- Supported directly observed complete root/selected-handler/assets provenance with source/build/binary/ROM/movie/readers sealed.
- Whole-request preflight and canonical-state commit; unsupported domain refuses unchanged and fallback is visible; first/late service failure never retries native root.
- Four independent material C0/C1/mixed0/mixed1 subjects, actual playback/menu resources and separate original-ROM route evidence.
- Integrator alone wires shared lib/state/ROM/callback/ABI/catalog, freezes merged source, verifies all affected families and publishes independent stages.
- Both scoped cleanup passes, retained source/readers/store/private inputs, retired implementer worktree and post-cleanup authentication before completion.

## Limitations and cleanup

- Admitted whole invocation may be bounded by genuine observed scripts; unsupported slot refuses entire root, never skips that slot.
- Proposed new module/test/manifest/adapter is unimplemented; all new gate argv are unrun and require separately reviewed adapter registration.
- No current supported exercise, native Scope admission, verified gameplay or original-ROM verdict is asserted.

After any future run, wait for writers, run both scoped project cleanup passes, preserve source/readers/store/private-input dependencies, retire integrated worktrees, and reauthenticate retained receipts. Record measured reclaimed/free bytes including zero. This research created no experiment or run; [cleanup](../cleanup.toml) records that result.

