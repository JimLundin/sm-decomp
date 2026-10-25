# Engine migration

![Five implementation lanes active; 40 of 44 required checks pass.](docs/codebase-map/work-summary.svg)

| Measure | Count |
| --- | ---: |
| Production Rust lines* | 115151 |
| Legacy bank Rust lines | 48664 |
| Verification Rust lines | 39336 |
| Total Rust | 154487 |
| Lines containing `unsafe` | 4323 |
| Warnings (no-default library) | 301 |
| Required checkpoint checks passing | 40/44 |

*Physical lines by path; inline verification hooks remain in production. Verification: `verification/`, `verify/`, or filenames starting `verify`.

Gripper shares checked patrol logic: 27 raw dereferences removed, 26 fewer production lines. Eight local rows pass.

[Current plan](docs/PLAN.md#work-order) | [Checkpoint recipes](movies/segments.toml) | [Codebase map](https://jimlundin.github.io/sm-decomp/#work-summary)
