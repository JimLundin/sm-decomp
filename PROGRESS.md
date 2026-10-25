# Engine migration

![Coverage at 34 of 44; legacy below 50000 lines.](docs/codebase-map/work-summary.svg)

| Measure | Count |
| --- | ---: |
| Production Rust lines* | 115401 |
| Legacy bank Rust lines | 49774 |
| Verification Rust lines | 39232 |
| Total Rust | 154633 |
| Lines containing `unsafe` | 4557 |
| Warnings (no-default library) | 357 |
| Required checkpoint checks passing | 34/44 |

*Physical lines by path; inline verification hooks remain in production. Verification: `verification/`, `verify/`, or filenames starting `verify`.

**Save/load migrated:** 127 legacy lines retired, 10 production lines saved. Eight rows pass; portable Session saves remain future work.

[Current plan](docs/PLAN.md#work-order) | [Checkpoint recipes](movies/segments.toml) | [Codebase map](https://jimlundin.github.io/sm-decomp/#work-summary)
