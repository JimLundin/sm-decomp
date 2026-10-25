# Engine migration

![Five implementation lanes active; 37 of 44 required checks pass.](docs/codebase-map/work-summary.svg)

| Measure | Count |
| --- | ---: |
| Production Rust lines* | 115211 |
| Legacy bank Rust lines | 49077 |
| Verification Rust lines | 39244 |
| Total Rust | 154455 |
| Lines containing `unsafe` | 4363 |
| Warnings (no-default library) | 302 |
| Required checkpoint checks passing | 37/44 |

*Physical lines by path; inline verification hooks remain in production. Verification: `verification/`, `verify/`, or filenames starting `verify`.

Crocomire attack callbacks use checked actors: unsafe -19; production +21 for scoped access. Eight local rows pass.

[Current plan](docs/PLAN.md#work-order) | [Checkpoint recipes](movies/segments.toml) | [Codebase map](https://jimlundin.github.io/sm-decomp/#work-summary)
