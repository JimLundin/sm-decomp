# Engine migration

![Coverage at 31 of 44; four old banks retired.](docs/codebase-map/work-summary.svg)

| Measure | Count |
| --- | ---: |
| Production Rust lines* | 115507 |
| Legacy bank Rust lines | 50077 |
| Verification Rust lines | 39228 |
| Total Rust | 154735 |
| Lines containing `unsafe` | 4585 |
| Warnings (no-default library) | 362 |
| Required checkpoint checks passing | 31/44 |

*Physical lines by path; inline verification hooks remain in production. Verification: `verification/`, `verify/`, or filenames starting `verify`.

**Draygon layer selection consolidated:** 25 production lines saved. Smoke4 passes; the unchanged Draygon branch remains uncovered.

[Current plan](docs/PLAN.md#work-order) | [Checkpoint recipes](movies/segments.toml) | [Codebase map](https://jimlundin.github.io/sm-decomp/#work-summary)
