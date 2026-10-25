# Engine migration

![Crystal Flash and Draygon consolidated; now: remaining Samus bank and Crocomire coverage.](docs/codebase-map/work-summary.svg)

| Measure | Count |
| --- | ---: |
| Production Rust lines* | 115944 |
| Legacy bank Rust lines | 51800 |
| Verification Rust lines | 38936 |
| Total Rust | 154880 |
| Lines containing `unsafe` | 4650 |
| Warnings (no-default library) | 367 |
| Required checkpoint checks passing | 26/44 |

*Physical lines by path; inline verification hooks remain in production. Verification: `verification/`, `verify/`, or filenames starting `verify`.

Change: **Draygon grab, struggle and release use owned state**; engine **+56 / -81 = -25**, legacy **-73**, unsafe **-6**, verification unchanged.

**84 exact controls; 5 mutation kills + failure-prefix check.** No natural Draygon movie coverage.

[Current plan](docs/PLAN.md#work-order) | [Checkpoint recipes](movies/segments.toml) | [Codebase map](https://jimlundin.github.io/sm-decomp/#work-summary)
