# Engine migration

![Room-seed coverage and parallel bank retirement.](docs/codebase-map/work-summary.svg)

| Measure | Count |
| --- | ---: |
| Production Rust lines* | 115602 |
| Legacy bank Rust lines | 50510 |
| Verification Rust lines | 39206 |
| Total Rust | 154808 |
| Lines containing `unsafe` | 4598 |
| Warnings (no-default library) | 362 |
| Required checkpoint checks passing | 27/44 |

*Physical lines by path; inline verification hooks remain in production. Verification: `verification/`, `verify/`, or filenames starting `verify`.

**Movement dispatch migrated:** 341 legacy lines retired, 4 production lines saved. Seven rows pass; unchanged cold movement families remain uncovered.

[Current plan](docs/PLAN.md#work-order) | [Checkpoint recipes](movies/segments.toml) | [Codebase map](https://jimlundin.github.io/sm-decomp/#work-summary)
