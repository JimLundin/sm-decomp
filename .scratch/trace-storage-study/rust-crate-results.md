# Combined Rust evidence crate

Implemented in `crates/evidence` on `research/trace-storage-format`. This is a
workspace library plus `sm-evidence` CLI, with an explicit new version-1 format.
The production capture pipeline and original evidence have not changed.

The crate reuses the other spike's 256-byte/fanout-16 Merkle model, selective image
diffs, and exact shared `(register, value)` PPU burst payloads. It combines those
with zstd block packs, compact physical Merkle references, frame directories,
bounded legacy parsing and independent observation/provenance roots. Commit uses
an exclusive writer lock, file sync, rename and directory sync. Readers check
index/block/object identities; complete-run verification checks the full closure.
Unknown records, noncanonical lines, frame resets and unfinished tails survive
byte for byte; later canonical events continue to be encoded. Frame directories
are split into blocks so the 130000-frame source fits without a giant object.

## Measurements

Decimal MB/GB; file lengths include all indexes and run objects. Source scopes
were temporarily hardlinked from original captures, never modified, and removed
in `finally`. Same selection as the previous comparison: hardware.log,
fixtures/*.wram and frame-*.bin. Frames are complete emulator state, PPUs, SRAM and
pixels, not only pictures. Exact restoration and source comparison passed for
1144 files, 5,734,473,474 bytes. These are two local measurements, not a forecast
for all future games/captures.

| Measure | Combined crate | Other Rust spike | Previous Python/C spike |
| --- | ---: | ---: | ---: |
| Large capture input | 5,264.04 MB | same | same |
| Stored large capture | **27.38 MB** | 277.52 MB logical / 612.68 MB allocated | 53.14 MB |
| Input/storage ratio | **192.3×** | 19.0× logical / 8.6× allocated | 99.1× |
| Large capture ingest + seal | 138.10 s* | 172.18 s | about 76 s |
| Full source comparison | 47.21 s* | 81.91 s | about 88 s |
| Both captures together | **27.46 MB (208.8×)** | not measured | 54.93 MB (104.4×) |
| Second capture incremental storage | 0.080 MB | not measured | about 1.79 MB |

The combined crate occupies 10.1× less file content and 22.4× less allocated disk
than the other Rust spike for the large capture. It also halves the previous
Python/C store size while retaining selective Merkle image diffs. It stores more
logical objects (189086 for the large capture), but they live in one file rather
than thousands of filesystem files. The second run reuses nearly all its pages,
bursts and segments; that strong sharing should not be assumed for unrelated runs.

*The first ingest includes full closure authentication with an initial unbuffered
text decoder. Verification and the second ingest used the subsequently buffered
reader. These implementations emit exactly the same stored format. The report
records the initial and current binary hashes and reauthenticates final closure
with the final binary. Do not present 138 seconds as an optimized converter
number. The Python/C converter remains faster at migration in this experiment.

Warm-store reads measured about **9–21 ms for trace frames** (including loading
the entire stream directory each call), and **3–5 ms for a 1.40 MB snapshot**.
Opening the store took 172 ms; one-process CLI reads including startup/index
loading took roughly 190–307 ms. Reads are slower than raw files, but seek directly
to relevant compressed data. A cached stream-directory API is a future improvement.

Direct native producer microbenchmark: **10 million synthetic events in 2.24 s**,
including canonical legacy-byte hashing and durable pack publication. It uses
repeating alternating PPU writes, a different distribution from the previous C
benchmark; the two numbers must not be interpreted as a language comparison.
`TraceWriter::push` avoids parsing legacy text but still formats canonical bytes
for its legacy restoration seal. This could be separated in a later native-only
format. **Live emulator overhead is not measured**; integration is required before
claiming frame-rate impact.

Raw measurements, exact roots and physical hashes are in `rust-benchmark.json`.
`tools/storage_rust_benchmark.py` reproduces matched-scope ingestion and exact
verification. `examples/record.rs` and `examples/read.rs` reproduce native writer
and warm-reader measurements. The retained store is
`out/storage-rust-final/store`; its trusted run ids are:

- Projectile: `ec931a1ceb3347bb6eec0f4bc52ba756490ed40df5eee4c2c35471ee7b180806`
- Room PLM: `9e1a5db7fd4aec5f9f9b20eb3b2a76e3c4cda8d9bfd9232a895f3aef3d180569`

## Verification and practical limits

Ten public-interface integration tests pass: lossless unknown/reset/tail handling,
burst ordering and duplication, busy-frame splitting, long frame directories,
odd/empty Merkle images and selective diffs, changed/reordered observations,
independent provenance, append/reopen/abort, writer/read-only ownership, stale
locks, symlinks, corrupted/truncated/missing packs, malformed objects/ids and CLI
ingest/cat/frame/extract/verify. Cargo fmt, focused Clippy with warnings denied,
and `git diff --check` pass. Both real capture closures authenticate, every source
member matches by SHA-256/length, and all original byte counts match.

Format/lifecycle details are in `crates/evidence/README.md`. This implementation
supports one writer and keeps object indexes in memory. Pack ordinals require
exporting the complete ordered store. No automatic garbage collection/repack,
stale-lock recovery, live C integration or acceptance-verdict integration is
included. A completed run is storage completeness, not a validation verdict.
