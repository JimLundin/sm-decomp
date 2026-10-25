# Engine migration

![Five implementation lanes active; 37 of 44 required checks pass.](docs/codebase-map/work-summary.svg)

| Measure | Count |
| --- | ---: |
| Production Rust lines* | 115299 |
| Legacy bank Rust lines | 49482 |
| Verification Rust lines | 39240 |
| Total Rust | 154539 |
| Lines containing `unsafe` | 4513 |
| Warnings (no-default library) | 357 |
| Required checkpoint checks passing | 37/44 |

*Physical lines by path; inline verification hooks remain in production. Verification: `verification/`, `verify/`, or filenames starting `verify`.

**Graphics-effect allocation shared:** typed definitions replace raw pointers; 44 production lines removed.

[Current plan](docs/PLAN.md#work-order) | [Checkpoint recipes](movies/segments.toml) | [Codebase map](https://jimlundin.github.io/sm-decomp/#work-summary)
