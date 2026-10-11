# Running the Rust port checks

Run these commands from the repository root:

```sh
python3 tools/port_batch.py check
python3 tools/port_batch.py run
python3 tools/port_batch.py status
```

`check` runs formatting, Rust tests, Clippy and the port harness tests.
`run` verifies the `gameplay` profile and runs four independent C/mixed processes
through 18,000 frames. Jobs run three at a time. The catalog and profiles live in
`tools/port/batch.json`; callback signatures and live eligibility remain declared
in the individual family manifests. The partial power-bomb entry is excluded
from live installation. A passing replay remains C/Rust integration evidence,
not original-ROM parity or full gameplay ownership.

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

`sm-pipeline` in `crates/pipeline` owns process leases, authenticated build reuse,
compiler dependency inventories and C translation-unit builds. Existing Python
commands adapt the family-specific harnesses to this Rust module. Worktrees
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
store after the comparison gates pass. `port_live.py --retain-private-stores`
keeps those stores for diagnostics. Plain and gzip capture remain sequential.

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
python3 tools/port_batch.py verify --profile all

# Verify or retry one family without rebuilding unchanged families.
python3 tools/port_batch.py verify --families projectile_flight

# Replay the already verified gameplay profile at a different horizon.
python3 tools/port_batch.py replay --frames 9000

# Select a batch explicitly or create one at a chosen path.
python3 tools/port_batch.py run --batch out/port/batches/review/batch.json --jobs 2

# Discover the current offline manifest frontier and run isolated case pairs.
python3 tools/port_batch.py verify --profile offline-all --case-jobs 4

# Select families affected by source changes relative to a commit.
python3 tools/port_batch.py verify --profile offline-all --changed-since HEAD~1
```

Catalog entries may declare `kind`, `manifest`, `live`, and a `prepare` command
with `{root}`, `{output}` and `{manifest}` placeholders. List imported generator
files in `prepare_inputs` so their contents also invalidate generated evidence.
Discovered manifests are offline only; live adapters require explicit catalog
entries. The `dispatcher-leaves` profile exercises the bank86 dispatcher harness.
The versioned compiler dependency contract seals actual source reads and canonical
memory, allowing unchanged narrow families to survive unrelated gameplay edits.

Use `--fresh` to force verification rebuilds. `--native-plm` requires passing
`plm-spawn` evidence and actual native setup execution during replay; a full PLM
slot pool or zero PLM calls cannot satisfy that gate. It is optional because the
current movie prefix does not exercise that boundary.

PLM instruction fixtures are generated automatically in the batch's asset
directory. Their bounded interpreter is verified offline and is excluded from
the whole live PLM loop. Individual harness commands remain available for focused
diagnosis; batch orchestration should use the checked-in runner.

Future `port_live.py` replays (including batch replay) use the Rust evidence store
by default. Hardware events stream into compressed binary records as the C host
runs. Snapshots, ROM views and fixture files are packed after each process exits;
the runner authenticates their restored byte hashes before publishing a durable
`evidence.json` pointer and removing staging files. Four independent subjects
still compare the same exact observation inventories. Identical pages, bursts
and captures share `out/evidence` across runs. Provenance is sealed separately
from observations; storage completeness does not grant gameplay acceptance.

```sh
# Authenticate the retained binary, subject pointers and packed observations.
python3 tools/port_authenticate_live.py out/port/example/RUN/verdict.json

# Restore a subject into a new directory, or one exact member/frame.
python3 tools/port_evidence.py out/port/example/RUN/c-0 restored
python3 tools/port_evidence.py out/port/example/RUN/c-0 frame.bin --member frame-001000.bin
python3 tools/port_evidence.py out/port/example/RUN/c-0 effects.log --member hardware.log --frame 100
```

`port_live.py --evidence-store PATH` selects another shared store. `--storage
plain` or `--storage gzip` keeps the legacy file layout; `--compress-effects`
remains an alias for gzip. Historical accepted evidence is unchanged and stays
readable. Failed producers never publish a subject pointer or release staging
files. Shared pipeline writers serialize while capturing; an unclean Rust writer
can leave `writer.lock` and `.pending` files, requiring manual recovery after
confirming that process has exited. Keep the store together with its dependent
run pointers; do not remove a shared pack as part of run cleanup.

To reclaim disposable Cargo build intermediates without changing existing
acceptance evidence:

```sh
python3 tools/port_cleanup.py                 # preview under out/
python3 tools/port_cleanup.py --apply         # remove eligible target/ directories
```

Cleanup skips unfinished runs, symlinks and every build directory named by an
artifact inventory, including nested inventories. Fixtures, replay observations,
verdicts, acceptance pointers and linked binaries remain retained. Existing
sealed offline runs often contain large full-WRAM fixtures; cleanup preserves
those files. New live runs discard their unsealed Cargo target directory after
completion automatically.

For larger savings, deduplicate identical sealed snapshots:

```sh
python3 tools/port_cleanup.py --deduplicate
python3 tools/port_cleanup.py --deduplicate --apply
```

This verifies SHA-256 and byte equality before atomically replacing duplicate
immutable WRAM/frame/terminal/ROM-view snapshots and sealed replay effect logs with hardlinks on the same
filesystem. Paths, bytes, verdicts and hashes remain unchanged. Only finished
accepted runs qualify; retained evidence must remain immutable because linked
snapshots share storage. Independent process comparison evidence stays intact.

`port_batch.py run` and `verify` share storage for immutable snapshots within
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
