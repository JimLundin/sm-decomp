# Engine migration

![Now: 26 of 44 gameplay checks pass; X-ray beam controls consolidated and sm_88 has 1,543 lines left. Next: replay timing and bomb jump. Outcome: safe owned state and less duplicate RAM code, then independent video/audio. Target: Map Rando.](docs/codebase-map/work-summary.svg)

| Measure | Count |
| --- | ---: |
| Production Rust lines* | 116062 |
| Legacy bank Rust lines | 52151 |
| Verification Rust lines | 39051 |
| Total Rust | 155113 |
| Lines containing `unsafe` | 4690 |
| Warnings (no-default library) | 367 |
| Required checkpoint checks passing | 26/44 |

*Physical lines by path; inline verification hooks remain in production. Verification: `verification/`, `verify/`, or filenames starting `verify`.

Change: **X-ray steering consolidated**; engine **+92 / -122 = -30**, legacy **-110**, unsafe **-11**, verification unchanged. `sm_88`: **1,543 lines left**.

Replacement **74%**; **101 comparisons**, **2.88 million steering evaluations**, **7 mutation kills**. Natural X-ray movie coverage remains pending.

[Current plan](docs/PLAN.md#work-order) | [Checkpoint recipes](movies/segments.toml) | [Codebase map](https://jimlundin.github.io/sm-decomp/#work-summary)
