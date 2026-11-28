# Building and checking

You need your own unheadered Super Metroid (Japan, USA) (En,Ja) ROM at `rom/sm.smc`, whose
SHA-256 is `ROM_SHA256` in the [replay data pins](../tools/src/sm_tools/ci/core/replay_data.py).
No ROM or game data is in this repository.
Credits and licences for the tools and data the project uses are [below](#credits-and-licence).

```sh
cargo build --release -p app             # the production application: target/release/super-metroid
cargo build --profile verify -p tools     # the verifiers: target/verify/tools
uv run --locked --project tools gate     # fmt, clippy, crate rules, doc links; tests and fault manifest validation for what changed
```

The tests are CI's rust job's `cargo nextest run --locked --workspace --no-fail-fast`
(`.github/workflows/ci.yml`), at the cargo-nextest version the home standards' `tools.env`
pins. Nextest runs the test binaries and their tests at once across the cores, where `cargo test
--workspace` runs the same tests one binary after another. `.config/nextest.toml` starts the
longest test first, verify's check of the committed runs against the lock. The workspace has no
doctests, which nextest would skip, so a crate that adds one also adds `cargo test --doc` to
the rust job.

The Python tools are one uv project in `tools/`, on the Python the
[standards](https://github.com/JimLundin/standards)' `tools.env` pins, which uv installs.
`tools/pyproject.toml` names each tool's command, and `uv run --locked --project tools run-tests`
runs their tests with pytest, each test module in its own process.

The application currently refuses power-on with an explicit missing-production-owner result.
Building it does not imply playable boot or whole-movie support. Use indexed scoped cases to
exercise available production entries; [STATUS](STATUS.md) gives selection and baseline commands.

Two profiles. `release` is the shipped game: one code generation unit and whole-program
link-time optimisation, slow to build. `verify` is the verifiers', their tests' and the
benchmarks': the same optimisation without link-time optimisation, incremental and in 16 units, so a
change rebuilds quickly. CI and every verifier command in [STATUS](STATUS.md) and
[ORACLE](ORACLE.md) build with `--profile verify`, whose output is in `target/verify/`.

The oracle drives lsnes rr2-beta23 as a separate process and links no emulator code. Set
`SM_LSNES` to its executable or put it on `PATH` as `lsnes` to capture
([ORACLE §2](ORACLE.md#2-toolchain-and-layout)); building and testing the oracle crate need neither.

`tools engine-size target/release/super-metroid scenarios/engine-size.toml` reports the release
binary's file size, its approximate stripped size and its section sizes against the committed
baseline, with the largest crates and functions by code bytes. `--write` records the build as the
new baseline. CI's engine-size job builds `app` in its release profile and puts that
report in its job summary when the engine's inputs change and on scheduled and dispatched runs.
No size fails it.

CI (`.github/workflows/ci.yml`) runs the rules, rust and python jobs (format, the crate rules, doc
links, the README, lints, tests and the Python tooling) and the replay and faults jobs for the ROM
content tests, the complete indexed replay baseline and the planted faults, each when a path it
reads changes. [STATUS](STATUS.md) shows how to run them locally.

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
`uv run --locked --project tools replay baseline` runs the required witnesses beside three shards of
the indexed check and `tools replay merge` joins the shard receipts into the one
receipt an unsharded check writes.

Locally, `uv run --locked --project tools replay case ID` builds the verifier from the checkout.
Set `SM_CACHE`, `SM_ROM` and `SM_AUDIO_NATIVE` to the sources the case uses.
For a library test, set its [fixture variables](../.github/workflows/ci.yml) to absolute
paths and run `uv run --locked --project tools required-witnesses TEST`. For example, the native
search contract reads `SM_ROM` and `SM_SEARCH_PACKET` and writes its receipt to `SM_SEARCH_RECEIPT`.

## Run the replay faults

Set `SM_ROM`, `SM_CACHE` and `SM_AUDIO_NATIVE` to the same fixtures used by replay.
Run the full manifest locally with one build:

```sh
uv run --locked --project tools check-mutants
```

CI runs the same full manifest. The checker builds one executable in a private directory.
Run `uv run --locked --project tools check-mutants validate` to check manifest coverage and source
anchors without building or running replay. Tree-sitter finds the
Rust function containing each edit. Every alternative starts with the original
function body and applies one manifest replacement. Cargo Gamma selects one
alternative at process startup. The checker counts the fault as killed when the case reports a
difference, a data failure or a failed checkpoint continuation, or when the replay crashes. It
then launches the same binary with the selector cleared and requires the full original receipt.
[Rust grammar](https://github.com/tree-sitter/tree-sitter-rust/tree/v0.24.2)
and [Cargo Gamma runtime](https://github.com/microsoft/ox-tools/blob/cargo-gamma-v0.8.0/crates/cargo-gamma-rt/src/lib.rs).

The manifest supplies the reviewed edits and the case each one must fail.
Gamma's CLI generates its own mutations, so the checker uses its runtime directly.
The played runner can exit successfully when it detects a fault. Its report line
names where it stopped. [Cargo Gamma mutation operators](https://github.com/microsoft/ox-tools/tree/cargo-gamma-v0.8.0/crates/cargo-gamma-engine/src/ops/registry)

Put each edit inside a gameplay or played-runner function. If a `const fn`
calls an instrumented function, list the caller in `runtime_callers`. Only the
private build evaluates those functions at runtime. Cargo checks the resulting
call chain and the worker checks that only the pinned Gamma runtime was added
to the replay build.

Each `RECEIPTS` line names the fault and gives the SHA-256 hashes of its
complete clean, faulty and restored receipts.

## Setting up on Linux

CI runs on Ubuntu 24.04. The following setup also supports local builds and replays.

**Tools.** Install rustup; the first `cargo` in the checkout installs the channel and
components of `rust-toolchain.toml` (`rustup show` does it at once). Install a C toolchain
(`build-essential`), git, [uv](https://docs.astral.sh/uv/) at the standards' `UV` pin,
Python 3.13 as `python3.13` or `python3` (the tools call `python3`), the
[Claude Code CLI](https://docs.claude.com/en/docs/claude-code) as
`claude` on `PATH`, `gh` logged in (`gh auth login`), `openssl` and `xz-utils`, and optionally
`cargo-deny` 0.22.0, which the gate runs where it is installed.

**Folders.** The tools keep their scratch, the lanes' worktrees and the seed under `$TEMP`
(else `$TMPDIR`, else `/tmp`). `/tmp` is often a RAM file system and is cleared on reboot, and the
seed's hard links need the seed and the lanes on one file system, so point `TEMP` at a disk
folder in your shell's profile, with the replay data's record cache beside it:

```sh
export TEMP=$HOME/scratch SM_CACHE=$HOME/scratch/collab/replay-data/cache
mkdir -p "$TEMP/collab"
```

**The ROM** goes at `rom/sm.smc` (above; `sha256sum rom/sm.smc` checks it).

**The replay data** is CI's record cache and native fixtures, one encrypted asset that the
`replay-data` tool [pins](../tools/src/sm_tools/ci/core/replay_data.py) and decrypts with a key
made from the ROM. It lives outside `/tmp`, so it survives a reboot; install it again when the pin
changes:

```sh
rm -rf "$TEMP/collab/replay-data"
uv run --locked --project tools replay-data fetch rom/sm.smc "$TEMP/collab/replay-data"
```

Then point the variables of the replay job's witness environment in `ci.yml` at it: `SM_CACHE`
at its `cache` folder, `SM_AUDIO_NATIVE` and `SM_CONNECTED_SOURCE` at its `native` folder,
`SM_SEARCH_PACKET` at `native/native-search-inputs-v1`, `SM_TOUCH_PACKET` at
`native/native-search-order-v1`, and the rest as that list names them.

**The seed** is the warm build new lanes start from: `uv run --locked --project tools worktree seed`
once, and again after big landings ([lane](../.agents/skills/lane/SKILL.md#resources)).

**The clean room.** `cargo cleanroom export` writes exports under `$CLEANROOM_ROOT` (default
`/cleanroom`, outside the home), and `cargo cleanroom run` starts a session on one in a box: a
docker container from the image `sm-cleanroom-root:1`, with no network, whose file system holds
the export, a temporary `/tmp`, and read-only `/usr`, toolchain, CLI and the `cleanroom` binary
with the files it reads. The session needs an API key: `ANTHROPIC_API_KEY` in the environment,
an `apiKeyHelper` or `ANTHROPIC_*` lines in `~/.claude/settings.json`, or a settings file named
by `CLEANROOM_AUTH`. A relay outside the box (`tools/cleanroom/box_relay.py`) adds it to each
request, so it never enters the box.

### Before any confined session on the box

- **The exports' root is outside the home.** Make `/cleanroom` once, owned by you and private
  (`sudo install -d -o "$USER" -m 700 /cleanroom`). The tool refuses a root under the home:
  host tools run git in the repositories they find there (VS Code's source control runs
  `git status` in every repository under its workspace, with the repository's own
  configuration), and the session writes the export. For the same reason, don't open
  `/cleanroom` in VS Code or run git there yourself; `cargo cleanroom import` reads an export
  through a mirror of its objects alone.
- **Docker runs as you.** `docker info` works without `sudo` (your user is in the `docker`
  group). The box needs nothing else changed on the host.
- **The box is proven.** `cargo test -p tools --no-default-features --bin cleanroom the_box`
  runs the box on a probe export: it reads the export and nothing beside it, the checkout or the
  decode notes. CI's rust job runs the same test. Before each round the root runs
  `cargo cleanroom run --probe DOMAIN` on the real export and CLI.

### First run on the box

1. `uv run --locked --project tools gate` prints `GATE OK`, and
   `uv run --locked --project tools pytest tools/tests/lane` passes.
2. The played runs reproduce `scenarios/replay/played.toml`: `uv run --locked --project tools
   played` plays them all, given `SM_CACHE` and `SM_ROM`, the ROM's absolute path (a lane's worktree
   has no `rom/`). Each `PLAYED` line shows the observed and reviewed counts and outcomes. The test
   passes when every run holds:

   ```sh
   SM_ROM=$PWD/rom/sm.smc uv run --locked --project tools played
   ```

## The README

The README is generated from the tree:

```sh
cargo run -p tools --bin tools -- report --write
```

writes the whole `README.md`. It states the project's thesis and reports progress from the
reviewed runs in [`scenarios/replay/played.toml`](../scenarios/replay/played.toml). The headline
bar covers the milestone in [`docs/architecture/domains.toml`](architecture/domains.toml) and
shows which frames pass from power-on or later seeds and which still need work. Each stretch in
[`scenarios/replay/stretches.toml`](../scenarios/replay/stretches.toml) shows its place on the
same movie scale, drawn `━` on `·`. Its power-on bar shows how much a single run from power-on
covers. A tick after the bar marks completion. The first incomplete stretch is in bold.
The tests hold each listed run to the lock and the stretches to the milestone's frames. When an integrator's work
moves a played run, its own commits change the run there and regenerate the README, and the root
reviews the figures at landing (Jim, 2026-11-25); full CI's replay witnesses hold every run to
the file.
CI's rules job runs `cargo run --locked -q -p tools --bin tools -- report --check`. The local
source gate also checks the generated README through verify's library tests when its inputs change.

## Credits and licence

[lsnes](https://tasvideos.org/EmulatorResources/Lsnes) runs as a separate process; none of its
code is linked or copied. The line compositor follows the PPU rules of
[LakeSnes](https://github.com/angelo-wf/LakeSnes) (MIT). Registry dependencies carry their own licences.
Patrick Johnston's [bank logs](https://patrickjohnston.org/bank/) inform names
and behaviour. Original contributions are MIT ([LICENSE](../LICENSE)). Super Metroid is a trademark
of Nintendo; no ROM or game data is included.
