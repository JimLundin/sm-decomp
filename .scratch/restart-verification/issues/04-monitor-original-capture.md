# Monitor bounded original-ROM DF45 capture

Status: resolved
Blocked by: 03
Triage: ready-for-agent
Owner: session `01a14300-1d4f-77e1-b198-a47e8d902bb7`
Started: 2026-10-16 09:10 UTC

## Scope

Continue the user's monitoring request through the next owned implementation:
restart ticket 07, a bounded original-ROM DF45 shot capture. The implementation
owner remains `01a1414e-7fc1-73c3-969e-a7294824368a`; roadmap owner watches gameplay
scope and continuity owner `01a143c7-78d1-7033-8eff-a361c16993dd` handles recovery
and next-task ownership. This monitor owns only these Markdown notes.

Watch reuse of the existing emulator/assets, bounded capture attempts, actual
observed input/call evidence, honest separation of original observations from
native C/Rust agreement, and retention/cleanup. Avoid duplicate captures,
builds or a new acceptance framework. Scope limits and concrete blockers should
be reported without erasing achieved native gameplay progress.

## Observations

### 09:10 UTC — bounded successor owned and inputs ready

- Scrolling completed at `f39ad13`; coordinator claimed ticket 07 at `a582f59`
  and recorded original runtime/input/isolated command scope at `4735e3d`.
- Owner reports pinned ROM/movie/emulator hashes match and existing runtime
  libraries load the retained emulator. No emulator rebuild is planned.
- Original capture tooling is a separate Rust crate, keeping archive handling
  out of the gameplay pilot build path. This is enabling work, not original
  behavior agreement or a captured shot yet.

### 09:14 UTC — bounded capture tool reaches first build

- The existing emulator/runtime inputs remain reused. The new Rust capture
  crate's first test/build command exited 0 in 26.220 seconds in its private
  worker target; no real original capture result exists yet.
- Ticket bounds are original frame 8,850 and 120 seconds. Inputs preserve the
  authentic prefix then author neutral/shot input; movie frames, actual polls
  and native host steps remain distinct. Observer records real call contexts
  rather than imposing a native/original alignment.

### 09:17 UTC — coordinator service failure, surviving work preserved

- Confirmed coordinator `systemError` / failed turn with
  `LanguageModelService:LlmSocketTimeout`. This is a model-service failure,
  not a failed original capture; no original capture has run yet.
- Roadmap notified the continuity owner, who retains sole responsibility for
  one resume of the existing coordinator. This watch confirmed the error and
  will not send a competing resume. The surviving capture writer retains its
  isolated source and passing first build/tests.

### 09:29 UTC — recovery and first bounded capture result

- Roadmap owner sent the single recovery prompt after the coordinator remained
  stopped; continuity owner confirmed recovery without sending another resume.
  The existing writer and source survived. Coordinator is active again.
- `restart/out/original/capture-01/result.toml` reports `failed` because
  `shot_created` was false. Completion reached frame 8,850 with 76,000 ledger
  events, zero snapshots and zero shot/impact/removal observations. Its
  `original_equivalence` field correctly remains `not-tested`.
- The ledger records controller read `$0040` and held/pressed `$0040` at
  observed frame 8,661 in DF45, game state 8, position (128,72). Thus the
  authored physical X input reached the original controller path; that alone
  did not establish a gameplay shot. Coordinator assigned the concrete input/
  call diagnosis to existing workers. No duplicate run by this monitor.

### 09:32 UTC — next attempt has a changed, source-supported hypothesis

- Coordinator's ticket records that the original Ceres arrival initializes
  front-facing pose 00; source skips firing for poses 00/9B. The next scenario
  uses normal Right input before X and captures input-time WRAM to establish
  the original pose, without RAM injection or longer replay bounds.
- Capture 01 occupies 16,741,415 apparent bytes. Retained failure evidence is
  useful; the next capture addresses a concrete missing observation rather
  than repeating unchanged inputs.
- The planned fixture worker owns only `restart/original/src/fixture.rs`,
  baseline `4b4318b`; root retains CLI/Cargo integration. Roadmap reviewed this
  narrow preparatory adapter against the existing 441-byte source-defined
  projection. No native-host API change or original comparison success has
  been claimed.

### 09:41 UTC — real shot and completed Fire pair precede later observer failure

- Capture 02 ran for 32.623 seconds including preparation and validation. The
  turn reached Fire, but an overly narrow position assertion rejected observed
  X=130. That assertion was corrected from diagnostic state; the movie was
  unchanged for capture 03.
- Capture 03 ran for 38.441 seconds on the same input and created one shot.
  It retained Fire entry event 73917, exit 73924 and return 73925 at frame
  8,661, plus five raw snapshots. A later DrawBomb entry (73975) failed an
  RTS/RTL stack witness. Overall capture remains failed, original equivalence
  not tested, lifecycle completion not demonstrated.
- Sent the coordinator and roadmap owner the concrete reuse opportunity:
  attempt the fixed fixture from this completed Fire pair if eligible, without
  waiting for unrelated Draw observer completion. Preserve full-run failure
  and exact earlier-pair eligibility separately; no blanket pass suggested.
- At 09:42 the coordinator explicitly adopted that reuse and integrated the
  fixture adapter (`60629e6` worker commit). Roadmap agrees the complete eligible
  Fire pair does not depend on whole-capture completion. The full lifecycle
  failure remains retained rather than rewritten as success.

### 09:59 UTC — second coordinator service timeout before first fixture

- The scoped prefix adapter finished at worker commit `06ecb5b`, with focused
  tests against capture 03 and negative controls passing. Coordinator's source
  research identified Draw's return hook two bytes before the actual RTS;
  that correction is deferred, without invalidating the earlier Fire pair.
- From approximately 09:48 the coordinator stopped advancing while still
  reporting active. Continuity owner was notified at 09:55; this monitor did
  not infer failure or restart it from inactivity alone.
- At 09:59 the session reported `systemError` / failed with
  `LanguageModelService:LlmSocketTimeout` (completion epoch 1792144683).
  Notified continuity owner for the single recovery and roadmap to avoid a
  duplicate. Captured state, source and diagnostics remain intact. The first
  actual C/Rust/original fixture has not run; next action is integration of
  the ready prefix adapter and that bounded comparison, not another capture.

### 10:08 UTC — first real comparison passes in under one second

- Coordinator recovered in the same session and integrated the prefix adapter
  as `c3f61fa`. Continuity and roadmap recovery notices arrived concurrently;
  both targeted the same coordinator, and this monitor confirmed it active and
  asked both to stop further recovery messages. No duplicate worker or capture
  was launched by this effort.
- `restart/out/original/fire-01/result.toml` reports `passed`: recovered C and
  Rust agree on all 131,072 WRAM bytes, and both match the original return on
  the predeclared 441 bytes. It uses capture 03's first eligible Fire pair and
  the retained native library. Coordinator measured 0.88 seconds, including
  authentication/validation, with no emulator replay.
- Full original WRAM, original live replay and hardware effect equivalence
  remain explicitly untested. One original sound-request marker and compared
  sound-queue storage are recorded, without audible/native-call-order claims.
- `fire-fault-01/result.toml` reports `mismatch` at WRAM `$0B64`: C/original
  141 versus Rust output 142. Coordinator measured 0.82 seconds. The adapter
  injects this byte/word difference after Rust execution, so this establishes
  output-comparison rejection, not a gameplay-source mutation. Sent a wording
  correction to the owner; no redundant new test requested.

### 10:09 UTC — bounded lifecycle correction also succeeds

- After the first comparison passed, `fac67e6` corrected the DrawBomb hook to
  the actual RTL opcode. Capture 04 completed in 37.918 seconds, with 8,850
  frames, 206 matched call snapshots, no unfinished calls and one observed
  creation, impact and removal. This fourth replay answered the specific
  lifecycle gap; it did not gate the already achieved first-pair comparison.
- `fire-02/result.toml` also passes from that complete capture, with identical
  scope: all C/Rust WRAM agrees, both 441-byte projections match the original.
  The capture itself correctly labels equivalence as untested; the separate
  fixture supplies its narrower agreement result.
- Initial cleanup and deduplication commands returned zero bytes for this
  narrow output format. Worker retirement and final retained-evidence
  authentication remain the integration owner's pending completion work.

## Completion evidence

Implementation ticket 07 resolved at `a987df5`. Capture 04 demonstrates original
shot creation, impact and removal (frame 8,694), with no unfinished calls. Fire
fixtures 01/02 pass the exact original 441-byte projection and full C/Rust WRAM;
fixture 02 took 1.63 seconds including full-capture authentication. Output fault
control rejects the expected mismatch. No whole-lifecycle parity, full original
WRAM, hardware or audible equivalence is claimed. The shared inventory publishes
`restart-first-original-fire-verified-20261016` for Fire only.

Read the owner's completed cleanup receipt at
`/home/jim/sm-decomp-restart/restart/out/original/evidence/cleanup.md` and verified
the two worker worktrees are absent. Both utility passes succeeded; 988,529,538
apparent bytes gross were removed across retired worktrees, disposable targets
and four unsealed rerecording files. Primary 1,052,961,365-byte target remains;
retained original evidence was approximately 288 MB. Owner reports all 953
retained-file hashes and 106 dependency hashes passing after cleanup. No
duplicate authentication or game run was launched by this monitor.

This monitor created only Markdown notes: no execution artifacts, worktrees or
recurring background process; cleanup 0 bytes. Continued user-authorized watch
moves to ticket 05 for the newly claimed Ceres door actor.
