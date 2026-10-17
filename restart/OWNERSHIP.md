# Player motion, persistence and actor ownership

The runner can select the verified Rust implementations independently. These
options apply to both `run` and `play`; use a current executable with its matching
prepared native library. Each run records the selected owner and typed counters
in `result.toml`.

| Option | Default | Rust scope |
| --- | --- | --- |
| `--player-motion c|rust` | `c` | Vertical movement roots, solid-enemy contact, fixed-point movement and pose feedback |
| `--persistence c|rust` | Inherits `--gameplay` | Save/Load and the bounded SRAM codec |
| `--waver c|rust` | `c` | Waver initialization, main logic, animation selection and event instruction |
| `--ripper c|rust` | `c` | Ripper and JetPowerRipper initialization and horizontal patrol |

Rust motion and actors use synchronous native terrain services. The admitted
terrain and state domains are finite; unsupported ordinary runtime states can
refuse unchanged to C. Ownership counters distinguish completed Rust calls from
C calls and refusals. Native enemy scheduling, common instructions, shot/frozen
behavior, contact, death and rendering retain their existing ownership.

Player motion has 223 actual-C fixture pairs, 12 admission checks and 13 fatal
controls. Its selected 24,832-input replay matches all 13 accepted route
artifacts. Rust completes 539 upward and 1,230 downward roots, including 33
solid-enemy downward branches. C handles 655 upward and 9,792 downward calls.
Six definitions earn live reach; upward NoColl is verified by fixtures only.
See [ticket 25](../.scratch/restart-pipeline/issues/25-rust-player-motion.md).

Persistence installation has a focused six-call composition check in addition
to the earlier codec evidence. The same player-motion replay naturally completes
one Save and seven Loads in Rust; two Saves refuse safely to C. This establishes
runtime reach, without a file-reopen/continue or durability claim. See
[ticket 26](../.scratch/restart-pipeline/issues/26-persistence-runtime.md).

Waver has 28 actual-C pairs, 14 admission checks and 19 fatal controls. Eight
additional calls compare production Waver, player and persistence installers
with all ten owners registered. Ripper has 52 actual-C patrol pairs and one
Waver coexistence pair, 13 admission checks and 19 fatal controls. Its separate
production composition check covers both patrol variants and Waver. These actor
checks are fixtures, with no natural live encounter or original-ROM parity
claim. See [ticket 27](../.scratch/restart-pipeline/issues/27-rust-waver.md) and
[ticket 28](../.scratch/restart-pipeline/issues/28-rust-ripper-patrol.md) for exact
source identities, check results, retained dependencies and cleanup.

Player and actor callback CSVs retain ordered scalar events. They cover both
Rust completions and any native fallback, so use the typed ownership counters
alongside the records. Global terrain counts can include C; they are not counts
of completed Rust branches.
