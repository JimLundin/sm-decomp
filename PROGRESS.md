# Engine migration

![Now: 25 of 44 gameplay checks pass; suit sequence owned. Next: remaining pickups and bosses, and equipment. Outcome: safe owned state and less duplicate RAM code, then independent video/audio. Target: Map Rando.](docs/codebase-map/work-summary.svg)

| Measure | Count |
| --- | ---: |
| Production Rust lines* | 116259 |
| Legacy bank Rust lines | 53593 |
| Verification Rust lines | 39028 |
| Total Rust | 155287 |
| Lines containing `unsafe` | 4850 |
| Warnings (no-default library) | 371 |
| Required checkpoint checks passing | 25/44 |

*Physical lines by path; inline verification hooks remain in production. Verification: `verification/`, `verify/`, or filenames starting `verify`.

Change: **Power Bomb pickup qualified: 25/44 checks**; capacity increase, message completion and **120 follow-up frames**, **0 failures**. Code counts unchanged. Published-TAS alignment remains **9,266 steps**.

[Current plan](docs/PLAN.md#work-order) | [Checkpoint recipes](movies/segments.toml) | [Codebase map](https://jimlundin.github.io/sm-decomp/#work-summary)
