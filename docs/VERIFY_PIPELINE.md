# The verification pipeline without Python

Jim (2026-11-03): "Preferably, I would want to avoid using Python if at all
possible. Our testing infrastructure should just be focused around the
verification pipeline, and we shouldn't really need Python to run that."

The verifier is already Rust: `sm_verify` (the `sm` crate: `sm/src/verify*.rs`,
`sm/src/verification/` and the `crates/sm_host/src` sources it compiles). This
document is about the pipeline around it: the
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

The Python the pipeline still runs; the scripts the finished steps replaced are
gone. "Caller" is who runs it.

### Planning and gating

| Script | What it does | Inputs, outputs | Caller |
| --- | --- | --- | --- |
| `.github/scripts/pack.py` | `partition`: splits a job's rows into worker processes by measured seconds | row_seconds.toml | replay_job.py |
| `.github/scripts/lock_gate.py` | `record-oracle`: re-records sampled locks privately and compares them | scenarios, lock; SEGMENTS | replay_job.py |
| `.github/scripts/full_route.py` | `restore`: decrypt and check a route bundle; `jobs`/`run`: a local route job; local `inputs`/`pack` author the route scenarios and bundles | FULL_ROUTE; route CSVs, lock; bundle file | replay_job.py (restore); bench.yml; people |

### Restoring private data (verify-replay jobs)

| Script | What it does | Inputs, outputs | Caller |
| --- | --- | --- | --- |
| `.github/scripts/prepare_corpus_inputs.py` | Per shard: downloads a seed bundle and restores it (via ci_seed_data.py), or downloads/caches the any% movie, checks its SHA-256 and converts it (via movie_inputs.py) | KIND, SEED_ASSET, SEED_SHA256, SEGMENTS, ROUTE, INPUTS | replay_job.py |
| `tools/ci_seed_data.py` | `restore`: checks a bundle's pinned SHA-256, decrypts it with the ROM-derived key, extracts only the members the selected rows need, checks their pins; `pack`: the author side | bundle, ROM, rows; files under movies/ and target/ | prepare_corpus_inputs.py; bench.yml; scenario.py cut; people |
| `tools/movie_inputs.py` | Parses LSMV/BK2 movies to controller words; builds the corpus route inputs (`extend_*` route builders); lag-aligned seeded slices; work-event maps | movie, ROM, capture logs; `.inputs` files | prepare_corpus_inputs.py; full_route.py inputs; scenario.py rows (movie maps) |

### Replaying (verify-replay jobs)

| Script | What it does | Inputs, outputs | Caller |
| --- | --- | --- | --- |
| `.github/scripts/replay_job.py` | Runs one matrix job: restores, splits rows into worker processes, runs `verify_dll.py --corpus` per process, filters console lines to what may be public, writes the summary; rom-import and record modes | env JOB (matrix entry), ORACLE, AUDIO_TRACE | verify.yml |
| `tools/verify_dll.py` | Builds `sm_verify` (`--build-only`: source hash, inventory, rustc chain, native archives); runs it (`--unit-tests`, single replays, `--corpus` driver with checkpoint prefixes, lock record/check, localisation, audio-trace checks) | many flags | verify.yml, bench.yml, gate1.yml, .github/actions/verifier, replay_job.py, lock_gate.py, full_route.py, scenario.py, people |
| `tools/scenario.py` | The scenario model: `rows`, `read_lock`, `write_lock`, `lock --check`; local `lock` (rehash, record) and `cut` | scenarios/, lock | every script above; people |

### Reporting

| Script | What it does | Inputs, outputs | Caller |
| --- | --- | --- | --- |
| `.github/scripts/coverage_tests.py` | Builds and runs the target packages' tests instrumented | RUSTFLAGS; target/coverage-unit | verify.yml (coverage) |
| `.github/scripts/coverage_report.py` | Merges profiles, runs `tools/coverage_summary.py` | profiles; coverage.md | verify.yml (coverage) |
| `tools/coverage_summary.py` | Engine coverage tables from llvm-cov output | llvm-cov JSON | coverage_report.py |
| `.github/scripts/row_seconds.sh` (shell) | Regenerates row_seconds.toml from CI logs | gh run logs | people |

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
| `lock-check` (done) | `scenario.py lock --check` | none |
| `relevant` (done) | replay_relevant.py's classification | `gh api` for passed heads, `git diff --name-only`, piped in |
| `test-summary` (done) | test_summary.py | none |
| `gate1` (done) | gate1.py, the Gate 1 row runner (`.github/workflows/gate1.yml`) | verify_dll.py and ci_seed_data.py `restore` until `replay` and `restore` port them; `curl` for a bundle, `sm_replay`, `xvfb-run sm_slice` |
| `volume` (measurement, not verification) | none: the [code volume](#code-volume) | none; Quality prints it and `tools/ram_ratchet.py` reads it |
| `restore` | prepare_corpus_inputs.py (seeds), ci_seed_data.py `restore`, full_route.py `restore` | `curl`, `sha256sum -c`, `openssl enc -d`, `tar -tz`/`-xz`; `restore` prints the allowed members, checks the listing and the restored pins |
| `replay` | replay_job.py, verify_dll.py's corpus driver and lock compare, pack.py `partition`, lock_gate.py `record-oracle` | none; it runs `sm_verify` |
| `build-verifier` | verify_dll.py `--build-only`, `--unit-tests` | rustc and native archives, as now |
| `movie-inputs` | movie_inputs.py, prepare_corpus_inputs.py (route) | `curl` for the movie |
| `coverage` | coverage_tests.py, coverage_report.py, coverage_summary.py | `llvm-profdata`, `llvm-cov` |
| none (done) | already_passed.py, prepare_rom.py, check_actor_exports.py, the inline result check | 2-5 lines each: `gh api --jq .total_count`, `openssl enc -d` with `sha1sum -c`, `sm_verify --host-capabilities \| grep`, `needs.*.result` expressions |

## Order of the port

Ordered by independence first, then by Python removed. Each step names how
it is proven equivalent while both exist; the Python is deleted only after
its check has passed in CI, and the check is deleted with it.

1. **`plan`: done.** Verify's verify-plan job runs `sm_pipeline plan`.
   `plan_golden.rs` holds golden plans for every scope over a frozen root
   (`sm_pipeline/fixture`, rows cut from the committed scenarios with their lock
   entries, seconds and capture indexes). full_route.py `jobs` stays for the
   local `run JOB`, and pack.py's `partition` until step 6 (replay uses it).
2. **Shell for the trivial scripts: done.** verify.yml's verify-plan `passed`
   step (`gh api --jq .total_count`), its Decrypt the ROM step (`openssl enc -d`,
   then `sha1sum --check`; bench.yml has the same lines), the capability step
   (`sm_verify --host-capabilities` and `grep -x`) and Verify result
   (`needs.*.result`, every failing job named).
3. **`relevant` and `test-summary`: done.** `relevant.rs` holds the pattern
   table and a `fnmatchcase` matcher, and its fixture holds the verdicts the
   Python gave; verify-plan's `relevant` step is shell around it. Quality runs
   `sm_pipeline test-summary`, and its fixture holds the Python's output on
   real and derived `test.log` files.
4. **`lock-check`: done.** `sm_pipeline lock-check [--root DIR]` prints
   scenario.py's line or exits 1 naming every problem; `lock_golden.rs` holds
   scenario.py's verdict over the plan fixture and broken copies of it. No CI
   step runs it (verify-plan's planner already checks the lock); the skills
   keep the Python command for local use.
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

## How the planner matches Python exactly

Why the planner does what it does where Python decided the shape (the
golden plans in `plan_golden.rs` depend on each):

- **Float sums.** CPython 3.12 and later sums floats with Neumaier
  compensation; `pack::py_sum` does the same, and the packing comparisons
  (`sum(...) <= cap`) depend on it. Running loads (`+=`) stay plain.
- **Rounding.** Python's `round` rounds ties to even (`round_ties_even`).
- **First minimum.** Python's `min` takes the first of equal loads
  (`route::lightest`).
- **Dict order.** Matrix objects keep Python's insertion order, including
  where a key is updated in place (`Object::set`).
- **Inherited environment.** The Python lock-job planner passed its environment to
  the corpus planner, so in the full scope the oracle sample's shards also join
  the lock jobs' corpus bins. The Rust planner does the same; it is a quirk to
  revisit once Python is gone, not a rule.
- **Movie maps.** scenario.py checks a movie-map pin's work events when its
  manifest is present. No scenario has one; the Rust planner refuses such a
  pin with a clear error until step 8 ports the check.

## Code volume

`sm_pipeline volume` (crates/sm_tools/src/bin/sm_pipeline/volume.rs) measures
code size in lexer tokens, so joining or splitting lines moves nothing but the
line count. It serves REVIEW's "After the decoupling: as few lines as possible"
beside `production_lines`, and like it is advisory for now.

- **Logic tokens.** Tokens outside comments, doc comments, `#[cfg(test)]` items
  and pure data items. Compound operators (`::`, `->`, `=>`, `..=`, `<<=`) are
  one token; a literal of any kind is one token.
- **Data tokens and entries.** The tokens of pure data items: a `const` or
  `static` with a value made only of literals, arrays, tuples, struct and
  tuple-struct literals and paths (with unary `-` and `&`). A call (a lowercase
  name before `(`), a macro, a closure, an operator, a cast, `if`, `match` or
  a block makes the item logic. Entries are the top-level elements of the
  item's outermost array or slice; a repeat array or a non-array value is one.
  Constants generated inside a macro invocation are logic.
- **Comment lines.** Lines holding any comment or doc comment; reported only.
- **Structure counts.** Lexical: `trait` followed by a name; `impl` at the start
  of an item (not `impl Trait` types); blanket impls, an `impl<...>` whose self
  type after `for` is one of its own type parameters (behind `&`, `&mut` or a
  lifetime too); `fn` followed by a name; match arms, `=>` outside
  `macro_rules!` bodies (a macro invocation's own `=>` syntax counts); and
  `macro_rules!` definitions.

The lexer handles line, nested block and doc comments, strings, byte and C
strings, raw strings with any number of `#`s, raw identifiers, character and
byte literals versus lifetimes and loop labels, and numbers with exponents and
suffixes. Its golden fixtures (`fixture/volume`) cover each case, and
`split.rs` and `joined.rs` hold the same code on 13 lines and on 2.

**Scope.** `production_lines`' files: `.rs` under `crates/*/src` and `sm/src`.
The engine is crates/sm_data, sm_game, sm_runtime and sm_app outside
`verification` directories, reported per package and per top-level sm_game
module (a file or directory under `src`). sm/src and crates/sm_host/src outside `verification` (the
oracle and the legacy game) and verification (`verification` directories and
crates/sm_tools/src outside its `fixture` test inputs) are separate rows, so
the engine measures do not move with them. `#[cfg(test)]` items are excluded
everywhere; `#[cfg(sm_verify_dll)]` items in engine files are counted, as
`production_lines` counts them.

**Running it.** `sm_pipeline volume [--root DIR]` prints the table and then
the ratchet's measures as `name: value` lines; `--baseline tools/ram_ratchet.py`
compares those with the file's `"name": value` entries and exits 1 when one
rose; `--file PATH...` measures single files. Quality builds sm_pipeline,
prints the report into the step summary, and runs `tools/ram_ratchet.py` with
`SM_PIPELINE` set, which adds `engine_logic_tokens`, `engine_data_tokens`,
`engine_traits` and `engine_blanket_impls` to its advisory measures. Locally
the ratchet builds the binary with rustc and skips these measures when it
cannot run it.
