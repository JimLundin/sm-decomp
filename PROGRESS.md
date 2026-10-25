# Engine migration

![Now: 26 of 44 gameplay checks pass; scripted movement owned and sm_90 has 1,176 lines left. Next: Crocomire and bomb jump. Outcome: safe owned state and less duplicate RAM code, then independent video/audio. Target: Map Rando.](docs/codebase-map/work-summary.svg)

| Measure | Count |
| --- | ---: |
| Production Rust lines* | 116092 |
| Legacy bank Rust lines | 52261 |
| Verification Rust lines | 39051 |
| Total Rust | 155143 |
| Lines containing `unsafe` | 4701 |
| Warnings (no-default library) | 370 |
| Required checkpoint checks passing | 26/44 |

*Physical lines by path; inline verification hooks remain in production. Verification: `verification/`, `verify/`, or filenames starting `verify`.

Change: **Scripted movement owned**; engine **+217 / -193 = +24**, legacy **-129**, unsafe **-6**, verification **+4**. `sm_90`: **1,176 lines left**.

Growth buys named handlers and lossless state boundaries; includes **2 test lines**. **158 comparisons, 6 mutation kills**; natural Draygon coverage pending.

[Current plan](docs/PLAN.md#work-order) | [Checkpoint recipes](movies/segments.toml) | [Codebase map](https://jimlundin.github.io/sm-decomp/#work-summary)
