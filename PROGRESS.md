# Engine migration

![Five implementation lanes active; 39 of 44 required checks pass.](docs/codebase-map/work-summary.svg)

| Measure | Count |
| --- | ---: |
| Production Rust lines* | 115146 |
| Legacy bank Rust lines | 48883 |
| Verification Rust lines | 39335 |
| Total Rust | 154481 |
| Lines containing `unsafe` | 4337 |
| Warnings (no-default library) | 301 |
| Required checkpoint checks passing | 39/44 |

*Physical lines by path; inline verification hooks remain in production. Verification: `verification/`, `verify/`, or filenames starting `verify`.

Eleven Ridley/Ceres operations have safe contracts; unsafe lines -10. Seven local rows pass.

[Current plan](docs/PLAN.md#work-order) | [Checkpoint recipes](movies/segments.toml) | [Codebase map](https://jimlundin.github.io/sm-decomp/#work-summary)
