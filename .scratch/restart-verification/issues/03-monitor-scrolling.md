# Monitor the room-scrolling second consumer

Status: resolved
Blocked by: 02
Triage: ready-for-agent
Owner: session `01a14300-1d4f-77e1-b198-a47e8d902bb7`
Started: 2026-10-16 08:15 UTC

## Scope

The user requested continued implementation monitoring. Coordinator
`01a1414e-7fc1-73c3-969e-a7294824368a` has separate user authorization to implement
room scrolling as the second consumer of the existing comparison loop.

Watch actual source/interface reuse, declared C/Rust observations and ownership,
measured iteration cost, justified regression selection, integration and cleanup.
Use existing producer results; do not launch duplicate builds, gameplay runs or
verification. Keep original-ROM agreement separate and avoid new acceptance
frameworks. Report concrete drift directly to the existing owner.

The roadmap owner `01a142f2-9750-79f3-9d1a-e34bcc44a445` is also monitoring;
coordination assigns this watch verification economics/coverage/retention while
that owner watches gameplay scope and original-reference feasibility. This
session owns only its `.scratch/restart-verification/` notes.

## Observations

### 08:15 UTC — authorized second consumer starts

- Restart baseline remains clean at `a82fe50`. The first Rust projectile pilot
  and its retained 17.479-second warm-loop measurement remain completed.
- Coordinator is active, inspecting room-scrolling scheduled calls before
  implementation. No new result or performance claim yet.
- Sent monitoring boundaries to implementation and roadmap owners. No new
  agent, build, verification run or source mutation was created by this monitor.

### 08:21 UTC — scope and reuse are concrete

- Ticket 06 owns `DisplayViewablePartOfRoom`, `UpdateScrollVarsUpdateMap` and
  `CalculateLayer2PosAndScrollsWhenScrolling` plus their helpers. Gameplay and
  native-host worktrees have disjoint writable paths.
- The existing `sm-pilot` is gaining a scenario selector; the trace crate remains
  unchanged. One fresh C reference for changed native observations is planned;
  subsequent candidate edits reuse it and the primary incremental cache.
- Acceptance includes material tile preparation/flushes and camera motion,
  both consumers together where admitted, a live fault and measured 30-second
  warm-loop target. No legacy family catalog or general framework was introduced.
- Roadmap owner confirmed the monitoring split. A separate continuity owner
  (`01a143c7-78d1-7033-8eff-a361c16993dd`) handles next-task ownership/continuation;
  this watch sends concrete verification findings to it and the integrator.

### 08:24 UTC — combined consumer scenario selected

- Coordinator selected a saved Ceres prefix through shaft descent, first door
  and the next room's camera motion, with one authored firing edge on the initial
  platform. It will require a material projectile lifecycle alongside scrolling.
- This uses one combined replay for the two consumers; reference/cache/first-
  mismatch code remains shared. Inputs must still be validated by the actual C
  run before the authored scenario can be reported as successful.

### 08:26 UTC — translated subsystem integrated

- Gameplay worker commit `aa0654a` integrated as `706c9a2`. Read its verification
  receipt: nine targeted tests pass in under 0.01 seconds, with 0.44-second test
  build and 0.14-second Clippy. Cases include wrapping/flip behavior, frozen RAM,
  sequential display flushes and owned failure after a service call.
- The coordinator published `translated`. Native bridge implementation and
  actual C/Rust comparison remain pending; no native success is inferred from
  worker unit tests.

### 08:28 UTC — pre-run materiality gap identified

- Coordinator found that Ceres's Mode7 flag can make ordinary row/column routines
  return early. The selected short route might exercise camera registers without
  material tile preparation. It is examining reachable states before claiming
  scenario coverage.
- Relayed this to the roadmap/gameplay-scope monitor to keep guidance disjoint.
  A necessary route extension is not broad-verification drift; any new warm-loop
  measurement must include the actual selected route.

### 08:32 UTC — combined C reference passes

- `out/scroll/runs/reference-01` passes in 11.74619 seconds over 9,380 steps,
  through the nearby stairs room rather than full Ceres escape. It records 50
  columns, 16 rows, horizontal/vertical camera movement and one complete shot.
- Bridge integrated as `e3d344f`; a Rust candidate comparison is next. No
  candidate equality or ownership is inferred from this reference pass.
- Asked the gameplay monitor to reconcile one materiality detail: display has
  17 flushes but the reference reports zero DMA bytes attributed to its entries.
  Later row/column consumption is material; avoid claiming more about initial
  display than the observer establishes. No additional run was requested here.

### 08:36 UTC — both Rust consumers match C

- `out/scroll/runs/rust-01` passes on the first candidate comparison in 12.78989
  seconds over 9,380 steps. Scrolling completions `[1,130,936]` have zero C
  entries/refusals; the projectile also completes its material lifecycle.
- Room-specific postconditions now record 16 corridor columns with 40,064 DMA
  bytes and 16 stair rows with 48,256 bytes during gameplay. Total descriptor
  consumption agrees with C: 50 columns and 16 rows.
- Initial display still records 17 completed synchronous service requests with
  zero transfer payload. Roadmap monitor confirmed counter semantics and sent
  this precise coverage limit to the owner. Later material transfers and native
  service controls must not be presented as live nonempty initial-display proof.
- Forced warm rebuild, documentation reuse, fault control and cleanup are still
  pending. This comparison duration is not yet the full edit-to-result measure.

### 08:47 UTC — session failure identified; recovery coordinated

- The pause after `rust-01` was followed by an explicit `systemError` from
  `wait_threads`: the implementation turn failed with
  `LanguageModelService:LlmSocketTimeout`. This is an agent-service failure,
  not a failed gameplay comparison or broad verification loop.
- Source and passing reference/candidate results remain on disk. Asked the
  continuity owner to send one resume to the existing integrator and notified
  the roadmap owner; this monitor does not send a competing resume or take over
  source. Remaining work is the fixed-host identity detail, existing warm/docs/
  fault/CLI checks and cleanup/handoff, with successful work preserved.

### 08:49 UTC — same session resumed

- Continuity owner sent one resume. Integrator is active again and a new
  `sm-pilot.rs` edit binds the authenticated fixed-host recipe into the reference
  key. Its update preserves the passed 9,380-step result and proceeds to the
  existing warm-loop/fault checks. No source restart or duplicate implementation.
- Recovery confirmation sent to the continuity owner. This monitor launched no
  recovery process or verification run itself.

### 08:51 UTC — warm second-consumer loop meets budget

- Integration committed as `6c095fd`. Reference identity now includes native
  library plus authenticated `library.json`, binding the existing fixed-host
  recipe while excluding candidate gameplay. `reference-02` passes under that
  corrected identity; no universal dependency framework was introduced.
- `dev-01` forces a rebuild by touching `room-scroll/src/scroll.rs` (no semantic
  source change), then runs both consumers against `reference-02`: passed in
  16.62219 seconds, including 0.58015 build, 0.08069 validation and 15.89678 run.
  This meets the existing 30-second target for this measured scenario only.
- Three focused scenario controls passed. Documentation reuse, actual scrolling
  fault, normal CLI and cleanup remain pending.

### 08:52 UTC — reuse and real fault detection pass

- `dev-docs-01` reuses the passing result in 0.20791 seconds with zero gameplay
  processes and unchanged candidate/native/reference identities.
- `fault-01` intentionally fails in 14.61808 seconds at first WRAM divergence,
  step 8,412, from the live scrolling mutation. No reference or comparator was
  altered to create that failure.
- Native service-control receipt is retained separately: real tiny descriptor
  transfers total 18 VRAM bytes; refusal/panic/after-service errors prohibit C
  retry. The receipt explicitly does not claim those injected descriptors as
  live nonempty initial-display coverage. This resolves the reporting concern.
- Normal CLI replay, status publication and cleanup remain with the integrator.

### 08:54 UTC — normal CLI runs both consumers

- `cli-rust-01` passes 9,380 steps in 9.03641 seconds. Scrolling completions
  `[1,130,936]` have zero C entries/refusals; projectile completions are
  `[1,802,971,971]`, with the cinematic C prefix separate. CPU fallback is zero.
- Existing runtime checks are now complete; cleanup/status publication and
  concise handoff remain. Sent this milestone to the continuity owner, which
  handles next-task ownership without duplicate verification work.

### 08:59 UTC — ticket resolved and cleanup complete

- Read resolved ticket 06 and `out/scroll/evidence/cleanup.md`. Both integrated
  worker worktrees are retired; preserved source/binaries/native package and
  results remain. Gross removal: 691,137,947 apparent / 715,116,544 allocated
  bytes, including relocated copies. Primary 720,538,508-byte cache is retained.
- Ordinary/deduplicating project cleanup passes succeeded with zero reclaimable
  bytes. Post-removal native/reference/result authentication reused the passing
  result in 0.530 seconds with zero game processes; final artifact hash check
  passed. This is retention validation, not another gameplay run.
- Handoff explicitly excludes live nonempty initial-display/background-layer
  proof and original-ROM agreement. Final status/documentation commit is pending.

## Completion evidence

Completed handoff: `f39ad13`, implementation `6c095fd`, on
`restart/playable-baseline`. The owner completed ticket 06, published stages,
retired both worktrees and authenticated 299 retained files. Runtime outcomes,
timings and limits are recorded above and in
[SCROLLING.md](/home/jim/sm-decomp-restart/restart/SCROLLING.md).

No broad verification churn was observed. One model-service socket timeout was
recovered in the same session through the continuity owner. The materiality and
fixed-host identity findings were resolved without a new verification framework.
This monitor created only Markdown, with no runs, tools or worktrees requiring
cleanup; monitor bytes reclaimed: 0. Producer cleanup evidence is recorded above.

The coordinator immediately claimed bounded original-ROM capture ticket 07;
the continuing watch is [04](04-monitor-original-capture.md). No recurring monitor
is installed outside this active conversation.
