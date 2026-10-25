# Engine migration

![Five implementation lanes active; 40 of 44 required checks pass.](docs/codebase-map/work-summary.svg)

| Measure | Count |
| --- | ---: |
| Production Rust lines* | 115129 |
| Legacy bank Rust lines | 48473 |
| Verification Rust lines | 39336 |
| Total Rust | 154465 |
| Lines containing `unsafe` | 4311 |
| Warnings (no-default library) | 301 |
| Required checkpoint checks passing | 40/44 |

*Physical lines by path; inline verification hooks remain in production. Verification: `verification/`, `verify/`, or filenames starting `verify`.

Statue and liquid effects share earthquake updates; statue phases consolidated. Nine fewer unsafe lines; seven local rows pass.

[Current plan](docs/PLAN.md#work-order) | [Checkpoint recipes](movies/segments.toml) | [Codebase map](https://jimlundin.github.io/sm-decomp/#work-summary)
