# Spike: content-addressed evidence store

Status: resolved
Type: prototype

Question: does modelling the evidence's structure (pages, upload bursts,
segments, runs) shrink real `out/port` evidence by orders of magnitude while
staying lossless and deterministic?

Answer: yes. Four real `projectile-movement-live` runs take 8.8 GB on disk
today (already hardlink-deduped; 32.6 GB logically). They become 305 MB of
canonical objects, or ~63–70 MB once each object is compressed. Every file
re-materializes with an identical sha256.

## Prototype

`crates/evidence` (std + `sha2`), binary `sm-evidence`:

- `ingest <store> <dir>...` stores each directory as a run tree and prints its root id (the seal)
- `verify <store> <id> <dir>` re-materializes every file and compares sha256 and length
- `cat`, `ls` and `diff-image` show stored objects

Object kinds (id = sha256 of `kind ‖ canonical payload`):

| Kind | Content |
|---|---|
| `B` blob | raw bytes: 256-byte page, upload payload, whole file, stream tail |
| `N` node | 16 child ids (image Merkle node) |
| `I` image | length, height, root node id |
| `G` segment | 64 frames of binary event records: scalar `Write` or `Block` → payload blob |
| `S` stream | line count, segment ids, optional verbatim tail blob |
| `T` tree | sorted `(name, entry kind, id)` |

Each trace line is re-rendered and compared byte-for-byte during ingest. Parsing
stops at the first non-canonical line, and the rest of the file is kept as a
verbatim tail. One interrupted run ends mid-line (`ppu 43187 0 34 15` with no
newline), and its tail costs 17 bytes. Without this handling, the first version
stored that 1.7 GB log as a single opaque blob.

Tests: `cargo test -p evidence` (round trip, cross-run identity, Merkle diff reads
only one path, burst sharing, canonical rejection, verbatim tails).

## Measurements

Input: `out/port/projectile-movement-live/*` (4 runs, 6,216 files,
traces up to 5 GB, 870 WRAM fixtures per variant).

| | Bytes |
|---|---|
| On disk today (`du`, hardlinks counted once) | 8.8 GB |
| Logical input (sum of file sizes, hardlinks counted each time) | 32.6 GB |
| Unique canonical objects | 305 MB |
| …with each object compressed (zlib-6 / xz-6 stand-ins for zstd) | 69.5 / 63.1 MB |

The shared column below is content that a later file or run referenced again
instead of storing:

| Kind | New objects | New MB | Shared refs | Shared MB |
|---|---|---|---|---|
| blob | 113,584 | 164.0 | 27,275,251 | 8,838 |
| segment | 1,897 | 134.0 | 8,578 | 571 |
| node | 12,964 | 6.3 | 1,596,402 | 816 |
| image / stream / tree | 728 | 0.3 | 5,397 | 0.6 |

Where the remaining compressed bytes go:

- Segments: 134 MB raw becomes 5.5–6.9 MB compressed. 21.9M scalar writes
  dominate the raw size, mostly BG3 scroll clears (`$2111`/`$2112`) repeated
  16× per frame. A codec handles that short-period repetition well.
- Upload payloads: ~126 MB raw (59k distinct) becomes about a third of that
  compressed. This is genuinely distinct graphics data.
- Whole files: 23.7 MB, mostly the linked `live` binaries (17.7 MB), which
  differ per run.
- WRAM pages and nodes: about 13 MB.

Cross-run: the three earlier runs added 1 new segment and shared 1,801. Their
cost was almost entirely their own binaries and the not-yet-seen pages.

Throughput: ~40 MB/s of trace text on one core (545 s for 4 × 5 GB). The time
goes to text parsing, which disappears once the C host writes records directly
(see below).

## Findings that change the design

1. **Loose objects are not viable on disk.** 129k files occupy 684 MB in 4 KiB
   blocks, against 305 MB of content. A real store needs pack files: append-only,
   with an id → offset index and per-object zstd. That's a storage detail and
   doesn't change ids.
2. **Seals are independent of the codec.** Ids hash canonical payloads, so
   choosing zlib, xz or zstd (or a dictionary) is purely a storage decision that
   can change without invalidating evidence.
3. **Text is the wrong boundary.** Parsing text costs most of the ingest time and
   needs the canonical-form/tail machinery. Emitting `Write`/`Block` records
   straight from the C host's `PortEffect` removes both. Text `hardware.log`
   remains available through `sm-evidence cat`.
4. **The register byte in upload payloads is redundant.** Payloads store
   `(reg, value)` pairs, but the register pattern is almost always
   alternating 0x18/0x19 or constant. Splitting it into a pattern field
   halves raw payload size. The codec already captures most of this, so it's
   low priority.
5. **Near-duplicate payloads aren't shared.** Exact dedup misses uploads that
   differ by a few bytes. A zstd dictionary per pack is the cheap next step;
   delta objects are the expensive one.
6. **Linked binaries are now a significant fixed cost (~6 MB per run).**
   Regenerate-on-demand policy would build on the store, not replace it.

## Not tested

- zstd itself (Python 3.8 has no binding here, and the crate isn't vendored).
  zlib/xz were used as lower/upper stand-ins.
- Other run families (`batches`, `controller-*`, `decomp`, `runs`), JSON-heavy
  batch directories (67k JSON files) and the `sync` CSVs.
- Concurrent writers, garbage collection and verdict/acceptance integration.
