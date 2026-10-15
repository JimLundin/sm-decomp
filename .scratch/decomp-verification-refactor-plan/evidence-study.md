# Offline evidence representation experiment

Status: implementation and bounded measurement active; no production adoption
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
16 actual-C Fixture pairs, two enemy-jump routines, 333 retained members. It has
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
aggregate simultaneous RSS promise.

## Commands and retained ownership

Private build target/TMPDIR/logs:
`/home/jim/sm-decomp-performance-evidence-proofs/build-20261015`.
The workspace-only ignored symlink `deps/rustysnes` points to
`/home/jim/sm-decomp/deps/rustysnes` so Cargo can resolve the existing patched
manifest. It grants no writable ownership of that checkout and is removed at
handoff without following it.

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

## Current result

Pending. Initial optimized compile passed in 1m57s. Final controls build is in
progress. Measurement has not started and no speedup is claimed.

## Cleanup and remaining work

Every measured sample invokes the frozen utility's ordinary and deduplicating
cleanup passes before fresh reader validation. The controls invoke both over
their owned tree, and the final study root receives both passes again. After
retaining exact reader/source and results, disposable build outputs and the
ignored dependency symlink are removed; worktree retirement belongs to root.
Canonical ticket status is unchanged by this worker.
