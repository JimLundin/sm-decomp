# Engine migration

![Crocomire defeat verified; next: room seeds and three parallel bank retirements.](docs/codebase-map/work-summary.svg)

| Measure | Count |
| --- | ---: |
| Production Rust lines* | 115931 |
| Legacy bank Rust lines | 51792 |
| Verification Rust lines | 38936 |
| Total Rust | 154867 |
| Lines containing `unsafe` | 4650 |
| Warnings (no-default library) | 367 |
| Required checkpoint checks passing | 27/44 |

*Physical lines by path; inline verification hooks remain in production. Verification: `verification/`, `verify/`, or filenames starting `verify`.

**Crocomire defeat verified:** 4,258 frames + two four-frame regressions, zero differences. Size unchanged.

[Current plan](docs/PLAN.md#work-order) | [Checkpoint recipes](movies/segments.toml) | [Codebase map](https://jimlundin.github.io/sm-decomp/#work-summary)
