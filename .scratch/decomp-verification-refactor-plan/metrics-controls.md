# Bounded batch measurement controls

Status: control design frozen before comparison; bounded controls and retained
samples completed in [metrics-results.md](metrics-results.md). No full-catalog
speedup or original-ROM claim is made by these controls.

## Workloads and accounting

Use one registered actual-C Offline family with its complete declared Fixture
domain. The small control is a correctness and instrumentation baseline, not a
substitute for the later complete required-catalog comparison.

Record these scenarios separately:

1. Fresh verification with an initially empty private artifact cache. The shared
   worker directory is not an artifact cache and is never cleared.
2. Authentication of the unchanged retained result: three alternating
   sampling-disabled / `SM_PIPELINE_METRICS=enabled` pairs. No new Fixture or
   compiler producer is permitted on these reuse samples.
3. Fresh verification with the same private artifact cache. Describe it as a
   warm namespace; inspect actual cache-hit records before claiming warm builds,
   since generated run paths may still create distinct build identities.
4. Rejected reuse followed by current-source rejection. Retain the failed reuse
   interval separately from the subsequent fresh work.
5. Interrupted batch work. Retain elapsed time, cancellation outcome and any
   admitted writer diagnostics; subsequent work must reacquire capacity.

The comparison for sampling overhead uses the exact same retained evidence and
source/tool/profile identities. It reports all six individual elapsed samples,
their median and range, and enabled-minus-disabled differences. Samples taken
during concurrent host compilation are explicitly contended observations.

The minimum useful change for this small latency workload is the greater of 5%
or 20 milliseconds, and it must also exceed observed sample variation. This is
not a promised speedup or a significance claim from three pairs. A material
regression or inconsistent coverage requires explanation before promotion.

## Bounds and controls

- Initial executable controls: 60 seconds each, declared before launch.
- Cargo compilation: a separate 900-second build budget, one Cargo job,
  locked/offline dependencies and a private target/TMPDIR.
- Four shared process slots through the existing worker directory; one small
  family and one active Fixture pair in the new latency controls.
- Sampling remains opt-in. Always-on monotonic spans and existing owned-child
  exit/`wait4` accounting remain useful without RSS/disk probes.
- Record host load and available memory, exact source/reader/compiler identities,
  profiles, declared/verified routines and cases, cache state and outcomes.
- Record logical file bytes/counts separately from unique allocated blocks.
  Neither is OS physical disk I/O; report unmeasured I/O as unavailable.

Batch sidecar elapsed time begins after CLI startup/tool identification. External
invocation time includes that startup and metrics publication. End-to-end control
time extends through both cleanup passes and independent post-cleanup admission.
Summed compiler/Fixture/process lifetimes can overlap each other and helper
lifetimes, and are never subtracted from or called batch wall time.

## Retention

All samples use a new explicit output directory. Keep complete Fixture evidence,
source snapshots and exact executable readers. After owned writers exit, run both
scoped cleanup passes, remove only the control's private unsealed cache/temporary
repository, and independently authenticate the retained passing verdicts again.
Record reclaimed bytes and remaining filesystem space. Root owns integrated
worktree retirement and the final admission after that retirement.
