# Engine migration

![Five implementation lanes active; 35 of 44 required checks pass.](docs/codebase-map/work-summary.svg)

| Measure | Count |
| --- | ---: |
| Production Rust lines* | 115343 |
| Legacy bank Rust lines | 49579 |
| Verification Rust lines | 39240 |
| Total Rust | 154583 |
| Lines containing `unsafe` | 4544 |
| Warnings (no-default library) | 357 |
| Required checkpoint checks passing | 35/44 |

*Physical lines by path; inline verification hooks remain in production. Verification: `verification/`, `verify/`, or filenames starting `verify`.

**Build repairs verified locally:** library/example, detached compilation, DLL and four smoke rows pass.

[Current plan](docs/PLAN.md#work-order) | [Checkpoint recipes](movies/segments.toml) | [Codebase map](https://jimlundin.github.io/sm-decomp/#work-summary)
