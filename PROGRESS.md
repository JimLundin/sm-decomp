# Engine migration

![Current work: gameplay checks and code consolidation lead to safe game state, an independent engine and Map Rando support.](docs/codebase-map/work-summary.svg)

| Measure | Count |
| --- | ---: |
| Production Rust lines* | 117662 |
| Verification Rust lines | 37504 |
| Total Rust | 155166 |
| Lines containing `unsafe` | 4982 |
| Warnings (no-default library) | 432 |
| Required checkpoint checks passing | 24/44 |

*Physical lines by path; inline verification hooks remain in production. Verification: `verification/`, `verify/`, or filenames starting `verify`.

This change: Reserve pickup verified; 24/44 checks pass. No production or verification Rust added.

[Current plan](docs/PLAN.md#work-order) · [Checkpoint recipes](movies/segments.toml) · [Codebase map](https://jimlundin.github.io/sm-decomp/)
