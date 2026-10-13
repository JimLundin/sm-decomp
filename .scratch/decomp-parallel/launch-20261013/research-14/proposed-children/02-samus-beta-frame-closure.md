# Complete the normal Samus beta-frame scope

Status: open
Blocked by: 01
Triage: needs-triage
Type: research

Proposal only; root renumbers dependencies in the shared tracker. Also needs child13's collision/hardware contract and reviewed input/alfa, pose/palette/minimap/gamma/hit dependencies. Consumers07/09 must retain this blocker.

## Question

Which directly exercised, coherent complete normal beta boundary can be promoted with every dynamic dependency ready? Root: `HandleSamusMovementAndPause` → `CallFrameHandlerBeta` → `Samus_FrameHandlerBeta_Func17`. Read [Answer](../Answer.md) and [scope holding](../scope-holding.toml). Do not mutate until actual root exercise and all required reviews exist.

## Work

Reuse existing samus/samus_control/collision/projectile/trail bodies and exact admitted domains. Review missing complete minimap, outer movement/gamma selectors, hit/bomb-jump, block/pose transition, palette, periodic-damage/pause/health services. Include Ceres gamma/timer/Ridley context only with its own reviewed evidence. A scheduled beta call is not all enemy contact; GameState8/alfa/EnemyMain/PLM/Eproj/camera/draw ordering remains shared scheduler authority.

Proposed exclusive implementation files: `crates/port/src/samus_movement_frame.rs`, `crates/port/tests/samus_movement_frame.rs`, `tools/port/samus-movement-frame.toml`. Input/alfa's separate later triplet is samus_input_frame; source ownership cannot overlap. lib/state/ROM, callback/catalog/CLI/live registration and main Git belong to the integrator. Preserve real collision Host mutable displacement, full refreshed WRAM and fatal-after-host/no native retry.

## Answer / Acceptance

Pin natural beta root+selectors, each source/ROM extent and dynamic closure; publish exact1–32 scope only after all review evidence is ready. If closure exceeds32, create dependency-ordered whole-boundary children. Do not omit missing services by marking an opaque gameplay call ready without its contract. Require actual-C full WRAM/raw returns/transient aliases and ordered effects, independent transition-priority/sequence/frozen/health/pause/host-displacement controls, nonzero Rust root live ownership in four independent subjects, matched original-ROM boundaries, visible remaining fallback and input adaptations. Existing projectile/trail acceptance and Ceres drawing refusal remain unchanged.

Proposed tests: `cargo test -p port --locked --offline --test samus_movement_frame`; regressions samus/samus_control/collision and projectile_frame/trails/game_host; reviewed family-verify manifest samus-movement-frame. Initial60s/integration3600s. Root freezes source and runs every affected Offline family and required live/original-ROM gates, then both cleanup passes, retirement/retention/authentication. These are proposed future gates, not executed acceptance.

## Completion evidence

Pending direct root provenance, ready dependencies, exact scope/gates, implementation children and cleanup. Research14 locates boundaries but establishes no new stage.
