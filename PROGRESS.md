# Engine migration

![Five implementation lanes active; 37 of 44 required checks pass.](docs/codebase-map/work-summary.svg)

| Measure | Count |
| --- | ---: |
| Production Rust lines* | 115350 |
| Legacy bank Rust lines | 49522 |
| Verification Rust lines | 39240 |
| Total Rust | 154590 |
| Lines containing `unsafe` | 4538 |
| Warnings (no-default library) | 354 |
| Required checkpoint checks passing | 37/44 |

*Physical lines by path; inline verification hooks remain in production. Verification: `verification/`, `verify/`, or filenames starting `verify`.

**Checkpoint reuse:** four local rows pass with zero setup frames; source, ROM and input guards retained.

[Current plan](docs/PLAN.md#work-order) | [Checkpoint recipes](movies/segments.toml) | [Codebase map](https://jimlundin.github.io/sm-decomp/#work-summary)
