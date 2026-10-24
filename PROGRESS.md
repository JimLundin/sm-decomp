# Engine migration

![Current work: gameplay checks and collision cleanup lead to safe game state, an independent engine and Map Rando support.](docs/codebase-map/work-summary.svg)

| Measure | Count |
| --- | ---: |
| Production Rust lines* | 116960 |
| Verification Rust lines | 38884 |
| Total Rust | 155844 |
| Lines containing `unsafe` | 4965 |
| Warnings (no-default library) | 378 |
| Required checkpoint checks passing | 24/44 |

*Physical lines by path; inline verification hooks remain in production. Verification: `verification/`, `verify/`, or filenames starting `verify`.

Warnings exclude repeated compiler diagnostics: **380 → 378** on the same basis.

This change: **7 unsafe functions retired**; production **+5** to preserve ordered state/output access. Verification +447 (36 formatting); Python +19; native +0.

[Current plan](docs/PLAN.md#work-order) · [Checkpoint recipes](movies/segments.toml) · [Codebase map](https://jimlundin.github.io/sm-decomp/)
