# Own complete equipment rendering, wireframe queue and menu spritemap effects

Type: task
Status: proposed; coordinator review/publication required
Origin: research 18 exact claim, attempt 1; see [Answer](../Answer.md) and [manifest](../children.toml).
Admission: `translation-scope-v1-after-supported-exercise-and-adapter`. No ownership or gameplay acceptance is granted.

This scope owns the actual output routines required by equipment categories. It does not own MainPauseRoutine or map restoration. The real DrawMenuSpritemap is included as a coherent render dependency instead of a new generic drawing callback.

Bind sprite table $82C569, selected spritemap record streams, selector animation table/state, reserve amount/palette/tank assets, wireframe pointer/bit-selection tile resources and full VRAM queue entries. Preflight selected lengths and queue extents. Preserve OAM wrap, extension bits, coordinates/tile attributes and canonical player/reserve input; do not impose new clipping.

Compare complete OAM/palette/WRAM/VRAM queue output and ordered DMA/NMI completion in material replay, including selector animation, empty/full/partial reserves, wireframe equipment combinations and independent wrong-table/length/cursor/palette/queue mutations.

## Exact proposed files and ownership

- `crates/port/src/pause_equipment_render.rs`
- `crates/port/tests/pause_equipment_render.rs`
- `tools/port/pause-equipment-render.toml`

The coordinator owns public lib/state/ROM/SRAM wiring, shared pipeline/ABI/catalog registration, main Git/index, tickets and inventory. Isolated workers are not alone in the codebase; they must retain other workers' changes and use only returned exact writable files/branch/worktree/baseline. These proposals authorize no shared edit.

## Exact routine boundaries

| Routine | Pinned source | Entry | Inherited definition SHA-256 |
| --- | --- | --- | --- |
| `DrawMenuSpritemap` | [src/sm_81.c:188](../../../../../deps/snesrev-sm/src/sm_81.c:188) | 0x81891F | `754fc667937e9382a0a8522d5a224f834ed5abb4515b9d1ab418d2d8f3abf7df` |
| `DrawPauseScreenSpriteAnim` | [src/sm_82.c:2292](../../../../../deps/snesrev-sm/src/sm_82.c:2292) | 0x82A881 | `fe0e660f06d02949ddda673743eddfa655ac102e290db4fcaa98ff7de8aadaab` |
| `EquipmentScreenDisplayReserveTankAmount` | [src/sm_82.c:2778](../../../../../deps/snesrev-sm/src/sm_82.c:2778) | 0x82B2A2 | `8d5290754c96a335f0871dbe6511973db886e755a6587ca7249e9376d6916a1c` |
| `EquipmentScreenDisplayReserveTankAmount_` | [src/sm_82.c:2789](../../../../../deps/snesrev-sm/src/sm_82.c:2789) | addressless | `1253d62dab82a6d448a57ed89d61c9c406a94c0cf254bcf27338389f3fcafab6` |
| `EquipmentScreenDisplayReserves_PaletteSetup` | [src/sm_82.c:2834](../../../../../deps/snesrev-sm/src/sm_82.c:2834) | 0x82B3F9 | `c4b9ffff23f65c8d3c7f053f0276a9565b17d1444a672d8d6890e0e793517316` |
| `EquipmentScreenDrawItemSelector` | [src/sm_82.c:2770](../../../../../deps/snesrev-sm/src/sm_82.c:2770) | 0x82B267 | `19633dc1379fba36dec4b635247b7afee6aee4a0011dd08468f210ac91887da1` |
| `WriteSamusWireframeTilemap` | [src/sm_82.c:2748](../../../../../deps/snesrev-sm/src/sm_82.c:2748) | 0x82B20C | `8b544e4a5a90e02fbcb2ee1583b25da2f282b09b3a5ff5865b110783990377be` |
| `WriteSamusWireframeTilemapAndQueue` | [src/sm_82.c:2735](../../../../../deps/snesrev-sm/src/sm_82.c:2735) | 0x82B1E0 | `26ead83d2a6d4a895d0960c575a63cbb21679cd990744d0068cb120768f9bc29` |

## Dependencies

- 18-exercise-contract-supplement
- 18-ui-transaction-adapter
- 22-23-full-render-NMI-state
- current-canonical-player-memory

Parent must replace dependency descriptions with exact completed local task paths, current input/dependency evidence hashes and supported observed-root receipts before native admission. Historical bounded leaves remain historical until current-source authentication.

## Proposed verification commands

All new targets/manifests/adapters here are unimplemented, and all commands are unrun. Review must register real adapter support and require nonzero executed tests/verification receipts before accepting these gates.

Submission, at most 60 seconds:

```sh
cargo test -p port --locked --offline --test pause_equipment_render
```

Merged gate, at most 3600 seconds:

```sh
cargo run -p pipeline --locked --offline --bin sm-pipeline -- family-verify --manifest tools/port/pause-equipment-render.toml --adapter pause-equipment-render --output out/port/pause-equipment-render --case-jobs 4
```

Exact four-subject material Live replay and independent original-ROM argv/timeouts are a required infrastructure/research deliverable before implementation scope publication. No invented current live flag or adapter string is admitted. Native scope argv must not override source/checkout/root/changed-since; the integrator separately records the exact baseline and all-affected selection on frozen source with workers 4.

## Acceptance and controls

- All8 routines own selector animation, reserve whole/partial digits and palettes, wireframe tilemap and ordered VRAM queue entries.
- Actual DrawMenuSpritemap processes all selected records and wraps OAM pointer at0 x1 ff, preserving extension bits and coordinates; no new clipping rule.
- Full asset/queue preflight binds naturally selected animation/sprite/wireframe/palette extents and canonical reserve/equipment state.
- Exact current-source actual-C full WRAM/raw return/ordered effect parity; independent semantic negative controls.
- Supported directly observed complete root/selected-handler/assets provenance with source/build/binary/ROM/movie/readers sealed.
- Whole-request preflight and canonical-state commit; unsupported domain refuses unchanged and fallback is visible; first/late service failure never retries native root.
- Four independent material C0/C1/mixed0/mixed1 subjects, actual playback/menu resources and separate original-ROM route evidence.
- Integrator alone wires shared lib/state/ROM/callback/ABI/catalog, freezes merged source, verifies all affected families and publishes independent stages.
- Both scoped cleanup passes, retained source/readers/store/private inputs, retired implementer worktree and post-cleanup authentication before completion.

## Limitations and cleanup

- Equipment category interaction, map rendering and MainPause orchestration belong to separate scopes.
- Proposed new module/test/manifest/adapter is unimplemented; all new gate argv are unrun and require separately reviewed adapter registration.
- No current supported exercise, native Scope admission, verified gameplay or original-ROM verdict is asserted.

After any future run, wait for writers, run both scoped project cleanup passes, preserve source/readers/store/private-input dependencies, retire integrated worktrees, and reauthenticate retained receipts. Record measured reclaimed/free bytes including zero. This research created no experiment or run; [cleanup](../cleanup.toml) records that result.

