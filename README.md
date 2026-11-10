# sm-decomp

Super Metroid (SNES) rebuilt in Rust as a model of the game's behaviour. Every behaviour is named,
so it can be reproduced exactly, adjusted, added or removed; the vanilla behaviours are proven
byte-exact against the original, unmodified ROM running on an emulator (the oracle). The engine
itself has no SNES emulation. Map Rando, with its custom worlds and opt-in mechanics, is the first
use case.

## Build and check

You need your own unheadered Super Metroid (Japan, USA) (En,Ja) ROM at `rom/sm.smc`, SHA-1
`da957f0d63d14cb441d215462904c4fa8519c613`. No ROM or game data is in this repository.

```sh
cargo build --release -p app -p tools
cargo test -p canonical -p canonical_derive -p data -p game -p runtime -p app -p verify -p tools \
  -p oracle_record -p clean_types -p clean_model_samus_motion -p clean_model_blocks \
  -p clean_model_camera -p clean_model_hud -p clean_model_progress
bash tools/lane/gate.sh   # fmt, clippy, crate rules and doc links
```

The oracle crate links a native LakeSnes archive: `bash tools/oracle/build_lakesnes.sh` builds it
with Zig 0.15.2 and prints the directory to put on `LIBRARY_PATH`. On a Windows host that blocks
fresh test executables, run tests with `tools/lane/run_tests.sh <crate>...`.

CI (`.github/workflows/ci.yml`) runs Quality (format, lints, tests, the crate rules and doc links)
and three verification jobs: the ROM content tests, the free-run gate against the lock, and the
lockstep ratchet. [STATUS](docs/STATUS.md) shows how to run them locally.

## Documentation

| Read | For |
| --- | --- |
| [AGENTS.md](AGENTS.md) | the rules every contributor and agent follows |
| [MODEL](docs/architecture/MODEL.md), [ARCHITECTURE](docs/architecture/ARCHITECTURE.md), [PRINCIPLES](docs/architecture/PRINCIPLES.md) | the design |
| [ORACLE](docs/ORACLE.md) | the oracle, the lock and the data bundles |
| [LEGACY](docs/LEGACY.md), [STATUS](docs/STATUS.md) | known differences; how progress is measured |
| [docs/behaviours/](docs/behaviours/) | what Super Metroid does, domain by domain |
| [crates/README.md](crates/README.md) | the crates and their dependencies |

## Credits and licence

[LakeSnes](https://github.com/angelo-wf/LakeSnes) (MIT) is linked into the oracle; its notices are
in [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md), with snesrev/sm's for two patch byte strings
kept for diagnosis. [lsnes](https://tasvideos.org/EmulatorResources/Lsnes) and
[MesenCE](https://github.com/nesdev-org/MesenCE) run as separate processes; none of their code is
linked or copied. Patrick Johnston's [bank logs](https://patrickjohnston.org/bank/) inform names
and behaviour. Original contributions are MIT ([LICENSE](LICENSE)). Super Metroid is a trademark
of Nintendo; no ROM or game data is included.
