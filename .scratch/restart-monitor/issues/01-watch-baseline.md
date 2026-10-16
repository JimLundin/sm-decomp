# Watch the restart through the first playable baseline

Status: resolved
Blocked by: None
Triage: ready-for-agent
Owner: monitor session `01a1415e-bc14-7361-b006-6ef8d1c46f2b`

Follow [the watch scope](../spec.md). User selected continued observation until
the first playable baseline. Raise concrete delays and course corrections in
the monitoring conversation. The user authorized the proposed coordinator
continuation message with "just do it" on 2026-10-16 local time.

## Observations

### 2026-10-15 21:05 UTC — initial checkpoint

- Restart branch began at `193cad9`; plan commit `941b522`.
- Bootstrap commit `58f00e2` was integrated as `f83deb0`. It adds one independent
  Rust workspace and an input doctor. The coordinator reports eight passing
  tests; this monitor has not rerun them.
- Runtime and Interface research are complete. Ticket 04 specifies a real
  recovered-C host, bounded boot/input experiment, shared interactive/headless
  Game and subsequent Ceres route. It is still open at this checkpoint.
- Boot, material controller response, Ceres escape, original-ROM comparison and
  restart Rust gameplay ownership are all unproved. Edit-to-result timing is
  not yet measured.
- No broad legacy verification or repeated verification loop was observed in
  the restart activity inspected. The coordinator is integrating the bootstrap.
- Watch item: ensure startup completion is followed by executable ticket 04;
  do not treat the input doctor as the playable baseline.

### 2026-10-15 21:08 UTC — startup ended before runnable work

- Restart HEAD is `8ceee31`; worktree is clean. Tickets 01–03 are resolved;
  ticket 04 is open and its listed dependencies are satisfied.
- Coordinator session is idle; its final message explicitly reports gameplay
  untested and the runnable C baseline as the next task.
- Alert: implementation has stopped at startup. This is an ownership/continuation
  gap, not an observed verification slowdown. No playable progress is claimed.
- Proposed correction: resume ticket 04 with bounded boot/controller response,
  then playable presentation and Ceres escape, using change-relevant checks.
- Asked the user whether to send that exact direction to the coordinator.
  Monitoring remains active; no cross-session message has been sent.

### 2026-10-15 21:36 UTC — waiting for continuation

- Repeated read-only observations through 21:35 UTC still show the coordinator
  idle, restart HEAD `8ceee31`, a clean worktree and ticket 04 open. No active
  implementation or new verification run is available to assess.
- The requested playable endpoint remains unmet. The watch awaits continuation
  of the implementation session. The user has not yet answered the question
  authorizing the proposed cross-session message, so none was sent.
- Do not interpret elapsed idle time as verification cost or an engineering
  failure. The producer ended at its startup handoff; this is a continuation gap.
- Resume from ticket 04 and the coordinator's latest state, without redoing the
  research or bootstrap tests. No background watcher or scheduled task has been
  installed; this record is the handoff for continued interactive monitoring.

### 2026-10-15 23:25 UTC — authorized continuation sent

- Rechecked the coordinator and worktree: still idle at `8ceee31`, ticket 04 open.
- User explicitly authorized sending the proposed continuation direction.
- Sent the message to coordinator `01a1414e-7fc1-73c3-969e-a7294824368a`; the tool
  returned that thread ID without error. Requested boot/input first, then playable
  presentation and Ceres escape through normal area 0 arrival, with focused
  checks, separate original-ROM reporting and no unrelated legacy verification.
- Monitoring resumed. No duplicate build, test or gameplay run was launched.

### 2026-10-15 23:34 UTC — first native boot observed

- Ticket 04 is claimed by the coordinator. Native host and presentation were
  implemented in isolated worktrees; Ceres route research completed read-only.
- Presentation worker commit `2cb6bed` integrated as `d7ae973`. Worker reports
  real desktop/PulseAudio smoke, 180 presented frames and zero queue resets.
- Read the existing native worker `out/native-host-smoke.log`: actual recovered
  C reaches game state 1 after ordinary boot; real polls show press, held and
  release values. 608 host steps, 458 polls/updates, zero recorded CPU fallback.
  This is boot/controller delivery evidence, not material in-game movement.
- Initial archive metadata/header collision errors were corrected; the successful
  warm smoke invocation took 0.615 seconds including the small Rust rebuild.
  This is one observed command duration, not a general performance guarantee.
- Integration and route driving are in progress. Ceres escape, normal area 0
  arrival and original-ROM agreement remain unproved. No broad legacy batch or
  unrelated refactor was observed. Monitor launched no verification runs.

### 2026-10-15 23:43 UTC — Ceres movement and real presentation

- Read coordinator `restart/out/runs/boot-02/result.toml`: ordinary reset reaches
  Ceres room `df45`, state 8, controller response true, 8,379 steps and 8,162
  polls/updates. Nonzero audio samples; zero recorded CPU fallback/instructions.
- Inspected the existing `boot-02/final.png`: Samus and the Ceres room are visible.
  The initial black frames were fixed by initializing the native rendering flag.
- `presentation-boot-01/result.toml` reports the same successful boot/movement
  domain through real desktop/audio presentation. Its 140-second elapsed time
  corresponds to real-time presentation, not headless feedback cost.
- Route attempts progress through successive rooms. The coordinator reports
  reaching Ridley and triggering the escape sequence normally. Escape and area 0
  arrival remain unproved; latest work addresses a raised doorway on the route.
- Recent incremental builds take 0.1–0.9 seconds and bounded headless route
  attempts roughly 9–21 seconds. Failures retain observations and inputs; changed
  route hypotheses motivate reruns. No broad verification gate observed.

### 2026-10-15 23:44 UTC — Ceres escape and area 0 arrival

- Read `restart/out/runs/ceres-06/result.toml`: passed, Ceres escape true, area 0
  arrival true, state 8, room `91f8`, 16,991 host steps and 16,068 polls/updates.
  Gameplay remains recovered C; recorded CPU fallback/instructions are zero.
- Inspected `ceres-06/final.png`: the ship is visible at the area 0 landing site.
  The producer's headless command completed in 15.258 seconds.
- The requested native gameplay milestone is reached. Original-ROM comparison
  remains explicitly not tested. Final runnable handoff and cleanup are pending.
- Sent the coordinator a completion-focused note: retain the successful route
  and useful failures, finish scoped cleanup and runnable instructions, and keep
  original comparison separate. A saved-input replay may address reproducibility;
  unrelated legacy verification should not delay this handoff.

## Completion evidence

Completed 2026-10-15 23:55 UTC (2026-10-16 local time).

- Integration branch: `restart/playable-baseline`, implementation commit
  `91f84a76e489a40f3e93a974ee6b5421a9f4827e` in `/home/jim/sm-decomp-restart`.
  Final handoff commit `c7e4983`; restart worktree observed clean.
- Read the resolved producer ticket:
  `/home/jim/sm-decomp-restart/.scratch/restart-pipeline/issues/04-runnable-baseline.md`.
  It records ordinary boot/controller response, real desktop/audio presentation,
  Ceres escape and normal area 0 arrival. Gameplay remains recovered C and
  original-ROM comparison remains untested.
- Saved-input run `restart/out/runs/ceres-recorded-final` reproduces the successful
  Ceres run's final memory, observations, inputs, image and result byte for byte.
  Final replay took 19.4035 seconds; 16 ordinary tests plus the relevant worker
  checks passed. No broad legacy verification was launched.
- Runnable instructions: `/home/jim/sm-decomp-restart/restart/README.md`.
  Executable: `restart/out/bin/sm-restart`. Full dependency and evidence locations
  are recorded in the producer ticket and `restart/out/handoff/`.
- Inspected the producer's cleanup record and retained post-cleanup logs. Both
  integrated worker worktrees and disposable Cargo target were removed. Producer
  records 889,340,102 gross apparent bytes removed, plus earlier worker cleanup;
  preserved copies mean this is not a net filesystem-space claim. All 768 retained
  files passed its hash check, and the retained CLI's input check passed.
- Monitor launched no builds, tests, gameplay processes, worktrees or run
  artifacts. Only Markdown notes were created; no monitor run intermediates
  require cleanup and monitor cleanup reclaimed 0 bytes. Final observed available
  disk: 108,472,246,272 bytes.
- Conclusion: the restart delivered a concrete native gameplay milestone, with
  short targeted iterations and bounded failures. The initial continuation gap
  was corrected; no recurrence of the broad-verification stall was observed.
  This watch ends at the requested endpoint; it is not ongoing automation.

## Comments

This is an advisory observation record, not a new verification system or a
condition on merging implementation work.
