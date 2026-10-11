# Recommended storage for validation traces and frames

Follow-up: [the global-store comparison](global-store-comparison.md) recommends
content-addressed logical artifacts backed by these compact physical packs.
It measures real cross-run sharing and distinguishes logical Merkle identities
from the physical object representation. The original bounded-pack measurements
below remain valid; global retention/concurrency require additional implementation.

Use a **versioned binary pack with independently compressed Zstandard chunks**.
Encode hardware effects as ordered typed records, and store snapshots as page
references into a compressed page pool. Keep a small JSON verdict/manifest. A
frame/sequence index makes first-divergence inspection possible without decoding
the whole run.

The useful optimization is eliminating repeated structure before compression:
decimal text and repeated frame numbers in traces, and nearly identical memory
pages in snapshots. Actual captures support this choice. A database or an image
format alone would not address both kinds of redundancy.

This research is on branch `research/trace-storage-format`, baseline `8bbef1f`,
in `/tmp/sm-trace-storage-study`. No production writer/reader or accepted capture
was changed. The separate `worktree-trace-storage-format` draft was read as
context; its numbers were not used as measured evidence here.

## Measured results

Read-only measurements use the installed libzstd **1.4.4**. This is a measurement
dependency, not a production codec choice. The script compares levels 1, 3 and 9
and gzip level 1; the following uses Zstandard level 3. Every candidate decoded
back to the exact original bytes. MB below means decimal MB; KB means decimal KB.

| Actual input | Existing logical bytes | Candidate encoded payload | Reduction |
| --- | ---: | ---: | ---: |
| Projectile trace, beginning/middle/end, three 16 MiB samples | 50.33 MB | 323.7 KB binary chunks | 155× combined |
| Room trace, beginning/middle/end, three 8 MiB samples | 25.17 MB | 172.3 KB binary chunks | 146× combined |
| Projectile WRAM, all 870 snapshots | 114.03 MB | 1.09 MB, 256-byte page pools | 104× |
| Projectile complete frame state, all 130 snapshots | 182.17 MB | 1.69 MB, 256-byte page pools | 108× |
| Room WRAM, first 128 of the available snapshots | 16.78 MB | 159.7 KB, 256-byte page pools | 105× |
| Room complete frame state, all 10 snapshots | 14.01 MB | 170.1 KB, 256-byte page pools | 82× |

These are **candidate payload sizes**, not final container sizes. A proposed
64-byte-per-chunk trace index adds 4,096 bytes across the projectile samples and
1,856 bytes across the room samples. Snapshot names/lengths/SHA-256 manifests,
compressed separately, add 22.7 KB / 5.1 KB for projectile WRAM/frame sets and
4.2 KB / 0.5 KB for room WRAM/frame sets. Final framing and integrity fields are
still to be implemented. Trace samples are contiguous byte ranges, not a
uniform frame census; do not project these ratios onto every run or claim that
the whole 4.97 GB projectile trace was packed.

Independent Zstandard over existing text yielded 20–28× in bounded chunks,
whereas binary chunks yielded 91–249× across the two runs. In the projectile
samples binary chunks occupied 118.5 / 114.5 / 90.7 KB; equivalent text chunks
occupied 791.5 / 850.6 / 807.7 KB. That is a measured 6.7–8.9× improvement from
the binary representation even after both representations are compressed.

The experimental binary schema is deliberately simple: signed frame delta
varint, stable PPU/APU tag, PPU selector/address/value bytes, or a full 32-bit
APU address plus value byte. It preserves duplicate writes and resets. It does
not claim to implement all historical event grammars or the CPU bus-event
schema. A production schema needs explicit profiles for each of those.

Snapshot candidates used **independent groups of up to 32 snapshots**, with a
fresh page dictionary in each group. Page IDs are assigned in first-appearance
order and stored as little-endian u32 references. The exact member length
reverses padding on the last page. Names and original member hashes belong in
the member index. The dictionary stores actual bytes, so page identity in this
experiment does not depend on a short hash or collision assumption.

| Projectile snapshot encoding, Zstandard level 3 | 870 WRAM snapshots | 130 full frames |
| --- | ---: | ---: |
| Each file independently compressed | 21.20 MB | 6.72 MB |
| Concatenate groups of 32 then compress | 1.31 MB | 3.05 MB |
| Full group base then XOR consecutive states | 1.51 MB | 3.46 MB |
| 256-byte page pool + per-snapshot page tables | 1.09 MB | 1.69 MB |
| 1 KiB page pool + tables | 1.07 MB | 1.82 MB |
| 4 KiB page pool + tables | 1.07 MB | 1.92 MB |

Page pools beat XOR chains on both measured WRAM/frame sets and reconstruct a
member without replaying every preceding snapshot. Start with 256-byte pages
for an implementation spike: that size performs well across both classes.
Allow page size in the versioned encoding; 4 KiB is marginally better for WRAM
and distinctly worse for these frame captures. Avoid declaring one universal
optimum from two runs.

## Account for existing deduplication

Raw pathname totals overstate current allocated storage when files are already
hardlinked. The projectile WRAM set has 567 distinct whole snapshots totaling
74.32 MB; even against that hypothetical perfect whole-file deduplication,
the 1.09 MB page payload plus manifest is about 67× smaller. All 130 frame
snapshots differ, so whole-file sharing does not reduce that set. These values
measure logical distinct bytes, not a complete physical-inode inventory of out/.

Both ROM samples contain only one distinct 4 MiB ROM view. Existing authenticated
whole-file sharing already handles most of their repetition. Compress that
distinct blob once (about 1.50 MB at level 3) and reference it from members;
do not advertise a 89× ROM saving relative to the already shared bytes. Historical
runs can contain several actual ROM views; preserve every distinct view and its
hash rather than substituting the initial ROM.

The source tree's `out/` measured approximately 113 GiB with `du` during this
study. That broad figure includes build products and unrelated experiments;
it is not a format-only savings estimate or a frozen inventory.

## Format outline

Use two artifact kinds (or two sections in a pack) so consumers can access them
independently:

- **Events:** header with schema/profile identity; ordered binary records;
  independent Zstandard chunks; frame/sequence-to-chunk index; completion footer.
  Start with groups of 64 observed frames and about 1 MiB uncompressed input,
  splitting a busy frame across chunks as needed. Preserve an absolute starting
  frame and event sequence in each chunk. Record empty frames explicitly in the
  index and distinguish input index, hardware frame and post-frame boundary.
- **Snapshots:** member index preserving original name, capture sequence, frame,
  serializer/layout, decoded length and SHA-256; bounded compressed page pools
  and per-member page tables. Start a new independent pool at 32 members or a
  byte budget (for example 64 MiB), whichever comes first. A frame remains the
  complete original emulator/PPU/SRAM/pixel state, not just an image.

A local self-contained pool avoids a global store's garbage-collection and
evidence dependency problems. Cross-run whole-pack hardlinks remain available
when deterministic bytes actually match. Global content-addressed chunks/pages
are a later option if measurements show substantial remaining duplication.

Keep **two identities**: SHA-256 of stored pack bytes for artifact integrity and
SHA-256 of exact decoded legacy bytes (or a separately versioned canonical record
encoding) for validation equality. Current live verdicts compare stored-file
hashes, so this distinction requires explicit consumer work. Codec upgrades may
change compressed bytes while decoded evidence remains equal. Pin implementation,
parameters, chunk boundaries and dictionary policy for reproducible writers;
never rely on codec version alone as the validation definition.

Require an authenticated footer with complete frame/event/member/chunk counts,
index coverage and logical digest. A truncated stream ending at a complete zstd
frame can still decompress successfully; successful decompression alone cannot
prove a complete run. Validate exact lengths, checked u64 offsets/counts,
overflowing varints, unknown tags/layouts, full input consumption and member
hashes. Zstandard's optional checksum is not a substitute for SHA-256.

## Implementation sequence

1. Build an opt-in standalone pack/unpack/verify reader and writer, with one
   implementation exposed to Rust and the generated C host. Keep production
   behavior and historical files until the new path has passed controls.
2. Implement binary event chunks and the byte/frame index, then page-based
   snapshot members. Support exact legacy text export and single-member extraction
   so current diagnostics can be adapted incrementally.
3. Validate the writer using independent legacy captures: exact decoded equality,
   changed/reordered/duplicate events, missing frames/members, frame-counter
   resets, bit corruption, truncation inside and between chunks, and historical
   format profiles. Measure actual producer throughput, peak disk/RAM and seek
   latency; Python benchmark timings are not generated-host throughput evidence.
4. Integrate opt-in streaming writers so large raw files are never created first.
   Teach comparators, fixtures and cleanup to resolve/authenticate members and
   packs. Four independent C/mixed processes must still execute and compare.
5. Make the format default after representative full runs pass. Treat historical
   migration separately: write and verify new artifacts, preserve old verdicts
   and seals with a conversion manifest, update consumers, then consider removal
   of originals. Do not rewrite historical acceptance as if it originally sealed
   the new format.

Arrow IPC is a credible standardized alternative when columnar analysis and
cross-language tooling dominate; it supports Zstandard-compressed batches.
Parquet suits aggregate queries across runs, and SQLite suits the catalog/member
index. For today's sequential comparison plus targeted extraction, the small
binary pack maps directly onto the existing verifier. No Arrow/Parquet/SQLite
size benchmark was performed, so this study does not claim measured superiority
over those formats.

## Reproduction and evidence

The checked-in experiment is `tools/storage_format_study.py`. It reads original
captures, reconstructs every candidate byte-for-byte, and retains only compact
measurement JSON. No packed captures or temporary build outputs are saved.

```sh
python3 tools/storage_format_study.py \
  --capture-root /home/jim/sm-decomp/out/port/projectile-movement-live/20261011T061431.667466Z/c-0 \
  --output .scratch/trace-storage-study/projectile-measurements.json --trace-mib 16
python3 tools/storage_format_study.py \
  --capture-root /home/jim/sm-decomp/out/port/room-plm-current-live/20261010T224032.887967Z/c-0 \
  --output .scratch/trace-storage-study/room-measurements.json --trace-mib 8 --snapshot-limit 128
```

The first snapshot measurements were started before chunked trace comparison was
added. Separate `projectile-trace-chunks.json` / `room-trace-chunks.json` were
generated with the same commands plus `--only-traces`; they contain the bounded
trace/index results used above. Both snapshot runs include codec round trips;
the chunk runs include independent chunk round trips. Built-in controls check
duplicate/event order, changed values, frame resets, varint truncation/overflow,
truncated compressed frames and same-process encoding determinism. The snapshot
manifest is re-read after each set to reject source changes during measurement.

`source-authentication.json` records verification of the original full trace,
all sealed WRAM fixtures and all frame files against both retained verdicts;
the verdicts themselves remained byte-identical. This authenticates historical
stored artifacts, not admission against today's evolving source. Raw trace
sample offsets, SHA-256 identities, codec versions, counts, sizes and rough
Python timings are in the measurement JSON files.

See [format-research.md](format-research.md) for primary-source citations,
existing writer/comparator references and format alternatives. Scope/verification
status and cleanup are recorded in [the research ticket](issues/01-format-investigation.md).
