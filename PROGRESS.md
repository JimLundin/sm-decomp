# Engine migration

![Phantoon fixed and qualified; 35 of44 required checks pass.](docs/codebase-map/work-summary.svg)

| Measure | Count |
| --- | ---: |
| Production Rust lines* | 115375 |
| Legacy bank Rust lines | 49698 |
| Verification Rust lines | 39232 |
| Total Rust | 154607 |
| Lines containing `unsafe` | 4554 |
| Warnings (no-default library) | 357 |
| Required checkpoint checks passing | 35/44 |

*Physical lines by path; inline verification hooks remain in production. Verification: `verification/`, `verify/`, or filenames starting `verify`.

**Crocomire animation shared:** four melt paths use one selector; eight local rows pass (6,480 frames).

[Current plan](docs/PLAN.md#work-order) | [Checkpoint recipes](movies/segments.toml) | [Codebase map](https://jimlundin.github.io/sm-decomp/#work-summary)
