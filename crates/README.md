# Crates

The rebuild's crates, as planned in
[ARCHITECTURE §4](../docs/architecture/ARCHITECTURE.md#4-packages-and-ownership) (D1).

| Crate | Owns | May depend on (workspace) |
| --- | --- | --- |
| `data` | IDs and stable keys, the ROM decoder and its tables, typed content, profiles, the portable records, the address map, the shared digest. No mutable state or rules. | `canonical`, `clean_types` (the asset key is the clean room's pose) |
| `game` | the `Session`, the `State`, the Tick and every gameplay domain. No pixels, samples, files, devices or addresses. | `data`, `canonical`, `clean_types`, `clean_model_samus_motion`, `clean_model_camera` |
| `runtime` | the line compositor, presentation, sound backends, storage. Feeds nothing back into the Session. | `data` |
| `app` | the run coordinator, Tick admission, the input profile, effect delivery. No gameplay. Its binary is `super-metroid`. | `data`, `game`, `runtime` |
| `oracle` | the emulator adapters, `Oracle`, seeds, the lock recorder; re-exports `oracle_record` as `oracle::record`. | `data`, `oracle_record` |
| `oracle_record` | `FrameRecord` and the types in it, the lsnes raw record decoder, the wait sites and the pad file reader, with no emulator: linking `oracle` needs the emulator archive. | none |
| `verify` | the oracle record decoder, the stream projections, facts, digests and comparison. No engine crate depends on it. | `data`, `game`, `runtime`, `oracle_record`, `canonical` (lockstep names differences with `Canonical::diff`) |
| `tools` | the command line over the above and the importers. No comparison logic of its own. | `verify`, `oracle`, `runtime` |
| `canonical` | the `Canonical` trait, its codec, diff and schema hash, and the primitive impls; re-exports the derive. Its own crate so `data` can derive it without depending on `game`. | `canonical_derive` |
| `canonical_derive` | the `Canonical` derive macro, dependency-free. | none |
| `clean_types` | the clean room's shared domain types (MODEL §5): `P.S` quantities, poses, cells, the `World` trait, the catalogue's kinds. No address or original name. | none |
| `clean_model_samus_motion` | Samus moving through the world (`crates/clean_model/samus-motion`), built from its spec in the clean room, and its catalogue. | `clean_types` |

`data`, `game`, `runtime` and `app` are the engine. Every new crate takes the workspace lint
set in the root `Cargo.toml` (PRINCIPLES K5, K7, K8); `game` also denies float arithmetic.

`tools/check_crate_rules.sh` checks the table's last column and the source rules of
[§4.2](../docs/architecture/ARCHITECTURE.md#42-dependency-rules-checked-in-ci): no
`cfg(verify)`, probe or observer in the engine, no SNES address literal in `game`, `runtime` or
`app`, no `#![feature]`, `allow` or `expect` in the engine, and no routine-label names. CI runs
it in Quality.
