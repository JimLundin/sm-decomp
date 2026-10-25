# Engine migration

![Unused completion hooks removed; now: Crystal Flash setup and Crocomire coverage.](docs/codebase-map/work-summary.svg)

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

Change: **unused player completion hooks removed**; production **-13**, legacy **-8**, verification and unsafe unchanged.

**2 maintained segments pass: 10,959 setup + 629 measured frames; 8 exact settlement pairs.** Required coverage remains 26/44.

[Current plan](docs/PLAN.md#work-order) | [Checkpoint recipes](movies/segments.toml) | [Codebase map](https://jimlundin.github.io/sm-decomp/#work-summary)
