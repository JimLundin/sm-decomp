# Engine migration

![Now: 26 of 44 gameplay checks pass; camera distances are consolidated and sm_90 has 1,039 lines left. Next: Crocomire coverage and Crystal Flash. Outcome: safe owned state and less duplicate RAM code, then independent video/audio. Target: Map Rando.](docs/codebase-map/work-summary.svg)

| Measure | Count |
| --- | ---: |
| Production Rust lines* | 115992 |
| Legacy bank Rust lines | 52014 |
| Verification Rust lines | 38944 |
| Total Rust | 154936 |
| Lines containing `unsafe` | 4679 |
| Warnings (no-default library) | 367 |
| Required checkpoint checks passing | 26/44 |

*Physical lines by path; inline verification hooks remain in production. Verification: `verification/`, `verify/`, or filenames starting `verify`.

Change: **Camera distances consolidated**; engine **+45 / -70 = -25**, legacy **-43**, unsafe **-4**, verification unchanged. `sm_90`: **1,039 lines left**.

Replacement **64%**; **340 exact comparisons and repeats**, **4 mutation kills**, **440 replay frames**. Broad aliases use bounded checks.

[Current plan](docs/PLAN.md#work-order) | [Checkpoint recipes](movies/segments.toml) | [Codebase map](https://jimlundin.github.io/sm-decomp/#work-summary)
