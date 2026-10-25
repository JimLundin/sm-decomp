# Engine migration

![Crocomire defeat verified; next: room seeds and three parallel bank retirements.](docs/codebase-map/work-summary.svg)

| Measure | Count |
| --- | ---: |
| Production Rust lines* | 115957 |
| Legacy bank Rust lines | 51744 |
| Verification Rust lines | 38936 |
| Total Rust | 154893 |
| Lines containing `unsafe` | 4648 |
| Warnings (no-default library) | 367 |
| Required checkpoint checks passing | 27/44 |

*Physical lines by path; inline verification hooks remain in production. Verification: `verification/`, `verify/`, or filenames starting `verify`.

**Crystal Flash setup migrated:** 48 legacy lines removed; production +26 to preserve admission/failure order. Four smoke rows pass; natural Crystal Flash remains uncovered.

[Current plan](docs/PLAN.md#work-order) | [Checkpoint recipes](movies/segments.toml) | [Codebase map](https://jimlundin.github.io/sm-decomp/#work-summary)
