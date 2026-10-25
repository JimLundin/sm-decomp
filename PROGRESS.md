# Engine migration

![Crocomire defeat verified; next: room seeds and three parallel bank retirements.](docs/codebase-map/work-summary.svg)

| Measure | Count |
| --- | ---: |
| Production Rust lines* | 115808 |
| Legacy bank Rust lines | 51343 |
| Verification Rust lines | 38936 |
| Total Rust | 154744 |
| Lines containing `unsafe` | 4609 |
| Warnings (no-default library) | 362 |
| Required checkpoint checks passing | 27/44 |

*Physical lines by path; inline verification hooks remain in production. Verification: `verification/`, `verify/`, or filenames starting `verify`.

**Room effects consolidated:** production -20, legacy -134, unsafe -4. sm89 retains its 48-line Ceres family.

[Current plan](docs/PLAN.md#work-order) | [Checkpoint recipes](movies/segments.toml) | [Codebase map](https://jimlundin.github.io/sm-decomp/#work-summary)
