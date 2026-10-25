# Engine migration

![Now: align recorded runs and retire collision code. Next: full-game checks and Samus banks. Outcome: safe owned state and less duplicate RAM code, then independent video/audio. Target: Map Rando.](docs/codebase-map/work-summary.svg)

| Measure | Count |
| --- | ---: |
| Production Rust lines* | 116549 |
| Legacy bank Rust lines | 55028 |
| Verification Rust lines | 38997 |
| Total Rust | 155546 |
| Lines containing `unsafe` | 4962 |
| Warnings (no-default library) | 375 |
| Required checkpoint checks passing | 24/44 |

*Physical lines by path; inline verification hooks remain in production. Verification: `verification/`, `verify/`, or filenames starting `verify`.

Change: **engine -121**, legacy **-287**; verification **+4** source-registration lines. Grapple-release clearance now owns its four distances; `sm_94`: **465 lines remain**.

Unsafe count is unchanged: this family was already safe Rust. Natural grapple movie coverage remains pending.

[Current plan](docs/PLAN.md#work-order) | [Checkpoint recipes](movies/segments.toml) | [Codebase map](https://jimlundin.github.io/sm-decomp/#work-summary)
