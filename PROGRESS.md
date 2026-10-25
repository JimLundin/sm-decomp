# Engine migration

![Now: 9,266 recorded steps aligned; demo setup owned. Next: door resume timing and suit sequence. Outcome: safe owned state and less duplicate RAM code, then independent video/audio. Target: Map Rando.](docs/codebase-map/work-summary.svg)

| Measure | Count |
| --- | ---: |
| Production Rust lines* | 116360 |
| Legacy bank Rust lines | 53989 |
| Verification Rust lines | 39023 |
| Total Rust | 155383 |
| Lines containing `unsafe` | 4881 |
| Warnings (no-default library) | 371 |
| Required checkpoint checks passing | 24/44 |

*Physical lines by path; inline verification hooks remain in production. Verification: `verification/`, `verify/`, or filenames starting `verify`.

Change: **+166 recorded steps aligned**, original state/position traces match; **0 replay failures**. Code counts unchanged. Next: resume gameplay after the fifth Ceres door.

[Current plan](docs/PLAN.md#work-order) | [Checkpoint recipes](movies/segments.toml) | [Codebase map](https://jimlundin.github.io/sm-decomp/#work-summary)
