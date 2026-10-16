# Verification that advances the restart

Date: 2026-10-16
Status: advisory findings delivered to the active owners
Main inspected: `193cad941713b3539d23e08238a79e7fb09dde61`

The useful unit of progress is a complete Rust gameplay behavior integrated
into the playable game. Verification should tell its owner whether that behavior
works, where it first diverges, and what remains foreign. It should not require
reconstructing the project's entire acceptance history during every edit.

The restart already demonstrates native boot, input, Ceres escape and Landing
Site arrival. That baseline still runs recovered C. The active
[projectile pilot](/home/jim/sm-decomp-restart/.scratch/restart-pipeline/issues/05-rust-projectile-pilot.md)
owns the first Rust lifecycle and measured comparison loop; this review grants
no new gameplay stage or acceptance verdict.

## What main teaches, with the history kept straight

- The [October 15 measurement](../decomp-batch-performance-20261015/findings.md)
  selected 160 of 177 families after documentation changes at `5c011eb`.
  Selection did not prove 160 compilations. Current main already delegates
  discovery to a semantic scanner that ignores literal documentation prose:
  [Discovery::source](/home/jim/sm-decomp/crates/pipeline/src/affected.rs:354),
  [attribute handling](/home/jim/sm-decomp/crates/offline-verification/src/source_facts.rs:441).
  Do not repeat the old diagnosis as a current measurement or port another
  dependency-discovery framework to solve the pilot.
- Main still records broad application sources, including cleanup and documents,
  in [HOST_SOURCES](/home/jim/sm-decomp/crates/pipeline/src/source_stamps.rs:5).
  Its [live verifier](/home/jim/sm-decomp/crates/pipeline/src/live.rs:17) uses
  `c-0`, `c-1`, `mixed-0`, `mixed-1` and prerequisite authentication. These
  mechanisms serve the old contract; importing that whole contract would also
  import its cost. Keep independent source identities and reuse immutable
  references in the new edit loop.
- The old [production checkpoint](../translation-scope-execution/production-migration/checkpoint.md)
  explicitly records working ownership/recovery machinery unable to advance
  held gameplay without acceptance adapters. Recover its source and knowledge;
  do not make recreating that machinery a prerequisite to playing Rust code.

See [main-reuse.md](main-reuse.md) for source-level algorithms, counterexamples
and small mechanisms worth selectively adapting. Old acceptance does not transfer
to the restart runtime.

## A bounded working process

| Change or event | Work to perform |
| --- | --- |
| Gameplay edit within the pilot | Incremental candidate build, explicit saved-input scenario, comparison against the existing C trace; focused regression cases when useful. |
| Documentation or presentation text only | Relevant text/build checks, if any; no gameplay process or reference regeneration. |
| Reference C, ROM, host scheduling/input or observation meaning changes | Review the changed dependency, run relevant controls, regenerate affected references. Candidate Rust alone is not a reference dependency. |
| First divergent observation | Stop there; retain its ordinal/component and candidate state. Rerun the needed interval with detailed C/candidate observations if diagnosis requires it. Change the hypothesis before another attempt. |
| Subsystem ready for integration | Fresh declared C/mixed comparison through the playable implementation, material lifecycle/ownership evidence, one compact result and scoped cleanup. Reuse valid independent original observations. |
| Shared scheduler/hardware change or broader gameplay milestone | Select the scenarios whose behavior can change; broaden coverage for identified dependencies and unresolved risks. |

The pilot's existing 30-second warm edit-to-result budget includes build,
validation, gameplay, comparison and retention. It is a measurement target,
not an excuse to weaken comparisons. Report cold preparation separately and
address the measured dominant cost before scaling. The trace worker's
[1.16-second synthetic comparison](/home/jim/sm-decomp-restart/restart/trace/VERIFICATION.md)
is only hashing overhead, not an end-to-end result. No performance measurements
were repeated by this review.

Keep a fixed recorded input sequence; use an adaptive controller only to discover
that sequence. Report gameplay reach, Rust ownership, original-ROM agreement and
feedback cost separately. Test counts, function counts and reaching input EOF
do not establish a new gameplay milestone. A mixed C/Rust run can be useful
progress while still leaving explicit foreign behavior.

Original comparison remains real work, but the user-authorized bounded pilot
does not wait for full TAS synchronization. The roadmap owner's
[feasibility note](../restart-collaboration/original-feasibility.md) identifies
the missing original shot-state/effect capture and causal input mapping. Do not
silently equate native host steps with original movie frames.

## Requested review of the pilot implementation

These observations concern files being actively written on 2026-10-16, not a
frozen or tested final implementation. The implementation owner owns fixes and
their relevant checks. Findings were sent directly, without launching runs.

1. **Candidate/reference separation is present.**
   `sm-pilot::identity` uses the recovered-C pin, native library hash, pristine
   ROM, saved inputs, observation version and scenario. The standalone projectile
   crate is absent. Native Rust candidate edits can therefore reuse C observations.
   Changes to Rust runner observation/input semantics need a corresponding
   observation/scenario version change; the runner source is not automatically
   part of that reference identity.
   Source: [pilot runner](/home/jim/sm-decomp-restart/restart/src/bin/sm-pilot.rs).

2. **There is one bounded preparation-cost concern.**
   Native `recipe_hash` includes complete `lib.rs`, `hooks.rs` and `prepare.rs`
   as well as the C sources. `verify_library` rejects a changed recipe. A Rust
   wrapper or comment edit can therefore require native preparation even when
   the compiled C is unchanged. This does not establish that ordinary projectile
   edits invalidate the trace. Measure it as a follow-up unless it blocks this
   pilot; if needed, split the specific C recipe/host-semantics identities rather
   than introducing generic dependency machinery.
   Source: [native preparation](/home/jim/sm-decomp-restart-pilot-native/restart/native-host/src/prepare.rs).

3. **Enforce the premise behind same-slot drawing evidence.**
   `Lifecycle` tracks movement, impact and removal per slot, but attributes an
   aggregate draw-counter increment to an active slot under the comment that
   the scenario has at most one active shot. Enforce that premise during the
   scenario, or correlate draw evidence directly with slots. This is a small
   clarification of the existing lifecycle requirement, not a new test suite.
   Source: `Lifecycle::observe` in the pilot runner linked above.

   Follow-up observed before handoff: the implementation owner added a sticky
   `overlapping_shots` flag, rejects it in `Lifecycle::complete`, reports it in
   the result and added a focused control. This is source-level adoption;
   this review did not run that control or claim a final pilot pass.

4. **The observation domain is substantial but should be described precisely.**
   The native draft captures SRAM, both PPUs including VRAM/OAM, DMA, CPU,
   APU/DSP/player state and audio queue, separately from full WRAM. It guards
   against ending a step inside the external collision-pointer context.
   Ownership counters are correctly separate from state equality. PPU and
   SpcPlayer pointer-free byte ranges rely on upstream zero initialization;
   this is reasonable for a pinned native build, not a portable snapshot format.
   Sources: [native capture](/home/jim/sm-decomp-restart-pilot-native/restart/native-host/src/pilot.c),
   [PPU reset](/home/jim/sm-decomp/deps/snesrev-sm/src/snes/ppu.c:119),
   [player allocation](/home/jim/sm-decomp/deps/snesrev-sm/src/spc_player.c:1513).

   The inspected effects record contains `WriteReg`, `RtlApuWrite`, `SpawnPLM`
   request/result and four scheduled entry boundaries. It does not contain
   `ReadReg` results or the ordered poll/update/NMI/wait events. The admitted
   candidate Interface only exposes WRAM, ROM and synchronous `SpawnPLM`, so
   this observation is not a request to rebuild a hardware verifier. Document
   the boundary and revisit it when an added scope needs those operations.
   Pixels/audio samples are not directly compared by these trace slices; do not
   label this audiovisual equivalence. Small external values worth including
   are `snes_frame_counter` and `currently_installed_bug_fix_counter`, which
   participate in native frame/ROM-patch behavior.
   Sources: native capture/preparation above and
   [RtlRunFrame](/home/jim/sm-decomp/deps/snesrev-sm/src/sm_rtl.c:394).

## Coordination and completion limits

Implementation owner: `01a1414e-7fc1-73c3-969e-a7294824368a`.
Roadmap/original-feasibility owner: `01a142f2-9750-79f3-9d1a-e34bcc44a445`.
Both received the findings and the historical correction. The roadmap owner
agreed to keep native recipe invalidation a measured follow-up and avoid
duplicate direction. This review neither edits their source nor replaces their
pilot acceptance or shared progress publication.

Only local Markdown was authored. No build, verification run, gameplay capture,
cleanup helper or new worktree was created. There are no owned run intermediates
to remove; reclaimed bytes: 0. Other owners' active worktrees and evidence remain
with them. Final completion and disk observation are in the
[ticket](issues/01-review-and-coordinate.md).
