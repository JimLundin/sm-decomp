# Engine migration

![Room-seed coverage and parallel bank retirement.](docs/codebase-map/work-summary.svg)

| Measure | Count |
| --- | ---: |
| Production Rust lines* | 115606 |
| Legacy bank Rust lines | 50851 |
| Verification Rust lines | 39207 |
| Total Rust | 154813 |
| Lines containing `unsafe` | 4596 |
| Warnings (no-default library) | 362 |
| Required checkpoint checks passing | 27/44 |

*Physical lines by path; inline verification hooks remain in production. Verification: `verification/`, `verify/`, or filenames starting `verify`.

**Room-effects bank retired:** all 45 runnable rows pass; two selected mutations caught. The final relocation saves 3 production lines; Ceres remains raw and uncovered.

[Current plan](docs/PLAN.md#work-order) | [Checkpoint recipes](movies/segments.toml) | [Codebase map](https://jimlundin.github.io/sm-decomp/#work-summary)
