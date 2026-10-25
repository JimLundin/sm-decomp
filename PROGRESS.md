# Engine migration

![Legacy below 50000 lines; shared sprite drawing verified.](docs/codebase-map/work-summary.svg)

| Measure | Count |
| --- | ---: |
| Production Rust lines* | 115439 |
| Legacy bank Rust lines | 49948 |
| Verification Rust lines | 39228 |
| Total Rust | 154667 |
| Lines containing `unsafe` | 4574 |
| Warnings (no-default library) | 362 |
| Required checkpoint checks passing | 31/44 |

*Physical lines by path; inline verification hooks remain in production. Verification: `verification/`, `verify/`, or filenames starting `verify`.

**Sprite drawing consolidated:** 54 production lines saved, 115 legacy lines retired; eight rows pass.

[Current plan](docs/PLAN.md#work-order) | [Checkpoint recipes](movies/segments.toml) | [Codebase map](https://jimlundin.github.io/sm-decomp/#work-summary)
