# Local verification pipeline

Rust owns process leases, authenticated compiler caches, cleanup, ordinary
offline families, dispatcher/callback/PLM verification, fixture generation,
packed live replay, decompressor acceptance and RNG jobs. Several worktrees share one capacity configured by
`SM_PIPELINE_WORKERS` (default 4) under the Git common checkout's `out/pipeline`.
Set the same `SM_PIPELINE_ROOT` to share an explicit directory.

```sh
cargo run -p pipeline --locked --offline -- batch verify --families system audio --jobs 2 --case-jobs 4
cargo run -p pipeline --locked --offline -- batch status
cargo run -p pipeline --locked --offline -- batch catalog
cargo run -p pipeline --locked --offline -- family-verify --manifest tools/port/system.toml --output out/port/system --case-jobs 4
cargo run -p pipeline --locked --offline -- family-authenticate --verdict RUN/verdict.toml
cargo run -p pipeline --locked --offline -- plm-fixtures --output-dir out/port/plm-fixtures
cargo run -p pipeline --locked --offline -- fixtures skree-particles out/port/skree-fixtures.toml
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
observed-projectile domains use the checked-in TOML case matrices as their
single declaration. `fixtures FAMILY OUTPUT.toml` validates and republishes
those matrices, preserving case order, continuations and actual-ROM inputs.

`fixture-sequence` captures actual C output chains for glass (128 frames),
sparks (allocation plus 96 frames) and `n00b-tube` (96 frames). Pass a TOML
request on stdin, for example:

```toml
checkout = "/home/jim/sm-decomp"
kind = "sparks"
source = "/home/jim/sm-decomp/out/port/SPARKS-FRAME-RUN"
output = "out/port/sparks-recaptured.toml"
root = "/home/jim/sm-decomp/out/pipeline"
workers = 4
```

Glass and tube also require `caller_source` naming their completed PLM caller
run. Capture uses a fresh output path and retains before/after WRAM, external
host state, C execution logs and SHA-256 expectations beside the TOML manifest.

Native batch workers own distinct immutable run directories; atomic batch saves
use a per-batch OS lock. Reuse authenticates current dependencies and full retained
outputs. Each case runs bounded Rust before C, then compares complete canonical
WRAM and return values. Frozen live authentication verifies all four subjects
through their retained reader and pointer seals.

Readiness, bounded sessions, pattern proof/promotion, retained demonstrations,
original-prefix diagnostics and storage studies also run natively. Small metadata and configuration use TOML; material states and ordered traces use versioned binary formats.

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
