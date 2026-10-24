# Engine migration

![Current work: gameplay checks and code consolidation lead to safe game state, an independent engine and Map Rando support.](docs/codebase-map/work-summary.svg)

| Measure | Count |
| --- | ---: |
| Production Rust lines* | 117745 |
| Verification Rust lines | 37504 |
| Total Rust | 155249 |
| Lines containing `unsafe` | 4993 |
| Warnings (no-default library) | 433 |
| Required checkpoint checks passing | 23/44 |

*Physical lines by path; inline verification hooks remain in production. Verification: `verification/`, `verify/`, or filenames starting `verify`.

This change: 120 production lines deleted; no verification, support or native code added.

[Current plan](docs/PLAN.md#work-order) · [Checkpoint recipes](movies/segments.toml) · [Codebase map](https://jimlundin.github.io/sm-decomp/)
