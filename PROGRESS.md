# Engine migration

![Now: 9,100 recorded steps aligned; demo input owned. Next: input timing and demo setup. Outcome: safe owned state and less duplicate RAM code, then independent video/audio. Target: Map Rando.](docs/codebase-map/work-summary.svg)

| Measure | Count |
| --- | ---: |
| Production Rust lines* | 116414 |
| Legacy bank Rust lines | 54148 |
| Verification Rust lines | 39023 |
| Total Rust | 155437 |
| Lines containing `unsafe` | 4892 |
| Warnings (no-default library) | 371 |
| Required checkpoint checks passing | 24/44 |

*Physical lines by path; inline verification hooks remain in production. Verification: `verification/`, `verify/`, or filenames starting `verify`.

Change: **door timing corrected**; **9,100 recorded steps**, **24/24 runnable checks requalified** (24/44 overall). Engine **+16** for the missing loading suspension; legacy, unsafe and verification unchanged.

[Current plan](docs/PLAN.md#work-order) | [Checkpoint recipes](movies/segments.toml) | [Codebase map](https://jimlundin.github.io/sm-decomp/#work-summary)
