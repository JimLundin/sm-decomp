# Engine migration

![Six implementation lanes active; 35 of 44 required checks pass.](docs/codebase-map/work-summary.svg)

| Measure | Count |
| --- | ---: |
| Production Rust lines* | 115343 |
| Legacy bank Rust lines | 49579 |
| Verification Rust lines | 39236 |
| Total Rust | 154579 |
| Lines containing `unsafe` | 4544 |
| Warnings (no-default library) | 357 |
| Required checkpoint checks passing | 35/44 |

*Physical lines by path; inline verification hooks remain in production. Verification: `verification/`, `verify/`, or filenames starting `verify`.

**Mother Brain spawns fixed:** 6,770 local frames pass. Full defeat remains pending.

[Current plan](docs/PLAN.md#work-order) | [Checkpoint recipes](movies/segments.toml) | [Codebase map](https://jimlundin.github.io/sm-decomp/#work-summary)
