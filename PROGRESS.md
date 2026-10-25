# Engine migration

![Room-seed coverage and parallel bank retirement.](docs/codebase-map/work-summary.svg)

| Measure | Count |
| --- | ---: |
| Production Rust lines* | 115585 |
| Legacy bank Rust lines | 50441 |
| Verification Rust lines | 39206 |
| Total Rust | 154791 |
| Lines containing `unsafe` | 4592 |
| Warnings (no-default library) | 362 |
| Required checkpoint checks passing | 27/44 |

*Physical lines by path; inline verification hooks remain in production. Verification: `verification/`, `verify/`, or filenames starting `verify`.

**Weather scrolling consolidated:** 69 legacy lines retired, 17 production lines saved. Four smoke rows pass; spores and fog remain uncovered.

[Current plan](docs/PLAN.md#work-order) | [Checkpoint recipes](movies/segments.toml) | [Codebase map](https://jimlundin.github.io/sm-decomp/#work-summary)
