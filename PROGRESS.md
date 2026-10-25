# Engine migration

![Five implementation lanes active; 40 of 44 required checks pass.](docs/codebase-map/work-summary.svg)

| Measure | Count |
| --- | ---: |
| Production Rust lines* | 115214 |
| Legacy bank Rust lines | 48376 |
| Verification Rust lines | 39348 |
| Total Rust | 154562 |
| Lines containing `unsafe` | 4305 |
| Warnings (no-default library) | 299 (Rio + Fireflea composition) |
| Required checkpoint checks passing | 40/44 |

*Physical lines by path; inline verification hooks remain in production. Verification: `verification/`, `verify/`, or filenames starting `verify`.

Rio uses checked actors: 49 raw dereferences removed; safety adds 69 production lines. Hosted verification pending.

[Current plan](docs/PLAN.md#work-order) | [Checkpoint recipes](movies/segments.toml) | [Codebase map](https://jimlundin.github.io/sm-decomp/#work-summary)
