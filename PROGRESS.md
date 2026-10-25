# Engine migration

![Crocomire defeat verified; next: room seeds and three parallel bank retirements.](docs/codebase-map/work-summary.svg)

| Measure | Count |
| --- | ---: |
| Production Rust lines* | 115858 |
| Legacy bank Rust lines | 51504 |
| Verification Rust lines | 38936 |
| Total Rust | 154794 |
| Lines containing `unsafe` | 4616 |
| Warnings (no-default library) | 365 |
| Required checkpoint checks passing | 27/44 |

*Physical lines by path; inline verification hooks remain in production. Verification: `verification/`, `verify/`, or filenames starting `verify`.

**Input dispatch consolidated:** production -4, legacy -29, unsafe -3; warnings -2. Six selected rows pass.

[Current plan](docs/PLAN.md#work-order) | [Checkpoint recipes](movies/segments.toml) | [Codebase map](https://jimlundin.github.io/sm-decomp/#work-summary)
