# Engine migration

![Now: 25 of 44 gameplay checks pass; sm_91 deleted. Next: Wave Beam, cannon and forced movement. Outcome: safe owned state and less duplicate RAM code, then independent video/audio. Target: Map Rando.](docs/codebase-map/work-summary.svg)

| Measure | Count |
| --- | ---: |
| Production Rust lines* | 116087 |
| Legacy bank Rust lines | 52594 |
| Verification Rust lines | 39043 |
| Total Rust | 155130 |
| Lines containing `unsafe` | 4722 |
| Warnings (no-default library) | 370 |
| Required checkpoint checks passing | 25/44 |

*Physical lines by path; inline verification hooks remain in production. Verification: `verification/`, `verify/`, or filenames starting `verify`.

Change: **sm_91 deleted**; engine **+234 / -279 = -45**, legacy **-240**, unsafe **-19**, verification **+3**.

Its shared families retired **1,949 legacy lines** with **408 fewer production lines**: **79% replacement**, including related sm_88/sm_90 work. Natural X-ray/boss/grapple coverage remains pending.

[Current plan](docs/PLAN.md#work-order) | [Checkpoint recipes](movies/segments.toml) | [Codebase map](https://jimlundin.github.io/sm-decomp/#work-summary)
