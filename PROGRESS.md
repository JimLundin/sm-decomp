# Engine migration

![Five implementation lanes active; 40 of 44 required checks pass.](docs/codebase-map/work-summary.svg)

| Measure | Count |
| --- | ---: |
| Production Rust lines* | 115146 |
| Legacy bank Rust lines | 48883 |
| Verification Rust lines | 39335 |
| Total Rust | 154481 |
| Lines containing `unsafe` | 4337 |
| Warnings (no-default library) | 301 |
| Required checkpoint checks passing | 40/44 |

*Physical lines by path; inline verification hooks remain in production. Verification: `verification/`, `verify/`, or filenames starting `verify`.

Botwoon: fresh defeat plus full follow-up window passes; 40/44 required checks. Five local rows pass, no code growth.

[Current plan](docs/PLAN.md#work-order) | [Checkpoint recipes](movies/segments.toml) | [Codebase map](https://jimlundin.github.io/sm-decomp/#work-summary)
