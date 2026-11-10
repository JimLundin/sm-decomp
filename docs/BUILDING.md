# Building and checking

You need your own unheadered Super Metroid (Japan, USA) (En,Ja) ROM at `rom/sm.smc`, SHA-1
`da957f0d63d14cb441d215462904c4fa8519c613`. No ROM or game data is in this repository.

```sh
cargo build --release -p app             # the game, as it ships: target/release/super-metroid
cargo build --profile verify -p tools     # the verifiers: target/verify/tools
cargo test -p canonical -p canonical_derive -p data -p game -p runtime -p app -p verify -p tools \
  -p oracle_record -p clean_types -p clean_model_samus_motion -p clean_model_blocks \
  -p clean_model_camera -p clean_model_hud -p clean_model_progress -p clean_model_samus_looks
bash tools/lane/gate.sh   # fmt, clippy, crate rules, doc links and the report
```

Two profiles. `release` is the shipped game: one code generation unit and whole-program
link-time optimisation, slow to build. `verify` is the verifiers', their tests' and the
benchmarks': the same optimisation without link-time optimisation, incremental and in 16 units, so a
change rebuilds quickly. CI and every verifier command in [STATUS](STATUS.md) and
[ORACLE](ORACLE.md) build with `--profile verify`, whose output is in `target/verify/`.

The oracle crate links a native LakeSnes archive: `bash tools/oracle/build_lakesnes.sh` builds it
with Zig 0.15.2 and prints the directory to put on `LIBRARY_PATH`. On a Windows host that blocks
fresh test executables, run tests with `tools/lane/run_tests.sh <crate>...`.

CI (`.github/workflows/ci.yml`) runs Quality (format, lints, tests, the crate rules, doc links and
the report) and three verification jobs: the ROM content tests, the free-run gate against the
lock, and the lockstep ratchet. [STATUS](STATUS.md) shows how to run them locally.

## The status and the report

The README's status and [the rebuild report](REPORT.md) are generated together from the tree:

```sh
cargo run -p tools --bin tools -- report --write
```

writes `docs/REPORT.md` and the README's text between `<!-- status:begin -->` and
`<!-- status:end -->`. `tools/check_report.sh`, in Quality and `tools/lane/gate.sh`, fails when
either differs from what the tree generates. The status reads the plan's domains, their waves,
their progress and the rounds landed from
[`docs/architecture/domains.toml`](architecture/domains.toml), which the root changes in the
commit that lands a round.
