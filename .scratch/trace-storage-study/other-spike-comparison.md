# Comparison with the other worktree's actual spike

**The measured results favor compressed packs for physical storage.** The other
spike adds useful exact burst sharing and Merkle subtree diffs, which should be
combined with compact compressed packs. Its current uncompressed loose-object
layout uses substantially more disk and takes longer to ingest the same capture.
Neither prototype has measured integrated emulator runtime overhead.

Other worktree: `/home/jim/sm-decomp/.claude/worktrees/trace-storage-format`,
`crates/evidence`, branch `worktree-trace-storage-format`, baseline8bbef1f with
uncommitted spike changes. Existing release binary SHA-256
`953cf9c865363305ce4a424a0db26a3eb1c00712039e2def69920e9a2ab50205`.
Source snapshot/context is retained under `other-spike-source/`; source remained
unchanged during our comparison. Its original files, build and stores were not
modified. The comparison ran its existing binary into a new disposable store.

## Matched actual measurements

Exactly the same5,264,039,623bytes in1,001files: all hardware events plus870WRAM
snapshots and130complete frame snapshots from projectile capture
`20261011T061431.667466Z/c-0`. The other CLI ingests a directory, so temporary
hardlinks built the matched scope without copying5GB or changing source bytes.

| Measurement | Packed spike | Other loose-object spike |
| --- | ---: | ---: |
| Actual file bytes, including our indexes/manifests | 53.14 MB | 277.52 MB |
| Actual allocated file blocks | 53.14 MB | 612.68 MB |
| Reduction from5.26GBlogicaloriginal | 99× | 19× logical /8.6× allocated |
| File/object count | 3 retained files for first run | 117,503 object files |
| Offline ingest/conversion, before separate exact decode check | about76.3s | 172.2s |
| Peak memory | Python whole benchmark117MiB; native16MiBmicrobenchmark | ingest32MiB |

The packed first-run figure53.14MB includes36,864bytesempty-storebaseline plus
53,101,850bytesadded. Earlier reports rounded53.1MB and99×. The whole two-run
store54.93MB is not the matched comparison scope.

Other allocated size is **11.5×** our allocated size; logical bytes are5.2×.
Directory metadata is excluded from the allocated-object count, so loose-object
actual overhead can be a little higher. Small leaves/nodes use at least one
filesystem allocation block each. Other ingest CPU:115.24suser+52.41ssystem.
Its172.2smeasurement does not fsync each object. Our timing sums67.77s native
conversion+3.69s import+3.55s snapshots+1.28s root publication and includes syncs,
but excludes83.59sseparate native exact decode/hash. It is not a controlled
end-to-end replay comparison.

Other store object payload allocation by kind is in `other-matched-benchmark.json`
or matched-ingest result. Ingest reports137.55MBblob objects (pages and burst
payloads),133.89MBevent segments,5.93MBMerkle nodes, plus small image/tree/stream
headers. Logical payload sharing alone does not make segment scalar records
or repeated hash references compact. Compression is still useful after typed
records and payload sharing.

The competitor's matched readback verification passed all1,001files and
5,264,039,623bytes in81.9seconds (including hashing original files). Our separate
native trace expansion/hash takes83.6seconds plus3.9secondssnapshot reconstruction;
those differ in verification scope and implementation, so there is no clear
logical-verification speed winner from these figures.

Its existing `/tmp/evidence-spike/ingest.txt` covers four broader run directories,
including ROMs, logs, source and multiple processes. Its aggregate304.67MBobject
bytes therefore must not be compared directly with our54.93MBtwo-C-capturestore.
That log records545.3singestion for the broader full-movie run, which is also
not the same workload as the matched172.2srun here.

## Capabilities and correctness

| Capability | Packed spike | Other spike |
| --- | --- | --- |
| Disk-backed cross-run content sharing | Yes | Yes |
| Exact legacy text/snapshot restoration | Verified on complete captures | Native reader and byte/hash comparison provided |
| Burst encoding | No, per-write records | Yes, exact consecutive `(register,value)` byte pairs in shared blobs |
| Merkle identities | Roots computed, compact page tables stored | Page/node objects stored; subtree diff implemented |
| Random trace frame extraction | Sealed per-frame offset index implemented |64-frame segments, no frame-read CLI/interface yet |
| Compression | Independent Zstandard3chunks/page pools | None in current crate |
| Root semantics | Separate provenance/run and comparable observation roots | Generic sorted directory tree; not separate comparable observation root |
| Completion handling | Authenticated trace footer/counts plus complete manifest | Preserves interrupted/unrecognized remainder as raw tail; no production completion/admission protocol |
| Concurrent publication, GC, live host integration | Not implemented | Not implemented |

The competing burst representation does preserve exact observed address/value
order and splits at interleaving/nonmember events, so it addresses the main
losslessness concern. It does not substitute an assumed source upload for actual
observed writes. Its `put_stream` stops typed encoding at an unknown tag or
backward-frame/noncanonical line and stores the whole remainder as a raw blob.
Our literal-record fallback continues typed encoding after those records. This
is a material format-size distinction on some captures, though the matched
projectile trace has no unknown tail.

Its `Store::put` reuses existing files based on path existence and `Store::get`
verifies SHA-256 on reading. Root equality therefore still needs authenticated
dependencies; presence alone cannot establish healthy stored evidence. Both
prototypes need production publication/validation controls before migration.

The other spec infers at most nine trajectories from nine distinct file sizes.
Equal length does **not** imply equal content. Cross-run sharing is real in our
measurements, but that broader estimate requires hashes, not sizes. Likewise,
rejecting compression because it is “within-file only” misses content-addressed
compressed objects/packs, which our spike actually implements.

## Read costs

Using its existing retained native store, `sm-evidence cat ... image` restored
frame-001000.bin exactly three times in114/69/44ms, including subprocess startup,
hash verification, Merkle page reads and stdout pipe. Compare cautiously with our
Python17–30ms warmed-reader medians across random frames: ours additionally has
about0.8slarge-run initialization before the first request. The other CLI has
better one-off startup; its page-per-file approach adds many syscalls. These
are warm-cache observations, not a controlled native-versus-native read trial.

Our indexed trace-frame requests measured6–7ms after initialization. The other
spike has a segment index but no equivalent frame extraction interface, so no
matching figure is available. Its tree diff can identify a one-page difference
through only three pairs of node reads in its synthetic control, a useful
capability our stored flat tables do not provide.

Our0.22s/ten-million-events direct recording result has no equivalent competitor
microbenchmark. Offline Rust ingestion time does not establish its live-observer
overhead. A side-by-side integrated writer replay is still needed for that.

## Outcome

Keep the content-addressed logical model, **compressed physical packs and compact
indexes**, and incorporate the other spike's exact burst payload sharing and
Merkle diff capability where they improve measured workloads. This comparison
does not support replacing compression with loose Merkle objects. A production
native implementation of the combined design is the next useful experiment.

Measured raw results and cleanup/status are recorded in
[ticket04](issues/04-other-spike-comparison.md).
