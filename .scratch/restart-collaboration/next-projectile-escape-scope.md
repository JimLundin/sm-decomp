# Successor: ordinary projectile updates during Ceres escape

Status: bounded research complete; advisory candidate for integrator selection
Date: 2026-10-16
Research owners: `/root/transition_boundary` (source closure), `/root` (host/runner)
Writable ownership: this file only; released at handoff

Recommend extending the **complete ordinary-projectile update boundary** through
the Ceres high-bit domain, including transformed trail placement. Retain the
affected player/explosion drawing exclusions and the existing Fire admission
until separately supported. This is a concrete update-and-trail extension,
not removal of the shared guard or a claim of a fully Rust-rendered escape shot.

## Sources and current admission

Recovered C pin: `578f90b3cc49557bb70060ad033bb90b8cf8ac50`. C citations below
resolve beneath:

`/home/jim/sm-decomp-restart/restart/out/scroll/native/2c9c9a3cf51a059220afac1ee04727c956e45d067e10e17405bf3adbd268fc7f/source/src`

Restart projectile source was last changed by
`ace9cdb3e0b8bcf5c227a4c8d3f317d273730557`; its inspected files were clean.
[lib.rs](/home/jim/sm-decomp-restart/restart/projectile/src/lib.rs:36) currently
uses one admission predicate for all four entries. It rejects equipped beams,
Hyper Beam, `ceres_status & 8000`, crash echoes, active bomb slots, and active
slots outside `(type,preinstruction) = (8000,AEF3)` or `(8700,B169)`
(`36–56`). Admission precedes mutation; subsequent failures are errors rather
than fallback (`59–73`). Preserve those inventory/error rules.

The C selector is specifically `ceres_status` at WRAM `093F`, not merely the
PPU Mode7 setting or `irq_enable_mode7`. `CeresDoor_Func_6b` publishes `8000`
when Ceres status is nonzero (`sm_a6.c:4602–4605`). Do not substitute a different
graphics flag for this branch condition.

## Exact defined update extension

The scheduled root remains `HandleProjectile` (`sm_90.c:2650–2665`): traverse
all ten slots, dispatch each active preinstruction, execute its instructions,
and honor the shared projectile index. The admitted ordinary lifetime is already
implemented by restart `core.rs:262–316`, including flight, collision, deletion,
instruction progression and impact-explosion no-op preinstruction.

The selected closure must explicitly include these helpers:

| Symbol | Source and responsibility |
| --- | --- |
| `ProjPreInstr_Beam_NoWaveBeam` | `sm_90.c:2683–2701`: four-tick trail allocation, acceleration, directional collision and offscreen deletion. Existing restart `core.rs:262–291` supplies this behavior. |
| `SpawnProjectileTrail` | `sm_90.c:3161`, `90:B657`: existing allocation boundary; restart `trails.rs:157–170` allocates the highest free left-trail slot and initializes both sides. Preserve this complete call. |
| `ProjectileTrail_Func5` | `sm_9b.c:76–104`, `9B:A3CC`: the missing high-bit placement branch. Obtain the instruction-dependent offset, transform the projectile point, add layer position back, then apply signed left/right trail offsets. |
| `CalcExplosion_Mode7` | `sm_8b.c:675–685`, `8B:8AD9`: shared projectile-to-screen transform, returning `Point16U`. |

Existing instruction-value lookup remains `ProjectileInsts_GetValue`
(`sm_93.c:115–119`). Existing collision/instruction/clear dependencies stay
within the established ordinary-beam closure; this extension does not admit
additional weapons or bomb preinstructions.

`CalcExplosion_Mode7` reads projectile X/Y, layer X/Y and M7 A/B/C/X/Y fields.
It computes signed products through `Smult16x16`, truncates each shifted product
to 16 bits and wraps the additions. It uses M7A again in the Y equation;
substituting M7D changes the reference. Its shown body writes no WRAM or
hardware and returns only coordinates. The existing main Rust implementation
in [samus_projectile_frame.rs:207](/home/jim/sm-decomp/crates/port/src/samus_projectile_frame.rs:207)
implements the products locally with no ROM lookup, effects or host call.
Reuse that arithmetic, rather than introducing a foreign transform service.
`projectile_flight.rs:383` and `:410` also contain a transform and its trail
placement use; that implementation was located, not independently re-audited.

The existing `Host::spawn_plm` contract can therefore remain unchanged
(`restart/projectile/src/lib.rs:30–34`). Collision still uses its synchronous
state exchange; transformed placement itself needs no service. Make admission
entry-specific so **only the fully extended update root** gains the high-bit
domain. Fire and drawing keep their current restrictions in this first scope.

## What actually differs in rendering

Fire's ordinary origin and lifetime setup remain world-coordinate operations
(`sm_90.c:3401–3446`; restart `core.rs:167–239`). `DrawFlareAnimationComponent`
has its own temporary Samus Mode7 transform (`sm_90.c:3650–3682`), but it is a
different drawing boundary, not justification for changing projectile origins.

The two existing draw roots have genuine reference gaps:

- `DrawPlayerExplosions2` transforms the point and uses `pt.y` for clipping,
  but passes uninitialized local `r18` as draw Y in its high-bit branch
  (`sm_93.c:168`, `192–202`). A visible ordinary beam reaches this problem.
- `DrawBombAndProjectileExplosions` calls the transform but discards its return,
  then uses `r18`/`r20` without establishing them for that branch
  (`sm_93.c:239–266`). The admitted ordinary impact type `8700` reaches this
  branch when the high bit is set. Types `0300` and `0500` have distinct paths
  but remain outside current inventory admission.

Main Rust already records these exact exclusions
(`samus_projectile_frame.rs:300–356`, introduced by
`c711faea56f20d42113eb63393cc4a76d4940557`). Its transform support is not proof
of a defined visible-render oracle. Do not silently initialize locals, patch
the reference, or accept compiler-dependent output as intended behavior.
The reference decision for active rendering remains unresolved. Elevated
`sub_9382FD`, crash echoes, charged beams, bombs and other weapons remain outside
this proposed extension.

## Retained evidence and compatible verification

[Ticket 11](/home/jim/sm-decomp-restart/.scratch/restart-pipeline/issues/11-full-ceres-rust-milestone.md:78)
records root-order Fire/update/player-draw/explosion-draw counts from
`restart/out/full-ceres-rust/runs/escape-01/result.toml`:

- Rust completions: `[0,4891,5343,5343]`.
- Refusals: `[0,706,707,707]`; C entries: `[0,2282,3885,737]`.
- No Fire call. The high bit appears in 706 sampled steps 14372–15077.

These are ticket-11 observations. Sampled high-bit
context does not identify every individual call's reason; add per-entry reason
observations before assigning all refusals to that condition. The exclusions
are intentional and the milestone exposed no gameplay failure.

Use the existing `Game` hooks and comparison runner. The unchanged full route
can establish admitted empty/update traversal, continued mixed ownership and
full-route regression agreement after a new named reference scenario. It cannot
prove active transformed trails or Mode7 OAM. Separately require a reached active
ordinary-projectile update case with the high bit set and an actual trail-allocation
tick; compare C/Rust state at update return, including both trail coordinates,
instruction state and flight/collision progression. Keep that bounded state
comparison separate from the unresolved active draw behavior.

The legacy short runner requires Fire and a shot lifecycle; the full saved pads
contain no Fire. Preserve the current short shot regression and give the full
route its own explicit material and ownership conditions.

## Host and runner recommendation

Host/runner source inspected at restart `d82f0bebec56c3f1b9d269b89567bacbb35d62e7`.
The same [installation path](/home/jim/sm-decomp-restart/restart/src/gameplay.rs:11)
serves the CLI and comparison runner. Its native callback already receives full
WRAM, the runtime ROM view and copied synchronous SpawnPLM services.
[hooks.rs:46](/home/jim/sm-decomp-restart/restart/native-host/src/hooks.rs:46)
and `:111–147` preserve the private transaction, exact service state exchange,
unchanged refusal and fatal owned errors. The pure transform needs no new host
capability or nesting rule.

For a full differential scenario, reuse `sm-pilot` and `sm-trace` with a new
explicit scenario identity and the unchanged 16,991 input pads:

- [sm-pilot.rs:250](/home/jim/sm-decomp-restart/restart/src/bin/sm-pilot.rs:250)
  currently selects short input authoring. [scrolling.rs:4](/home/jim/sm-decomp-restart/restart/src/bin/sm-pilot/scrolling.rs:4)
  truncates to 9,380 and adds the shot at 8,380. A full scenario must preserve
  the original pads rather than select either existing authoring branch.
- [sm-pilot.rs:541](/home/jim/sm-decomp-restart/restart/src/bin/sm-pilot.rs:541)
  rejects any projectile refusal, while `:639–645` requires all four completed
  roots to be nonzero and C entries to equal their activation prelude. The
  proposed update-only extension keeps strict post-activation update ownership;
  any permitted draw/Fire refusal must be explicitly attributed to its retained
  entry/domain and counted. Zero Fire is valid for these saved inputs.
- The short material requirements at `:652–675` include a complete shot and
  exactly the original crossings. Reuse the existing full-Ceres route predicate
  for escape and area 0, preserving the separate short material regression.
- [sm-pilot.rs:157](/home/jim/sm-decomp-restart/restart/src/bin/sm-pilot.rs:157)
  already admits the full input extent. Its identity at `:176–189` binds source,
  prepared host, ROM, inputs, schema and named scenario. Capture one current-host
  C reference and reuse it for candidate edits. The old CSV/final-state agreement
  in ticket 11 does not substitute for the full-state/effect reference.

The decisive active update/trail check must observe **the update return** before
either unresolved renderer runs. Use a bounded, well-formed actual-C Fixture
pair through the existing native-control approach, covering a real ordinary
beam, the trail-allocation tick, transformed coordinates and subsequent
flight/collision/instruction state. The current
[pilot-host-control.rs](/home/jim/sm-decomp-restart/restart/native-host/examples/pilot-host-control.rs)
checks host refusal/error/service behavior; it is not already such an active
escape fixture. Extend that bounded diagnostic approach rather than adding a
general snapshot API or a second comparison runner. Fixture state injection
does not grant live-from-reset coverage.

If a live active-shot case is later authored, capture its ordinary C input route
first and identify the call context without forcing RAM or skipping drawing.
Full host-step hashes include downstream render state, so a mismatch caused by
the undefined C draw locals cannot be normalized away or blamed on update math.
Do not declare active escape-lifetime parity from the unchanged no-Fire route,
focused update results, or empty draw traversal alone. A fully Rust-rendered
escape shot remains a separate reference/renderer decision.

Suggested writable split after integrator selection: gameplay owner adapts only
`restart/projectile/**`; native owner handles any bounded update-return fixture
and observation support in `restart/native-host/**`; integrator owns the named
scenario, installation, source freeze, regression, stage publication and cleanup.
This packet assigns no code ownership and adds no gate to active camera work.

Cleanup: read-only source/evidence inspection and this draft only. No builds,
tests, replays, authentication, worktrees or generated intermediates. Reclaimed
bytes: 0. Other agents' files and shared progress were preserved.
