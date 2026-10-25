# Engine migration

![Coverage at 31 of 44; four old banks retired.](docs/codebase-map/work-summary.svg)

| Measure | Count |
| --- | ---: |
| Production Rust lines* | 115493 |
| Legacy bank Rust lines | 50063 |
| Verification Rust lines | 39228 |
| Total Rust | 154721 |
| Lines containing `unsafe` | 4585 |
| Warnings (no-default library) | 362 |
| Required checkpoint checks passing | 31/44 |

*Physical lines by path; inline verification hooks remain in production. Verification: `verification/`, `verify/`, or filenames starting `verify`.

**Crocomire collapse consolidated:** 14 production lines saved by sharing ordered spawn calls; eight replay rows pass.

[Current plan](docs/PLAN.md#work-order) | [Checkpoint recipes](movies/segments.toml) | [Codebase map](https://jimlundin.github.io/sm-decomp/#work-summary)
