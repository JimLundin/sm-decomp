# Binary Offline family fixtures

Files in `tools/port/*.smf` are authoritative Fixture inputs. The catalog and
source inventory remain small TOML documents. Verification, discovery and progress
open the binary format directly; TOML Fixture input is available only through the
explicit import command.

The format removes TOML parsing and millions of small byte-vector allocations.
One opened file backs all write spans. Import and generation resolve overlapping
writes, combine nearby spans and share identical payloads. Execution uses ordinary
slice copies without decompression.

## Inspect, edit and compare

```sh
cargo run -p pipeline --locked --offline --bin sm-pipeline -- fixture inspect tools/port/system.smf
cargo run -p pipeline --locked --offline --bin sm-pipeline -- fixture inspect tools/port/system.smf CASE_NAME
cargo run -p pipeline --locked --offline --bin sm-pipeline -- fixture inspect tools/port/system.smf > /tmp/system-edit.toml
# Edit /tmp/system-edit.toml, then import it atomically:
cargo run -p pipeline --locked --offline --bin sm-pipeline -- fixture import /tmp/system-edit.toml tools/port/system.smf
cargo run -p pipeline --locked --offline --bin sm-pipeline -- fixture diff BEFORE.smf AFTER.smf
```

Text exports are temporary views, not a second checked-in source. Rust generators
use the same binary writer as import. An unchanged inspection/import round trip
produces identical bytes. `diff` lists changed, added and removed cases and reports
changes to family metadata, routines or case order.

The standalone camera and projectile-trail producers also publish `.smf` through
the pipeline writer. Their fixture commands take the pipeline executable explicitly:
`camera-scroll-inventory fixtures PIPELINE OUTPUT.smf` and
`projectile-trails-producer fixtures CHECKOUT ROM PIPELINE OUTPUT.smf`.
Captured-fixture commands document the same argument in their usage output.

For readable Git diffs, configure the driver declared by `.gitattributes`:

```sh
git config diff.smfixture.textconv 'cargo run --quiet -p pipeline --locked --offline --bin sm-pipeline -- fixture inspect'
git diff -- tools/port/system.smf
```

## Layout

All integers use little endian. Offsets and lengths are checked before any shared
span is exposed. The current format is `SMFIXT01`; incompatible schema changes
require a new magic/version.

| Header offset | Bytes | Meaning |
| --- | --- | --- |
| 0 | 8 | `SMFIXT01` |
| 8 | 8 | Typed metadata length |
| 16 | 8 | Case count |
| 24 | 8 | Write-span count |
| 32 | 8 | Payload length |
| 40 | 24 | Reserved; must be zero |

Four contiguous sections follow the header:

1. Typed metadata in the explicit `Wire` field order in
   `crates/offline-verification/src/fixture_format.rs`. The public
   `offline_verification::fixtures` interface owns authoring and codec access.
   Strings and vectors have a 32-bit
   length, optional values have a one-byte presence tag, and booleans are 0 or 1.
   Machine-sized fields are encoded as 64-bit integers. No textual document is
   embedded. All Manifest, Routine and Case fields are preserved except writes,
   which are represented by the following sections.
2. A 16-byte directory entry per case: first span index and span count, both
   64-bit. Top-level cases precede cases in routine declaration order.
3. A 16-byte record per span: WRAM offset (32-bit), byte count (32-bit), and
   offset within the payload section (64-bit). Each case's spans are ordered,
   nonempty and nonoverlapping, and fit canonical 128 KiB WRAM.
4. Shared raw payload bytes. Identical spans can refer to the same byte range.

Roots start with their declared fill, defaulting to zero. Import removes writes
equal to that fill and bridges gaps of at most 16 known fill bytes when combining
spans. Continuations retain every explicitly written byte, including zeros, and
never bridge untouched gaps: those bytes belong to the verified predecessor.
Captured `before_file` inputs remain unpatchable and keep their exact path and
SHA-256 identity. Expected captures, arguments, return values, host words, case
order, adapter declarations and dependencies keep their existing meanings.

The reader rejects invalid magic, flags, lengths, counts, payload references,
overlap and truncated or trailing data. Existing SHA-256 source and evidence
seals authenticate the complete binary file. Fresh admission and later reuse
still authenticate current inputs.

## Verification records

New family runs retain `manifest.smf` and a compact `case.sha256` identity per
case, along with the existing before/after WRAM evidence. Prepared native work
serializes its authenticated source contract and normalized fixture identity;
the decoded Manifest is an in-memory cache and is reopened for each operation.
Fixture task lookup is built once during authentication, avoiding repeated
whole-family ordering for individual producer records. Producer checks share
one parsed native intent per authentication.

Native receipts require indexed producer attribution and explicit candidate
ownership. Completed producers release unused reservations before more work is
admitted. There are no inline-receipt, old-reservation or implicit single-module
identity modes. Receipt enumeration and byte accounting each authenticate the
index and its chunks in one pass.

Producing verification results costs full WRAM writes and SHA-256 checks. Bounded
reads return a single snapshot for hashing and state comparison; Fixture
admission similarly authenticates the exact bytes it decodes. Authentication
reuses binary identities from its fresh artifact inventory and reopens current
inputs on each operation.

This removes half the captured-input read volume and one third of the ordinary
root-case WRAM-authentication read volume, including its inventory pass. Warm-cache
timing gains were modest and noisy; these measurements do not establish a complete
verification speedup. Details are in
`.scratch/fixture-format-performance/evidence-audit/README.md`.

Historical formats and results are not supported. The migration executables,
obsolete readers and archived runs created for this format study were retired;
current Rust tests exercise the supported format and verification contracts.

## Migration evidence

The complete 177-family migration compared all metadata and full modeled WRAM
for 149,120 cases, including 6,956 continuations. The 1,123 captured cases retained
their asset references and were checked with varied arbitrary capture inputs;
the migration did not rewrite or reauthenticate those external captures.

Source fixture size fell from 272,739,363 to 40,045,625 bytes. Final-source
warm-cache read/SHA-256/decode medians for floater-spike-motion,
mother-brain-bomb-update and ridley-subsystem were 2.52, 3.20 and 2.50 ms
over five repetitions. The original TOML medians were 2,924.57, 1,069.33 and
635.93 ms over three repetitions. Other machine work was active.
These measure input loading, not complete actual-C verification or gameplay.
Measurements and cleanup records are in
`.scratch/fixture-format-performance/`.
