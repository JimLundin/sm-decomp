# Engine migration

![Five implementation lanes active; 39 of 44 required checks pass.](docs/codebase-map/work-summary.svg)

| Measure | Count |
| --- | ---: |
| Production Rust lines* | 115211 |
| Legacy bank Rust lines | 49077 |
| Verification Rust lines | 39335 |
| Total Rust | 154546 |
| Lines containing `unsafe` | 4363 |
| Warnings (no-default library) | 302 |
| Required checkpoint checks passing | 39/44 |

*Physical lines by path; inline verification hooks remain in production. Verification: `verification/`, `verify/`, or filenames starting `verify`.

Gravity Suit + Grapple: fresh awards, message completion and full follow-up windows pass. Verification code +91; production unchanged.

[Current plan](docs/PLAN.md#work-order) | [Checkpoint recipes](movies/segments.toml) | [Codebase map](https://jimlundin.github.io/sm-decomp/#work-summary)
