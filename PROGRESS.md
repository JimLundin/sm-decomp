# Engine migration

![Five implementation lanes active; 37 of 44 required checks pass.](docs/codebase-map/work-summary.svg)

| Measure | Count |
| --- | ---: |
| Production Rust lines* | 115340 |
| Legacy bank Rust lines | 49512 |
| Verification Rust lines | 39240 |
| Total Rust | 154580 |
| Lines containing `unsafe` | 4546 |
| Warnings (no-default library) | 357 |
| Required checkpoint checks passing | 37/44 |

*Physical lines by path; inline verification hooks remain in production. Verification: `verification/`, `verify/`, or filenames starting `verify`.

**Backend contracts corrected:** checked RAM remains; hardware calls require an initialized backend. Eight local rows pass.

[Current plan](docs/PLAN.md#work-order) | [Checkpoint recipes](movies/segments.toml) | [Codebase map](https://jimlundin.github.io/sm-decomp/#work-summary)
