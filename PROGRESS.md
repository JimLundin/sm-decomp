# Engine migration

![Crocomire defeat verified; next: room seeds and three parallel bank retirements.](docs/codebase-map/work-summary.svg)

| Measure | Count |
| --- | ---: |
| Production Rust lines* | 115643 |
| Legacy bank Rust lines | 51079 |
| Verification Rust lines | 38936 |
| Total Rust | 154579 |
| Lines containing `unsafe` | 4608 |
| Warnings (no-default library) | 362 |
| Required checkpoint checks passing | 27/44 |

*Physical lines by path; inline verification hooks remain in production. Verification: `verification/`, `verify/`, or filenames starting `verify`.

**Morph eye-beam animation consolidated:** production -37, legacy -136. Four smoke rows pass, including the full Morph pickup window.

[Current plan](docs/PLAN.md#work-order) | [Checkpoint recipes](movies/segments.toml) | [Codebase map](https://jimlundin.github.io/sm-decomp/#work-summary)
