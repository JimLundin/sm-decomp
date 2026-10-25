# Engine migration

![Five implementation lanes active; 37 of 44 required checks pass.](docs/codebase-map/work-summary.svg)

| Measure | Count |
| --- | ---: |
| Production Rust lines* | 115274 |
| Legacy bank Rust lines | 49329 |
| Verification Rust lines | 39244 |
| Total Rust | 154518 |
| Lines containing `unsafe` | 4406 |
| Warnings (no-default library) | 312 |
| Required checkpoint checks passing | 37/44 |

*Physical lines by path; inline verification hooks remain in production. Verification: `verification/`, `verify/`, or filenames starting `verify`.

Menu windows: shared edge updates and HDMA tables; production -71, legacy -193. Five local rows pass.

[Current plan](docs/PLAN.md#work-order) | [Checkpoint recipes](movies/segments.toml) | [Codebase map](https://jimlundin.github.io/sm-decomp/#work-summary)
