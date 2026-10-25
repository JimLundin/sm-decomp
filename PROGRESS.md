# Engine migration

![Five implementation lanes active; 40 of 44 required checks pass.](docs/codebase-map/work-summary.svg)

| Measure | Count |
| --- | ---: |
| Production Rust lines* | 115200 |
| Legacy bank Rust lines | 48852 |
| Verification Rust lines | 39336 |
| Total Rust | 154536 |
| Lines containing `unsafe` | 4337 |
| Warnings (no-default library) | 301 |
| Required checkpoint checks passing | 40/44 |

*Physical lines by path; inline verification hooks remain in production. Verification: `verification/`, `verify/`, or filenames starting `verify`.

Crocomire return motion shares acceleration and phase updates: 36 raw dereferences removed; production +47 for checked actor access. Eight local rows pass.

[Current plan](docs/PLAN.md#work-order) | [Checkpoint recipes](movies/segments.toml) | [Codebase map](https://jimlundin.github.io/sm-decomp/#work-summary)
