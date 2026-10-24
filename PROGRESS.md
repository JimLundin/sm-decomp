# Engine migration

![Current work: gameplay checks and code consolidation lead to safe game state, an independent engine and Map Rando support.](docs/codebase-map/work-summary.svg)

| Measure | Count |
| --- | ---: |
| Production Rust lines* | 116955 |
| Verification Rust lines | 38437 |
| Total Rust | 155392 |
| Lines containing `unsafe` | 4977 |
| Warnings (no-default library) | 432 |
| Required checkpoint checks passing | 24/44 |

*Physical lines by path; inline verification hooks remain in production. Verification: `verification/`, `verify/`, or filenames starting `verify`.

This change: **72 fewer production lines**. Verification +338 (231 new; 107 formatting); support/native +0; unsafe operations removed: 0.

[Current plan](docs/PLAN.md#work-order) · [Checkpoint recipes](movies/segments.toml) · [Codebase map](https://jimlundin.github.io/sm-decomp/)
