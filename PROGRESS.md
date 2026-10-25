# Engine migration

![Crocomire defeat verified; next: room seeds and three parallel bank retirements.](docs/codebase-map/work-summary.svg)

| Measure | Count |
| --- | ---: |
| Production Rust lines* | 115828 |
| Legacy bank Rust lines | 51477 |
| Verification Rust lines | 38936 |
| Total Rust | 154764 |
| Lines containing `unsafe` | 4613 |
| Warnings (no-default library) | 363 |
| Required checkpoint checks passing | 27/44 |

*Physical lines by path; inline verification hooks remain in production. Verification: `verification/`, `verify/`, or filenames starting `verify`.

**X-ray dispatch consolidated:** production -30, legacy -27, unsafe -3; warnings -2. Four smoke rows pass; natural X-ray remains uncovered.

[Current plan](docs/PLAN.md#work-order) | [Checkpoint recipes](movies/segments.toml) | [Codebase map](https://jimlundin.github.io/sm-decomp/#work-summary)
