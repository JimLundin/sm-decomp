# Engine migration

![Now: correct recorded-run timing; collision bank retired. Next: full-game checks and Samus input. Outcome: safe owned state and less duplicate RAM code, then independent video/audio. Target: Map Rando.](docs/codebase-map/work-summary.svg)

| Measure | Count |
| --- | ---: |
| Production Rust lines* | 116478 |
| Legacy bank Rust lines | 54543 |
| Verification Rust lines | 39010 |
| Total Rust | 155488 |
| Lines containing `unsafe` | 4928 |
| Warnings (no-default library) | 375 |
| Required checkpoint checks passing | 24/44 |

*Physical lines by path; inline verification hooks remain in production. Verification: `verification/`, `verify/`, or filenames starting `verify`.

Change: **sm_94 deleted**; engine **-39**, legacy **-329**, unsafe **-24**, verification **-11**. Next: Samus input. Bank-retirement changes: **541 net replacement lines / 734 removed (73.7%)**, above the 70% guide.

[Current plan](docs/PLAN.md#work-order) | [Checkpoint recipes](movies/segments.toml) | [Codebase map](https://jimlundin.github.io/sm-decomp/#work-summary)
