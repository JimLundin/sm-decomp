# Monitor the Rust projectile pilot

Status: resolved
Blocked by: 01
Triage: ready-for-agent
Owner: session `01a14300-1d4f-77e1-b198-a47e8d902bb7`
Started: 2026-10-16 05:21 UTC

## Scope

The user requested monitoring of the existing implementation. Watch coordinator
`01a1414e-7fc1-73c3-969e-a7294824368a` and restart ticket 05 through its completed
pilot handoff, or report a concrete blocker. Use existing source, results and
thread updates. Send actionable concerns directly to the owner, as authorized
by the user's collaboration and monitoring requests.

Watch for material Rust spawn/movement/impact/drawing/removal, C/mixed agreement,
required failure controls, reference reuse, full warm-loop timing, integration
and cleanup. Keep original-ROM agreement independent. Flag repeated unchanged
failures, unrelated broad verification, new general infrastructure without a
measured blocker, and idle executable work. These are observations of the
existing ticket, not added acceptance gates.

This monitor owns only its Markdown notes. It does not duplicate tests, builds,
captures, implementation, shared progress publication or cleanup of another
owner's active files. No recurring/background monitor is installed.

## Observations

### 05:21 UTC — integration underway; first C reference fails explicitly

- Restart HEAD `ace9cdb` includes projectile gameplay, native hooks and compact
  traces. The coordinator is active; runner integration remains dirty/owned.
- Read `restart/out/pilot/runs/reference-01/invocation.toml` and `stderr.log`:
  the first C reference stopped after 8.90 seconds with `native effect trace
  capacity exceeded`. No final reference/pilot success is claimed.
- Coordinator reports examining event volume and the fixed-capacity limit.
  This is a specific failure with an active diagnosis, not yet a repeated
  verification stall. No unrelated broad run was observed.
- Sent the coordinator the monitoring scope and ownership boundary.

### 05:23 UTC — source integration and bounded overflow diagnosis

- Coordinator committed the runner as `ae94633`; `a4715ff` and `e3b030c`
  bring worker verification/cleanup records into the integration branch.
- Native worker is inspecting room-load event volume, including possible repeated
  polling or duplicate instrumentation. Its stated next experiment preserves
  all ordered effects while measuring the burst. No repeated unchanged run or
  expansion into a general verification framework was observed.

### 05:24 UTC — translated publication; observation contract now explicit

- Read the integrated worker records. Projectile worker reports six focused
  tests passing in under 0.01 seconds; native control/example covers ownership
  refusal, fatal-error behavior and real synchronous PLM calls. These are worker
  controls, not final live pilot acceptance.
- Native README now declares framebuffer/PCM, register reads/results,
  poll/update/NMI/wait events and the frame/bug-fix counters in observations.
  This supersedes the earlier WIP omission findings.
- Coordinator published `translated` and continued integration checks. Its
  workspace test/Clippy command took about 17 seconds and exited nonzero; a
  targeted lint correction followed with a successful 1.44-second Clippy run.
  These are the independent restart workspace's checks, not the legacy family
  catalog. No new live success was reported.

### 05:28 UTC — full native scenario clears capacity failure

- Native worker replaced the fixed effect buffer with bounded growth and
  diagnostics, preserving ordered events. Its `off-full` replay of the pilot's
  saved inputs exits 0 in 6.308 seconds, recorded in its
  `out/trace-capacity/full-scenario.log`.
- This is a worker replay with the revised host, not yet a completed coordinator
  reference or C/Rust agreement. Integration owner remains active.
- Actual burst: 6,563,600 bytes at step 8164, with register writes/reads; peak
  allocation 8 MiB. The run reports zero CPU fallback. Its two firing calls are
  fewer than the scenario's three intended shots; sent that concrete observation
  to the coordinator for input/lifecycle inspection before claiming success.

### 05:31 UTC — lifecycle criterion catches an input-authoring gap

- Read coordinator `reference-02/result.toml`: two spawned, impacted, drawn and
  completed shots; no overlap; `milestone-not-reached` because three were required.
  Reference run took 11.18 seconds as reported by the coordinator.
- Coordinator confirmed the planned left shot fell during the turn animation.
  Scenario v2 adds a released interval after turning and an upward aiming interval.
  The three-shot criterion is retained; `reference-03` is running. This is a
  changed, concrete input hypothesis rather than repetition of the failed run.

### 05:33 UTC — first material Rust/native comparison passes

- `reference-03` passes scenario v2: 3 spawned/removed, 2 complete wall-impact
  lifecycles, no overlap. The third shot need not hit terrain; the existing
  criterion requires two full impact lifecycles plus three removals.
- `rust-01` stopped on an unadmitted menu-time invocation at step 2075. The
  coordinator moved Rust activation to actual state 8 / area 6 / room DF45 and
  records the C prelude separately. This makes the ordinary-shot scope explicit.
- Read `rust-02/result.toml` and invocation: passed, 8,777 host steps, activation
  step 8262, Rust completions `[3, 414, 516, 516]`, zero refusals, no C entries
  after activation, 3 spawned/removed, 2 complete drawn impact lifecycles.
  Comparison uses `reference-03`; invocation elapsed 15.5175 seconds.
- This is a real native C/Rust scenario pass. It is not yet the full warm
  edit/build/run measurement, final control package, cleanup or original-ROM
  agreement. The coordinator is continuing those remaining obligations.

### 05:35 UTC — measured candidate loop below budget

- Read `dev-01/invocation.toml`: passed in 17.47897 seconds, including 0.46864
  seconds for a forced incremental candidate rebuild, 0.07603 validation and
  16.89129 run. It reuses the same native library and `reference-03`.
- This meets the ticket's 30-second warm-loop target for this measured scenario;
  it does not predict later subsystems or cold build cost. Deliberate-fault and
  README-only reuse controls are being recorded next.

### 05:36 UTC — cheap documentation path and real failure detection

- `dev-readme-01/invocation.toml` reports `reused`, 0.20944 seconds and zero
  gameplay processes, retaining the same candidate/native/reference identities.
- `fault-01` exits nonzero in 15.26547 seconds. `mismatch.txt` reports first
  WRAM divergence at step 8390 from the deliberately altered projectile hook.
- Sent a completion-focused note to the coordinator: retain these results,
  complete remaining explicit controls/handoff/cleanup, and avoid repeating
  broader passing checks without a relevant change. No broad-verification drift
  has been observed during this watch.

### 05:39 UTC — normal CLI integration passes; cleanup in progress

- `cli-rust-01/result.toml` reports the normal CLI running the same Rust hooks,
  completions `[3,414,516,516]`, zero refusals and zero CPU fallback. The owner
  compared its final WRAM and image with `reference-03`; both `cmp` calls passed.
- Final focused pilot tests, workspace formatting/Clippy and normal CLI build/run
  complete successfully. Cleanup/handoff remain in progress.
- Sent a retention clarification: preserve an intentionally reusable primary
  incremental cache if appropriate, while retiring disposable worker outputs;
  otherwise make the next cold build cost explicit. No new cache system or
  cleanup gate was requested.

### 05:41 UTC — completion-phase check

- No new run, commit or cleanup artifact appeared for several minutes after
  the successful CLI check. Coordinator still reports active, not stopped.
- Asked for the concrete remaining blocker/next action and encouraged a concise
  runnable handoff and scoped cleanup. This is a completion-drift check, not
  evidence of another verification failure or a declared blocked implementation.

### 05:47 UTC — source checkpoint and cache-retention decision

- Source/handoff checkpoint `010ab94` is committed on `restart/playable-baseline`.
  Coordinator explicitly reports no blocker and no further gameplay runs planned.
- The primary `restart/target` will remain an intentionally reusable developer
  cache, not a retained evidence dependency. Integrated worker worktrees/targets
  will be retired after preserving required sources/logs/binaries.
- The observed completion-phase delay ended with a concrete source checkpoint;
  cleanup, stage publication and ticket closure remain underway.

### 05:49 UTC — worker retirement and retained checks observed

- `git worktree list` now contains only the integration worktree for the restart;
  all three integrated pilot worker worktrees are retired.
- Read the owner's post-cleanup hash/input/native-library logs. Retained source
  checks report `OK`; the input doctor passes and the native library is still
  located in the retained pilot preparation. These are retention checks, not
  another gameplay acceptance run.
- Ticket 05 is still claimed pending the final completion/stage record.

## Completion evidence

Completed 2026-10-16 05:51 UTC.

- Pilot handoff `a82fe50`, implementation checkpoint
  `010ab94f147dd50ae9a7319712b4ef70ce1966d9`, branch `restart/playable-baseline`.
  Final Git inspection confirms the handoff and clean worktree. Ticket 05 is
  resolved; runnable commands are in
  [PILOT.md](/home/jim/sm-decomp-restart/restart/PILOT.md).
- Read the producer's completed ticket and retained results. Native C/Rust
  agreement covers 8,777 host steps, with 3 spawned/removed shots and 2 complete
  individually tracked drawn wall-impact lifecycles. Required Rust completions
  are `[3,414,516,516]`, without refusal or C execution after activation.
  The normal CLI uses the same gameplay implementation.
- Measured forced incremental rebuild + candidate comparison: 17.479 seconds.
  README-only reuse: 0.209 seconds, zero game processes. Deliberate gameplay
  fault: rejected at first WRAM divergence, step 8,390. Cold costs and tested
  scope are explicit. Original-ROM agreement remains untested; the missing
  original shot capture and causal mapping are documented separately.
- Shared inventory contains separate verified, integrated and live-exercised
  observations for the restart scope. These do not transfer original production
  acceptance or establish full-game ownership.
- Producer cleanup removed all three integrated worker worktrees, 815,327,837
  apparent bytes gross. Required artifacts were copied first; all 405 retained
  files passed post-cleanup hashing, and retained native-library/input checks
  passed. The 520,106,092-byte primary Cargo cache remains explicitly reusable.
- Monitor launched no build, gameplay, test, capture, cleanup helper or worktree.
  Only its owned Markdown notes changed. Monitor cleanup has no run intermediates;
  reclaimed bytes: 0. Other owners' files and evidence were preserved. Final
  observed available disk: 107,837,714,432 bytes.
- Conclusion: the pilot delivered real Rust gameplay and demonstrated a bounded
  feedback loop. Failures led to specific changed hypotheses, not broad repeated
  verification. Monitoring flagged the missing third firing event, encouraged
  completion after a handoff delay, and preserved the reusable-cache distinction.
  This bounded implementation watch is complete; no recurring monitor is running.
