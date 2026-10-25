# Engine migration

![Now: 25 of 44 gameplay checks pass; Mother Brain player reactions owned. Next: Wave Beam, reset and X-ray. Outcome: safe owned state and less duplicate RAM code, then independent video/audio. Target: Map Rando.](docs/codebase-map/work-summary.svg)

| Measure | Count |
| --- | ---: |
| Production Rust lines* | 116131 |
| Legacy bank Rust lines | 52913 |
| Verification Rust lines | 39040 |
| Total Rust | 155171 |
| Lines containing `unsafe` | 4743 |
| Warnings (no-default library) | 371 |
| Required checkpoint checks passing | 25/44 |

*Physical lines by path; inline verification hooks remain in production. Verification: `verification/`, `verify/`, or filenames starting `verify`.

Change: **Mother Brain player scripts shared**; engine **+91 / -117 = -26**, legacy **-103**, unsafe **-11**, verification **+4**. `sm_91`: **318 lines left**. Encounter movie coverage is pending.

[Current plan](docs/PLAN.md#work-order) | [Checkpoint recipes](movies/segments.toml) | [Codebase map](https://jimlundin.github.io/sm-decomp/#work-summary)
