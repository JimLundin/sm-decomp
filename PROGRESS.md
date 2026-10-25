# Engine migration

![Five implementation lanes active; 39 of 44 required checks pass.](docs/codebase-map/work-summary.svg)

| Measure | Count |
| --- | ---: |
| Production Rust lines* | 115157 |
| Legacy bank Rust lines | 48974 |
| Verification Rust lines | 39335 |
| Total Rust | 154492 |
| Lines containing `unsafe` | 4351 |
| Warnings (no-default library) | 301 |
| Required checkpoint checks passing | 39/44 |

*Physical lines by path; inline verification hooks remain in production. Verification: `verification/`, `verify/`, or filenames starting `verify`.

Menu graphics share palette loaders and nine queue writers: production -54, legacy -103. Six local rows pass.

[Current plan](docs/PLAN.md#work-order) | [Checkpoint recipes](movies/segments.toml) | [Codebase map](https://jimlundin.github.io/sm-decomp/#work-summary)
