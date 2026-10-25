# Engine migration

![Five implementation lanes active; 37 of 44 required checks pass.](docs/codebase-map/work-summary.svg)

| Measure | Count |
| --- | ---: |
| Production Rust lines* | 115190 |
| Legacy bank Rust lines | 49056 |
| Verification Rust lines | 39244 |
| Total Rust | 154434 |
| Lines containing `unsafe` | 4382 |
| Warnings (no-default library) | 308 |
| Required checkpoint checks passing | 37/44 |

*Physical lines by path; inline verification hooks remain in production. Verification: `verification/`, `verify/`, or filenames starting `verify`.

Ten checked explosion-state operations now have safe signatures. Backend contracts retained; five local rows pass.

[Current plan](docs/PLAN.md#work-order) | [Checkpoint recipes](movies/segments.toml) | [Codebase map](https://jimlundin.github.io/sm-decomp/#work-summary)
