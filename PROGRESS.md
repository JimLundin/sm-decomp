# Engine migration

![Coverage at 31 of 44; four old banks retired.](docs/codebase-map/work-summary.svg)

| Measure | Count |
| --- | ---: |
| Production Rust lines* | 115573 |
| Legacy bank Rust lines | 50260 |
| Verification Rust lines | 39228 |
| Total Rust | 154801 |
| Lines containing `unsafe` | 4591 |
| Warnings (no-default library) | 362 |
| Required checkpoint checks passing | 31/44 |

*Physical lines by path; inline verification hooks remain in production. Verification: `verification/`, `verify/`, or filenames starting `verify`.

**Samus bank retired:** 49 runnable rows pass; all 3 selected mutations caught. Final step saves 12 production lines; camera compatibility still uses raw RAM.

[Current plan](docs/PLAN.md#work-order) | [Checkpoint recipes](movies/segments.toml) | [Codebase map](https://jimlundin.github.io/sm-decomp/#work-summary)
