# sm-decomp

**The goal** is to model Super Metroid's behaviour, all of it, as named behaviours, so each one can
be reproduced exactly, adjusted (the space jump timing), added (respin) or removed (lag). In the
end only this engine runs; the emulator is the oracle that proves the vanilla behaviours exact.

The engine is Rust with no SNES emulation; the vanilla behaviours are proven byte-exact against
the original, unmodified ROM. Map Rando, with its custom worlds and opt-in mechanics, is the first
use case. To build it and run the checks, see [BUILDING](docs/BUILDING.md).

<!-- status:begin -->
<!-- Generated with docs/REPORT.md by `cargo run -p tools --bin tools -- report --write` (verify::report) from the tree; `tools/check_report.sh` fails when it is stale. Never edit it by hand. -->

## Where it stands

Of the 36 clean-room domains in the [whole-game plan](docs/architecture/domains.md), **6 are in progress** and **0 done**; 30 have not started. **36 rounds** have landed on 5 models; the plan estimates 147 rounds for the whole game.

### The models

Each model is built in the clean room from its spec alone, then checked frame by frame against the original game: its harness frames are the recorded frames on which it matched the original on every compared field, of the frames it ran over. Score and budget are weighted tokens; fit is score over budget.

| Model | Plan | Status | Rounds | Vectors | Harness frames | Score | Budget | Fit |
| --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| samus-motion | S1, S2 (in progress); S3, S4 (not started) | in progress | 17 | 284 | 17,999 of 25,426 | 8,475 | 7,884 | 1.07 |
| camera | CAM (in progress) | in progress | 6 | 83 | 20,513 of 25,426 | 1,883 | 1,692 | 1.11 |
| progress | P1 (in progress) | in progress | 6 | 66 | 25,419 of 25,426 | 979 | 1,092 | 0.90 |
| hud | H2 (in progress) | in progress | 3 | 72 | 20,614 of 25,426 | 2,978 | 2,028 | 1.47 |
| blocks | W2 (in progress) | in progress | 4 | 100 | 20,530 of 21,857 | 2,643 | 1,992 | 1.33 |

### The whole-game plan

The plan's waves ([domains.md §4](docs/architecture/domains.md#4-order)), each domain with its plan id; rounds are the plan's estimates, platform work is in lanes.

| Wave | Clean-room domains | Platform work | Started | Rounds planned |
| --- | --- | --- | ---: | ---: |
| 0 | **S1 Samus's motion: follow-ups** (in progress); **CAM The camera** (in progress) | V1 Tooling: domain free run, RNG per consumer, cache tier (3) | 2 of 2 | 2 |
| 1 | **S2 Liquids, the speed booster, pushes** (in progress); S5 How Samus looks; **H2 The HUD and the mini-map** (in progress); **W2 Blocks that react** (in progress); **P1 Progress** (in progress) | E1 Sprite drawing (2); E2 Tile writes (3); A1 Instruction lists as assets (3) | 4 of 5 | 15 |
| 2 | R1 Doors and elevators; W1 Beams, missiles and bombs; W3 Items to pick up; S3 The shinespark and echoes; S6 The grapple beam; H3 Message boxes | R2 Room loading (4) | 0 of 6 | 20 |
| 3 | C1 Fighting enemies; S4 Hurt and knockback, bomb jumps; H1 Energy, reserves and death; S7 The x-ray scope; R3 Liquids and room weather; R4 Room ambience; P2 Saving, loading and stations; F3 Game over | A2 Enemy and room importers (3); E3 The line program (3) | 0 of 8 | 23 |
| 4 | C2-C11 Enemy families; B1 Spore Spawn; B2 Bomb Torizo, Golden Torizo; B3 Botwoon; B4 Crocomire; B5 Kraid; B6 Phantoon; B7 Draygon; B8 Ceres Ridley and the escape; B9 Ridley; B10 Mother Brain and the escape; F1 The pause screen | - | 0 of 12 | 71 |
| 5 | F2 Title, intro, file select, options; F4 Cinematics and credits; U1 Audio, the CPU side | E4 Mode 7 and mid-line writes (2); T1 Slowdown and loading time (6) | 0 of 3 | 16 |

### The engine

The engine's code still to be rebuilt (`crates/game` without the bridges), beside the rebuilt models. Logic-equivalent lines are score ÷ 6, the old engine's density.

| Code | Files | Logic-equivalent lines | Score | Lines |
| --- | ---: | ---: | ---: | ---: |
| not yet rebuilt | 44 | 5,040 | 30,241 | 6,920 |
| bridges from the engine to the models | 8 | 2,449 | 14,693 | 2,473 |
| rebuilt models, shared types included | 59 | 2,956 | 17,737 | 7,701 |

### Verification

The original game on an emulator (the oracle) recorded the lock's rows; the engine is held to them. See [STATUS](docs/STATUS.md) for how each is run.

| Check | State |
| --- | --- |
| the lock | 642 rows of play, 449,543 frames |
| the free-run gate: the engine runs a row alone | 8 green (row, stream) pairs over 2 rows |
| the lockstep ratchet: the engine re-seeded each Tick | 1,848 of 1,903 Ticks fully equal over 2 rows |
| the harnesses: each model on recorded frames | 44 rows cached, 25,470 frames |

Every figure, file by file and area by area, is in [the rebuild report](docs/REPORT.md).
<!-- status:end -->

## Documentation

| Read | For |
| --- | --- |
| [AGENTS.md](AGENTS.md) | the rules every contributor and agent follows |
| [MODEL](docs/architecture/MODEL.md), [ARCHITECTURE](docs/architecture/ARCHITECTURE.md), [PRINCIPLES](docs/architecture/PRINCIPLES.md) | the design |
| [the domain plan](docs/architecture/domains.md), [PROCESS](docs/cleanroom/PROCESS.md) | the whole game as clean-room domains; how a round runs |
| [ORACLE](docs/ORACLE.md) | the oracle, the lock and the data bundles |
| [LEGACY](docs/LEGACY.md), [STATUS](docs/STATUS.md) | known differences; how progress is measured |
| [docs/behaviours/](docs/behaviours/) | what Super Metroid does, domain by domain |
| [crates/README.md](crates/README.md) | the crates and their dependencies |

## Credits and licence

[LakeSnes](https://github.com/angelo-wf/LakeSnes) (MIT) is linked into the oracle: the build
fetches its pinned sources and their licence; the repository carries only the project's fixes to it
(`crates/oracle/native/fixes/`).
[lsnes](https://tasvideos.org/EmulatorResources/Lsnes) and
[MesenCE](https://github.com/nesdev-org/MesenCE) run as separate processes; none of their code is
linked or copied. The crates in `vendor/` are registry crates, each with its own licence files.
Patrick Johnston's [bank logs](https://patrickjohnston.org/bank/) inform names
and behaviour. Original contributions are MIT ([LICENSE](LICENSE)). Super Metroid is a trademark
of Nintendo; no ROM or game data is included.
