# Evidence storage

A Rust workspace library and `sm-evidence` CLI for lossless validation captures.
It combines the other trace-storage spike's 256-byte Merkle images and literal
PPU burst pairs with compressed, indexed immutable packs. It is independent of
the capture pipeline; existing evidence is not migrated automatically.

```sh
cargo build -p evidence --release
sm-evidence ingest STORE CAPTURE_DIRECTORY [PROVENANCE.json]
sm-evidence verify STORE RUN_ID [CAPTURE_DIRECTORY]
sm-evidence cat STORE RUN_ID frame-000000.bin > restored.bin
sm-evidence frame STORE RUN_ID hardware.log 60 > frame-60.log
sm-evidence extract STORE RUN_ID NEW_DIRECTORY
sm-evidence stats STORE
```

Ingestion prints the observation root, completed run id and physical pack seal.
Retain the run id externally as the trusted pointer. A completed run proves its
referenced closure is readable and internally consistent; the validation system
must still decide whether those observations are accepted. `verify` authenticates
physical packs and the entire run closure, and optionally compares exact bytes
and directory membership against the original capture. Extraction creates a new
directory and never overwrites existing files. CLI errors can leave partial output
in that new directory; delete or inspect it before retrying.

The library exposes `Store`, Merkle image writes/diffs, chunked files, typed event
recording, full/frame trace reads, directory trees and completed runs. For native
producers, `TraceWriter::push(Event::Ppu { ... })` avoids legacy text parsing;
`literal` retains unknown records and explicitly associates them with a frame.
`finish` returns a stream id; `Store::commit` publishes the pack. Uncommitted ids
are usable by that writer but unavailable to other processes. The caller must
publish a run pointer only after commit succeeds.

## Logical format, version 1

Object ids are SHA-256 of `SM-EVIDENCE\0`, version byte `1`, kind byte and canonical
payload. Unsigned integers use minimal unsigned LEB128; references are 32-byte
ids. Logical identities do not depend on compression or pack location.

| Kind | Payload |
| --- | --- |
| `B` | Literal bytes, at most 2 MiB |
| `N` | Ordered Merkle children, 16 at most for an image |
| `I` | Byte length, minimal tree height, root id if nonempty |
| `F` | Byte length, chunk count, ordered blob ids; fixed 1 MiB chunks |
| `G` | One-frame event segment: tag `0` + device/register/value; tag `1` + port/value; tag `2` + device/burst-blob id |
| `D` | Up to 1024 ordered (typed flag, frame-presence flag, optional frame, id) entries |
| `S` | Legacy byte/newline/event counts, SHA-256 of restored bytes, segment count, ordered directory-block ids |
| `T` | UTF-8 names in byte order, name length/name, entry kind (`f/i/s/t`), id |
| `R` | Completed flag `1`, observation-tree id, canonical provenance-JSON blob id, file/byte counts |

Images use 256-byte pages and fanout 16. Final pages keep their exact length.
`diff_images` visits only unequal branches and returns differing page offsets for
images of the same size. Equal root comparisons assume previously authenticated
closures; equality alone cannot detect a missing object.

Event segments hold at most 32768 events. Consecutive PPU writes to data ports
4, 24, 25 or 34 with the same device become a burst at 16 writes. The shared
payload contains observed `(register, value)` pairs, preserving alternating ports,
repeated values, ordering and interleaved scalar/APU events. Ingestion recognises
canonical `ppu` and `apu` lines. Other lines, noncanonical decimal formatting,
unknown bytes and interrupted tails remain literal in blocks of at most 64 KiB.
Frame decreases are supported. Very long literal lines are streamed in bounded
pieces and remain unassigned to frames. Frame reads include explicitly assigned
literal records; the full stream preserves every byte, including unassigned ones.
Native and legacy ingestion can group literal bytes differently; compare restored
SHA-256 when comparing these representations. Typed-only input receives the same
stream root through both interfaces.

Directory ingestion rejects symlinks and special files, supports UTF-8 names and
64 levels of nesting. Snapshot suffixes `.wram/.rom/.bin/.srm/.sram` use images;
`hardware.log` and `routine-calls.log` use trace encoding; other files use chunks.
Limits are 1 GiB per image, 60000 chunks per generic file (about 60 GiB), 2000000
trace segments, 2 MiB per logical object and 256 MiB per pack index. A store's
indexes are held in memory, so store size is not an unlimited-memory promise.

## Physical packs and lifecycle

A single writer owns `writer.lock` using exclusive creation. Readers never remove
that lock or modify files. The writer appends to a `.pending` file; normal aborts
remove its pending file and lock. A process crash can leave them behind. Readers
ignore pending files. After confirming the process has exited, an operator may
remove its lock and pending file; committed packs remain usable. No automatic
stale-lock stealing is provided.

Each pack starts with `SMEPACK1`, then zstd level-3 blocks of about 1 MiB. Each
physical object has a flag byte plus payload. Merkle-node references become
little-endian 32-bit store ordinals; readers expand them to canonical ids and
verify object identities. The index is `SMEINDEX`, initial ordinal/count (u64 LE),
then 89-byte entries: id(32), kind(1), block offset(8), compressed/raw lengths(4
 each), raw-block SHA(32), object position/length(4 each). The 80-byte footer holds
index offset(8), index SHA(32), preceding-body SHA(32), `SMEEND01`(8). The filename
is pack sequence plus full physical SHA. Index seals and bounds are checked on
open; raw-block SHA and canonical object ids are checked on read; `verify_packs`
also authenticates the body and filename seals. An 8-block LRU bounds decoded
block caching. Duplicate insertion authenticates existing content before reuse.

Commit writes the index/footer, syncs the file, renames it into the sealed pack
namespace, and syncs the directory. Readers should reopen to see new packs.
Physical ordinals require packs in sequence: export the complete store, not an
arbitrary pack subset. Garbage collection, repacking, concurrent writers,
automatic recovery, and live C/emulator integration are future work. SHA-256
identities provide integrity against a trusted external root, not signatures or
protection against a writer that can replace both evidence and its trusted pointer.
The version is a new format and does not read either spike's old physical files.

See `.scratch/trace-storage-study/rust-crate-results.md` for measurements and the
exact comparison scope. `tools/storage_rust_benchmark.py` reproduces the real
capture benchmark; `examples/record.rs` measures direct native event recording.
