# Engine migration

![Phantoon fixed and qualified; 35 of44 required checks pass.](docs/codebase-map/work-summary.svg)

| Measure | Count |
| --- | ---: |
| Production Rust lines* | 115395 |
| Legacy bank Rust lines | 49718 |
| Verification Rust lines | 39232 |
| Total Rust | 154627 |
| Lines containing `unsafe` | 4553 |
| Warnings (no-default library) | 357 |
| Required checkpoint checks passing | 35/44 |

*Physical lines by path; inline verification hooks remain in production. Verification: `verification/`, `verify/`, or filenames starting `verify`.

**Wave initialization shared:** intro and Phantoon use one initializer; seven local rows pass (15,037 frames).

[Current plan](docs/PLAN.md#work-order) | [Checkpoint recipes](movies/segments.toml) | [Codebase map](https://jimlundin.github.io/sm-decomp/#work-summary)
