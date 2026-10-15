# Offline evidence representation experiment

Status: experiment complete; reject whole-run packing for this performance refactor
Owner: performance session `01a13e7f-ab01-7413-a1c2-288b08a4b719`, delegated
`/root/evidence_prototype`
Branch: `performance/evidence-study-20261015`
Baseline: `8bf8acf610e1a94a1a3bccd5d718dae4a4244857`

This is ticket 05's isolated Rust representation experiment. The canonical ticket
and plan remain in `/home/jim/sm-decomp/.scratch/decomp-verification-refactor-plan`.
Source ownership is the example and its controls under `crates/evidence/examples/`
and this handoff. No production code, Cargo manifests or lockfiles are changed.

## Question and scope

Does replacing completed Offline staging with an existing indexed evidence store
reduce the complete representation lifecycle, including independent reading and
cleanup? Compare 256-byte Merkle images against raw 1-MiB file chunks (one payload
blob for each 128-KiB state), and compare a direct private store against four
bounded private writers followed by authenticated ordinal-remapping import.

This is **storage-only**. It does not execute a new Fixture, bypass staging from a
real producer, authenticate a new Native attempt, or implement a fused Offline
admission reader. Byte-for-byte preservation of original producer receipts is
necessary for this study but is not a new producer-attribution contract.

The complete read-only input is the source-authenticated ordinary run at
`/home/jim/sm-decomp-offline-architecture-proofs/plm-fix-20261015/ordinary/1792064366867709-1385229-0`:
16 actual-C Fixture pairs, two enemy-jump routines, 333 retained members and
43,675,132 logical bytes. It has
already passed both cleanup passes and standalone authentication. All members
are copied, including the unchanged original verdict, binaries, ROM, source,
empty stdout/stderr, return observations, fork identities and timing records.
There are no host/callback/chain observations in this particular declaration;
its results cannot substitute for those recipes' later gates.

The preparation pins the original verdict and authenticated receipt, hashes every
listed original artifact against the untouched verdict, and records every
retained member's bytes, mode and digest. The input manifest's digest stays
outside candidate output. A fresh reader checks exact logical membership and
restored bytes against that manifest. The original external source/reader
dependencies continue to be required; the study is not a receipt relocation.

## Predeclared bounded method

Use the same optimized example executable for all variants. Each sample has a
60-second timeout, at most 256 MiB input, at most 64 MiB per file, 4096 members,
and at most four private writer threads. Staging preserves original hard links
inside the new copy without linking to the original proof. Unique-inode allocated
bytes and inode counts are reported; no flat-file inflated baseline is used.

Each candidate receives three alternating adjacent directory/candidate pairs
(candidate first on the middle repetition). This is 18 independent sample
processes. Store directories and pending outputs are private. No active cache,
installed policy, baseline source or original evidence is modified.

For a positive storage-performance decision, require at least a 15% lower median
elapsed lifecycle from completed staging, improvement in all three paired
observations, and no material regression in independent authentication or
extraction. Large retained-size savings alone do not pass. Contention and small
sample size limit any inference about whole-batch speedup.

Every mode reports the common simulated capture separately. The primary comparison
starts after that capture and includes staged inventory authentication, encoding,
hashing/compression, complete logical closure construction, physical commit,
reopening/closure validation, import where used, pointer publication, staging
release, both supported cleanup passes, fresh post-cleanup reading, exact
extraction and original byte comparison, and extraction removal. Per-writer
encoding/commit durations overlap for four-writer mode and are not summed as
exclusive wall-time stages.

Store encoding/hashing/compression are one measured stage; the public Store API
does not expose an honest split between them. The raw file strategy still uses
the store's existing Zstandard blocks. The directory path adds no per-file fsync
that current Offline output does not already perform. `/proc/self/io` counts are
kernel process I/O accounting, not a physical-device throughput measurement;
`rchar` includes cache hits. CPU ticks and process peak RSS are recorded separately.
Cleanup subprocess CPU is reported as waited-child CPU; process RSS is not an
aggregate simultaneous RSS promise. The sample timer excludes process startup,
input-manifest parsing and the common cleanup-reader seal check; the external
matrix timer includes those costs. All representation stages listed above are
inside the sample timer.

## Commands and retained ownership

Private build target/TMPDIR/logs:
`/home/jim/sm-decomp-performance-evidence-proofs/build-20261015`.
The workspace-only ignored symlink `deps/rustysnes` pointed to
`/home/jim/sm-decomp/deps/rustysnes` so Cargo can resolve the existing patched
manifest. It granted no writable ownership of that checkout and was removed
without following it after the final build.

Build with Rust only, locked/offline and one Cargo job:

```sh
CARGO_TARGET_DIR=/home/jim/sm-decomp-performance-evidence-proofs/build-20261015/target \
TMPDIR=/home/jim/sm-decomp-performance-evidence-proofs/build-20261015/tmp \
CARGO_BUILD_JOBS=1 SM_PIPELINE_ROOT=/home/jim/sm-decomp/out/pipeline \
SM_PIPELINE_WORKERS=4 cargo build -p evidence --locked --offline --release \
  --example offline_representation_prototype
```

The CLI has `prepare`, `sample`, `measure`, `verify`, `check-matrix` and `controls`
commands. `verify` requires the independently held input and publication digests;
`check-matrix` also pins the matrix digest. The intentional `crash-writer` command
is a private study fault injection entry point and exits 23 before committing.
Full exact commands, retained executable/source identities, raw measurements and
cleanup receipts will be recorded in the final handoff.

## Measured result

All 18 alternating samples completed in 16.29 seconds total, under the 180-second
matrix budget and 60-second individual limits. Every sample reconstructed and
compared every member against the original bytes, ran both cleanup passes and
authenticated with a new reader after cleanup. Input files were already in the
filesystem cache; each candidate wrote a new private store. No cache was cleared.

| Representation | Samples | Median from staging | Observed range | Retained allocated bytes | Files / distinct inodes | Process peak RSS |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Current directory with supported deduplication | 9 | 0.2628 s | 0.2459–0.3581 s | 40,206,336 | 333 / 301 | 4,896–4,980 KiB |
| Store, Merkle 256-byte images | 3 | 1.0246 s | 1.0160–2.2210 s | 11,038,720 | 1 / 1 | 40,032–40,148 KiB |
| Store, raw 1-MiB file chunks | 3 | 0.9034 s | 0.8800–1.0997 s | 9,912,320 | 1 / 1 | 39,816–39,952 KiB |
| Four private raw writers plus authenticated import | 3 | 1.8181 s | 1.3659–2.1444 s | 9,953,280 | 1 / 1 | 56,928–58,000 KiB |

The common simulated capture had medians of 0.0282–0.0314 seconds and is excluded
from the table's elapsed column. Median measured self plus waited-child CPU was
0.28 s for directories, 0.99 s for Merkle, 0.87 s for raw and 1.94 s for private
raw/import. Private writer CPU overlaps; CPU durations are not wall-time parts.

Packing saved about 75% of retained allocated bytes in the raw strategy, but
lost elapsed time in every comparison. Median fresh post-cleanup authentication
was 0.0305 s for directories, 0.1291 s for Merkle, 0.1112 s for raw and 0.1321 s
for private raw/import. Median exact extraction and original-byte comparison was
0.0834 / 0.2206 / 0.1774 / 0.1897 seconds respectively. These are complete-member
restorations, not comparisons of declared roots alone.

Four writers reduced their encoding/commit phase to a 0.3097-second median from
the single raw writer's 0.3945 seconds, but private-store validation added
0.1211 seconds and authenticated import/combined closure/commit added 0.4359
seconds. The final imported raw run identity exactly matches the direct raw run,
which confirms that store ordinals were remapped without changing observations.

The host had eight logical CPUs, about 12.5 GB RAM, existing swap use and active
independent mutation/Native sessions. The immediate pre-matrix process snapshot
showed no high-CPU compiler or producer, and one-minute load averages during the
matrix were 0.18–0.36. Long pack outliers and changing system disk usage still
preclude attributing all variation to this implementation. No whole-catalog or
whole-verification speedup is established.

This is a small declaration whose retained compiler binaries, worker archive and
ROM dominate its bytes. The result rejects **packing this complete run as a
performance improvement**. It does not prove that packing only state-heavy
observations, a cold filesystem workload, or a direct producer transport can
never win. Those are separate experiments, not benefits measured here.

## Read and failure controls

This host has no `/proc/self/io`. The first measurement refused during resource
setup before writing any staging; its diagnostics and exact failed executable
remain retained. Both failed roots received both zero-byte cleanup passes.
Resource rows now explicitly say `io_counters_available = false`, and unavailable
byte counters are absent rather than reported as zero. Initial optimized build
took 1m57s; the subsequent controls and availability rebuilds passed. This change
is solely instrumentation in the isolated example.

Two additional, separately labeled `strace -f -c` samples counted explicit I/O
syscalls including study children and cleanup. They are excluded from the timing
comparison because tracing adds overhead. Directory: 9,848 `read` calls, 30
`pread64`, 1,012 `copy_file_range` and six `write`. Raw: 10,510 `read`, 30
`pread64`, 506 `copy_file_range` and 372 `write`. Each trace recorded one failed
read. These are call counts, not actual payload bytes or physical disk traffic;
they show why retained file count alone is not a read-cost measurement.

All 16 explicit negative checks returned normal Interface errors (exit 1), with
no panic, signal or timeout accepted as a refusal. They covered altered/missing/
extra directory members, equal wrong Rust/C outputs, missing producer records,
truncated state, altered/missing/truncated packs, unexpected private-store files,
extra/duplicate logical tree members and a missing referenced chunk. Interruption
before pointer rename refused. An actual child exited 23 before committing its
pending pack; the unpublished root refused before and after removal of that
exited writer's pending file/lock, while the previous committed closure remained
authenticatable. The fault injector changes only private study copies.

All 16 checks were repeated successfully after final cleanup. The study does not
claim framed-transport validation or Native producer-contract acceptance, because
it implements neither interface. Existing original receipts remain byte-identical.

## Selected narrower ticket 06

Revise ticket 06 to **bounded reuse of authenticated bytes within one case or
operation admission, retaining the current directory format**. No new store
format, producer capability, historical reader or receipt conversion is selected
by this study.

- Fuse inventory hashing and semantic comparison using bounded owned bytes for
  the active case or operation. Cover every required member, returns, host state,
  ordered effects, chain inputs and producer membership; do not replace these
  checks with matching declared hashes.
- Reuse only within the same admission and for bytes actually authenticated in
  that scope. Every independent/restarted/post-cleanup admission reopens and
  authenticates its own closure. Source freshness remains outside the payload
  reuse cache.
- Preserve missing/extra/duplicate/truncation/corruption controls, exact source
  and attempt/producer identities, failure ordering and memory/output bounds.
- Measure the change through the real ordinary and Native admission Interfaces
  before claiming a performance benefit. This study measured no fused-reader
  speedup. Native operation bundling is owned by `/root/native_admission`.

A future direct producer transport remains unmeasured and is not ruled out by
this storage-only loss. Such work must include actual producer/recipe hooks,
complete output framing, Native capability/reservation changes and independent
post-cleanup admission. It should not be implemented now merely to recover
the lost packing cost.

## Reproducible retained evidence

Study root: `/home/jim/sm-decomp-performance-evidence-proofs/20261015`.
The implementation freeze is commit
`30cd749c272a42a62c7d3a57bb7d35634291013a`; subsequent changes are study prose.
Exact evidence/pipeline-model sources, Cargo manifests and lockfile are copied
under `source/`, checked against `logs/source-files.sha256` after build removal.
All project commands and source remain Rust.

| Item | SHA-256 |
| --- | --- |
| `ordinary/input.toml` | `1e5c68b7a147debf27c8991d11a23e91fc2dd3b52ae8d754d814266f98d628b5` |
| `ordinary/matrix.toml` | `afe5bc3343179f02a656d704fd7da1d11cd2d135d4a0cce8a1ff944a826b68c3` |
| `ordinary/controls.toml` | `81ac649b60c6bc36c61261a608da8db9eefaa3d6766d4b75e44423ae98a0c80a` |
| `tools/offline-representation-prototype` | `4cfaf8ab5644ae4ccbec562ff1ff89f6c80d8b8d4627a4c2ce38ea8a874d7c8b` |
| Original verdict and final original authentication output | `50cc84fd73fb6b4d14c635fcd763d3411b9051056a4f70f590258e12bde1c166` |

The full executed argv/budgets and resource measurements are in `logs/*.time`,
`ordinary/matrix.toml` and per-sample receipts. Standard current readers still
reside under the original proof's `plm-fix-20261015/tools`: `sm-pipeline` hash
`84107b83ceb4295db660e1cbf5504268bad031ce056a1a175de58b5025ad0146` and
`sm-evidence` hash `733507a6a9c4966072ff2dcfa96acd34ef943ec76bbdfb898f30b160baf4ad09`.
Retain that proof parent's frozen source, ROM/source prerequisites and original
receipts. Copying the verdict does not relocate its externally referenced paths.

Independent positive readback, after build removal:

```sh
/home/jim/sm-decomp-performance-evidence-proofs/20261015/tools/offline-representation-prototype \
  check-matrix /home/jim/sm-decomp-performance-evidence-proofs/20261015/ordinary \
  1e5c68b7a147debf27c8991d11a23e91fc2dd3b52ae8d754d814266f98d628b5 \
  afe5bc3343179f02a656d704fd7da1d11cd2d135d4a0cce8a1ff944a826b68c3
```

`logs/post-cleanup-matrix.stdout` records all 18 successful complete-member
readbacks; `logs/post-cleanup-controls.toml` records all 16 post-cleanup refusals.
The unchanged current `sm-evidence verify STORE RUN ORIGINAL_DIRECTORY` also
verified all 333 files / 43,675,132 bytes for each packing strategy, using raw run
`817776aac366ac3f9837e5436e90145fc60df855d728c3daecf6c38f8b03c69f`
and Merkle run
`e28eb29f7dcbdb25a7f2190f683b825941b332c6bfe0771a827f112a3dd30938`.
See `logs/current-reader-*.stdout` and stderr. Their concurrently performed final
readback timings are correctness evidence, not comparison samples.

The frozen `family-authenticate` command with the unchanged original
`ordinary-authenticate.toml` returned the exact original verdict bytes in
`logs/original-authenticated-after-study.toml`: 16 actual-C pairs, two routines,
`offline-verified`. No producer was launched by that readback.

## Cleanup and handoff

All owned writers exited before cleanup. Every sample had both supported cleanup
passes; its byte-preserving hard links were already deduplicated. Controls'
ordinary cleanup reclaimed zero and their deduplicating pass reclaimed
10,354,688 bytes across 287 checked snapshots. Final whole-study passes reclaimed
zero and 20,971,520 bytes respectively, checking 767 snapshots. Failed setup roots
and the private build root each received both successful zero-byte passes.

The private Cargo target was then removed with scoped `rm -r --`, reclaiming
76,251,136 measured allocated bytes; its empty TMPDIR added 4,096 bytes. Exact
source, compiled study reader, failed diagnostics and build logs remain. The
ignored dependency symlink was unlinked without touching the real RustySNES
checkout. `logs/disposable-build-bytes.txt`, `logs/*cleanup*.toml` and
`logs/removed-dependency-symlink.txt` retain these facts.

After removal, all 18 matrix readbacks, all 16 negative controls, all three
standard-reader checks and original family authentication passed. The retained
source checksum list passed independently. Measured retained study allocation
was 799,518,720 bytes, plus 20,480 bytes of separate build logs; measured available
filesystem space was 122,843,992,064 bytes. These are point-in-time observations
on a host where other sessions continue writing.

No production source, shared cache, installed policy, main Git/index, other
worktree or original receipt was modified. Canonical ticket 05 remains claimed
for root to resolve with ownership/integration records; ticket 06's revision is
recommended above. The assigned worktree and branch remain available. Root owns
its retirement and the subsequent retained readback; this evidence has no
runtime dependency on that worktree or its deleted build target.
