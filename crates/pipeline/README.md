# Local verification pipeline

`pipeline` composes the CLI, Batch, Translation approval, Live replay and cleanup.
`offline-verification` owns complete Offline family verification, recipes and
retained evidence admission, including the explicit Fixture authoring interface.
`decomp-source` owns pinned Recovered C source and inventories. `tool-execution`
owns shared process leases, authenticated compiler caches and bounded input reads.
Several worktrees share one capacity configured by
`SM_PIPELINE_WORKERS` (default 4) under the Git common checkout's `out/pipeline`.
Set the same `SM_PIPELINE_ROOT` to share an explicit directory.

For affected-family explanations and automatic local subagent assignments,
follow [the coordinator workflow](../../docs/agents/assignments.md).
`batch plan --changed-since BASE` is read-only; `assign next` reserves a shared
claim before returning work. Translation research promotes only complete scopes.

Fixture execution overlaps independent chains within that shared capacity.
Continuations run only after their predecessor passes every comparison; failures
leave causal `blocked.toml` records. Verdict case order stays canonical.
Ordinary/projectile families on Linux x86_64 use preloaded subject workers that
fork a fresh child for each fixture. Worker parents never execute gameplay;
unchanged ROM data is loaded once. Callback/PLM families use prompt fresh-process
execution through the same scheduler. `family-verify --execution fresh` selects
fresh processes for comparison; `--execution fork` explicitly requires a supported
ordinary family. The default `auto` selects the supported path.

Per-case state, returns, host words, callback captures, stdout/stderr and command
timings remain sealed. Fixture metrics are aggregated in `family.metrics.toml`;
fork receipts retain per-child CPU and reaping details. Aggregate helper CPU
includes its children; fixture identity records mark CPU unavailable to avoid
counting it twice. RSS sampling can count shared fork pages more than once and
miss brief child peaks. Compiler/general `exec` commands keep their own receipts.
Execution source and worker artifacts are part of family admission. Only the
current indexed evidence format is supported. Existing sealed artifacts are
retained without rewriting; moved source inputs can require fresh verification.

```sh
cargo run -p pipeline --locked --offline -- batch verify --families system audio --jobs 2 --case-jobs 4
cargo run -p pipeline --locked --offline -- batch status
cargo run -p pipeline --locked --offline -- batch catalog
cargo run -p pipeline --locked --offline -- family-verify --manifest tools/port/system.smf --output out/port/system --case-jobs 4
cargo run -p pipeline --locked --offline -- family-authenticate --verdict RUN/verdict.toml
cargo run -p pipeline --locked --offline -- plm-fixtures --output-dir out/port/plm-fixtures
cargo run -p pipeline --locked --offline -- fixtures skree-particles out/port/skree-fixtures.smf
cargo run -p pipeline --locked --offline -- batch replay --batch BATCH/batch.toml --frames 1000
cargo run -p pipeline --locked --offline -- routine-live --accepted ACCEPTANCE/decomp-acceptance.toml --output out/decomp/live
cargo run -p pipeline --locked --offline -- decomp-jobs --accepted ACCEPTANCE/decomp-acceptance.toml --output out/decomp/jobs
cargo run -p pipeline --locked --offline -- lsnes-replay movies/super-metroid-any-3653.lsmv --frames 390 --out out/sync/lsnes.csv
cargo run -p pipeline --locked --offline -- batch verify --families plm-instructions --case-jobs 4
cargo run -p pipeline --locked --offline -- live-authenticate RUN/verdict.toml
cargo run -p pipeline --locked --offline -- cleanup --root out/port --apply
cargo run -p pipeline --locked --offline -- cleanup --root out/port --deduplicate --apply
```

Newer blue-ring, bomb, glass, spark, tube, projectile contact/rendering and
observed-projectile domains use the checked-in binary case matrices as their
single declaration. `fixtures FAMILY OUTPUT.smf` validates and republishes
those matrices, preserving case order, continuations and actual-ROM inputs.

`fixture-sequence` captures actual C output chains for glass (128 frames),
sparks (allocation plus 96 frames) and `n00b-tube` (96 frames). Pass a TOML
request on stdin, for example:

```toml
checkout = "/home/jim/sm-decomp"
kind = "sparks"
source = "/home/jim/sm-decomp/out/port/SPARKS-FRAME-RUN"
output = "out/port/sparks-recaptured.smf"
root = "/home/jim/sm-decomp/out/pipeline"
workers = 4
```

Glass and tube also require `caller_source` naming their completed PLM caller
run. Capture uses a fresh output path and retains before/after WRAM, external
host state, C execution logs and SHA-256 expectations beside the binary manifest.

Native batch workers own distinct immutable run directories; atomic batch saves
use a per-batch OS lock. Reuse authenticates current dependencies and full retained
outputs. Each case runs bounded Rust before C, then compares complete canonical
WRAM and return values. Frozen live authentication verifies all four subjects
through their retained reader and pointer seals.

Readiness, bounded sessions, pattern proof/promotion, retained demonstrations,
original-prefix diagnostics and storage studies also run natively. Offline family
fixtures use the indexed binary `.smf` format; see
[inspection and editing](../../docs/FIXTURE_FORMAT.md). Small metadata and
configuration use TOML; material states and ordered traces use versioned binary formats.

Commands without options accept a complete TOML document on stdin, terminated
by EOF, and print a TOML document. The interactive `cache` command reads a
TOML request and emits TOML replies prefixed by a four-byte little-endian byte
length. A cache miss names the pending artifact; writing `commit\n` publishes
it only after source identities are checked again. `lease ROOT WORKERS` emits
a length-prefixed TOML receipt and holds capacity until stdin closes.
Historical sealed evidence keeps its original format. JSON ingestion is limited to external reference corpora.
The no-Python integration test checks maintained source, recipes and CI.
Final migration controls and cleanup are tracked in `.scratch/rust-tooling`.

## Controlled pipeline measurements

Normal verification keeps process exit/CPU accounting and elapsed receipts,
with detailed stage timing and periodic RSS/disk scans disabled by default.
Set `SM_PIPELINE_METRICS=enabled` to collect detailed telemetry; use `disabled`
to explicitly suppress it. Benchmark commands always enable detailed telemetry.
The large-workload 2% telemetry overhead target is not yet established, so
routine decomp iterations do not pay its unproved cost.

`sm-pipeline benchmark prepare CONFIG.toml` prepares authenticated native live
seeds at 1,000 and 45,000 inputs, using system, decompressor and RNG. Preparation
is a separate cold/empty-store experiment. `benchmark freeze` seals an existing
pair of seed runs; `benchmark validate` checks the frozen source, predecessors,
compiler/executable identities, native cache seals and shared-store inventories.
`benchmark run` uses three alternating repeats at each extent and capacities
two/four. The two worker roots must already have their matching immutable
`workers/capacity.toml` and empty `build-cache` directories must be absent.

Configuration uses `pipeline-benchmark-v1`: an absolute `pipeline` executable,
`output`, complete `request` for `live-verify`, `capacity_roots` keyed by `2` and
`4`, `cache_seed`, `store_seed`, `seed_verdicts` and frozen identity inventories.
`prepare` fills the initially empty seed verdicts/inventories and source revision.
The host budgets are fixed at 8 GiB aggregate sampled RSS, 64 GiB staging/private
allocation and a 50 GiB free disk reserve. Sources and compilation environment
must stay fixed throughout preparation and the matrix. Each sample independently
copies the same complete native cache/store seed and reads its bytes before
starting the measured boundary. Final cache clones remain with their samples.

Run with host `/proc` visibility; a private PID namespace cannot prove the host
is quiet. The runner records CPU/load/memory/disk observations and process
state/wchan samples. These classify broad activity and waiting; they do not
attribute CPU to text formatting or parsing symbols. Resource or interference
failures, drift, cache misses and live refusals retain diagnostics and exclude
the sample from medians. No runner lease wraps native live orchestration.

The measured boundary includes live generation/builds, four fresh captures,
publication, authentication, cleanup/dedup and retained authentication. Seed
preparation and post-run identity checks are outside it. Receipts, per-stage
overlap summaries and host observations stay outside canonical evidence. Stage
durations are nested/concurrent and must not be added to claim total wall time
or a removable bottleneck share. The final existing 129,712-input projectile
route remains a separate pipeline regression gate. Work status and frozen
contracts are in `.scratch/pipeline-upgrade/`.
