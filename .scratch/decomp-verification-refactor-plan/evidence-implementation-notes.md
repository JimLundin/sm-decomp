# Evidence prototype implementation inputs

Status: source observations for tickets 05/06; no prototype result yet
Source: extracted production freeze `4a736587af557de9e44b3126ff08f6d6d11beee8`

## Reuse existing storage code

`evidence::Store` already supports typed payloads, authenticated reads, private
writers and `import_store`. Import re-encodes store-local ordinals; no pack copying.
Its current block target is 1 MiB, maximum block is 4 MiB, and index ceiling is
256 MiB. Each write block uses Zstandard level 3. Each read handle caches eight
decoded blocks, authenticating actual payload identities. One mutable Store is
not a thread-shared writer.

`put_image` splits an image into 256-byte pages with fanout 16. A 128-KiB WRAM
state therefore visits 512 pages before tree nodes. Compare this with one raw
`BLOB` object per state and larger independently authenticated chunks; do not
infer the fastest choice from deduplication ratio. Include commit's physical
hashing, reopened verification, import and extraction in the prototype.

Existing `pipeline::storage_studies` supplies bounded Rust study receipts,
disk/inode accounting, retained reader capture and extraction checks. Its current
benchmark assumes Live `hardware.log` and snapshot directories, so it is not an
Offline benchmark without an explicit current Offline input adapter. The
`evidence` crate's integration tests cover missing/corrupt objects and import.
Do not revive the historical Python benchmark or historical study readers.

## Current Offline reads

`family::case` holds actual Rust/C state bytes while comparing them, after the
producer wrote separate files. It separately hashes `before.wram` and records
return/state/capture hashes. Whole-run `inventory` later reads artifacts again.
Independent admission inventories the run at its start, then rereads state and
return members for comparison and chain checks. Recipe checks also read their
own host/callback/PLM/Ridley members. Optimize all complete logical observations;
a simple generic WRAM-only record cannot represent these recipes.

Packing a finished directory cannot claim to eliminate the initial producer
write/read cycle. A framed output transport needs actual recipe/producer hooks,
bounded collector accounting and Native output-capability integration. Start with
the smallest complete real recipe and compare raw outputs exactly. Keep the
staging-only and direct-output experiments separately labeled.

A possible narrower candidate is a private admission reader that fuses member
hash collection and semantic comparison, retaining bounded owned bytes for the
active case. It must still authenticate every listed member, reject extra or
missing files, preserve normalized captured-event identities, and perform source
freshness reads outside the cache. A whole-family byte cache can exceed memory
on large families; do not substitute an unbounded map for repeated I/O.

## Historical measurements are context only

The old trace-storage and decomp-pipeline studies showed that compact Live packs
can add capture/sealing cost and that a shared writer lock can serialize workers.
They concern old source, different workloads and uncontrolled host activity.
Use them to select controls, never as current Offline speedup measurements.

The current host has eight logical CPUs, about 12.5 GB RAM and existing swap use.
The mutation session and the extraction's frozen-source gates are active.
Coordinate controlled samples after those workloads settle; any earlier sample
must record contention. No shared cache, old evidence or active run may be cleared.

## Cleanup

This note used source/document reads only and created no execution artifacts.
