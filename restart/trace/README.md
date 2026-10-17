# Compact reference traces

`sm-trace` stores three SHA-256 hashes per ordered native observation: canonical
128 KiB WRAM, caller-supplied external-state bytes, and caller-supplied ordered
effect bytes. It never persists full per-step state. The caller defines and
identifies the exact observed byte schema; empty external/effect slices mean
those components were empty, not that unobserved hardware was verified.

## Interface

```rust,ignore
use sm_trace::{Identity, Reference, Writer};

let identity = Identity {
    reference: fixed_c_library_sha256,
    rom: pristine_and_patched_rom_identity,
    host: fixed_schedule_and_host_identity,
    input: native_input_bytes_sha256,
    observation: explicit_byte_schema_identity,
    scenario: scenario_identity,
};

// Actual C execution, with no candidate identity in this reference key.
let mut writer = Writer::create("reference.trace", &identity)?;
writer.record(1, &wram, &external_state_bytes, &ordered_effect_bytes)?;
let sealed = writer.finish()?;

// Authenticate the complete stored reference before running/comparing a candidate.
let reference = Reference::load("reference.trace", &identity)?;
assert_eq!(reference.summary(), sealed);
let mut comparison = reference.comparison();
comparison.check(1, &candidate_wram, &candidate_external, &candidate_effects)?;
let agreement = comparison.finish()?;
```

Each `Identity` field must be nonempty and match exactly. Supply canonical
composite identity strings when a field needs more than one hash. The caller
must keep candidate identities out of these reference fields. This crate does
not generate fixtures, execute C, interpret game state, choose comparison masks,
or establish required Rust ownership; those remain the caller's responsibility.

Ordinals start at 1 and are consecutive. `Writer` creates a new path and refuses
to overwrite existing files. `finish` writes the footer and flushes/synchronizes
the completed file. An error poisons the writer, and unfinished files remain
invalid rather than becoming accepted partial references. No empty trace passes.

`Reference::load` validates the entire file, schema, checksum, identity, count
and record order before exposing hashes. It retains only immutable expected
hashes, so later file changes cannot alter an active comparison. `summary()`
returns the count and full-file SHA-256; callers can pin this digest separately
when retaining a trusted producer result. The embedded checksum protects file
integrity and is not a digital signature or independent proof of its producer.

`comparison.check` reports the first differing ordinal or component, checking
WRAM, external state, then effects. It never searches ahead. Failures stick even
if callers try to continue after an error. `finish` rejects missing trailing
records; checking after the last expected record rejects extra observations.
Always call it before claiming agreement. Successful completion returns the
authenticated reference summary, not a claim about unobserved state.

`Error::difference()` exposes `ordinal`, `component`, `expected`, `actual`,
`expected_ordinal` and `actual_ordinal`. Component hashes are present for state
or effect mismatches; ordinal fields explain missing, extra or reordered records.
`Reference::expected(ordinal)` exposes all three expected hashes for diagnostics.
The caller can rerun to capture full state at that exact point.

## Version 1 file format

All numeric fields are unsigned little-endian. Hashes are raw 32-byte SHA-256.
The complete file is capped at 64 MiB; loading reads at most that limit plus one
byte even if its length changes while being read. Identity JSON is capped at
16 KiB and rejects missing, duplicate or unknown fields.

| Field | Bytes | Meaning |
| --- | --- | --- |
| Magic | 8 | ASCII `SMTRACE1` |
| Identity length | 4 | Number of following JSON bytes |
| Identity | Variable | The six declared `Identity` string fields |
| Records | 104 each | Ordinal (8), WRAM hash (32), external hash (32), effects hash (32) |
| Footer magic | 8 | ASCII `SMEND001` |
| Record count | 8 | Nonzero number of preceding consecutive records |
| Checksum | 32 | SHA-256 of every preceding byte, including header and footer count |

There is exactly one footer at end-of-file, with no trailing bytes or sidecar.
The count must match the payload's exact extent. `Summary::sha256` hashes the
whole file including the embedded checksum, permitting an independent retained
seal to bind these expected hashes.

## Worker checks and integration

This worker crate temporarily has its own `[workspace]`. Integration removes
that section, adds `trace` to the restart workspace, and moves its two profile
overrides to the workspace root:

```toml
[profile.dev.package.sha2]
opt-level = 3

[profile.dev.package.digest]
opt-level = 3
```

The dependency optimization matters because every observation hashes all WRAM.
The crate itself and game code can remain in ordinary debug builds. Root's
workspace lockfile should own dependencies after integration; the worker lockfile
records the independently checked package set.

```sh
cargo test --manifest-path restart/trace/Cargo.toml --locked --offline -j1
cargo clippy --manifest-path restart/trace/Cargo.toml --locked --offline -j1 --all-targets -- -D warnings
cargo fmt --manifest-path restart/trace/Cargo.toml --check
```

Ten focused tests cover compact round-trip, all six identity fields, every
single-byte corruption/truncation of a small reference, appended data, malformed
count/order even after checksum recomputation, each state/effect mismatch,
ordered candidate comparison, missing/extra candidate records, partial writers,
invalid WRAM length, existing-file preservation, immutable loaded expectations,
and oversized input rejection. Tests remove their own private files.

The `hash-cost` example measures one fixed 10,000-step synthetic workload. Its
external state is 64 KiB and effects are 32 bytes per step. Pass a fresh output
path whose parent exists. This example measures storage/hash overhead only;
it is not a gameplay or original-ROM comparison.
