# Working storage spike: results

The spike works on complete real captures. **5.73 GB of traces and snapshots
became a 54.9 MB disk-backed store: about 104× smaller, or 99.0% less storage.**
That total includes compressed objects, SQLite lookup indexes and sealed run
manifests. Direct native binary logging was about 10× faster than plain-text
logging and 16× faster than gzip text in the measured writer microbenchmark.
Snapshot extraction is slower than reading a cached raw file.

This is throwaway code on `research/trace-storage-format`, under
`/tmp/sm-trace-storage-study`. It does not change production captures or the
validation pipeline. The reproducible entry point is `tools/storage_spike.py`.

## What was built

- An append-only compressed object pack with a disk-backed SQLite object/page
  index. Existing page and trace content is reused across runs.
- 256-byte snapshot pages, compact u32 physical references, exact lengths and
  canonical SHA-256 Merkle snapshot roots. The prototype computes the Merkle
  root but persists compact page tables, not separately traversable branch nodes.
- Independently compressed binary trace chunks, per-frame offsets and event
  ordinals, exact legacy text hashes, chunk hashes and completion counts.
  Unknown event types use a literal record, preserving exact bytes. The full
  room capture contains one `room-plm` event absent from earlier samples.
- Separate canonical run/provenance and observation roots. Root publication
  follows durable object writes. A reopened reader can extract a snapshot or
  retrieve one trace frame without decoding preceding frames.
- A native C trace encoder/decoder and a direct recording microbenchmark with
  plain text, gzip1, Zstandard3 text, binary Zstandard3 and no-logging modes.

Burst aggregation is not implemented: every observed write is retained as a
typed record. Garbage collection, concurrent producers, portable closure export,
production integration and a native snapshot reader/writer remain future work.

## Complete-capture storage

All byte counts below use decimal MB/GB. Original scopes contain hardware.log,
all sealed WRAM fixtures and all complete frame files; exclude ROM views, build
products, unrelated logs and verdict/source files.

| Scope | Original logical bytes | Actual new store bytes | Reduction |
| --- | ---: | ---: | ---: |
| Projectile, 129,712 frames: 280 million events, 1,000 snapshots | 5,264 MB | 53.1 MB | 99× |
| Room, 9,000 frames: 26.5 million events, 142 snapshots, added to same store | 470 MB | 1.79 MB added | 262× incremental |
| Combined store, including empty-store baseline | 5,734 MB | 54.93 MB | 104.4× |

Counting source hardlinks once **within each selected capture**, originals
allocate 5,691.6 MB and the new store allocates 54.94 MB, a 103.6× reduction.
The baseline does not enumerate inode sharing across all out/ directories.
These are actual complete captures, not extrapolated trace samples. Do not
apply this ratio to all out/: builds, ROMs and other data have different behavior.

Final disk layout:

| File class | Actual bytes |
| --- | ---: |
| Compressed object pack | 22.82 MB |
| SQLite physical index, pages, frame lookup and duplicated manifest data | 28.36 MB |
| Two inspectable sealed JSON run manifests | 3.75 MB |
| Total | 54.93 MB in four files |

Within the pack, projectile trace chunks occupy20.89MB, snapshot page blocks
1.14MB and snapshot tables0.75MB. The room run adds only four new trace chunk
objects, one page block and74snapshot table objects; its10frame snapshots already
exist in the longer capture. Final counts:43,241pages and2,873packed objects
(2,090trace chunks,12page blocks,771distinct snapshot tables).

Separate self-contained trace containers, before cross-run sharing, are23.30MB
for the4.97GB projectile trace and1.10MB for the439MB room trace. Those files
include hashes and per-frame indexes and were removed after verified import.

SQLite/index overhead dominates much of the total. It deliberately prioritizes
inspectability and a runnable spike. Compact binary indexes and eliminating
duplicated manifests are plausible improvements, but no further savings are
claimed without measurement. The earlier1.23MBglobal snapshot-payload estimate
excluded these costs and is not the size of a working global store.

## Write/runtime cost

Native C microbenchmark: ten million recorded events, median of three runs,
actual file writes plus close/fsync. Events are preloaded from a16MiBreal mid-run
trace sample and repeatedly recorded. Binary mode includes frame-index writes,
logical/physical chunk hashes and Zstandard3 compression. It does not include
SQLite insertion/global pack publication, legacy-text hash formatting or any
emulator/game execution.

| Writer | Time for10million events | Output bytes |
| --- | ---: | ---: |
| No logging loop | 0.042 s | 0 |
| Existing-style `fprintf` text | 2.31 s | 176.9 MB |
| Existing-style gzip1 text | 3.52 s | 16.80 MB |
| Zstandard3 text | 2.85 s | 8.88 MB |
| Typed binary + Zstandard3 + hashes/index | 0.220 s | 1.14 MB |

CPU medians are approximately0.040 /2.302 /3.510 /2.836 /0.217seconds.
Binary recording adds about**18ns/event**over the no-logging loop in this
microbenchmark; existing plain logging adds about226ns/event. Applying those
costs to the279.5millionevent capture suggests about**5CPU seconds for binary
recording vs63CPU seconds for text**, plus store publication/snapshot work.
This is a workload projection, not an end-to-end replay measurement. It cannot
be converted into an emulator slowdown percentage without timing an integrated
writer in otherwise identical replays. Storage microbenchmarks also benefit
from cached inputs, regular repeated events and a single writer.

No separate native snapshot-write benchmark was built. Python offline packing
of all1,000projectile snapshots takes3.55seconds; this reads296MBofstate, computes
legacy hashes and Merkle identities, deduplicates pages, writes the store and
syncs. That averages3.6mspercapturedsnapshot, not per game frame. Capture cadence
and state complexity vary; this does not establish a native live-observer cost.

## Offline migration and verification

| Operation | Projectile | Room |
| --- | ---: | ---: |
| Convert existing full text trace, native streaming encoder | 67.8 s | 8.16 s |
| Exact expansion of every record + compare original SHA-256 | 83.6 s | 6.42 s |
| Import already packed trace into content-addressed store | 3.69 s | 0.17 s |
| Pack all snapshots with original-seal checks, Python | 3.55 s | 0.51 s |
| Publish sealed roots/indexes | 1.28 s | 0.078 s |
| Verify/decompress all stored trace objects, Python | 4.39 s | 0.29 s |
| Reconstruct/hash every complete snapshot, Python | 3.89 s | 0.31 s |

Offline parsing of old decimal text is much more expensive than writing binary
directly. Both native trace conversions computed the exact original sealed
SHA-256, and independent native decoders reproduced it. All1,142snapshot members
were checked against retained source verdicts and reconstructed after reopening
the store. Verification of stored chunks validates hashes and identities; exact
legacy expansion additionally formats/hashes every record and is more expensive.
Hashing just the entire22.8MBphysical pack took0.054seconds in an additional run;
that authenticates stored bytes against a trusted physical seal and does not
perform logical reconstruction or completeness checks by itself.

## Random access and memory

Ten randomly selected frame snapshots and ten trace frames per run, with the
prototype's object cache cleared between requests. OS file cache remains warm;
these are not cold-disk benchmarks. All objects are hashed on reading; snapshots
also verify leaf/root and legacy hashes. Initial manifest opening is excluded
from per-request medians and measured separately.

| Read operation | Projectile median | Room median |
| --- | ---: | ---: |
| Full1.40MBframe snapshot extraction | 29.8 ms | 17.3 ms |
| Same raw frame read from warm file cache | 0.40 ms | 0.43 ms |
| One indexed trace frame | 7.0 ms | 6.4 ms |

Largest sampled snapshot extraction:113ms. Largest sampled trace-frame lookup:
13ms. Median compressed bytes fetched per snapshot:516KB/259KB; per trace frame:
4.2KB/1.9KB. The Python per-page hashing and assembly are the main visible
tradeoff; a native reader needs measurement before claiming its performance.

Reopening the store and loading43kpage-index rows took74–102ms. First validation
and construction of the sealed run's frame index took719ms for the large run
and26ms for the small run. Subsequent reads reuse that authenticated index.
Thus command startup for a large run is approximately0.8seconds before its first
request. A sealed binary index could improve this, but was not benchmarked.

Peak Python benchmark RSS:about117MiB. Native writer process:about16MiB in an
additional run, including an approximately11MBpreloaded event sample used only
by the benchmark. Live event buffering is capped around1MiBplus compression
workspace; no measured integrated-host peak memory is claimed. The benchmark's
aggregate child RSS field includes compilation and should not be interpreted as
native writer memory.

Machine: x86_64, Intel i7-1165G7 @2.80GHz, four cores/eight threads; Linux/WSL
filesystem, Python3.8.10, system libzstd1.4.4, native `cc -O3`, single-threaded
codec. Timings are approximate and subject to other workspace activity.

## Reproduce/use

```sh
PYTHONDONTWRITEBYTECODE=1 python3 tools/storage_spike.py benchmark \
  --output out/storage-spike/new-run --events 10000000
PYTHONDONTWRITEBYTECODE=1 python3 tools/storage_spike.py extract \
  --store out/storage-spike/final/store \
  --run 71de132bc5e7aaa54c0979f757d69632ed947675f6be9729d4fce077a502b258 \
  --member frame-001000.bin --output /tmp/storage-spike-frame.bin
```

`verify --store <directory> --run <root>` performs full Python logical trace and
snapshot reconstruction. For the large trace it is intentionally much slower
than the native verify measurements; production should use the native reader.
`extract` refuses to overwrite its output.

The retained working store is
`/tmp/sm-trace-storage-study/out/storage-spike/final/store`. Complete raw
measurements are in this directory's sibling `benchmark.json` and copied into
the issue directory. Additional source files record native memory, reader-open
cost, chunk verification and20independent indexed-frame extraction checks.

The next implementation step is to integrate the native event writer behind
the existing host recorder, move snapshot processing/indexing into native code,
and measure identical replay horizons. Global concurrent publication, GC and
self-contained export must precede a production global store. Accepted evidence
should remain on its current format until those controls pass.

Cleanup and retained evidence dependencies are recorded in
[ticket03](issues/03-storage-spike.md).
