# Engine migration

![Now: gameplay checks and collision cleanup. Next: safe game state, an independent engine and Map Rando.](docs/codebase-map/work-summary.svg)

| Measure | Count |
| --- | ---: |
| Production Rust lines* | 116760 |
| Verification Rust lines | 38993 |
| Total Rust | 155753 |
| Lines containing `unsafe` | 4964 |
| Warnings (no-default library) | 378 |
| Required checkpoint checks passing | 24/44 |

*Physical lines by path; inline verification hooks remain in production. Verification: `verification/`, `verify/`, or filenames starting `verify`.

Change: **engine -202** (-198 excluding reflow); verification **+111**; support/native **0**; total **-91**.

The table includes 2 added inline verification lines in production. Unsafe text: 2 attributes removed, 1 verification call added. Warnings exclude repeated diagnostics.

[Current plan](docs/PLAN.md#work-order) | [Checkpoint recipes](movies/segments.toml) | [Codebase map](https://jimlundin.github.io/sm-decomp/#work-summary)
