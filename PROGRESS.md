# Engine migration

![Crocomire defeat verified; next: room seeds and three parallel bank retirements.](docs/codebase-map/work-summary.svg)

| Measure | Count |
| --- | ---: |
| Production Rust lines* | 115876 |
| Legacy bank Rust lines | 51664 |
| Verification Rust lines | 38936 |
| Total Rust | 154812 |
| Lines containing `unsafe` | 4627 |
| Warnings (no-default library) | 367 |
| Required checkpoint checks passing | 27/44 |

*Physical lines by path; inline verification hooks remain in production. Verification: `verification/`, `verify/`, or filenames starting `verify`.

**Samus drawing consolidated:** production -81, legacy -80, unsafe -21. Four changed rows + four smoke rows pass.

[Current plan](docs/PLAN.md#work-order) | [Checkpoint recipes](movies/segments.toml) | [Codebase map](https://jimlundin.github.io/sm-decomp/#work-summary)
