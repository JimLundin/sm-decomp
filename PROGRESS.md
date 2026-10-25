# Engine migration

![Crocomire melt repair verified; next: wall break and Crystal Flash setup.](docs/codebase-map/work-summary.svg)

| Measure | Count |
| --- | ---: |
| Production Rust lines* | 115931 |
| Legacy bank Rust lines | 51792 |
| Verification Rust lines | 38936 |
| Total Rust | 154867 |
| Lines containing `unsafe` | 4650 |
| Warnings (no-default library) | 367 |
| Required checkpoint checks passing | 26/44 |

*Physical lines by path; inline verification hooks remain in production. Verification: `verification/`, `verify/`, or filenames starting `verify`.

Change: **Crocomire melting uses the full graphics buffer**; production, legacy, unsafe and verification counts unchanged.

**4 regression frames pass; missing-plane mutation caught.** Boss completion remains pending final-source qualification.

[Current plan](docs/PLAN.md#work-order) | [Checkpoint recipes](movies/segments.toml) | [Codebase map](https://jimlundin.github.io/sm-decomp/#work-summary)
