# Engine migration

![Five implementation lanes active; 40 of 44 required checks pass.](docs/codebase-map/work-summary.svg)

| Measure | Count |
| --- | ---: |
| Production Rust lines* | 115177 |
| Legacy bank Rust lines | 48715 |
| Verification Rust lines | 39336 |
| Total Rust | 154513 |
| Lines containing `unsafe` | 4327 |
| Warnings (no-default library) | 301 |
| Required checkpoint checks passing | 40/44 |

*Physical lines by path; inline verification hooks remain in production. Verification: `verification/`, `verify/`, or filenames starting `verify`.

Mother Brain purple-breath spawn parameter corrected. Six local rows pass; full defeat remains pending. No code growth.

[Current plan](docs/PLAN.md#work-order) | [Checkpoint recipes](movies/segments.toml) | [Codebase map](https://jimlundin.github.io/sm-decomp/#work-summary)
