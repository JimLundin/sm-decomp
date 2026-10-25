# Engine migration

![Now: recheck door timing; demo input owned. Next: full-game checks and demo setup. Outcome: safe owned state and less duplicate RAM code, then independent video/audio. Target: Map Rando.](docs/codebase-map/work-summary.svg)

| Measure | Count |
| --- | ---: |
| Production Rust lines* | 116398 |
| Legacy bank Rust lines | 54148 |
| Verification Rust lines | 39023 |
| Total Rust | 155421 |
| Lines containing `unsafe` | 4892 |
| Warnings (no-default library) | 371 |
| Required checkpoint checks passing | 24/44 |

*Physical lines by path; inline verification hooks remain in production. Verification: `verification/`, `verify/`, or filenames starting `verify`.

Change: **demo input owned**; engine **+198 / -189 = +9** for **18 owned bytes**, legacy **-175**, unsafe **-13**, verification **+7**. `sm_91`: **844 lines left**. Next: demo setup.

[Current plan](docs/PLAN.md#work-order) | [Checkpoint recipes](movies/segments.toml) | [Codebase map](https://jimlundin.github.io/sm-decomp/#work-summary)
