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
| Required checkpoint checks passing | 29/44 |

*Physical lines by path; inline verification hooks remain in production. Verification: `verification/`, `verify/`, or filenames starting `verify`.

**Plasma pickup qualified:** Fresh Plasma award, completed message and 120-frame tail.

[Current plan](docs/PLAN.md#work-order) | [Checkpoint recipes](movies/segments.toml) | [Codebase map](https://jimlundin.github.io/sm-decomp/#work-summary)
