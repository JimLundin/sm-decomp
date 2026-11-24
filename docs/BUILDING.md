# Building and checking

You need your own unheadered Super Metroid (Japan, USA) (En,Ja) ROM at `rom/sm.smc`, SHA-1
`da957f0d63d14cb441d215462904c4fa8519c613`. No ROM or game data is in this repository.

```sh
cargo build --release -p app             # the production application: target/release/super-metroid
cargo build --profile verify -p tools     # the verifiers: target/verify/tools
cargo test -p canonical -p canonical_derive -p data -p game -p runtime -p app -p verify -p tools \
  -p oracle_record -p types -p samus_movement -p world_blocks \
  -p world_camera -p interface_hud -p run_progress -p samus_appearance
bash tools/lane/gate.sh   # fmt, clippy, crate rules, doc links and the report
```

The application currently refuses power-on with an explicit missing-production-owner result.
Building it does not imply playable boot or whole-movie support. Use indexed scoped cases to
exercise available production entries; [STATUS](STATUS.md) gives selection and baseline commands.

Two profiles. `release` is the shipped game: one code generation unit and whole-program
link-time optimisation, slow to build. `verify` is the verifiers', their tests' and the
benchmarks': the same optimisation without link-time optimisation, incremental and in 16 units, so a
change rebuilds quickly. CI and every verifier command in [STATUS](STATUS.md) and
[ORACLE](ORACLE.md) build with `--profile verify`, whose output is in `target/verify/`.

The oracle crate links a native LakeSnes archive: `bash tools/oracle/build_lakesnes.sh` builds it
with Zig 0.15.2 and prints the directory to put on `LIBRARY_PATH`. On a Windows host that blocks
fresh test executables, run tests with `tools/lane/run_tests.sh <crate>...`.

CI (`.github/workflows/ci.yml`) runs Quality (format, lints, tests, the crate rules, doc links and
the report) and verification jobs for the ROM content tests and the complete indexed replay baseline. [STATUS](STATUS.md) shows how to run them locally.

## Run one verification check

Choose a pushed branch or tag in **Actions → CI → Run workflow** and enter an indexed case ID
or a fully qualified `verify` library test name. The same selection works from the command line:

```sh
gh workflow run ci.yml --ref BRANCH -f target=connected-ordinary/carried-running-camera
gh workflow run ci.yml --ref BRANCH -f target=replay::native_search::tests::native_search_contract
```

CI installs the pinned fixtures and checks the selected case against its reviewed baseline,
or runs exactly one matching library test, including ignored tests. A failing job prints the
command to rerun it on the same commit. An empty target runs full CI, where
[`tools/ci/replay_parallel.sh`](../tools/ci/replay_parallel.sh) runs the required witnesses beside
three shards of the indexed check and `tools replay merge` joins the shard receipts into the one
receipt an unsharded check writes.

Locally, `bash tools/ci/replay.sh --case ID` builds the verifier from the checkout.
Set `SM_CACHE`, `SM_ROM` and `SM_AUDIO_NATIVE` to the sources the case uses.
For a library test, set its [fixture variables](../.github/workflows/verify.yml) to absolute
paths and run `python3 tools/ci/required_witnesses.py TEST`. For example, the native search
contract reads `SM_ROM` and `SM_SEARCH_PACKET` and writes its receipt to `SM_SEARCH_RECEIPT`.
On a Windows host that blocks new executables, the hosted command above runs the same test.

## The status and the report

The README and [the rebuild report](REPORT.md) are generated together from the tree:

```sh
cargo run -p tools --bin tools -- report --write
```

writes `docs/REPORT.md`, the timeline's picture `docs/timeline.svg` and the whole `README.md`.
`tools/check_report.sh`, in Quality and `tools/lane/gate.sh`, fails when any of them differs from
what the tree generates. The report describes the current clean models and indexed replay
claims. Its replay results separate scoped, integrated and whole-movie witnesses and distinguish
supported behaviour from expected unsupported boundaries. Historical claims have explicit
transition dispositions. The README states the project's thesis and its progress in frames of
the milestone's movie: the frames the reviewed played runs of
[`scenarios/replay/played.toml`](../scenarios/replay/played.toml) hold, over the milestone and
rungs of [`docs/architecture/domains.toml`](architecture/domains.toml). The report's tests hold
each listed run to the lock; the root changes a run there when a reviewed played run changes it.
