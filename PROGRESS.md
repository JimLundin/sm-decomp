# Engine migration

![Now: align recorded runs and retire collision code. Next: full-game checks and Samus banks. Outcome: safe owned state and less duplicate RAM code, then independent video/audio. Target: Map Rando.](docs/codebase-map/work-summary.svg)

| Measure | Count |
| --- | ---: |
| Production Rust lines* | 116517 |
| Legacy bank Rust lines | 54872 |
| Verification Rust lines | 39021 |
| Total Rust | 155538 |
| Lines containing `unsafe` | 4952 |
| Warnings (no-default library) | 375 |
| Required checkpoint checks passing | 24/44 |

*Physical lines by path; inline verification hooks remain in production. Verification: `verification/`, `verify/`, or filenames starting `verify`.

Change: **8,250 aligned steps pass**; menu-state bug fixed. Engine **+1**; verification **+20** for the shared state expectation. Next: door timing; required coverage remains **24/44**.

[Current plan](docs/PLAN.md#work-order) | [Checkpoint recipes](movies/segments.toml) | [Codebase map](https://jimlundin.github.io/sm-decomp/#work-summary)
