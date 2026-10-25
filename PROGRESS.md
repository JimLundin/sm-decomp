# Engine migration

![Legacy below 50000 lines; four banks retired.](docs/codebase-map/work-summary.svg)

| Measure | Count |
| --- | ---: |
| Production Rust lines* | 115411 |
| Legacy bank Rust lines | 49901 |
| Verification Rust lines | 39228 |
| Total Rust | 154639 |
| Lines containing `unsafe` | 4567 |
| Warnings (no-default library) | 358 |
| Required checkpoint checks passing | 31/44 |

*Physical lines by path; inline verification hooks remain in production. Verification: `verification/`, `verify/`, or filenames starting `verify`.

**Crocomire melt animation consolidated:** 12 production lines saved; eight replay rows pass.

[Current plan](docs/PLAN.md#work-order) | [Checkpoint recipes](movies/segments.toml) | [Codebase map](https://jimlundin.github.io/sm-decomp/#work-summary)
