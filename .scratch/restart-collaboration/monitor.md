# Watch the first Rust projectile delivery

Status: resolved — watched implementer through completed pilot and cleanup
Started: 2026-10-16
Monitor: `01a142f2-9750-79f3-9d1a-e34bcc44a445`
Implementer: `01a1414e-7fc1-73c3-969e-a7294824368a`

The user explicitly requested continued monitoring while implementation proceeds.
Watch restart ticket 05 through its playable result, measured loop and cleanup,
or until the user redirects the watch. The earlier bounded research handoff is
complete; this is an ongoing watch. Use existing source, commands and retained
results. Launch no duplicate build, gameplay, authentication or verification run.
Send concrete corrective feedback when needed; do not add an approval or acceptance
gate. Preserve the implementer's code, claims, worktrees and run output.

Watch gameplay delivery, repeated failures and changed hypotheses, the amount of
work each edit triggers, lifecycle/ownership honesty, measured feedback cost,
integration continuity and final cleanup. Original-ROM comparison remains a
separate documented gap and does not block this authorized native pilot.

## Observations

- At start, restart HEAD is `ace9cdb`: reference-trace, native-hook and ordinary
  projectile implementations are integrated; runner wiring is in progress.
  The previous implementation turn suffered a model socket timeout. The user
  resumed it with `continue`; the owner is active again. This interruption is
  distinct from verification cost.
- The first reference run failed during setup on the fixed effect-log capacity.
  The implementer retained the failure and is checking event volume with the
  native-host worker. A targeted capacity correction addresses an observed
  blocker. No broad legacy gate or repeated unchanged experiment is observed.
- No successful C/Rust projectile comparison or measured warm loop is established
  at this checkpoint. Source integration is useful progress, not gameplay acceptance.

### Setup trace correction

- The integrated workspace reports 34 passing tests and strict Clippy. The
  initial lint failure was followed by a small runner refactor and a 1.438-second
  successful lint retry; no broad gameplay rerun accompanied that correction.
- Native worker `out/trace-capacity/full-scenario.log` completes 8,754 host steps
  in DF45 with zero CPU fallback. It records a maximum 6,563,600 effect bytes at
  step 8,164, explaining the old 4 MiB overflow. The bounded growable buffer
  retained the stream using an 8 MiB allocation. This is the worker's existing
  native run, not a C/Rust comparison performed by the monitor.
- Sent this measured result to the integrator. Next meaningful evidence is the
  fresh reference and candidate comparison. The current correction is targeted
  to a demonstrated setup failure; no new acceptance gate is requested.

### First native C/Rust gameplay result

- `reference-02` completed but failed the three-shot requirement: the left shot
  occurred during the turn animation. The implementer preserved the criterion,
  added released/aiming steps and versioned the input scenario.
- `reference-03` passes over 8,777 host steps: three shots spawn and clear, two
  complete correlated movement/drawing/wall impact/explosion/removal, and no
  overlapping shots are reported. Original comparison remains not tested.
- `rust-01` refused at step 2,075 in intro state 30/room 0000. Sent a focused
  scope recommendation: retain the C-owned intro and activate required Rust at
  the reached admitted gameplay boundary, preserving full-run comparison.
- `rust-02` then passes against the retained C trace. The implementer reports
  15.5 seconds for the run. The measured incremental-build loop, deliberate
  failure and documentation-only no-run controls are next. No broad translation
  or legacy gate was introduced to handle the intro refusal.

### Measured loop and negative controls

- Inspected `rust-02/result.toml` and `invocation.toml`: 15.5175 seconds, Rust
  activation at step 8,262, completions `[3, 414, 516, 516]`, zero refusal,
  no C entries in the owned interval and no foreign calls. C intro counts remain
  explicit. Native capture source includes framebuffer and PCM in external state.
- `dev-01/invocation.toml`: passed in 17.478974399 seconds including a forced
  incremental candidate rebuild of 0.468641255 seconds. This is one measured
  pilot loop, not a timing guarantee for every future edit.
- `dev-readme-01/invocation.toml`: same candidate/reference identity, reused
  the passing result in 0.209440383 seconds with `game_processes = 0`.
- `fault-01/mismatch.txt`: deliberate real projectile fault rejects at the
  first observed WRAM divergence, step 8,390. Failed invocation and raw diagnostic
  state remain retained. Original-ROM comparison remains untested.
- Sent these results to the implementer. Continue monitoring the scoped final
  checks, runnable handoff and cleanup; the pilot has now demonstrated gameplay
  and measured feedback rather than merely adding verification infrastructure.

### Durable source and cleanup

- The normal CLI run `cli-rust-01` completes the saved scenario in 8.0854 seconds
  with matching final WRAM/PNG and the same Rust completion counts. The integrator
  ran the focused final checks; the monitor did not repeat them.
- After screenshot inspection at 05:38 UTC, the session showed roughly seven
  minutes without visible command/file/run progress while still reporting active.
  No build/gameplay process was running. Sent recovery/checkpoint guidance;
  activity resumed and the owner reported no engineering blocker. This delay
  is not attributed to verification cost.
- Source checkpoint `010ab94` is committed and the restart worktree was observed
  clean. The implementer is preserving exact source/evidence and the primary
  reusable Cargo cache, and removing integrated worker worktrees/disposable builds.
  Cleanup and final publication are still pending at this observation.

## Cleanup

Monitor creates only Markdown notes and performs read-only observation. No owned
run intermediates exist; implementation cleanup stays with the integration owner.

## Completion evidence

Completed 2026-10-16 after the implementer finished and its thread was observed
idle with a completed final turn. Final restart handoff is `a82fe50`; gameplay
implementation is `010ab94`. The restart worktree is clean and ticket 05 resolved.

Read the producer's completed ticket, retained commands/results, cleanup logs
and final handoff. Three worker worktrees were removed, totaling 815,327,837
apparent bytes gross. The 520,106,092-byte primary Cargo cache is deliberately
retained for reuse and is not an evidence dependency. Post-cleanup checks report
all 405 retained artifact hashes passing, prepared-native cache authentication,
and valid ROM/archive inputs. Runnable artifacts are under `restart/out/pilot/bin`;
commands and limitations are in [PILOT.md](/home/jim/sm-decomp-restart/restart/PILOT.md).
Separate translation, verification, integration and live-exercise observations
are published. Original-ROM shot comparison remains explicitly untested.

The observed pilot delivered material Rust gameplay and a 17.48-second measured
warm loop, with zero gameplay processes for a README-only edit and a rejecting
deliberate-fault control. Failures led to specific corrections: bounded trace
capacity, input timing during turns and Rust activation scope. No broad legacy
verification churn was observed. One session delay after screenshot inspection
was recorded separately from engineering/verification cost.

Monitor cleanup: no build, test, replay, authentication, worktree or run output
was created; only Markdown notes and read-only observations, including one saved
image inspection. Reclaimed bytes: 0. Final available disk observed:
107,837,722,624 bytes. This watch ends at the implementer's completed bounded
pilot; it is not a background scheduled monitor for future tasks.
