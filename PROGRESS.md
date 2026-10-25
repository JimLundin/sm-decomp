# Engine migration

![Now: align recorded runs and retire collision code. Next: full-game checks and Samus banks. Outcome: safe owned state and less duplicate RAM code, then independent video/audio. Target: Map Rando.](docs/codebase-map/work-summary.svg)

| Measure | Count |
| --- | ---: |
| Production Rust lines* | 116516 |
| Legacy bank Rust lines | 54871 |
| Verification Rust lines | 39001 |
| Total Rust | 155517 |
| Lines containing `unsafe` | 4952 |
| Warnings (no-default library) | 375 |
| Required checkpoint checks passing | 24/44 |

*Physical lines by path; inline verification hooks remain in production. Verification: `verification/`, `verify/`, or filenames starting `verify`.

Change: **engine -33**, legacy **-157**, unsafe **-10**; verification **+4** source-registration lines. Inside-block sampling now shares owned terrain effects; `sm_94`: **308 lines remain**.

[Current plan](docs/PLAN.md#work-order) | [Checkpoint recipes](movies/segments.toml) | [Codebase map](https://jimlundin.github.io/sm-decomp/#work-summary)
