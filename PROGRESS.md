# Engine migration

![Crocomire drop repair verified; next: finish the fight and adopt Crystal Flash and Draygon reductions.](docs/codebase-map/work-summary.svg)

| Measure | Count |
| --- | ---: |
| Production Rust lines* | 115992 |
| Legacy bank Rust lines | 51924 |
| Verification Rust lines | 38946 |
| Total Rust | 154938 |
| Lines containing `unsafe` | 4662 |
| Warnings (no-default library) | 367 |
| Required checkpoint checks passing | 26/44 |

*Physical lines by path; inline verification hooks remain in production. Verification: `verification/`, `verify/`, or filenames starting `verify`.

Change: **shared enemy scratch restored**; engine **+120 / -87 = +33**, legacy **-1**, unsafe unchanged, verification **+2**. Correctness repair; no bank retired.

**167,281 setup + 4 regression frames**, **20 original-CPU comparisons**, **2 mutation kills**. Crocomire defeat remains pending; the 26 required recipes were not all rerun.

[Current plan](docs/PLAN.md#work-order) | [Checkpoint recipes](movies/segments.toml) | [Codebase map](https://jimlundin.github.io/sm-decomp/#work-summary)
