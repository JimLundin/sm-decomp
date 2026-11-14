# Crates

The rebuild's crates, as planned in
[ARCHITECTURE §4](../docs/architecture/ARCHITECTURE.md#4-packages-and-ownership) (D1).

| Crate | Owns | May depend on (workspace) |
| --- | --- | --- |
| `data` | IDs and stable keys, the ROM decoder and its tables, typed content, profiles, the portable records, the address map, the shared digest. No mutable state or rules. | `canonical`, `types` (the asset key is the clean room's pose) |
| `game` | the `Session`, the `State`, the Tick and every gameplay domain. No pixels, samples, files, devices or addresses. | `data`, `canonical`, `types`, `samus_movement`, `samus_appearance`, `world_blocks`, `world_camera`, `interface_hud`, `run_progress` |
| `runtime` | the line compositor, presentation, sound backends, storage. Feeds nothing back into the Session. | `data` |
| `app` | the run coordinator, Tick admission, the input profile, effect delivery. No gameplay. Its binary is `super-metroid`. | `data`, `import` (the player's ROM at startup), `game`, `runtime` |
| `import` | the importers ([assets.md §5](../docs/architecture/assets.md#5-the-importers)): the only code that knows the ROM's layout of the content it writes, into `data`'s formats, and the content cache ([assets.md §6](../docs/architecture/assets.md#6-loading-at-runtime)). `game` and `runtime` never reach it. | `data`, `canonical` (the cache is written with the codec) |
| `oracle` | the emulator adapters, `Oracle`, seeds, the lock recorder; re-exports `oracle_record` as `oracle::record`. | `data`, `oracle_record` |
| `oracle_record` | `FrameRecord` and the types in it, the lsnes raw record decoder, the wait sites and the pad file reader, with no emulator: linking `oracle` needs the emulator archive. | none |
| `verify` | the oracle record decoder, the stream projections, facts, digests and comparison. No engine crate depends on it. | `data`, `import` (it builds the content from the ROM), `game`, `runtime`, `oracle_record`, `canonical` (lockstep names differences with `Canonical::diff`), `types` and the clean models (the domain harnesses run them) |
| `tools` | the command line over the above. No comparison logic of its own. | `verify`, `oracle`, `runtime`, `import` |
| `canonical` | the `Canonical` trait, its codec, diff and schema hash, and the primitive impls; re-exports the derive. Its own crate so `data` can derive it without depending on `game`. `canonical::model` holds a clean model's State, which derives serde's traits, by the same codec (MODEL §1 decision 29). | `canonical_derive` |
| `canonical_derive` | the `Canonical` derive macro, dependency-free. | none |
| `types` | the clean room's shared domain types (`crates/clean/types`, MODEL §5): `P.S` quantities, poses, cells, the `World` trait, the catalogue's kinds. No address or original name. | none |
| `samus_movement` | Samus moving through the world (`crates/clean/samus_movement`), built from its spec in the clean room, and its catalogue. | `types` |
| `world_blocks` | the blocks that break and come back, and the scroll cells (`crates/clean/world_blocks`): the places reactions run in, their scripts, the starts by family and the room's stand-ins, built from its spec in the clean room, and its catalogue. | `types` |
| `world_camera` | the camera following Samus (`crates/clean/world_camera`), built from its spec in the clean room, and its catalogue. | `types` |
| `interface_hud` | the HUD (`crates/clean/interface_hud`): the status bar, the selected item and the mini-map, built from its spec in the clean room, and its catalogue. | `types` |
| `run_progress` | what the game remembers of a run (`crates/clean/run_progress`): the reports that change it, the questions it answers and the room states it picks, built from its spec in the clean room, and its catalogue. | `types` |
| `samus_appearance` | her animation's timing (`crates/clean/samus_appearance`), part 1 of how Samus looks: the clip's steps, the pose's start, her moves' writes and the run's boost stages, built from its spec in the clean room, and its catalogue. | `types` |
| `weapons_projectiles` | the uncharged power beam (`crates/clean/weapons_projectiles`): firing, inherited motion, contact, lifetime and picture intents, built from its spec in the clean room, and its catalogue. | `types` |

The clean crates are the folders of `crates/clean/`, one level deep, each a workspace member
(`crates/clean/*`). A model's package is its domain's name with dashes as underscores, and the
name starts with the domain's area (`docs/architecture/domains.toml`): samus, weapons, world,
enemy, boss, interface, run, screens or audio. The shared types are `types`.

The table lists workspace packages only. A clean crate may also take the registry crates of
`tools/cleanroom/allowed_deps.txt` (MODEL §1 decision 12), and an engine or verification crate
takes one of them, at the same pin, where its types cross the bridge: `game` and `verify` depend
on `enum-map` because the progress model's tables are `EnumMap`s.

`data`, `game`, `runtime` and `app` are the engine. Every new crate takes the workspace lint
set in the root `Cargo.toml` (PRINCIPLES K5, K7, K8); `game` also denies float arithmetic.

`tools/check_crate_rules.sh` checks the table's last column, that `data`, `game` and `runtime`
reach no `import`, and the source rules of
[§4.2](../docs/architecture/ARCHITECTURE.md#42-dependency-rules-checked-in-ci): no
`cfg(verify)`, probe or observer in the engine, no SNES address literal in `game`, `runtime` or
`app`, no `#![feature]`, `allow` or `expect` in the engine, and no routine-label names. CI runs
it in Quality.

`vendor/` is not ours: it holds published registry crates whose build scripts Defender blocks on
the development machine, each with its script's output baked in by `tools/vendor_nobuild.sh` and
patched in by the root `Cargo.toml`. CI's `tools/vendor_nobuild_check.sh` runs the real scripts
and fails if a copy's baked output differs.
