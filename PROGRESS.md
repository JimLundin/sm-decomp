# Engine migration

![Now: correct recorded-run timing; input consolidated. Next: full-game checks and demo playback. Outcome: safe owned state and less duplicate RAM code, then independent video/audio. Target: Map Rando.](docs/codebase-map/work-summary.svg)

| Measure | Count |
| --- | ---: |
| Production Rust lines* | 116389 |
| Legacy bank Rust lines | 54323 |
| Verification Rust lines | 39016 |
| Total Rust | 155405 |
| Lines containing `unsafe` | 4905 |
| Warnings (no-default library) | 372 |
| Required checkpoint checks passing | 24/44 |

*Physical lines by path; inline verification hooks remain in production. Verification: `verification/`, `verify/`, or filenames starting `verify`.

Change: **input consolidated**; engine **+138 / -227 = -89**, legacy **-220**, unsafe **-23**, verification **+6**. `sm_91`: **1,017 lines left**. Next: demo playback.

[Current plan](docs/PLAN.md#work-order) | [Checkpoint recipes](movies/segments.toml) | [Codebase map](https://jimlundin.github.io/sm-decomp/#work-summary)
