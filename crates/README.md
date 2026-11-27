# Crates

The rebuild's crates, as planned in
[ARCHITECTURE §4](../docs/architecture/ARCHITECTURE.md#4-packages-and-ownership) (D1).

| Crate | Owns | May depend on (workspace) |
| --- | --- | --- |
| `data` | IDs and stable keys, the ROM decoder and its tables, typed content, profiles, the portable records, the address map, the shared digest. No mutable state or rules. | `canonical`, `types` (the asset key is the clean room's pose) |
| `game` | production admission, complete owned clean State, concrete selected recipes and their adapters. No pixels, samples, files, devices or addresses. | `data`, `canonical`, `types`, `samus_movement`, `samus_appearance`, `world_blocks`, `world_camera`, `interface_hud`, `run_progress`, `boss_ceres_ridley`, `weapons_projectiles`, `enemy_interactions`, `world_doors`, `audio_transport`, `game_time`, `samus_health`, `world_quake`, `world_haze`, `game_random`, `screens_boot`, `screens_arrival`, `screens_title`, `screens_file_select`, `screens_options`, `screens_playback` |
| `runtime` | the line compositor. Presentation, sound backends and storage are planned; compositor output feeds nothing back into production State. | `data` |
| `app` | `super-metroid` currently returns `UnsupportedPowerOn`. Application admission and effect delivery are the target. | `data`, `import`, `game`, `runtime` |
| `import` | the importers ([assets.md §5](../docs/architecture/assets.md#5-the-importers)): the only code that knows the ROM's layout of the content it writes, into `data`'s formats, and the content cache ([assets.md §6](../docs/architecture/assets.md#6-loading-at-runtime)). `game` and `runtime` never reach it. | `data`, `canonical` (the cache is written with the codec) |
| `oracle` | the emulator adapters, `Oracle`, seeds, the lock recorder; re-exports `oracle_record` as `oracle::record`. | `data`, `oracle_record` |
| `oracle_record` | `FrameRecord` and the types in it, the lsnes raw record decoder, the wait sites and the pad file reader, with no emulator: linking `oracle` needs the emulator archive. | none |
| `verify` | independent record/input admission, observation projections, the indexed production replay runner and its baseline. No engine crate depends on it. | `data`, `import` (it builds the content from the ROM), `game`, `runtime`, `oracle_record`, `canonical` (state codecs), `types` and the clean models (typed input/state observation edges) |
| `tools` | `tools replay` delegates to `verify::replay`. Oracle capture has its own `oracle` binary. | `verify`, `oracle`, `runtime`, `import` |
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
| `enemy_interactions` | ordinary power-shot contact with single-box Ceres doors (`crates/clean/enemy_interactions`): stateless eligibility, overlap and stable shot consumption, built from its spec in the clean room. | none |
| `boss_ceres_ridley` | Ceres door behaviour and its catalogue (`crates/clean/boss_ceres_ridley`). | `types` |
| `world_doors` | Admitted door travel and its catalogue (`crates/clean/world_doors`). | `types` |
| `audio_transport` | admitted request delivery, resident music/effect progress and timed physical reply publications (`crates/clean/audio_transport`), built from its spec in the clean room, and its catalogue. | `types` |
| `game_time` | elapsed play time and advancement at qualified gameplay opportunities. | `types` |
| `samus_health` | current energy, periodic loss and low-energy warning requests. | `types` |
| `world_quake` | room quake duration and phase, quiet-room advancement and displacement. | `types` |
| `game_random` | the carried random sequence and the value returned by each completed draw. | `types` |
| `world_haze` | Ceres backdrop haze fade, intensity and band intents. | `types` |
| `screens_boot` | the console's power-on (`crates/clean/screens_boot`): the logo brightening, held and dimming, the sound driver's load and the settle until the title, built from its spec in the clean room, and its catalogue. | `types` |
| `screens_arrival` | her arrival at Ceres as a new game begins: the dark wait, the arrival and the fade in up to play. | `types` |
| `screens_title` | the title (`crates/clean/screens_title`): the load, the skipped opening, the title screen fading in, counting down and left for file select, built from its spec in the clean room, and its catalogue. | `types` |
| `screens_file_select` | file select (`crates/clean/screens_file_select`): the load, the menu fading in, the cursor, a file chosen and its helmet turning, and the fade to the options menu, built from its spec in the clean room, and its catalogue. | `types` |
| `screens_options` | the options menu (`crates/clean/screens_options`): the load, the cursor, the language, the special settings and their dissolves, and start game's hand-offs, built from its spec in the clean room, and its catalogue. | `types` |
| `screens_playback` | The recorded pad and control handoffs for the intro's battle and discovery (`crates/clean/screens_playback`). | `types` |

The clean crates are the folders of `crates/clean/`, one level deep, each a workspace member
(`crates/clean/*`). A model's package is its domain's name with dashes as underscores, and the
name starts with the domain's area (`docs/architecture/domains.toml`): samus, weapons, world,
enemy, boss, interface, run, screens or audio. The shared types are `types`.

The dependency column lists permitted workspace packages. The `app` manifest currently lists `data`,
`game` and `runtime`; [startup import](../docs/architecture/assets.md#6-loading-at-runtime) is planned through `import`. The `tools` binary requires
`engine-cli`, which enables `verify` and `runtime` among workspace packages. The `oracle`
binary dispatches independently through `oracle::cli`. See D17 and the [post-M0 window work](https://github.com/JimLundin/sm-decomp/issues/495) for presentation.

The table lists workspace packages only. A clean crate may also take the registry crates of
`tools/cleanroom/allowed_deps.txt` (MODEL §1 decision 12), and an engine or verification crate
inherits the same exact version from the root `[workspace.dependencies]` where its types cross the bridge: `game` and `verify` depend
on `enum-map` because the progress model's tables are `EnumMap`s.

`data`, `game`, `runtime` and `app` are the engine. Every new crate takes the workspace lint
set in the root `Cargo.toml` (PRINCIPLES K5, K7, K8); `game` also denies float arithmetic.

`deny.toml` holds the table's last column: `cargo deny check bans` refuses any other edge
between workspace crates, so `data`, `game` and `runtime` reach no `import` by any path. A new
crate takes its line there. The source rules of
[§4.2](../docs/architecture/ARCHITECTURE.md#42-dependency-rules-checked-in-ci) are the workspace
lints (no `cfg(verify)`, `#![feature]` or `allow`) and `tools/check_crate_rules.sh` (no probe or
observer in the engine, no SNES address literal in `game`, `runtime` or `app`, no `expect` in
the engine, and no routine-label names). CI runs both in Quality.

Cargo checks registry downloads against the checksums in `Cargo.lock`.
[`deny.toml`](../deny.toml) lists the versions permitted to run a build script.
[`check_crate_rules.sh`](../tools/check_crate_rules.sh) checks that those scripts come from crates.io.
Clean-room exports take their locked sources along for offline builds.
