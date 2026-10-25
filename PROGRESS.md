# Engine migration

![Six implementation lanes active; 35 of 44 required checks pass.](docs/codebase-map/work-summary.svg)

| Measure | Count |
| --- | ---: |
| Production Rust lines* | 115337 |
| Legacy bank Rust lines | 49573 |
| Verification Rust lines | 39236 |
| Total Rust | 154573 |
| Lines containing `unsafe` | 4544 |
| Warnings (no-default library) | 357 |
| Required checkpoint checks passing | 35/44 |

*Physical lines by path; inline verification hooks remain in production. Verification: `verification/`, `verify/`, or filenames starting `verify`.

**Menu drawing shared:** 38 fewer production lines; seven local rows pass (17,380 frames).

[Current plan](docs/PLAN.md#work-order) | [Checkpoint recipes](movies/segments.toml) | [Codebase map](https://jimlundin.github.io/sm-decomp/#work-summary)
