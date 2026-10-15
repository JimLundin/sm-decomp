# Evidence prototype implementation inputs

Status: ticket 05 resolved; source observations for the selected ticket 06 reader
Source: extracted production freeze `4a736587af557de9e44b3126ff08f6d6d11beee8`

The completed storage experiment selected existing directory/operation-index
formats with bounded payload reuse inside each independent admission. Its exact
measurements and retirement are in [ticket 05](issues/05-offline-evidence-prototype.md).
The storage alternatives below remain research context, not selected production
changes. Native operation reuse is implemented and its combined gates are active;
see [Native recovery](native-finalization-recovery.md).

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

## Current ordinary reader implementation guidance

These additional observations use the frozen assembly `7b27ea1` and do not
change its source. They prepare the ticket 06 implementation after the captured
input/identity interfaces are ready.

`family::authenticate_contents` currently inventories/hashes the entire run,
decodes the declaration and authenticates sources, then checks every case.
`inventory_bounded` omits the verdict and existing metrics sidecars and bounds
file count, discovered paths and depth. Preserve those exact membership rules.
`inputs::artifact_digest` permits the one large original Ridley event snapshot
only for its authenticated captured declaration and exact known hash; it is an
exception for streaming hashing, not permission to retain that asset in a case
cache or enlarge all file-read bounds.

The comparison owners mix generic bounded readers and direct `fs::read` calls:

- `family.rs`: case declaration identity, WRAM, return text, captured output,
  projectile host and predecessor WRAM/host checks.
- `callback.rs`: output metrics, the complete ordered C/Rust callback file sets,
  full WRAM and captured output. `callback_files` currently loads each complete
  set into a map; convert paired-member comparison to bounded windows rather than
  retaining all callback images in another whole-family cache.
- `plm_spawn.rs`: metrics, input/output WRAM, expected flags/amount and captures.
- `ridley_subsystem_adapter.rs`: WRAM, host, PPU, ordered effects/services,
  independent captured companions and predecessor inheritance. Its current
  `check` reads several files then hashes/reads them again. External original
  capture freshness remains a separate obligation.

A small private retained-reader Module can separate namespace discovery from
payload validation, authenticate each consumed member while reading it, and
explicitly authenticate all unconsumed members before success. If expected
digests are used during early metadata checks, they are still untrusted until
that final complete admission succeeds. An owning case reader can serve the
same immutable bytes and hash to recipe comparison; no process-global path/hash
cache or unbounded run payload map is appropriate. Source/compiler/input reads
outside the admitted retained run must keep their independent freshness checks.

Actual continuations may patch inherited WRAM through `case.writes`; matching
only the predecessor digest cannot replace applying those writes and comparing
the resulting complete bytes. Branching continuations can revisit an older
predecessor. Bound any retained predecessor frontier explicitly, or use fresh
bounded predecessor windows and account for their rereads separately. Preserve
host and PPU inheritance as well as WRAM. A later independent admission always
starts a fresh reader, regardless of any prior invocation cache.

Keep existing mismatch diagnostics and missing/corrupt/extra-member refusals.
Budget/path/deadline errors cannot become cache misses that bypass validation.
Use the public actual-C family and recipe controls, plus a read-byte structural
control on complete retained outputs. Counters must distinguish repeated reads
inside one case window from a necessary independent admission or predecessor
window. Integrate source stamps for the new reader so it remains part of the
verification implementation identity.
