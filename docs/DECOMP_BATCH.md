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

Future `port_live.py` replays (including batch replay) use the Rust evidence store
by default. Hardware events stream into compressed binary records as the C host
runs. Snapshots, ROM views and fixture files are packed after each process exits;
the runner authenticates their restored byte hashes before publishing a durable
`evidence.json` pointer and removing staging files. Four independent subjects
still compare the same exact observation inventories. Identical pages, bursts
and captures share `out/evidence` across runs. Provenance is sealed separately
from observations; storage completeness does not grant gameplay acceptance.

For projectile integration, `port_live.py --capture-active-eprojs` also retains
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

The `observed-projectiles` profile includes native room allocation and the
complete scheduler through the separate `--projectile-verdict` contract. Its
immutable admission checks recognize nine observed bank86 definitions and their
native script continuations. Refused calls select C before mutation; accepted
Rust failures terminate the replay. The synchronous bridge carries all eight
external projectile host words, with full-pool allocation and disabled scheduler
paths requiring no host reads. This remains bounded projectile ownership.

Native projectile evidence includes first-eight active and fireball captures,
first-two enabled scheduler captures per observed ID, and periodic/terminal
sixteen-byte host snapshots. Per-ID counters distinguish C, Rust and support
refusals. Full129,712-input admission requires all nine IDs to have exclusive
Rust scheduler calls and complete WRAM/host captures. Controlled offline
`continue_from` fixtures inherit the preceding matching full WRAM and host
state; their `case_order` records cross-routine allocation/frame sequences.

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
