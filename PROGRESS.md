# Engine migration

![Now: 9,266 recorded steps aligned; suit sequence owned. Next: audio upload timing and equipment. Outcome: safe owned state and less duplicate RAM code, then independent video/audio. Target: Map Rando.](docs/codebase-map/work-summary.svg)

| Measure | Count |
| --- | ---: |
| Production Rust lines* | 116259 |
| Legacy bank Rust lines | 53593 |
| Verification Rust lines | 39028 |
| Total Rust | 155287 |
| Lines containing `unsafe` | 4850 |
| Warnings (no-default library) | 371 |
| Required checkpoint checks passing | 24/44 |

*Physical lines by path; inline verification hooks remain in production. Verification: `verification/`, `verify/`, or filenames starting `verify`.

Change: **Varia/Gravity sequence shared**; engine **+306 / -407 = -101**, legacy **-396**, unsafe **-31**, verification **+5**. `sm_91`: **591 lines left**. Next: equipment.

[Current plan](docs/PLAN.md#work-order) | [Checkpoint recipes](movies/segments.toml) | [Codebase map](https://jimlundin.github.io/sm-decomp/#work-summary)
