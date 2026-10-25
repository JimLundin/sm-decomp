# Engine migration

![Now: 25 of 44 gameplay checks pass; Closing-gate parameter fixed. Next: Wave Beam, X-ray and bank deletion. Outcome: safe owned state and less duplicate RAM code, then independent video/audio. Target: Map Rando.](docs/codebase-map/work-summary.svg)

| Measure | Count |
| --- | ---: |
| Production Rust lines* | 116132 |
| Legacy bank Rust lines | 52834 |
| Verification Rust lines | 39040 |
| Total Rust | 155172 |
| Lines containing `unsafe` | 4741 |
| Warnings (no-default library) | 371 |
| Required checkpoint checks passing | 25/44 |

*Physical lines by path; inline verification hooks remain in production. Verification: `verification/`, `verify/`, or filenames starting `verify`.

Change: **Closing-gate projectile parameter fixed**; engine **+1**, legacy **0**, unsafe **0**, verification Rust **0**. The 10-frame regression catches the old fault; Wave acquisition remains pending.

[Current plan](docs/PLAN.md#work-order) | [Checkpoint recipes](movies/segments.toml) | [Codebase map](https://jimlundin.github.io/sm-decomp/#work-summary)
