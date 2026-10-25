# Engine migration

![Late item and area coverage progressing.](docs/codebase-map/work-summary.svg)

| Measure | Count |
| --- | ---: |
| Production Rust lines* | 115585 |
| Legacy bank Rust lines | 50441 |
| Verification Rust lines | 39229 |
| Total Rust | 154814 |
| Lines containing `unsafe` | 4593 |
| Warnings (no-default library) | 362 |
| Required checkpoint checks passing | 31/44 |

*Physical lines by path; inline verification hooks remain in production. Verification: `verification/`, `verify/`, or filenames starting `verify`.

**Wrecked Ship qualified:** Wrecked Ship area and entry room CBD5 observed; all 300 replay frames match.

[Current plan](docs/PLAN.md#work-order) | [Checkpoint recipes](movies/segments.toml) | [Codebase map](https://jimlundin.github.io/sm-decomp/#work-summary)
