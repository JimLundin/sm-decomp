# Engine migration

![Now: 26 of 44 gameplay checks pass; X-ray table pointers are removed and sm_88 has 1,454 lines left. Next: Crocomire coverage and Crystal Flash. Outcome: safe owned state and less duplicate RAM code, then independent video/audio. Target: Map Rando.](docs/codebase-map/work-summary.svg)

| Measure | Count |
| --- | ---: |
| Production Rust lines* | 115959 |
| Legacy bank Rust lines | 51925 |
| Verification Rust lines | 38944 |
| Total Rust | 154903 |
| Lines containing `unsafe` | 4662 |
| Warnings (no-default library) | 367 |
| Required checkpoint checks passing | 26/44 |

*Physical lines by path; inline verification hooks remain in production. Verification: `verification/`, `verify/`, or filenames starting `verify`.

Change: **X-ray table pointers removed**; engine **+346 / -379 = -33**, legacy **-89**, unsafe **-17**, verification unchanged. `sm_88`: **1,454 lines left**.

**497 exact comparisons**, **7 mutation kills + 1 failure-prefix check**, **1,200 replay frames**. Natural X-ray coverage remains pending.

[Current plan](docs/PLAN.md#work-order) | [Checkpoint recipes](movies/segments.toml) | [Codebase map](https://jimlundin.github.io/sm-decomp/#work-summary)
