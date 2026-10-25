# Engine migration

![Crocomire defeat verified; next: room seeds and three parallel bank retirements.](docs/codebase-map/work-summary.svg)

| Measure | Count |
| --- | ---: |
| Production Rust lines* | 115862 |
| Legacy bank Rust lines | 51533 |
| Verification Rust lines | 38936 |
| Total Rust | 154798 |
| Lines containing `unsafe` | 4619 |
| Warnings (no-default library) | 367 |
| Required checkpoint checks passing | 27/44 |

*Physical lines by path; inline verification hooks remain in production. Verification: `verification/`, `verify/`, or filenames starting `verify`.

**Frame dispatch migrated:** production -14, legacy -131, unsafe -8. Seven selected rows pass, including death.

[Current plan](docs/PLAN.md#work-order) | [Checkpoint recipes](movies/segments.toml) | [Codebase map](https://jimlundin.github.io/sm-decomp/#work-summary)
