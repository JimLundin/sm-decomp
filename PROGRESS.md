# Engine migration

![Now: 25 of 44 gameplay checks pass; player commands and equipment owned. Next: remaining pickups and bosses, Mother Brain scripts and reset. Outcome: safe owned state and less duplicate RAM code, then independent video/audio. Target: Map Rando.](docs/codebase-map/work-summary.svg)

| Measure | Count |
| --- | ---: |
| Production Rust lines* | 116157 |
| Legacy bank Rust lines | 53016 |
| Verification Rust lines | 39036 |
| Total Rust | 155193 |
| Lines containing `unsafe` | 4754 |
| Warnings (no-default library) | 371 |
| Required checkpoint checks passing | 25/44 |

*Physical lines by path; inline verification hooks remain in production. Verification: `verification/`, `verify/`, or filenames starting `verify`.

Change: **commands + equipment shared**; engine **+524 / -626 = -102**, legacy **-577**, unsafe **-96**, verification **+8**. `sm_91`: **421 lines left**; `sm_90`: **1,514**.

[Current plan](docs/PLAN.md#work-order) | [Checkpoint recipes](movies/segments.toml) | [Codebase map](https://jimlundin.github.io/sm-decomp/#work-summary)
