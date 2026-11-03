# The verification pipeline without Python

Jim (2026-11-03): "Preferably, I would want to avoid using Python if at all
possible. Our testing infrastructure should just be focused around the
verification pipeline, and we shouldn't really need Python to run that."

The verifier is already Rust: `sm_verify` (sources in `sm/src/verify*.rs` and
`sm/src/verification/`). This document is about the pipeline around it: the
Python that plans a Verify run, restores its private data, drives the
verifier, compares locks and reports. It lists every script, gives the Rust
that replaces it, and orders the port so each step deletes Python and is
proven equivalent before its Python goes.

## Destination

- **One Rust command-line tool, `sm_pipeline`**, a binary of `sm_tools`
  (`crates/sm_tools/src/bin/sm_pipeline/`). It uses only the standard library,
  so a workflow builds it in seconds with
  `rustc --edition=2024 -O crates/sm_tools/src/bin/sm_pipeline/main.rs`,
  without building `sm_game`, and `cargo test -p sm_tools` runs its tests.
  Its subcommands own the pipeline's decisions: what runs, what each job
  restores, what a replay may print, whether a lock matches.
- **Plain shell in the workflows** for what is only plumbing: `gh api`,
  `curl`, `openssl enc`, `tar`, `sha256sum`, `llvm-profdata`.
- **The verifier stays the verifier.** Per-row driving that is only argv
  building (today `verify_dll.py --corpus`) moves into `sm_pipeline replay`,
  which runs `sm_verify` directly.
- The scenario model (`tools/scenario.py` `rows`, `read_lock`, `lock --check`)
  is ported once, in `sm_pipeline`'s `scenario` module, and every subcommand
  reads it.

No new crates.io dependency is needed for the CI side: TOML is read by a small
subset reader (`toml.rs`), JSON is written by `json.rs`, SHA-256 needs a
~100-line implementation, and gzip/tar/AES stay with `tar` and `openssl` in
shell.

## Inventory

Lines are as of `85f0078c6`. "Caller" is who runs it today.

### Planning and gating (verify-plan job)

| Script | Lines | What it does | Inputs, outputs | Caller |
| --- | --- | --- | --- | --- |
| `.github/scripts/plan_verify.py` | 205 | Plans the whole Verify run for a scope (gate, full, rows, coverage) as one job matrix plus the gate's floor jobs | env SCOPE, SELECTED, ORACLE_SAMPLE, COVER_ROUTES; scenarios, lock, row_seconds.toml, movies/manifest.toml; writes GITHUB_OUTPUT `matrix`, `first`, `first_count`, `movie_file`, `movie_sha256` and the step summary | ci.yml verify-plan |
| `.github/scripts/plan_corpus.py` | 102 | Shards corpus rows by the seed bundle or movie route they restore; adds the rotating faithful-oracle sample | env SELECTED, ORACLE_SAMPLE; writes a shard matrix | plan_verify.py and lock_gate.py, as subprocesses |
| `.github/scripts/pack.py` | 152 | Packs rows into jobs by measured seconds (budget, floor rows, partition into worker processes) | row_seconds.toml | plan_verify.py; replay_job.py (`partition`) |
| `.github/scripts/lock_gate.py` | 151 | `jobs`: the full scope's engine-only lock jobs; `oracle_sample`/`oracle_slot`; `record-oracle`: re-records sampled locks privately and compares them | scenarios, lock; SEGMENTS | plan_verify.py; replay_job.py (`record-oracle`) |
| `.github/scripts/full_route.py` | 305 | `plan`/`jobs`: route rooms packed into jobs; `restore`: decrypt and check a route bundle; `run`; local `inputs`/`pack` author the route scenarios and bundles | FULL_ROUTE; route CSVs, lock; bundle file | plan_verify.py, lock_gate.py (plan, jobs); replay_job.py (restore); people (inputs, pack) |
| `.github/scripts/already_passed.py` | 11 | Skips a scheduled run whose commit already passed | gh api; writes `skip` | ci.yml verify-plan |
| `.github/scripts/replay_relevant.py` | 85 | Skips a pull request's replays when every changed file is in a list of irrelevant patterns | event JSON, gh api, git diff; writes `skip` | ci.yml verify-plan |

### Restoring private data (verify-replay jobs)

| Script | Lines | What it does | Inputs, outputs | Caller |
| --- | --- | --- | --- | --- |
| `.github/scripts/prepare_rom.py` | 11 | Decrypts the ROM from the ci-data checkout, checks its SHA-1, deletes the checkout | SM_ROM_KEY; `rom/sm.smc` | ci.yml (verify-replay, verify-replay-first) |
| `.github/scripts/prepare_corpus_inputs.py` | 44 | Per shard: downloads a seed bundle and restores it (via ci_seed_data.py), or downloads/caches the any% movie, checks its SHA-256 and converts it (via movie_inputs.py) | KIND, SEED_ASSET, SEED_SHA256, SEGMENTS, ROUTE, INPUTS | replay_job.py |
| `tools/ci_seed_data.py` | 336 | `restore`: checks a bundle's pinned SHA-256, decrypts it with the ROM-derived key, extracts only the members the selected rows need, checks their pins; `pack`: the author side | bundle, ROM, rows; files under movies/ and target/ | prepare_corpus_inputs.py; scenario.py cut; people |
| `tools/movie_inputs.py` | 1781 | Parses LSMV/BK2 movies to controller words; builds the corpus route inputs (`extend_*` route builders); lag-aligned seeded slices; work-event maps | movie, ROM, capture logs; `.inputs` files | prepare_corpus_inputs.py; full_route.py inputs; scenario.py rows (movie maps) |

### Replaying (verify-replay jobs)

| Script | Lines | What it does | Inputs, outputs | Caller |
| --- | --- | --- | --- | --- |
| `.github/scripts/replay_job.py` | 194 | Runs one matrix job: restores, splits rows into worker processes, runs `verify_dll.py --corpus` per process, filters console lines to what may be public, writes the summary; rom-import and record modes | env JOB (matrix entry), ORACLE, AUDIO_TRACE | ci.yml |
| `tools/verify_dll.py` | 1426 | Builds `sm_verify` (`--build-only`: source hash, inventory, rustc chain, native archives); runs it (`--unit-tests`, single replays, `--corpus` driver with checkpoint prefixes, lock record/check, localisation, audio-trace checks) | many flags | ci.yml, .github/actions/verifier, replay_job.py, lock_gate.py, full_route.py, scenario.py, people |
| `tools/scenario.py` | 526 | The scenario model: `rows`, `read_lock`, `write_lock`, `lock --check`; local `lock` (rehash, record) and `cut` | scenarios/, lock | every script above; people |
| `.github/scripts/check_actor_exports.py` | 20 | Checks the built verifier's source stamp, library and three actor capabilities | sm_verify | ci.yml verify-build |

### Reporting

| Script | Lines | What it does | Inputs, outputs | Caller |
| --- | --- | --- | --- | --- |
| `.github/scripts/test_summary.py` | 89 | Summarises a `cargo test` log per test binary and fails when an expected one did not run | test.log, package dirs | ci.yml Quality |
| `.github/scripts/coverage_tests.py` | 70 | Builds and runs the target packages' tests instrumented | RUSTFLAGS; target/coverage-unit | ci.yml verify-build (coverage) |
| `.github/scripts/coverage_report.py` | 45 | Merges profiles, runs `tools/coverage_summary.py` | profiles; coverage.md | ci.yml verify-coverage |
| `tools/coverage_summary.py` | 251 | Engine coverage tables from llvm-cov output | llvm-cov JSON | coverage_report.py |
| ci.yml "Verify result" (inline) | 12 | Requires every planned Verify job to pass | `needs` JSON | ci.yml |
| `.github/scripts/row_seconds.sh` | shell | Regenerates row_seconds.toml from CI logs | gh run logs | people |

### Outside the verification pipeline

Quality runs these too, but they are code generators, measurements or native
installers rather than verification: `tools/ram_ratchet.py`,
`tools/cycle_report.py`, `tools/oracle_patch_policy.py` (with
`tools/snesrev_patches.py`), `tools/authored_inputs.py`,
`tools/scenario_seeds.py`, the two `mutations.py --check-anchors`,
`tools/install_native_*.py`, `tools/install_lakesnes.py` and the mypy step.
They are a separate track; the last step below lists them.

## Subcommands

| Subcommand | Replaces | Shell around it |
| --- | --- | --- |
| `plan` (done) | plan_verify.py, plan_corpus.py, pack.py's planning, lock_gate.py `jobs`/`oracle_sample`, full_route.py `plan`/`jobs`, scenario.py `rows`/`lock --check` | none |
| `lock-check` | `scenario.py lock --check` | none |
| `relevant` | replay_relevant.py's classification | `gh api` for passed heads, `git diff --name-only`, piped in |
| `test-summary` | test_summary.py | none |
| `restore` | prepare_corpus_inputs.py (seeds), ci_seed_data.py `restore`, full_route.py `restore` | `curl`, `sha256sum -c`, `openssl enc -d`, `tar -tz`/`-xz`; `restore` prints the allowed members, checks the listing and the restored pins |
| `replay` | replay_job.py, verify_dll.py's corpus driver and lock compare, pack.py `partition`, lock_gate.py `record-oracle` | none; it runs `sm_verify` |
| `build-verifier` | verify_dll.py `--build-only`, `--unit-tests`, check_actor_exports.py | rustc and native archives, as now |
| `movie-inputs` | movie_inputs.py, prepare_corpus_inputs.py (route) | `curl` for the movie |
| `coverage` | coverage_tests.py, coverage_report.py, coverage_summary.py | `llvm-profdata`, `llvm-cov` |
| none | already_passed.py, prepare_rom.py, check_actor_exports.py, the inline result check | 2-5 lines each: `gh api --jq .total_count`, `openssl enc -d` with `sha1sum -c`, `sm_verify --host-capabilities \| grep`, `needs.*.result` expressions |

## Order of the port

Ordered by independence first, then by Python removed. Each step names how
it is proven equivalent while both exist; the Python is deleted only after
its check has passed in CI, and the check is deleted with it.

1. **`plan`: done in this slice.** About 2,600 lines of Rust as rustfmt
   lays them out, and 500 of tests, including the TOML reader, JSON writer
   and scenario model the later steps reuse.
   *Proof:* `cargo test -p sm_tools` runs `equivalence.rs`, which plans every
   scope with both planners over the committed scenarios and requires
   byte-identical GITHUB_OUTPUT and step summaries: gate; full with and
   without the oracle sample at three pinned slots (`ORACLE_SLOT`); rows for
   four selections (every fifth row, a mixed sample with long-prefix,
   checkpoint and any% rows, one corpus row, one room) and two that must fail
   in both; coverage with and without routes. *Switch:* one line in ci.yml
   verify-plan (below). *Deletes after the switch:* plan_verify.py and
   plan_corpus.py (307 lines), lock_gate.py `jobs`/`matrix`/`oracle_sample`,
   full_route.py `plan`/`jobs`/`job_list`. pack.py stays until step 6 (replay
   uses `partition`).
2. **Shell for the four trivial scripts** (already_passed.py, prepare_rom.py,
   check_actor_exports.py, the inline result check; about 55 lines). Fully
   independent. *Proof:* the same run's outputs (`skip`, the decrypted ROM's
   SHA-1, the capability list, the result) on one gate run and one scheduled
   run.
3. **`relevant` and `test-summary`** (174 lines of Python, about 200 of
   Rust). Independent. *Proof:* `relevant` reproduces replay_relevant.py's
   verdict over the changed-file lists of the last 50 merged pull requests
   (a unit test over the pattern table, plus a one-off comparison script in
   CI); `test-summary` reproduces the Markdown table byte for byte from the
   `test.log` of recent Quality runs, and fails on the same missing binaries.
4. **`lock-check`** (exposes what `plan` already ports; removes the Quality
   need for scenario.py `lock --check`). *Proof:* the equivalence test
   already requires the same locked set; add negative cases (a stale entry,
   a missing pin, an unlocked lockable row) that both must reject.
5. **`restore`** (prepare_corpus_inputs.py's seed half, ci_seed_data.py
   `restore`, full_route.py `restore`: about 250 lines of Python, about 400
   of Rust with SHA-256). Needs the scenario model only. *Proof:* a CI job
   restores every pinned bundle with both into two directories and requires
   identical `sha256sum` listings of the restored trees, and that both reject
   a bundle with a wrong digest, an unexpected member and a member outside
   the allowed names.
6. **`replay`** (replay_job.py, verify_dll.py's `--corpus` driver,
   `compare_lock`, `localize`, lock_gate.py `record-oracle`, pack.py: about
   900 lines of Python, about 1,200 of Rust). The largest step and the one
   that removes Python from every replay job. Depends on 5. *Proof:* first a
   `--dry-run` in both that prints each `sm_verify` argv and environment per
   row, compared over the whole corpus and both routes; then one gate and one
   full run with the Rust driver whose per-row PASS/FAIL lines, console and
   summary lines equal the Python run's on the same commit.
7. **`build-verifier`** (verify_dll.py's build half and `--unit-tests`:
   about 400 lines). After 6, verify_dll.py has no CI caller left but this.
   *Proof:* identical source inventory and source stamp, the same rustc
   argv, and the built verifier's `--host-capabilities` and units pass.
8. **`movie-inputs`** (movie_inputs.py and prepare_corpus_inputs.py's route
   half: about 1,800 lines of Python). Independent of 5-7 but the most code
   per Python line removed. *Proof:* every `corpus-*.inputs` route file and
   every route room's inputs match their pinned `input_sha256` in the lock,
   which already records the Python output's digest.
9. **`coverage`** (coverage_tests.py, coverage_report.py,
   coverage_summary.py: 366 lines). *Proof:* identical coverage.md from one
   coverage run's profiles.
10. **The author side**: scenario.py `lock`/`cut`, full_route.py
    `inputs`/`pack`, ci_seed_data.py `pack`. These run on a person's machine,
    not in CI. *Proof:* regenerating scenarios/lock.toml and the route
    scenario files yields byte-identical files; a repacked bundle restores
    to identical files.
11. **Outside the pipeline**: the Quality generators and measurements listed
    above. Each is either a generator whose output is committed (proof:
    identical regenerated files) or a measurement (proof: identical report).

After step 9 the CI side has no Python; after 10, none of the verification
tooling does; the mypy step goes with the last script.

## Switching the planner (step 1)

The other lane owns `.github/workflows`. The change is the `plan` step of
ci.yml's verify-plan job, which keeps its env (SCOPE, SELECTED,
ORACLE_SAMPLE, COVER_ROUTES and GITHUB_OUTPUT) as they are:

```yaml
        run: rustc --edition=2024 -O -o "$RUNNER_TEMP/sm_pipeline" crates/sm_tools/src/bin/sm_pipeline/main.rs && "$RUNNER_TEMP/sm_pipeline" plan
```

`rustc` picks up rust-toolchain.toml through rustup on the runner, as the
detached job's rustc steps do. Once that change has run green on a gate and
a scheduled full run, delete the Python listed under step 1 and the
equivalence test with it.

## How the planner matches Python exactly

Notes for anyone changing either planner while both exist:

- **Float sums.** CPython 3.12 and later sums floats with Neumaier
  compensation; `pack::py_sum` does the same, and the packing comparisons
  (`sum(...) <= cap`) depend on it. Running loads (`+=`) stay plain.
- **Rounding.** Python's `round` rounds ties to even (`round_ties_even`).
- **First minimum.** Python's `min` takes the first of equal loads
  (`route::lightest`).
- **Dict order.** Matrix objects keep Python's insertion order, including
  where a key is updated in place (`Object::set`).
- **Inherited environment.** `lock_gate.jobs` passes its environment to
  plan_corpus.py, so in the full scope the oracle sample's shards also join
  the lock jobs' corpus bins. The Rust planner does the same; it is a quirk to
  revisit once Python is gone, not a rule.
- **Movie maps.** scenario.py checks a movie-map pin's work events when its
  manifest is present. No scenario has one; the Rust planner refuses such a
  pin with a clear error until step 8 ports the check.
