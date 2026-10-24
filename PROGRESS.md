# Engine migration

![Now: Power Bomb checks and collision-bank retirement. Next: safe state, an independent engine and Map Rando.](docs/codebase-map/work-summary.svg)

| Measure | Count |
| --- | ---: |
| Production Rust lines* | 116670 |
| Legacy bank Rust lines | 55315 |
| Verification Rust lines | 38993 |
| Total Rust | 155663 |
| Lines containing `unsafe` | 4962 |
| Warnings (no-default library) | 376 |
| Required checkpoint checks passing | 24/44 |

*Physical lines by path; inline verification hooks remain in production. Verification: `verification/`, `verify/`, or filenames starting `verify`.

Change: **engine -90** (-88 excluding closure reflow); verification **0**; progress command **+3**. Bank `sm_94`: **734 lines remain**.

Unsafe text fell by 2 removed attributes; existing unsafe operations remain. Warnings exclude repeated diagnostics.

[Current plan](docs/PLAN.md#work-order) | [Checkpoint recipes](movies/segments.toml) | [Codebase map](https://jimlundin.github.io/sm-decomp/#work-summary)
