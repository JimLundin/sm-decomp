# Engine migration

![Coverage at 31 of 44; four old banks retired.](docs/codebase-map/work-summary.svg)

| Measure | Count |
| --- | ---: |
| Production Rust lines* | 115493 |
| Legacy bank Rust lines | 50063 |
| Verification Rust lines | 39228 |
| Total Rust | 154721 |
| Lines containing `unsafe` | 4585 |
| Warnings (no-default library) | 362 |
| Required checkpoint checks passing | 31/44 |

*Physical lines by path; inline verification hooks remain in production. Verification: `verification/`, `verify/`, or filenames starting `verify`.

**Input import memory:** 2.79 GiB ? 90.5 MiB; identical replay inputs, 9.35s ? 5.75s. Engine code unchanged.

[Current plan](docs/PLAN.md#work-order) | [Checkpoint recipes](movies/segments.toml) | [Codebase map](https://jimlundin.github.io/sm-decomp/#work-summary)
