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

One batch command runs at a time to protect the record; verification jobs inside
it run concurrently. Long jobs print progress every 30 seconds, with full output
kept in their log directories.

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
```

Use `--fresh` to force verification rebuilds. `--native-plm` requires passing
`plm-spawn` evidence and actual native setup execution during replay; a full PLM
slot pool or zero PLM calls cannot satisfy that gate. It is optional because the
current movie prefix does not exercise that boundary.

PLM instruction fixtures are generated automatically in the batch's asset
directory. Their bounded interpreter is verified offline and is excluded from
the whole live PLM loop. Individual harness commands remain available for focused
diagnosis; batch orchestration should use the checked-in runner.

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
