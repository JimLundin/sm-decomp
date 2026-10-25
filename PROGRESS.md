# Engine migration

![Room-seed replay verified; next: late pickup coverage and bank retirement.](docs/codebase-map/work-summary.svg)

| Measure | Count |
| --- | ---: |
| Production Rust lines* | 115629 |
| Legacy bank Rust lines | 50974 |
| Verification Rust lines | 39208 |
| Total Rust | 154837 |
| Lines containing `unsafe` | 4605 |
| Warnings (no-default library) | 362 |
| Required checkpoint checks passing | 27/44 |

*Physical lines by path; inline verification hooks remain in production. Verification: `verification/`, `verify/`, or filenames starting `verify`.

**Room-seed replay qualified:** 600 matching frames and the expected next room. Production unchanged; verification +272 for the generic importer.

[Current plan](docs/PLAN.md#work-order) | [Checkpoint recipes](movies/segments.toml) | [Codebase map](https://jimlundin.github.io/sm-decomp/#work-summary)
