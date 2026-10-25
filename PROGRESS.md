# Engine migration

![Five implementation lanes active; 41 of 44 required checks pass.](docs/codebase-map/work-summary.svg)

| Measure | Count |
| --- | ---: |
| Production Rust lines* | 115210 |
| Legacy bank Rust lines | 48327 |
| Verification Rust lines | 39348 |
| Total Rust | 154558 |
| Lines containing `unsafe` | 4303 |
| Warnings (no-default library) | 299 |
| Required checkpoint checks passing | 41/44 |

*Physical lines by path; inline verification hooks remain in production. Verification: `verification/`, `verify/`, or filenames starting `verify`.

Mother Brain: fresh defeat + full tail, 1,380 frames, zero differences. Hosted validation passed.

[Current plan](docs/PLAN.md#work-order) | [Checkpoint recipes](movies/segments.toml) | [Codebase map](https://jimlundin.github.io/sm-decomp/#work-summary)
