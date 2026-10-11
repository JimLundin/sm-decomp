# Parallel decomp pipeline investigation

Status: resolved

Integration branch: `decomp-pipeline-performance`.
Worktree: `/tmp/sm-decomp-pipeline`, based on `16f1883`.

Investigate offline verification and live capture/validation throughput, with
the goal of supporting heavily parallel implementation efforts. Measure actual
pipeline paths, identify serialization and invalidation boundaries, and leave
an implementation order with explicit correctness gates. Active uncommitted
port work in `/home/jim/sm-decomp` is outside this study's mutation scope.

Deployment target: several worktrees on one machine first (user clarification).
Use a shared resource budget across worktrees; multi-machine distribution is
outside this first implementation plan.

## Required work

- 01: source review, representative measurements, and recommended architecture.
- Create dependency-ordered implementation tickets from measured findings.

Retain the established full canonical WRAM, return, ordered hardware-effect,
progress, exclusive Rust execution and four independent process checks.
Historical binary/observation authentication is separate from current-source
admission and original-ROM parity. Benchmark runs do not grant new admission.

## Measurement approach

Use the frozen 1,000-frame storage study's exact retained live and reader
binaries for a differential throughput loop. Compare each subject's complete
packed inventory and root to its frozen counterpart. Compare serial shared-store,
parallel shared-store and parallel private-store capture. Run real offline family
verification with one and multiple workers. Record stage timings and cleanup.

## Completion evidence

Investigation complete on `decomp-pipeline-performance` at base `16f1883`.
See [findings.md](findings.md), [source-review.md](source-review.md) and resolved
[ticket 01](issues/01-investigation.md). Nine four-subject live capture experiments,
eight offline family runs, a 475-case narrow compiler experiment and an unrelated
dependency invalidation probe completed. Retained evidence reauthenticated after
scoped cleanup. Implementation is separately specified in open tickets 02–08;
this investigation does not claim those changes are implemented.

## Authorized implementation

User requested implementation on this dedicated worktree. Required tickets 02–08
are now the completion scope; investigation evidence above remains historical.

## Language direction

User requested keeping the new pipeline machinery in Rust. `crates/pipeline`
implements shared worker leases, authenticated cache publication, conservative
module discovery, exact compiler inventories and pinned C translation-unit builds.
Existing Python verification/admission harnesses use adapters to this CLI. Packed
verified inventories and ordinal-remapping imports remain in `crates/evidence`.
This change does not migrate all historical harness code or alter gameplay logic.

## Implementation completion

Tickets 02–08 resolved on `decomp-pipeline-performance`. Rust owns the new
scheduling/cache/dependency/C-build core; evidence storage/import is Rust.
See [implementation.md](implementation.md) for measurements, exact passing
serial/parallel 1000/9000/45000 evidence, 52 harness tests, Rust tests/Clippy,
formatting limitation, cleanup and post-cleanup authentication.
