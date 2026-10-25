# Engine migration

![Now: 26 of 44 gameplay checks pass, including Wave Beam; sm_91 deleted and cannon control owned. Next: Crocomire and scripted movement. Outcome: safe owned state and less duplicate RAM code, then independent video/audio. Target: Map Rando.](docs/codebase-map/work-summary.svg)

| Measure | Count |
| --- | ---: |
| Production Rust lines* | 116068 |
| Legacy bank Rust lines | 52390 |
| Verification Rust lines | 39047 |
| Total Rust | 155115 |
| Lines containing `unsafe` | 4707 |
| Warnings (no-default library) | 370 |
| Required checkpoint checks passing | 26/44 |

*Physical lines by path; inline verification hooks remain in production. Verification: `verification/`, `verify/`, or filenames starting `verify`.

Change: **Wave Beam pickup checked**; **565 frames, 0 failures**. Required coverage **25 → 26/44**; production and verification Rust unchanged.

Next: **Crocomire** and **scripted movement**. Wave combat coverage remains incomplete.

[Current plan](docs/PLAN.md#work-order) | [Checkpoint recipes](movies/segments.toml) | [Codebase map](https://jimlundin.github.io/sm-decomp/#work-summary)
