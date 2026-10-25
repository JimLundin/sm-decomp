# Engine migration

![Crystal Flash consolidated; next: Draygon and Crocomire defeat.](docs/codebase-map/work-summary.svg)

| Measure | Count |
| --- | ---: |
| Production Rust lines* | 115969 |
| Legacy bank Rust lines | 51873 |
| Verification Rust lines | 38936 |
| Total Rust | 154905 |
| Lines containing `unsafe` | 4656 |
| Warnings (no-default library) | 367 |
| Required checkpoint checks passing | 26/44 |

*Physical lines by path; inline verification hooks remain in production. Verification: `verification/`, `verify/`, or filenames starting `verify`.

Change: **Crystal Flash ammo consumption consolidated**; engine **+40 / -63 = -23**, legacy **-51**, unsafe **-6**, verification **-10**.

**138 exact controls + 9 retained cases; 10 mutation kills + failure-prefix check.** No natural Crystal Flash movie coverage.

[Current plan](docs/PLAN.md#work-order) | [Checkpoint recipes](movies/segments.toml) | [Codebase map](https://jimlundin.github.io/sm-decomp/#work-summary)
