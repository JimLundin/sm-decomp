# Building and checking

You need your own unheadered Super Metroid (Japan, USA) (En,Ja) ROM at `rom/sm.smc`, SHA-1
`da957f0d63d14cb441d215462904c4fa8519c613`. No ROM or game data is in this repository.
Credits and licences for the tools and data the project uses are [below](#credits-and-licence).

```sh
cargo build --release -p app             # the production application: target/release/super-metroid
cargo build --profile verify -p tools     # the verifiers: target/verify/tools
bash tools/lane/gate.sh   # fmt, clippy, crate rules, doc links; tests and fault plan for what changed
```

The tests are the `cargo test` line of Quality (`.github/workflows/quality.yml`), which names
every crate but `oracle`.

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
the README) and verification jobs for the ROM content tests and the complete indexed replay
baseline. [STATUS](STATUS.md) shows how to run them locally.

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

## Compare selectable replay faults with literal edits

Set `SM_ROM` and `SM_CACHE` to the ROM and the lock-row cache, then run
`python3 tools/pilot_mutants.py`. To use the pinned CI fixtures, run
`gh workflow run ci.yml --ref BRANCH -f target=fault-pilot`.
The command takes two private copies of the same Git commit. It reads the field replacements and expected stops from
`tools/mutants.toml`. In the first copy it puts both replacements into one
replay binary. Each process selects one fault by its ordinal or runs clean.
In the second copy the literal runner compiles each edit. The command compares
the complete clean, faulty and restored receipts between the two methods.

| Fault | First differing cut | Field |
| --- | ---: | --- |
| Suppress Samus's reaction to the tail hit | 609 | `movement.pose` |
| Reverse her protection flicker | 614 | `samus-oam.piece-count` |

The pilot uses Cargo Gamma's runtime for selection. Its CLI discovers mutations
with its own operators. We found no documented way to import the reviewed field
replacements into the CLI. The test build pins `cargo-gamma-rt` to 0.8.0 because
upstream treats it as part of Cargo Gamma. The pilot checks the ordinary
dependency list and compares the resolved packages and features before building
with the runtime. [Gamma runtime](https://github.com/microsoft/ox-tools/blob/cargo-gamma-v0.8.0/crates/cargo-gamma-rt/src/lib.rs)
and [mutation operators](https://github.com/microsoft/ox-tools/tree/cargo-gamma-v0.8.0/crates/cargo-gamma-engine/src/ops/registry).

`cargo-mutants` 27.1.0 runs Cargo for each source edit. `cargo-schemata` 0.6.0
can compile many mutations together, but its published operators do not express
these two field substitutions and its runner credits a failing test process.
The played replay exits successfully when it reports a difference. `failspot`
0.2.0 supports explicit alternatives through a public API. It would also need an
enum for the sites and code in the replay process to enable one.
[cargo-mutants execution](https://github.com/sourcefrog/cargo-mutants/blob/v27.1.0/src/lab.rs#L242-L292),
[cargo-schemata commands and operators](https://github.com/t09tanaka/cargo-schemata/blob/v0.6.0/README.md)
and [failspot API](https://docs.rs/failspot/0.2.0/failspot/).

Keep the literal CI workers for the full manifest. The pilot can generate a
switch for a struct field. For example, `movement-double-update` adds statements
and still requires a literal build. Move a fault only after its own clean,
faulty and restored receipts agree and the worker plan accounts for it once.

## Setting up on Linux

The repository's tools run on Linux as they do on CI's ubuntu-24.04 runner. The Windows
workarounds (the DLL test runner, the clean room's Python test command) don't apply there:
`tools/lane/run_tests.sh` is `cargo test`. Only the clone and the GitHub issues carry over from
another machine; everything below is set up once.

**Tools.** Install rustup; the first `cargo` in the checkout installs the channel and
components of `rust-toolchain.toml` (`rustup show` does it at once). Install a C toolchain
(`build-essential`), git, Python 3.13 as `python3.13` or `python3` (the tools call `python3`;
`SM_PY` overrides it), the [Claude Code CLI](https://docs.claude.com/en/docs/claude-code) as
`claude` on `PATH`, `gh` logged in (`gh auth login`), `openssl` and `xz-utils`, and optionally
`shellcheck` and `cargo-deny` 0.22.0, which the gate runs where it is installed.

**Folders.** The tools keep their scratch, the lanes' worktrees and the seed under `$TEMP`
(else `$TMPDIR`, else `/tmp`). `/tmp` is often a RAM file system and is cleared on reboot, and the
seed's hard links need the seed and the lanes on one file system, so point `TEMP` at a disk
folder in your shell's profile, with the record cache's folder beside it:

```sh
export TEMP=$HOME/scratch SM_CACHE=$HOME/scratch/collab/ci-cache/rows
mkdir -p "$TEMP/collab"
```

**The ROM** goes at `rom/sm.smc` (above; `sha1sum rom/sm.smc` checks it).

**The record cache** is CI's, rebuilt from the encrypted asset that `CACHE_ASSET` and
`CACHE_SHA256` pin in `.github/workflows/verify.yml`, with a key made from the ROM. It lives at
`$SM_CACHE`, outside `/tmp`, so it survives a reboot; rebuild it when the pin changes:

```sh
asset=$(sed -n 's/^ *CACHE_ASSET: //p' .github/workflows/verify.yml | head -1)
sum=$(sed -n 's/^ *CACHE_SHA256: //p' .github/workflows/verify.yml | head -1)
curl -fL --retry 8 -o "$TEMP/$asset" "https://github.com/JimLundin/sm-decomp/releases/download/ci-data/$asset"
echo "$sum  $TEMP/$asset" | sha256sum --check
SM_SEED_PASSPHRASE=$({ printf 'sm-decomp-seeds-v1'; cat rom/sm.smc; } | sha256sum | cut -d' ' -f1)
export SM_SEED_PASSPHRASE
mkdir -p "$SM_CACHE"
set -o pipefail   # a failed decrypt must not leave a partial cache unnoticed
openssl enc -d -aes-256-cbc -pbkdf2 -iter 200000 -md sha256 -pass env:SM_SEED_PASSPHRASE \
  -in "$TEMP/$asset" | tar -xJ -C "$SM_CACHE"
rm "$TEMP/$asset"
```

The native fixtures some indexed cases also read come from the `tools/ci/fetch_*.sh` scripts
that `verify.yml`'s replay job runs, each given the ROM and one folder. `fetch_audio_native.sh`
needs `AUDIO_NATIVE_ASSET` and `AUDIO_NATIVE_SHA256`, pinned beside the cache's in `verify.yml`:

```sh
export AUDIO_NATIVE_ASSET=$(sed -n "s/^ *AUDIO_NATIVE_ASSET: //p" .github/workflows/verify.yml | head -1 | tr -d '"')
export AUDIO_NATIVE_SHA256=$(sed -n "s/^ *AUDIO_NATIVE_SHA256: //p" .github/workflows/verify.yml | head -1 | tr -d '"')
native=$TEMP/collab/audio-native
for f in audio_native sender_offers movement_touchdown connected_ordinary connected_run_slope \
  movement_early_map shot_contact native_search_packets health_tail; do
  bash tools/ci/fetch_$f.sh rom/sm.smc "$native" || break
done
```

Then point the variables of `verify.yml`'s witness environment at it: `SM_AUDIO_NATIVE` and
`SM_CONNECTED_SOURCE` at the folder, `SM_SEARCH_PACKET` at `$native/native-search-inputs-v1`,
`SM_TOUCH_PACKET` at `$native/native-search-order-v1`, and the rest as that list names them.

**The seed** is the warm build new lanes start from: `bash tools/lane/worktree.sh seed` once, and
again after big landings ([lane](../.agents/skills/lane/SKILL.md#resources)).

**The clean room.** `tools/cleanroom/export.sh` writes exports under `$CLEANROOM_ROOT` (default
`/cleanroom` off Windows, outside the home), and `tools/cleanroom/launch.sh` keeps its record in `~/.cleanroom`. Off Windows
the session runs in a box: a docker container with no network whose file system holds only the
export, the lane's home and temporary folder, and read-only `/usr`, toolchain, CLI and settings.
Inside it the launch's pattern rules still deny every place beside the export's path, as a
second layer; its header says what each layer holds and why. The session needs an API key:
`ANTHROPIC_API_KEY` in the environment, an `apiKeyHelper` or `ANTHROPIC_*` lines in
`~/.claude/settings.json`, or `CLEANROOM_AUTH`. A relay outside the box adds it to each request,
so it never enters the box.

### Before any confined session on the box

The operating system confines the session, and the pattern rules are a second layer. Before the
first clean-room lane on a machine, and after a new CLI version:

- **The exports' root is outside the home.** Make `/cleanroom` once, owned by you and private
  (`sudo install -d -o "$USER" -m 700 /cleanroom`). export.sh and launch.sh refuse an export
  under the home: host tools run git in the repositories they find there (VS Code's source
  control runs `git status` in every repository under its workspace, with the repository's own
  configuration), and the session writes the export. For the same reason, while a session runs,
  don't open `/cleanroom` (or a folder above it) in VS Code, and don't run git or a shell whose
  prompt runs git there; the launch reports what the box left in `.git` afterwards, but a tool
  that reads it during the session would already have run it.
- **Docker runs as you.** `docker info` works without `sudo` (your user is in the `docker`
  group). The box needs nothing else changed on the host: bubblewrap and the CLI's own sandbox
  need unprivileged user namespaces, which Ubuntu's AppArmor restricts, and docker doesn't.
- **The box is proven.** `bash tools/cleanroom/launch_config_test.sh` prints `LAUNCH CONFIG TEST
  OK`: it runs a launch with a fake CLI that tries the home, cargo's home, the canary record,
  the temporary folder, the checkout, a search from `/`, the docker socket, writes into the
  root, the lane and the export's git configuration, the network, the key and the relay's other
  targets, and each must fail, while the export, the relay and the test runner work. The fake
  CLI then commits a canary token, removes it, and plants a replace ref, a commondir with a
  textconv driver, a nested repository and a renamed `.git`; the launch must report each as a
  breach, and nothing planted may run. canary.sh must still find a removed token behind a
  textconv driver or a replace ref.
- **The key stays outside the box.** The dry run's `settings.json` holds the placeholder
  `cleanroom-relay` and `env.txt` holds no key. Other `ANTHROPIC_*` lines (custom headers, other
  keys) stay with the relay too; the launch names them.
- **The probe passes in the box** (below): every CHECK refused, and nothing outside the export
  is in the box to find, so CHECKs 29 to 31 (searches from `/` and the home) find nothing.

### First run on the box

1. `bash tools/lane/gate.sh` prints `GATE OK`, and `bash tools/lane/self_tests.sh` (all of them,
   with no base) prints `SELF TESTS OK`. It includes `launch_config_test.sh`, which checks the
   launch's Linux rules without a session.
2. The played runs reproduce `scenarios/replay/played.toml`: `tools/lane/played.sh` plays them
   all, given `SM_CACHE` and `SM_ROM`, the ROM's absolute path (a lane's worktree has no `rom/`).
   Each run prints a `PLAYED` line with its cuts held and stop beside its run's `held` and `stop`,
   ending `HOLDS` when they match, and the test passes when every run holds:

   ```sh
   SM_ROM=$PWD/rom/sm.smc bash tools/lane/played.sh
   ```

3. The launch probe and the canary ([launch.sh](../tools/cleanroom/launch.sh)'s header). On a
   fresh box the canary means something only once the forbidden sources are there to plant:
   copy them under `$TEMP` where `canary.sh` looks (`c4a/logs*`, `banklogs`, `collab/reviews`,
   `bank*`) or name them in `CANARY_ALSO`, and check `canary.sh list` shows them before the
   probe counts. Its CHECKs 1 to 3 name files there.

   ```sh
   bash tools/cleanroom/canary.sh plant
   bash tools/cleanroom/canary.sh list
   CLEANROOM_ALLOW_MISSING=1 bash tools/cleanroom/export.sh launch-probe
   ex=${CLEANROOM_ROOT:-/cleanroom}/launch-probe t=~/.cleanroom/transcripts/probe.jsonl
   CLEANROOM_DRY_RUN=1 bash tools/cleanroom/launch.sh --probe "$ex" "$t"
   bash tools/cleanroom/launch.sh --probe "$ex" "$t"
   bash tools/cleanroom/canary.sh check "$t" "$ex"
   rm -rf "$ex"
   ```

   The dry run writes the lane record and stops: read its `settings.json` deny list, `env.txt`
   and `box.txt`, the box's `docker run`. The probe passes when it prints `LAUNCH OK launch-probe`, the transcript shows
   every CHECK refused and every TASK done (TASK 7's runner ends `vector_runner: all targets
   passed`), and the canary check prints `CANARY CLEAN`.

## The README and the report

The README is generated from the tree:

```sh
cargo run -p tools --bin tools -- report --write
```

writes the whole `README.md`. It states the project's thesis and its progress over the
milestone's movie as bars and percentages, without frame numbers or counts: the frames the
reviewed played runs of [`scenarios/replay/played.toml`](../scenarios/replay/played.toml) hold,
over the milestone of [`docs/architecture/domains.toml`](architecture/domains.toml), as one bar
across the movie that marks the frames continuous from power-on, those that pass from later
seeds and those that don't pass yet. The stretches being worked, from
[`scenarios/replay/stretches.toml`](../scenarios/replay/stretches.toml), each show where they lie
in the movie on the same scale, drawn `━` on `·`, and a bar of their frames that pass in the
headline's marks, or a tick when all do; the first not yet whole is in bold. Its tests hold each
listed run to the lock and the stretches to the milestone's frames. When an integrator's work
moves a played run, its own commits change the run there and regenerate the README, and the root
reviews the figures at landing (Jim, 2026-11-25); full CI's replay witnesses hold every run to
the file.
`tools/check_readme.sh` in Quality, and verify's lib tests, which `tools/lane/gate.sh` runs when
the README's inputs change, fail when the README differs from what the tree generates.

The report is printed, not committed:

```sh
cargo run -p tools --bin tools -- report
```

prints the current clean models' size, budget and fit by domain and area, their lines, the
bridges, replay providers and engine services, the indexed replay evidence, the domain graph and
each model's structure. Its replay results separate scoped, integrated and whole-movie witnesses
and distinguish supported behaviour from expected unsupported boundaries.

## Credits and licence

[LakeSnes](https://github.com/angelo-wf/LakeSnes) (MIT) is linked into the oracle: the build
fetches its pinned sources and their licence; the repository carries only the project's fixes to it
(`crates/oracle/native/fixes/`).
[lsnes](https://tasvideos.org/EmulatorResources/Lsnes) and
[MesenCE](https://github.com/nesdev-org/MesenCE) run as separate processes; none of their code is
linked or copied. The crates in `vendor/` are registry crates, each with its own licence files.
Patrick Johnston's [bank logs](https://patrickjohnston.org/bank/) inform names
and behaviour. Original contributions are MIT ([LICENSE](../LICENSE)). Super Metroid is a trademark
of Nintendo; no ROM or game data is included.
