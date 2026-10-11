# Packed storage in the live pipeline

`port_live.py`, including invocation by `port_batch.py replay/run`, now defaults
to shared packed evidence at `out/evidence`. `--evidence-store` overrides that
location; `--storage plain/gzip` keeps legacy output. Hardware bytes flow through
a FIFO into the Rust encoder during replay, avoiding a full text log on disk.
Snapshots and all fixtures are staged until the producer finishes. The encoder
commits the trace, constructs the complete tree/run, reopens it, and hashes its
restored members. Python compares those against all staged file hashes, syncs
and atomically publishes `evidence.json`, then releases staging files.

Pointers cover original ROM copies and capture manifests as well as every prior
comparison member. Existing exact inventory/progress/counter admission remains
unchanged. Four subjects have separate provenance/run ids; identical observation
objects share the store. All pointer roots are included in the verdict. Failures
preserve staging/trace receipts and never publish an accepted subject. A crashed
collector may leave a crate writer lock/pending file; recovery stays manual.
Pipeline writers serialize using an advisory lock and never steal that lock.

`port_authenticate_live.py` supports packed and historical loose evidence.
`port_evidence.py SUBJECT NEW_DIRECTORY` restores a whole subject; `--member`
restores one member, with `--frame` optionally selecting a trace frame. Each run
retains its exact reader binary. Shared packs are dependencies of every reference
and must not be removed when deleting an individual run. Production integration
currently covers the main four-process port live/batch runner; older frozen
`decomp_live.py` acceptance harnesses retain their established layout.

## Evidence

Four independent 1000-frame C/mixed replays pass with both storage formats.
The exact observation inventories match across packed and plain runs. Retained
binary/pointer/closure authentication passes for both formats. Snapshot/ROM
fixture duplication and excluded manifests account for different full tree roots
between C and mixed; the prior comparison inventory stays equal.

- Logical bytes from all four subjects: 319326808 (~319 MB).
- Shared packed bytes including indexes: 3196451 (~3.20 MB), ~100x smaller.
- Repeating the same four observations added zero packed bytes.
- Packed C producer: 2.64–2.71 seconds; plain C producer: 1.97–2.08 seconds.
- Packed C post-run sealing: 4.41–4.48 seconds.
- Packed mixed producer: 2.31–3.57 seconds; plain: 1.62–1.84 seconds.
- Packed mixed sealing: 2.14–3.00 seconds.

This short local run suggests roughly 30–40% capture overhead for the C subjects,
plus explicit full closure checking. The mixed sample had concurrent activity and
more timing variance. Do not extrapolate this to long-game frame rate; samples
were sequential and filesystem caches/scheduling were uncontrolled. C currently
formats events to its pipe and Rust parses them. Direct typed producer integration
is a later performance opportunity. See `pipeline-results.json` for exact times,
roots, run paths, binary/verdict seals and physical pack hashes.

Main repository gameplay edits were concurrently active, so an exact source
snapshot under `out/port/storage-pipeline/source` was used for real verification.
Required source files remain there without Git metadata, build outputs or
symlinks. Adding the evidence crate changed Cargo.lock, correctly invalidating
the decompressor predecessor. It was freshly accepted using all frozen synthetic,
captured/original checks and negative controls; the default acceptance pointer
now authenticates with the current lockfile. Original historical runs were not
modified.

Ten Rust integration tests, five pipeline capture/failure tests, 16 live harness
tests (one preexisting skip), and three cleanup tests pass; focused Clippy with
warnings denied, cargo fmt and diff whitespace checks pass. Missing packs,
producer failure, timeout/reaping, pointer tampering and writer locks are covered.

Scoped cleanup apply/deduplicate succeeded on the pipeline run directory and
fresh decompressor acceptance run: 0 build bytes and 477861689 duplicate snapshot
bytes reclaimed. Identified unsealed timing captures/source symlinks were removed
(319347708 file bytes); build helpers remove disposable targets automatically.
Final verdict/pack hashes reauthenticate after cleanup. About 266.49 GB remained.
Root workspace target predates this task and was used by concurrent gameplay
work; it remains shared, outside the scoped evidence runs.
