# Engine migration

![Current work: gameplay checks and code consolidation lead to safe game state, an independent engine and Map Rando support.](docs/codebase-map/work-summary.svg)

| Measure | Count |
| --- | ---: |
| Production Rust lines* | 117104 |
| Verification Rust lines | 37843 |
| Total Rust | 154947 |
| Lines containing `unsafe` | 4978 |
| Warnings (no-default library) | 432 |
| Required checkpoint checks passing | 24/44 |

*Physical lines by path; inline verification hooks remain in production. Verification: `verification/`, `verify/`, or filenames starting `verify`.

This change: **558 fewer production lines**; four projectile modules retired. Verification +339; support/native +0. Unsafe operations removed: 0.

[Current plan](docs/PLAN.md#work-order) · [Checkpoint recipes](movies/segments.toml) · [Codebase map](https://jimlundin.github.io/sm-decomp/)
