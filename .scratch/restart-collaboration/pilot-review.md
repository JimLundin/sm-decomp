# First projectile pilot: focused review

Date: 2026-10-16
Status: advisory review delivered; implementation still in progress
Subject: ticket 05 at `c177b4b45d1c46e3ca84661ce62520a309c2ae74`

The ticket's explicit scenario and reusable C trace are aligned with the restart
roadmap. The implementation owner confirmed four entries: `FireUnchargedBeam`,
`HandleProjectile`, `DrawPlayerExplosions2`, and `DrawBombAndProjectileExplosions`.
Including firing closes a gap in the older main-branch scheduled frame scope.
The proposed scenario starts from the native boot-02 prefix at DF45, fires an
ordinary shot and allows its lifecycle to finish. C-owned input, cooldown and
enemy contact remain declared. Escape/Mode7 is outside this pilot.

This reviews the proposed contract and existing baseline source. It is not an
acceptance verdict on the workers' evolving implementation.

## Points sent to the implementation owner

1. **Use one fixed input sequence for C and candidate.** The baseline runner can
   call `Route::buttons` using the candidate's current state. That is useful for
   route discovery, but the resulting saved sequence should drive both comparison
   subjects so an adaptive controller cannot conceal a changed trajectory.
   Source: [runtime input selection](/home/jim/sm-decomp-restart/restart/src/runtime_cli.rs:336).

2. **Observe one material lifecycle.** Correlate spawn, movement, terrain impact,
   drawing and removal for the same projectile slot. A disappearing projectile
   alone could be offscreen cleanup. Count the proposed neutral interval in actual
   updates, or label it as host steps; the native runtime explicitly distinguishes
   those events. Sources: [ticket acceptance](/home/jim/sm-decomp-restart/.scratch/restart-pipeline/issues/05-rust-projectile-pilot.md:29),
   [native counters](/home/jim/sm-decomp-restart/restart/native-host/src/lib.rs:26).

3. **Separate ownership from equality.** The baseline's `c_gameplay_calls` counts
   native scheduler entries, not all internal C calls. The pilot needs evidence
   for each required owned entry and declared foreign services. Those ownership
   values naturally differ between C and mixed execution; enforce them separately
   from gameplay-state/effect equality. Source: [counter contract](/home/jim/sm-decomp-restart/restart/native-host/src/lib.rs:33).

4. **Make host observation coverage explicit.** The trace worker's inspected
   draft hashes supplied WRAM, external state and ordered effects and explicitly
   delegates byte coverage to its caller. The host must supply the complete
   declared domain and consistent observation boundaries in both subjects.
   Trace infrastructure alone cannot prove that domain is complete. Source:
   `/home/jim/sm-decomp-restart-pilot-trace/restart/trace/src/lib.rs`, read-only
   observation of the work-in-progress draft on 2026-10-16.

These points clarify existing ticket requirements; they are not a request for a
new family framework, additional broad replay or repeated passing checks.

## Cleanup and limits

Only Markdown review notes were authored. No build, test, gameplay process,
capture or run output was created. There are no run intermediates to clean;
reclaimed bytes: 0. Existing source/evidence/worktrees were read only.
