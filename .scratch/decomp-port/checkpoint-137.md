# Ticket137 checkpoint

Saved on2026-10-11, branch `decomp-port-integration`. Ticket137 remains claimed.
The broader decomp goal remains active. This checkpoint records verified bounded
projectile ownership; original-ROM movie parity and whole-game ownership remain
outside the evidence established here.

## Resume

1. Inspect the existing full-route runner before starting another replay:
   - Process747533: `tools/port_live.py`, corrected native projectile contract,
    129,712inputs, gzip storage.
   - Run: `out/port/observed-projectile-corrected-live-full/20261011T135146.556016Z`.
   - Tool session69671, launched through a small Python subprocess wrapper.
   - Both C subjects finished. Mixed0 reached15,000inputs at this checkpoint;
     mixed1 has not started. Read `mixed-0/replay.stdout` for current progress.
   - Sources are frozen. Preserve this run and let all four subjects finish.
2. When `verdict.json` exists, require `mixed-C-Rust-integration`, all nine IDs
   with exclusive nonzero Rust scheduler counts, matching state/effects,
   zeroCPUfallback, and explicit support-refusal counts. Diagnose a failed
   verdict before making any ownership claim.
3. Inspect retained per-ID WRAM/host captures for the later965E/9688/9696 path.
   Run scoped ordinary cleanup, deduplication and `port_authenticate_live.py`.
   Record reclaimed bytes, free space, and authentication under ticket137.
4. Add full-route evidence to the corrected batch and ticket137, refresh this
   handoff, and resolve137 only after all its completion criteria are met.

## Verified

- Native Ridley93CA, CeresA2EE/A31B/commonA301, and950C no-op are implemented.
- Immutable support selection and synchronous eight-word host ABI are tested.
  Accepted Rust failures terminate; refusal selects C before mutation.
-294porttests, strict all-target Clippy,14focusedABI/batchcontrols pass.
  Earlier broader harness run:36tests, one opt-in skip. New Rust files pass
  rustfmt; existing projectile/PLM/room formatting discrepancies remain.
- Corrected final offline batch:
  `out/port/observed-projectile-corrected-final/batch.json`.
  All six families/12,094actual-C cases authenticate after cleanup.
  Authentication: `observed-projectile-corrected-offline-authentication.json`.
-45,000live verdict:
  `out/port/observed-projectile-corrected-live-45000/20261011T135145.693993Z/verdict.json`.
  Four subjects match607artifacts. Per mixedsubject:8Rustallocations,
 31,305Rustscheduler calls, six observed IDs exclusively Rust, zero refusals
  andCPUfallback. Cleanup reclaimed377,696,038bytes; authentication passes.
  Capture inspection: `observed-projectile-live-45000-inspection.json`.

## Diagnostic context

Full C captures revealed support refusal at x41/37/32 in the second orientation.
Direct Rust dispatch already matched C. The corrected envelope validates native
wrapped coordinate origins and contiguous scan indices, retaining terrain
extent/cycle checks. Eight ownership tests pass; all six later captures now
admit. Original refusal and corrected hashes remain in
`observed-projectile-full-capture-preflight.json`.

Earlier45000runner exited143 before mixed gameplay while waiting the shared
store lock; its two packed C baselines are authenticated diagnostics. The old
full runner was stopped after its complete C baseline was durably packed.
That baseline authenticates and corroborates every sampled later capture.
Their rejected verdicts remain retained; staging/targets were cleaned.

## Cleanup and workspace

Completed-run cleanup is recorded in `observed-projectile-cleanup.json` and
`observed-projectile-build-cleanup.json`. Corrected batch dedup reclaimed
2,241,724,416bytes. Running full-route cleanup is pending its completion.
Preserve shared `out/evidence`, packed subject pointers, retained readers,
sealed binaries/sources/native ROM scripts, and diagnostic verdicts.

No implementer worktrees were created. Preserve unrelated
`.scratch/workspace-cleanup/`, `/tmp/sm-decomp-pipeline`, and the existing
trace/review worktrees. The pipeline effort has its own branch and processes.
