# Engine migration

![Coverage at 31 of 44; four old banks retired.](docs/codebase-map/work-summary.svg)

| Measure | Count |
| --- | ---: |
| Production Rust lines* | 115532 |
| Legacy bank Rust lines | 50126 |
| Verification Rust lines | 39228 |
| Total Rust | 154760 |
| Lines containing `unsafe` | 4587 |
| Warnings (no-default library) | 362 |
| Required checkpoint checks passing | 31/44 |

*Physical lines by path; inline verification hooks remain in production. Verification: `verification/`, `verify/`, or filenames starting `verify`.

**Crocomire reactions consolidated:** 22 production lines saved; eight replay rows pass.

[Current plan](docs/PLAN.md#work-order) | [Checkpoint recipes](movies/segments.toml) | [Codebase map](https://jimlundin.github.io/sm-decomp/#work-summary)
