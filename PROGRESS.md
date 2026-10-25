# Engine migration

![Now: 26 of 44 gameplay checks pass; bomb jump uses owned state and sm_90 has 1,082 lines left. Next: Crocomire coverage and camera consolidation. Outcome: safe owned state and less duplicate RAM code, then independent video/audio. Target: Map Rando.](docs/codebase-map/work-summary.svg)

| Measure | Count |
| --- | ---: |
| Production Rust lines* | 116017 |
| Legacy bank Rust lines | 52057 |
| Verification Rust lines | 38944 |
| Total Rust | 154961 |
| Lines containing `unsafe` | 4683 |
| Warnings (no-default library) | 367 |
| Required checkpoint checks passing | 26/44 |

*Physical lines by path; inline verification hooks remain in production. Verification: `verification/`, `verify/`, or filenames starting `verify`.

Change: **Bomb jump uses owned state**; engine **+60 / -105 = -45**, legacy **-94**, unsafe **-7**, verification **-107**. `sm_90`: **1,082 lines left**.

Replacement **60%**; **172 exact comparisons and repeats**, **6 mutation kills**, **800 gameplay frames**. Horizontal jumps use bounded checks.

[Current plan](docs/PLAN.md#work-order) | [Checkpoint recipes](movies/segments.toml) | [Codebase map](https://jimlundin.github.io/sm-decomp/#work-summary)
