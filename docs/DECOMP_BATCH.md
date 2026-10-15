# Running the Rust port checks

Run these commands from the repository root:

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked --offline -- -D warnings
cargo test --workspace --locked --offline
cargo run -p pipeline --locked --offline -- batch run
cargo run -p pipeline --locked --offline -- batch status
```

`check` runs formatting, Rust tests, Clippy and the port harness tests.
`run` verifies the `gameplay` profile and runs four independent C/mixed processes
through 18,000 frames. Jobs run three at a time. The catalog and profiles live in
`tools/port/batch.toml`; callback signatures and live eligibility remain declared
in the individual family manifests. The partial power-bomb entry is excluded
from live installation. A passing replay remains C/Rust integration evidence,
not original-ROM parity or full gameplay ownership.

Use the optimized Cargo `verification` profile for repeated Offline batches:

```sh
cargo run --profile verification -p pipeline --locked --offline --bin sm-pipeline -- batch verify --profile offline-all --jobs 2 --case-jobs 4
```

The first `--profile` selects the controller's build configuration; the second
selects the complete Offline catalog domain. Tool compilation happens once per
source change and is separate from verification time. The `verification` profile
also applies to the separate sandbox and dependencies, keeps line tables in debug
companions, and emits tools under `target/verification/`. Build a matched pair with:

```sh
cargo build --profile verification --locked --offline -p pipeline -p translation-execution --bin sm-pipeline --bin sm-translation-sandbox
```

Managed Translation execution uses the exact tools in its sealed Installation;
see [the production command](TRANSLATION_EXECUTION.md#production-command) for its
installed launcher and ownership. A local Cargo build does not change that
installation. Retain debug companions beside copied tools when retaining a build
for diagnosis.

Batch records and complete per-job stdout/stderr live under ignored
`out/port/batches/`. The first verification creates a batch; later commands use
the latest batch. Interrupted or failed work can be resumed by repeating the
command. Passing evidence is reused only after its retained artifacts, source,
dependencies and manifest are authenticated. Stale families are rebuilt;
rejected evidence never proceeds to replay. Replay itself always starts fresh
processes. Existing failed attempts remain retained.

One command owns each batch record; distinct batch records and worktrees can run
concurrently. Long jobs print progress every 30 seconds, with full output kept
in their log directories.

`sm-pipeline` composes the batch. `offline-verification` owns captured inputs,
complete family execution and admission; `tool-execution` owns shared process
leases and authenticated compiler preparation/artifacts. Worktrees
share a budget and cache under the main checkout's `out/pipeline` (located through
Git's common directory). `SM_PIPELINE_ROOT` selects another shared directory;
`SM_PIPELINE_WORKERS` sets its capacity, defaulting to four or the machine's CPU
count if smaller. Every user of one directory must select the same capacity.
Use a new directory when changing capacity between experiments. OS locks release
when a lease holder exits. Each batch still owns its evidence independently.

`--jobs` controls family scheduling, `--case-jobs` controls isolated fixture pairs,
and `--live-jobs` controls the four fresh live subjects. All compilers, fixture
processes and captures use the same shared worker budget. Within a fixture pair,
Rust admission always runs before C. Packed live subjects capture to private
stores concurrently, then import authenticated observations into the shared
store after the comparison gates pass. `sm-pipeline live-verify --retain-private-stores`
keeps those stores for diagnostics. Native capture uses packed binary records.

An ordinary batch captures one catalog/input context and shares declaration
decoding and compatible compiler preparation across its families. One ready-work
executor serves the admitted family window; waiting chains consume no producer
slot. Helper residency is bounded by the shared worker count, with at most one
Rust and one C helper per executor. A failed comparison or completion publication
stops new admission and settles work already admitted. Canonical case order and
complete comparison requirements are unchanged. Native waves and Original family
ordering retain their existing reservations and execution order.

`work-graph.toml` files show advisory progress. Recovery authenticates the original
requirements, producer operations and verdicts; missing or edited progress files
cannot grant acceptance or request another producer. Set
`SM_PIPELINE_METRICS=enabled` to retain detailed preparation, build, execution and
admission timing. Compare complete wall time separately because those phases can
overlap.

The cache seals compiler identity/options/environment and source contents. It
checks artifact bytes on every hit and rechecks inputs before publication under
a per-key lock. Generated C wrappers are rebuilt and relinked each time; pinned
objects use the compiler's preprocessor dependency inventory. Cargo artifacts
include workspace manifests, lock/config/toolchain and package sources. Cache
entries are disposable build products, retained separately from run cleanup;
they never substitute for a fresh live replay. Do not remove a shared cache while
workers are publishing to it. Per-command timings and cache hit records remain
in evidence directories, including failed builds.

For explicit control:

```sh
# Offline verification of every family, including bounded PLM instructions.
cargo run -p pipeline --locked --offline -- batch verify --profile all

# Verify or retry one family without rebuilding unchanged families.
cargo run -p pipeline --locked --offline -- batch verify --families projectile_flight

# Replay the already verified gameplay profile at a different horizon.
cargo run -p pipeline --locked --offline -- batch replay --frames 9000

# Select a batch explicitly or create one at a chosen path.
cargo run -p pipeline --locked --offline -- batch run --batch out/port/batches/review/batch.toml --jobs 2

# Discover the current offline manifest frontier and run isolated case pairs.
cargo run -p pipeline --locked --offline -- batch verify --profile offline-all --case-jobs 4

# Select families affected by source changes relative to a commit.
cargo run -p pipeline --locked --offline -- batch verify --profile offline-all --changed-since HEAD~1

# Explain the same conservative selection without running fixtures.
cargo run -p pipeline --locked --offline -- batch plan --profile offline-all --changed-since HEAD~1
```

Catalog entries declare `kind`, `manifest` and `live`. Native batches reject
configured `prepare` commands; PLM instructions use built-in Rust preparation.
Discovered manifests are offline only; live adapters require explicit catalog
entries. The `dispatcher-leaves` profile exercises the bank86 dispatcher harness.
The versioned compiler dependency contract seals actual source reads and canonical
memory, allowing unchanged narrow families to survive unrelated gameplay edits.

Selection explains changed inputs and uncertainty, including current Rust module
discovery, untracked files and captured assets. It grants no evidence reuse or
Live admission. Verification still authenticates current source; skipped historical
records remain in the batch. The receipt's `selected` list names invocation work.
Rust dependency facts are syntax-aware: words in documentation comments do not
create input edges. Unresolved macros, generated inputs and external inputs with
insufficient history keep a family affected. A smaller selected set therefore
requires complete dependency evidence; a requested fresh domain still executes
all of its declared Fixture pairs.
For exclusive claims, translation research/promotion and isolated implementation
worktrees, follow [subagent assignments](agents/assignments.md).

Use `--fresh` to force verification rebuilds. `--native-plm` requires passing
`plm-spawn` evidence and actual native setup execution during replay; a full PLM
slot pool or zero PLM calls cannot satisfy that gate. It is optional because the
current movie prefix does not exercise that boundary.

PLM instruction fixtures are generated automatically in the batch's asset
directory. Their bounded interpreter is verified offline and is excluded from
the whole live PLM loop. Individual harness commands remain available for focused
diagnosis; batch orchestration should use the checked-in runner.

Future `sm-pipeline live-verify` replays (including batch replay) use the Rust evidence store
by default. Hardware events stream into compressed binary records as the C host
runs. Snapshots, ROM views and fixture files are packed after each process exits;
the runner authenticates their restored byte hashes before publishing a durable
`evidence.toml` pointer and removing staging files. Four independent subjects
still compare the same exact observation inventories. Identical pages, bursts
and captures share `out/evidence` across runs. Provenance is sealed separately
from observations; storage completeness does not grant gameplay acceptance.

For projectile integration, `sm-pipeline live-verify --capture-active-eprojs` also retains
the first eight outer calls per installed symbol whose before-state has an
active enemy projectile slot. These complete before/after WRAM captures use
`fixtures/active-port-<symbol-index>-<sequence>-{before,after}.wram`, alongside
the ordinary first-eight call captures. Capture selection does not alter game
state. Inspect actual OAM or contact changes before claiming those behaviors;
an active pool alone can still be filtered by priority, visibility or flags.

The `projectile-render-contacts` profile selects priority rendering and the two
contact detectors. The `projectile-render-hits` profile selects rendering and
the two hit handlers; leaving the C detectors installed gives native handler
wrappers their own measurable calls. The ordinary gameplay profile includes
the live-exercised rendering and detectors. Final frozen fixture evidence can
be resumed explicitly with
`--batch out/port/projectile-contact-render-final/batch.json`; the runner checks
its current manifests, source dependencies and retained artifacts before reuse.

The `observed-projectiles` profile includes both native graphics allocation
entry points and the complete scheduler through the separate
`--projectile-verdict` contract. Its
immutable admission checks recognize nine observed bank86 definitions and their
native script continuations. Refused calls select C before mutation; accepted
Rust failures terminate the replay. The synchronous bridge carries all eight
external projectile host words, with full-pool allocation and disabled scheduler
paths requiring no host reads. `SpawnEprojWithGfx` retains its native three-word
C signature; the two-argument Rust fixture seam packs enemy slot and definition
into the low/high halves of its second argument, declared explicitly with
`packed_args: "eproj-with-gfx-v1"`. This remains bounded projectile ownership.

Native projectile evidence includes first-eight active and fireball captures,
first-two enabled scheduler captures per observed ID, and periodic/terminal
sixteen-byte host snapshots. Per-ID counters distinguish C, Rust and support
refusals. Full129,712-input admission requires all nine IDs to have exclusive
Rust scheduler calls and complete WRAM/host captures. Allocation counters and
first-two per-ID captures separately prove Ridley and Ceres native allocation;
full-route admission requires 16 Ridley, four pad and four platform allocations
exclusively Rust. Every native entry must execute with zero support refusals.
Controlled offline
`continue_from` fixtures inherit the preceding matching full WRAM and host
state; their `case_order` records cross-routine allocation/frame sequences.

```sh
# Authenticate the retained binary, subject pointers and packed observations.
cargo run -p pipeline --locked --offline -- live-authenticate out/port/example/RUN/verdict.toml

# Restore a subject into a new directory, or one exact member/frame.
cargo run -p pipeline --locked --offline -- packed-materialize out/port/example/RUN/c-0 restored
cargo run -p pipeline --locked --offline -- packed-materialize out/port/example/RUN/c-0 frame.bin --member frame-001000.bin
cargo run -p pipeline --locked --offline -- packed-materialize out/port/example/RUN/c-0 effects.log --member hardware.log --frame 100
```

`sm-pipeline live-verify --evidence-store PATH` selects another shared store.
Native runs use packed storage. Historical plain/gzip accepted evidence stays
readable under its retained reader. Failed producers never publish a subject pointer or release staging
files. Shared pipeline writers serialize while capturing; an unclean Rust writer
can leave `writer.lock` and `.pending` files, requiring manual recovery after
confirming that process has exited. Keep the store together with its dependent
run pointers; do not remove a shared pack as part of run cleanup.

To reclaim disposable Cargo build intermediates without changing existing
acceptance evidence:

```sh
cargo run -p pipeline --locked --offline -- cleanup --root out
cargo run -p pipeline --locked --offline -- cleanup --root out --apply
```

Cleanup skips unfinished runs, symlinks and every build directory named by an
artifact inventory, including nested inventories. Fixtures, replay observations,
verdicts, acceptance pointers and linked binaries remain retained. Existing
sealed offline runs often contain large full-WRAM fixtures; cleanup preserves
those files. New live runs discard their unsealed Cargo target directory after
completion automatically.

For larger savings, deduplicate identical sealed snapshots:

```sh
cargo run -p pipeline --locked --offline -- cleanup --root out --deduplicate
cargo run -p pipeline --locked --offline -- cleanup --root out --deduplicate --apply
```

This verifies SHA-256 and byte equality before atomically replacing duplicate
immutable WRAM/frame/terminal/ROM-view snapshots and sealed replay effect logs with hardlinks on the same
filesystem. Paths, bytes, verdicts and hashes remain unchanged. Only finished
accepted runs qualify; retained evidence must remain immutable because linked
snapshots share storage. Independent process comparison evidence stays intact.

`sm-pipeline batch run` and `verify` share storage for immutable snapshots within
that finished batch automatically.

Finished implementer Git worktrees and their build outputs are removed after
integration. Where an existing evidence seal names an implementer source path,
only the exact sealed Rust source snapshot remains at that path so historical
verification still works. These small snapshots are evidence dependencies;
they contain no Git worktree metadata or retained build tree.

`--room-plm` verifies terrain, room and hardcoded allocation evidence and requires
exercised native room/hardcoded setup calls during live replay. Unlike
`--native-plm`, its admission gate does not require terrain collision callbacks.
Unsupported room setups remain counted C ownership. The PLM rendering/frame
engine remains separately owned.
