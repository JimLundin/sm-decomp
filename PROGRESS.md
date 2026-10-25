# Engine migration

![Five implementation lanes active; 37 of 44 required checks pass.](docs/codebase-map/work-summary.svg)

| Measure | Count |
| --- | ---: |
| Production Rust lines* | 115345 |
| Legacy bank Rust lines | 49522 |
| Verification Rust lines | 39240 |
| Total Rust | 154585 |
| Lines containing `unsafe` | 4414 |
| Warnings (no-default library) | 312 |
| Required checkpoint checks passing | 37/44 |

*Physical lines by path; inline verification hooks remain in production. Verification: `verification/`, `verify/`, or filenames starting `verify`.

**Checked word access:** 123 redundant unsafe blocks cleared; all 59 runnable local rows pass. Persistent ownership remains open.

[Current plan](docs/PLAN.md#work-order) | [Checkpoint recipes](movies/segments.toml) | [Codebase map](https://jimlundin.github.io/sm-decomp/#work-summary)
