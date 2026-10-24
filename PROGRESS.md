# Engine migration

![Current work: gameplay checks and code consolidation lead to safe game state, an independent engine and Map Rando support.](docs/codebase-map/work-summary.svg)

| Measure | Count |
| --- | ---: |
| Production Rust lines* | 117027 |
| Verification Rust lines | 38099 |
| Total Rust | 155126 |
| Lines containing `unsafe` | 4977 |
| Warnings (no-default library) | 432 |
| Required checkpoint checks passing | 24/44 |

*Physical lines by path; inline verification hooks remain in production. Verification: `verification/`, `verify/`, or filenames starting `verify`.

This change: **76 fewer production lines** (77 physical; one formatting-only). Verification +256; support/native +0; unsafe operations removed: 0.

[Current plan](docs/PLAN.md#work-order) · [Checkpoint recipes](movies/segments.toml) · [Codebase map](https://jimlundin.github.io/sm-decomp/)
