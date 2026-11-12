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

| Milestone | Frames | The engine plays alone | Still needed |
| --- | ---: | ---: | --- |
| **Power-on through Ceres**, the any% movie from power-on | 15,196 | 0 (0.0 %) | 28 of 28: S1, S2, S5, CAM, H2, P1, W2, E1, E2 (in progress); S4, H1, P2, W1, C1, B8, R1, R3, R4, F2, F4, U1, E3, E4, A1, A2, R2, T1, V2 (not started) |

The headline milestone ([MODEL §1](docs/architecture/MODEL.md#1-the-goal-and-what-is-exact), decision 14): the engine replays the stretch from power-on with nothing fed back, every compared cut and byte matching. Until it does, the frames it plays alone are those inside the movie's rows whose every stream the free-run gate holds green, each row run from its own seed. Still needed are the plan's domains and platform work the stretch touches ([domains.md §4](docs/architecture/domains.md#4-order)) that are not done.

Of the 36 clean-room domains in the [whole-game plan](docs/architecture/domains.md), **7 are in progress** and **0 done**; 29 have not started. **47 rounds** have landed on 6 models; the plan estimates 147 rounds for the whole game.

![How much of each movie the checks cover](docs/timeline.svg)

How much of each whole movie the lock's rows, the harnesses, the models, the free-run gate and lockstep cover, frame by frame; the figures are in [the report's timeline](docs/REPORT.md#timeline).

### The models

Each model is built in the clean room from its spec alone, then checked frame by frame against the original game: its harness frames are the recorded frames on which it matched the original on every compared field, of the frames it ran over. Score and budget are weighted tokens; fit is score over budget.

| Model | Plan | Status | Rounds | Vectors | Harness frames | Score | Budget | Fit |
| --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| samus-movement | S1, S2 (in progress); S4, S3 (not started) | in progress | 19 | 326 | 18,195 of 36,311 | 8,926 | 8,823 | 1.01 |
| world-camera | CAM (in progress) | in progress | 6 | 83 | 20,676 of 36,311 | 1,839 | 1,762 | 1.04 |
| run-progress | P1 (in progress) | in progress | 8 | 76 | 35,919 of 36,311 | 819 | 1,061 | 0.77 |
| interface-hud | H2 (in progress) | in progress | 5 | 84 | 20,778 of 34,377 | 2,695 | 2,112 | 1.28 |
| world-blocks | W2 (in progress) | in progress | 4 | 100 | 20,693 of 32,006 | 2,395 | 2,074 | 1.15 |
| samus-appearance | S5 (in progress) | in progress | 5 | 98 | 18,396 of 34,377 | 2,191 | 1,412 | 1.55 |

### The whole-game plan

The plan's waves ([domains.md §4](docs/architecture/domains.md#4-order)), each domain with its plan id; rounds are the plan's estimates, platform work is in lanes.

| Wave | Clean-room domains | Platform work | Started | Rounds planned |
| --- | --- | --- | ---: | ---: |
| 0 | **S1 Samus's motion: follow-ups** (in progress); **CAM The camera** (in progress) | V1 Tooling: domain free run, RNG per consumer, cache tier (3) | 2 of 2 | 2 |
| 1 | **S2 Liquids and pushes** (in progress); **S5 How Samus looks** (in progress); **H2 The HUD and the mini-map** (in progress); **W2 Blocks that react** (in progress); **P1 Progress** (in progress) | E1 Sprite drawing (2); E2 Tile writes (3); A1 Instruction lists as assets (3) | 5 of 5 | 15 |
| 2 | R1 Doors and elevators; W1 Beams, missiles and bombs; C1 Fighting enemies; H1 Energy, reserves and death; S4 Hurt and knockback, bomb jumps | R2 Room loading (4); A2 Enemy and room importers (3); V2 Whole-movie run: one free run from power-on, sparse then dense checkpoints (2) | 0 of 5 | 21 |
| 3 | B8 Ceres Ridley and the escape; F2 Title, intro, file select, options; F4 Cinematics and credits; R3 Liquids and room weather; R4 Room ambience; P2 Saving, loading and stations; U1 Audio, the CPU side | E3 The line program (3); E4 Mode 7 and mid-line writes (2); T1 Slowdown and loading time (6) | 0 of 7 | 30 |
| 4 | W3 Items to pick up; H3 Message boxes; C2-C11 Enemy families; B1 Spore Spawn; B2 Bomb Torizo, Golden Torizo; B3 Botwoon; B4 Crocomire; B5 Kraid; B6 Phantoon; B7 Draygon; B9 Ridley; B10 Mother Brain and the escape | - | 0 of 12 | 66 |
| 5 | S3 The shinespark and echoes; S6 The grapple beam; S7 The x-ray scope; F1 The pause screen; F3 Game over | - | 0 of 5 | 13 |

### The engine

The engine's code still to be rebuilt (`crates/game` without the bridges), beside the rebuilt models. Logic-equivalent lines are score ÷ 6, the old engine's density.

| Code | Files | Logic-equivalent lines | Score | Lines |
| --- | ---: | ---: | ---: | ---: |
| not yet rebuilt | 48 | 4,831 | 28,988 | 6,787 |
| bridges from the engine to the models | 10 | 2,670 | 16,020 | 2,762 |
| rebuilt models, shared types included | 71 | 3,254 | 19,526 | 8,657 |

### Verification

The original game on an emulator (the oracle) recorded the lock's rows; the engine is held to them. See [STATUS](docs/STATUS.md) for how each is run.

| Check | State |
| --- | --- |
| the lock | 651 rows of play, 460,437 frames |
| the free-run gate: the engine runs a row alone | 8 green (row, stream) pairs over 2 rows |
| the lockstep ratchet: the engine re-seeded each Tick | 1,848 of 1,903 Ticks fully equal over 2 rows |
| the harnesses: each model on recorded frames | 53 rows cached, 36,364 frames |

Every figure, file by file and area by area, is in [the rebuild report](docs/REPORT.md); how each model is built, module by module, is in [its structure section](docs/REPORT.md#structure).
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
