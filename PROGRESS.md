# Engine migration

![Now: gameplay checks and collision cleanup. Next: safe game state, an independent engine and Map Rando.](docs/codebase-map/work-summary.svg)

| Measure | Count |
| --- | ---: |
| Production Rust lines* | 116960 |
| Verification Rust lines | 38884 |
| Total Rust | 155844 |
| Lines containing `unsafe` | 4965 |
| Warnings (no-default library) | 378 |
| Required checkpoint checks passing | 24/44 |

*Physical lines by path; inline verification hooks remain in production. Verification: `verification/`, `verify/`, or filenames starting `verify`.

Published engine counts; collision draft excluded. Warnings exclude repeated diagnostics.

[Current plan](docs/PLAN.md#work-order) | [Checkpoint recipes](movies/segments.toml) | [Codebase map](https://jimlundin.github.io/sm-decomp/#work-summary)
