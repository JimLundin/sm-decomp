# Engine migration

![Now: 25 of 44 gameplay checks pass; Player initialization owned. Next: Wave Beam, X-ray and bank deletion. Outcome: safe owned state and less duplicate RAM code, then independent video/audio. Target: Map Rando.](docs/codebase-map/work-summary.svg)

| Measure | Count |
| --- | ---: |
| Production Rust lines* | 116131 |
| Legacy bank Rust lines | 52834 |
| Verification Rust lines | 39040 |
| Total Rust | 155171 |
| Lines containing `unsafe` | 4741 |
| Warnings (no-default library) | 371 |
| Required checkpoint checks passing | 25/44 |

*Physical lines by path; inline verification hooks remain in production. Verification: `verification/`, `verify/`, or filenames starting `verify`.

Change: **Player initialization owned; request cleanup shared**; engine **+99 / -99 = 0**, legacy **-79**, unsafe **-2**, verification **0**. `sm_91`: **238 lines left**. Ordered reset compatibility keeps this change size-neutral.

[Current plan](docs/PLAN.md#work-order) | [Checkpoint recipes](movies/segments.toml) | [Codebase map](https://jimlundin.github.io/sm-decomp/#work-summary)
