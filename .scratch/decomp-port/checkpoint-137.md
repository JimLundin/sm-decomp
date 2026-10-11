# Ticket 137 checkpoint

Saved on 2026-10-11, branch `decomp-port-integration`. Implementation checkpoint:
`91c513c` — “Checkpoint native projectile ownership with verified combat replay”.
Ticket 137 remains claimed; the broader decomp goal remains active.

## Resume

1. Inspect the existing full-route runner before starting another replay:
   - Process 747533: `tools/port_live.py`, 129,712 inputs, gzip storage.
   - Run: `out/port/observed-projectile-corrected-live-full/20261011T135146.556016Z`.
   - Tool session 69671, launched through a Python subprocess wrapper.
   - Both C subjects finished. Mixed 0 passed 89,000 inputs with zero CPU
     fallback; mixed 1 was pending at this refresh. Read each subject's
     `replay.stdout` for current progress.
   - Sources are frozen. Let all four subjects finish and retain this evidence.
2. Require a finished `mixed-C-Rust-integration` verdict, matching four-process
   artifact inventories/progress, zero CPU fallback, explicit support-refusal
   counts, and all nine IDs with exclusive nonzero Rust scheduler counts.
   Diagnose failure before making an ownership claim. Scheduler ID counts are
   enabled outer-entry slot appearances; newborn same-frame slots are excluded.
3. Finish all-four-subject capture inspection, then scoped cleanup and retained
   authentication. Record the verdict and results under ticket 137. Commands:

   ```sh
   python3 tools/port_cleanup.py --root out/port/observed-projectile-corrected-live-full/20261011T135146.556016Z --apply
   python3 tools/port_cleanup.py --root out/port/observed-projectile-corrected-live-full/20261011T135146.556016Z --deduplicate --apply
   python3 tools/port_authenticate_live.py out/port/observed-projectile-corrected-live-full/20261011T135146.556016Z/verdict.json --output .scratch/decomp-port/observed-projectile-live-full-authentication.json
   ```

4. Close the newly identified Ceres allocation gap before resolving ticket 137:
   - Current live adapter owns `SpawnEprojWithRoomGfx` (16 full-route Ridley
     calls). Ceres uses three-argument `SpawnEprojWithGfx`, which remains C.
     Current observed offline manifests also omit that native entry point.
   - After this replay finishes, add immutable support selection, synchronous
     host-aware ABI and same-bank routing for `SpawnEprojWithGfx`. Maintained
     `spawn_with_gfx_host` already exists. Preserve native parameter/gfx reads,
     highest-free allocation, partial resets and accepted-error termination.
   - Add actual-C full-WRAM/raw-return/eight-host-word fixtures for all free
     slots and saturation, canonical enemy aliases and gfx/palette OR behavior.
     Add compiled refusal, transactional error and same-bank ABI controls.
   - Add per-entry and per-ID allocation counters/captures. Require a fresh
     four-process full-route replay with 16 Ridley, four Ceres pad and four
     Ceres platform allocations exclusively Rust, plus all nine scheduler IDs.
     Internal impact/split births remain owned by the complete Rust scheduler.
   - Refresh affected source seals, run required checks, clean finished runs,
     authenticate retained evidence, and only then resolve ticket 137.

## Verified

- Native Ridley 93CA, Ceres A2EE/A31B/common A301 and 950C are implemented.
- Immutable bounded support selection and synchronous eight-word host ABI are
  tested. Accepted Rust failures terminate; refusal selects C before mutation.
- 294 port tests, strict all-target Clippy and 14 focused ABI/batch controls pass.
  Earlier broader harness run: 36 tests, one opt-in skip. New Rust files pass
  rustfmt; preexisting projectile/PLM/room formatting discrepancies remain.
- Corrected offline batch:
  `out/port/observed-projectile-corrected-final/batch.json`.
  Six families / 12,094 actual-C cases authenticate after cleanup.
  Record: `observed-projectile-corrected-offline-authentication.json`.
- 45,000-input live verdict:
  `out/port/observed-projectile-corrected-live-45000/20261011T135145.693993Z/verdict.json`.
  Four subjects match 607 artifacts. Each mixed subject executes eight Rust
  RoomGfx allocations and 31,305 Rust scheduler calls, six observed IDs
  exclusively Rust, with zero support refusals and CPU fallback.
  Post-cleanup authentication passes; dedup reclaimed 377,696,038 bytes.
  Capture record: `observed-projectile-live-45000-inspection.json`.
- Full-route partial inspection:
  `observed-projectile-live-full-inspection.json`. Later 965E/9688/9696 first-two
  WRAM/host captures match both C baselines and mixed 0. Material observations
  include impact point (22,120) and newborn lower-slot vertical splits at
  (22,106)/(22,134). Mixed 1 comparison and final verdict remain pending.

This establishes bounded projectile behavior. Ceres initializer implementation
and scheduler evidence do not establish exclusive live Ceres allocation.
Whole-game ownership and original-ROM movie parity remain outstanding.

## Diagnostic context

Full C captures exposed support refusal at x=41/37/32 in the second orientation.
Direct Rust dispatch already matched C. The corrected envelope validates wrapped
coordinate origins and contiguous scan indices, retaining terrain extent/cycle
checks. Eight ownership tests pass; all six later captures admit. Original
refusal/source hash and corrected comparisons are retained in
`observed-projectile-full-capture-preflight.json`.

Earlier 45,000-input runner exited 143 before mixed gameplay while waiting for
the shared-store lock; its packed C baselines are authenticated diagnostics.
The old full runner was stopped after its complete C baseline was durably
packed. That baseline authenticates and corroborates sampled later captures.
Rejected diagnostic verdicts remain retained; their staging/targets were cleaned.

## Cleanup and workspace

Completed-run cleanup is recorded in `observed-projectile-cleanup.json` and
`observed-projectile-build-cleanup.json`. Corrected batch dedup reclaimed
2,241,724,416 bytes. The current full-route cleanup is deferred because processes
still write its subjects; run the scoped commands above after they finish.
The handoff refresh performs read-only inspection without builds/extractions;
its temporary inspection script was removed (4,316 bytes reclaimed). Remaining
free space at inspection: 257,736,871,936 bytes.

Preserve shared `out/evidence`, packed pointers/readers, sealed source/binaries,
ROM/native scripts, host sidecars and diagnostic verdicts. No implementer
worktrees were created. Preserve unrelated `.scratch/workspace-cleanup/`,
`/tmp/sm-decomp-pipeline`, and existing trace/review worktrees.
