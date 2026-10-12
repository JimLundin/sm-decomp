# Evidence store: content-addressed trajectories

Status: open

## Problem

Oracle-vs-port evidence under `out/` has grown to 111 GB (2026-10-11), with 82 GB
in `out/port`. Measured with hardlinked inodes counted once:

| Artifact | Size | Files |
|---|---|---|
| `fixtures/*.wram` snapshots (128 KiB each) | 36 GB | 275k |
| plain `hardware.log` event streams | 25 GB | 858 |
| `*.json` (mostly per-case batch/run files) | 7 GB | 285k |
| `*.rom` views | 6.5 GB | 1.5k |
| `hardware.log.gz` | 3.9 GB | 41 |
| `frame-*.bin` periodic state | 3.5 GB | 2.5k |
| build products | ~16 GB | — |

Today's formats store every observation as an independent flat file: text
lines, or full memory images. Gzip and hardlinks paper over this. The data
itself has much more structure than that.

## What the data actually is

The evidence is a **deterministic trajectory**: a pure function of
(ROM, inputs, binary). The trajectory is observed as memory images and as
writes to hardware ports. Measured on
`projectile-movement-live/20261011T061431.667466Z/c-0`, plus a scan of `out/port`:

1. **Runs repeat each other.** All four variants of one run write byte-identical
   5 GB event streams. Across `out/port`, 21.8 GB of distinct plain logs come in
   only 9 distinct sizes (9.2 GB), so most are likely the same movie trajectory
   recomputed and stored again. When verification passes, the port side
   duplicates the oracle side by definition.
2. **State changes sparsely over time.** 870 WRAM snapshots (114 MB) contain only
   3,059 distinct 256-byte pages. Consecutive snapshots differ in about 25 of 512
   pages.
3. **Event streams are mostly block transfers of recurring payloads.** 93% of
   event lines are data-port bursts (VMDATA 24/25, OAMDATA 4, CGDATA 34), meaning
   DMA uploads written out one byte per line. A 1 GB prefix holds 126,592 bursts
   but only 9,774 distinct payloads (20.8 MB). The same graphics are uploaded
   over and over.
4. **Whole frames do not repeat** (only 3× from frame-level dedup), so the
   redundancy sits below frame level, in pages and payloads.

Generic compression finds some of this by accident (gzip 29×, xz 63×, within
one file only). A format that models (1)–(3) directly gets it by construction,
across files and runs, and makes comparison cheaper as well.

## Design: a content-addressed object store

The model is git's object store, specialised for SNES evidence: one store for
all of `out/` (`out/store/`), with immutable objects named by the sha256 of
their canonical encoding.

### Objects

- **`blob`**: raw bytes. Holds a 256-byte memory page, a DMA payload, or a small
  file (ROM, source snapshot).
- **`image`**: a memory image as a fixed-fanout Merkle tree of page ids. The
  128 KiB of WRAM becomes 512 page ids under a small tree, and a ROM view
  becomes a tree too. A new snapshot costs only its changed pages plus a few
  tree nodes. Two images are equal exactly when their root ids are equal, and a
  diff walks down only the subtrees that differ. That replaces the byte-by-byte
  `cmp`/`read_bytes()` comparisons in `port_live.py` and `diff`.
- **`events`**: one segment of a port's event stream, as binary typed records.
  `Write{frame Δ, port, reg, value}` covers scalar register writes.
  `Block{frame Δ, port, target, start, len, payload: blob id}` covers a burst
  to a data port. The writer detects bursts, so the C host's `PortEffect`
  calls don't change. A stream is a list of segments, each N frames long.
- **`trajectory`**: the run's index. It records ROM id, inputs id and binary id,
  and per frame the event-segment id plus any image ids captured at that frame
  (periodic `frame-*.bin`, decompressor before/after, terminal WRAM/SRAM). This
  replaces `calls.jsonl` filename pointers and the `fixtures/` directory.
- **`run`**: the sealed root. It references the trajectory, the verdict fields
  and the build/log blobs.

### Sealing and determinism

The seal is the `run` object id: the sha256 of its canonical encoding, which
transitively covers every object it reaches. That's a hash of the new format
itself, and it's deterministic by construction:

- Each object kind has exactly one canonical byte encoding: fixed-width
  little-endian integers or LEB128 varints, fields in a fixed order, and
  lists in trajectory order or sorted. No timestamps, no maps.
- Burst detection and segment boundaries are pure functions of the event
  sequence.
- Ids are computed over the canonical encoding before any storage
  compression. On-disk zstd for large objects is a storage detail that can
  change without changing any seal.

Identical trajectories produce identical ids, so cross-run sharing is automatic
and the hardlink machinery in `port_live.py`/`port_cleanup.py` is no longer
needed. The 9 distinct log sizes above would collapse to at most 9 trajectories,
and those would share most of their payloads.

### Expected size

Taking the measured run's 1 GB event prefix: 20.8 MB of unique payloads, plus
about 126k block records, plus about 4M scalar writes as varints. That's
around 40× before any codec, and payloads are shared across every run of the
same movie. WRAM: 3,059 pages, about 0.8 MB plus tree nodes, against 114 MB.
zstd on stored objects stays available for leftovers (logs, JSON). It isn't the
mechanism.

### Option: don't store what can be regenerated

Because the trajectory is a pure function of (ROM, inputs, binary) and verdicts
already keep the linked binary, a run could store just the per-frame object ids
(32 B/frame) plus keyframe images every K frames, then re-materialize events or
images on demand by replaying from the nearest keyframe. Then a passing run
costs almost nothing beyond its digest chain. This is a policy on top of the
store, not a different format, and it's deferred until the store exists.

### Interfaces

`crates/artifact` holds the encoder and store, written once:

- as a Rust library for `snes`, `diff`, `decomp` and `port`.
- as a C ABI `staticlib` linked into the generated hosts. It replaces the
  `fopen`/`gzopen` effect writer and the `fwrite(g_ram)` snapshot writers in
  `tools/port_live.py`.
- as the `sm-evidence` CLI: `put`, `cat <id>` (writes back the legacy text log or
  raw image byte-for-byte), `diff <id> <id>`, `verify <run>`, `convert <dir>`,
  `gc`. Python tools call the CLI, so Python 3.8 needs no new bindings.

### Alternatives considered

- **Compress existing files (gzip/zstd/xz).** Within-file only, the format stays
  opaque, comparisons still decode everything, and no cross-run sharing. Rejected.
- **Parquet/Arrow for events.** Columnar dictionary/RLE/delta encoding fits scalar
  writes and DuckDB can query it. But it has no content addressing across
  files, no natural form for memory images, and byte output depends on the
  writer version. Columnar segment layout can still be revisited inside
  `events` objects.
- **Generic dedup backups (restic/borg/casync).** These give chunk dedup but are
  opaque. They don't model images or bursts, and their ids aren't stable seals.

## Migration

`sm-evidence convert` walks the accepted runs and ingests each artifact. It
checks that `cat` reproduces the old bytes against the old sealed sha256, writes
a `run` object recording each `converted_from` name and sha256, and only then
removes the originals. Acceptance pointers move to run ids.

## Out of scope

- Build products (~16 GB), which `tools/port_cleanup.py` already handles.
- Committed `tools/port/*.json` fixtures (53 MB in git).
