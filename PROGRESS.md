# Engine migration

![Phantoon fixed and qualified; 35 of44 required checks pass.](docs/codebase-map/work-summary.svg)

| Measure | Count |
| --- | ---: |
| Production Rust lines* | 115401 |
| Legacy bank Rust lines | 49774 |
| Verification Rust lines | 39232 |
| Total Rust | 154633 |
| Lines containing `unsafe` | 4557 |
| Warnings (no-default library) | 357 |
| Required checkpoint checks passing | 35/44 |

*Physical lines by path; inline verification hooks remain in production. Verification: `verification/`, `verify/`, or filenames starting `verify`.

**Phantoon qualified:** room-setup spawn value fixed; regression, boss tail and smoke4 pass (6257 frames).

[Current plan](docs/PLAN.md#work-order) | [Checkpoint recipes](movies/segments.toml) | [Codebase map](https://jimlundin.github.io/sm-decomp/#work-summary)
