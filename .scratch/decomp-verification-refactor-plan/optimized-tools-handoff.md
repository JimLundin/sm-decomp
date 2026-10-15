# Ticket 08 optimized controller and sandbox

Status: source implementation and provisional controls complete; merged by root.
Final controller/test reader and isolated Installation await the combined source
freeze. No production tool, policy, claim or authority has been changed.

Owner `/root/shared_preparation`, session
`01a13e7f-ab01-7413-a1c2-288b08a4b719`. Worktree
`/home/jim/sm-decomp-performance-tools`, branch
`performance/optimized-tools-20261015`, exact initial source
`83c0f4fb154e4440bc1130ba0a7068a14221733a`.

Own root Cargo profiles, narrowly justified pipeline startup/identity code and
scoped tests, plus this handoff/candidate preparation documents. Root owns Ceres;
bulk owns capture/generation/provenance; dependency owner owns scanner/planner
and then scheduling. No shared registration or installed policy/registry/claim
mutation belongs to this worker.

Cargo target `/home/jim/sm-decomp-offline-verifier/target` was released
by ticket 04 after copying its reader/evidence and used exclusively for the
completed 08 experiments below. **Root now owns its exclusive Cargo writer**;
this worker must not build into it without an explicit return of ownership.
Builds use locked/offline,
`CARGO_BUILD_JOBS=1`, `SM_PIPELINE_ROOT=/home/jim/sm-decomp/out/pipeline` and
`SM_PIPELINE_WORKERS=4`. Proof root:
`/home/jim/sm-decomp-performance-tools-proof`. Local ignored dependency link
`deps/rustysnes` resolves to the existing pinned checkout; its bytes are unchanged.

## Initial observations

The root manifest defines no custom profiles. Default development and release
dependency cohorts already exist in the assigned target, but their source must
be reestablished before matched measurements. The current controller eagerly
hashes its complete executable before nearly every CLI dispatch. Only
`offline::host_inputs` consumes the cached `binary_digest`; installed execution
and preparation separately authenticate/seal controller and helper bytes.
Confined producer dispatch already avoids this ambient initialization and must
retain that behavior.

Initial measurement scope is bounded utility startup and a complete small
Offline family, separately from tool build time. First reproduce the startup
executable reads on exact source and establish the complete workload baseline.
Then compare deferred unused hashing and matched optimization for the controller,
dependencies and separate sandbox. Keep exact identity for producers/admission;
do not use stripped-plan timeouts as throughput evidence. No full-batch speedup
can be inferred from utility startup alone.

## Completion evidence

### Provisional source checkpoints

- `6a3cfd1`: bounded actual-executable startup probe, independently compilable
  with `rustc --test`; Linux `strace` is an opt-in diagnostic dependency only.
- `564b2c1`: defer the controller digest until Offline host inputs consume it.
  Open the selected image at the original startup boundary and retain that file
  descriptor; cache one digest or failure. Path replacement cannot redirect the
  deferred read. Early confined producer dispatch and installed admission remain
  unchanged.
- `5b6e59d`: matched `verification` profile for both tools and dependencies:
  release inheritance, optimization 2, 16 codegen units, no LTO, line tables,
  packed split debug information and stripped in-image debug information.

### Completed startup controls

The original development controller built from `83c0f4f` is retained at
`baseline-dev/tools/sm-pipeline` below the proof root: 365,502,352 bytes,
SHA-256 `94a22aabf077460560b3c4d2235dca026bba281bc795bd09de105fa0552d58be`.
Build time was 165.51 seconds, peak RSS 2,413,168 KiB. Its matching sandbox is
95,362,136 bytes, SHA-256
`948ef48e324e8ce9adfd415948b0a5038d43aa737f17b25123a3a5568ffb9c5e`.

The deferred development controller from `564b2c1` is retained at
`deferred-dev/tools/sm-pipeline`: 365,559,280 bytes, SHA-256
`f73f39e75968a4ac76da6ca6cdf9fbc0f025261ae683653beb96368ba93ed517`.
Build time was 121.57 seconds, peak RSS 2,815,936 KiB; the sandbox digest is
unchanged. `logs/startup-red-bounded.log` records the expected utility failure
and passing Offline identity control on the original image. Both utility and
Offline read the entire 365,502,352-byte image. `logs/startup-green.log` records
both passing controls: utility reads zero image bytes and Offline reads all
365,559,280 bytes once. Syscall traces and exact requests are retained under
`startup-red-bounded/` and `startup-green/`. These traced times are structural
diagnostics, not performance samples.

Earlier `probe-build.toml`, `startup-red.log` and
`startup-red-compiled.log` diagnostic failures remain retained: missing run
directory, missing test executable, and an invalid initial Offline probe request,
respectively. The final probe sends deliberately invalid TOML so refusal occurs
after identity consumption, before verification. No failed sample is accepted.

### Completed matched profile comparison

The development pair was rebuilt from exact profile-comparison source
`5b6e59d` in 13.32 seconds (`logs/build-dev-profile-source-check.log`) and retained
at `profile-dev/tools/`. The controller SHA-256 is
`968ff77c8c9af108eb6bcaab9dec5970f29f12705aa25d96a98eb982f82516e6`;
the sandbox retains the digest above. Use this pair, not the original startup
baseline, for profile comparisons.

A 900-second bounded, one-Cargo-job `cargo build -p pipeline -p
translation-execution --bin sm-pipeline --bin sm-translation-sandbox --profile
verification --locked --offline` completed from the same source in **755.27
seconds**, peak RSS **2,771,584 KiB**. Its log is
`logs/build-verification-profile.log`. Both executables and actual emitted `.dwp`
companions are independently retained under `profile-verification/tools/`:

| File | Bytes | SHA-256 |
| --- | ---: | --- |
| `sm-pipeline` | 45,589,648 | `52355cfcc022ebc7adab8fc621f48b9a7417fdecd70df20e53440e0ba86c035b` |
| `sm-pipeline.dwp` | 61,194,808 | `9124b5d29c23cd4c31be3d59a033cc481e2fef8056b5f71d874a436b726befb3` |
| `sm-translation-sandbox` | 6,056,632 | `869a02ec707bc98db0dc715768fa583d69c1ae46286b1fa95ddba63f0c2f46de` |
| `sm-translation-sandbox.dwp` | 16,277,872 | `4d679927a3c51ae43179bbe366ee34e6818420fc467787639c66e7d2177a042c` |

Build time is separate from runtime. Dependency build caches differed between the
earlier development baseline and the new optimized profile, so those build times
are not a controlled build-speed comparison. The declared runtime target was a
repeatable improvement of roughly 10% or more for complete small-family
verification, including independent authentication and both cleanup passes.

The actual CLI probe in `crates/pipeline/tests/tool_profiles.rs` creates twelve
actual-C/Rust Fixture pairs: eight roots and four continuations. It preserves all
WRAM bytes and complete case records, checks the exact controller digest in
`pipeline-inputs.toml`, independently authenticates, runs both cleanup passes and
authenticates again. All profiles use fresh Fixture execution, four shared worker
slots and case_jobs 4. Separate private build-cache namespaces share the existing
worker pool; no shared cache was cleared. The common captured source is retained
under `family-profile-comparison/source/` and matches the measured `5b6e59d`
implementation. Host load is recorded before each sample; root was independently
building its integration reader during this period. These are ordinary Offline
controls, not Native, full-catalog, Live or Original-ROM acceptance.

The initial 60-second bounded development pilot passed in 17.09 seconds including
setup (`logs/family-pilot.log`); its complete cold sample was 12.881 seconds. The
declared comparison then completed in 66.04 seconds within its 180-second bound,
including six additional cross-profile reader admissions:

| Complete run | Development seconds | Verification seconds |
| --- | ---: | ---: |
| Cold private cache observation | 11.634 | 1.114 |
| Warm pair 0, development first | 12.639 | 1.117 |
| Warm pair 1, verification first | 11.770 | 1.106 |
| Warm pair 2, development first | 11.426 | 1.103 |

The median complete sample improved **11.770 → 1.106 seconds** (90.6% lower,
10.65x for this workload). All twelve case records and actual complete outputs
matched across profiles. The profile clears the targeted benefit criterion;
these numbers do not establish a whole-batch speedup. Exact results, requests,
per-stage timings, cache/compiler records and cleanup receipts are retained under
`family-profile-comparison/` and `logs/family-profile-comparison.log`.

### Startup, sandbox and installed-image controls

Untraced alternating utility samples (`logs/startup-alternating.log`,
`startup-alternating/measurements.toml`) compare the original/deferred development
images with the same empty cleanup-preview request. Original times were
4.354/3.983/3.909 seconds; deferred times were 0.059/0.003/0.006 seconds. These
support the structural zero-unused-image-read result; they are not a batch gate.

Commit `a643f8b` updates ten existing test initialization sites to the retained
image identity. The actual descriptor replacement unit test passed after a
105-second test-reader build (`logs/deferred-descriptor-unit-fixed.log`). Its
reader is `identity-controls/tools/sm-pipeline-tests`, SHA-256
`2570af4613605d8271a03e55af88dd5412fa88c0a30e4dcdbbbc4ce934d1eec7`.
The original failed unit compile remains retained: obsolete test initializers and
the absent local ignored pinned-C link were corrected. Dependency bytes were not
changed.

The sandbox performance probe observes `/proc/<pid>/exe` device/inode identity
for its own live descendants, confirming the selected helper actually executes.
It keeps the original per-operation five-second limits and performance/refusal
assertions. Each sequential probe owns one shared worker slot. Three alternating
pairs passed, with all 24 selected helper images observed; `sandbox-pilot/` holds
the preceding four-operation development pilot. Median complete operation times:

| Operation / retained files | Development seconds | Verification seconds |
| --- | ---: | ---: |
| File / 0 | 0.0572 | 0.0525 |
| File / 256 | 0.2578 | 0.2060 |
| Pipe / 0 | 0.0785 | 0.0739 |
| Pipe / 256 | 0.2878 | 0.2270 |

The file-heavy cases improved about 20–21%; the smaller differences without
retained files are noisier. `sandbox-profile-comparison/` retains all requests,
selected paths and CPU/timing observations. The unchanged complete
`confinement_runtime` controls passed **16/16 for each helper** within their
original limits, under `sandbox-runtime-controls/`. The 15 ignored entries are
the existing child probes and optional stress test, not claimed coverage.

The disposable installed-image control passed in 28.86 seconds within its
60-second bound. It first validated a complete isolated Installation, then
modified only private copied helper/controller images, verifying the exact
installed-tool identity refusal and that no execution state was created. The
helper was restored and the complete Installation revalidated before testing the
controller. No active installed image was changed. Full results are under
`tool-tamper-controls/`; its exact reader is
`tool-tamper-controls/tools/sm-pipeline-tests`, SHA-256
`c593c5c858cc1fcef5d23c06fa99fa0bac42c21137a8a698f33654a80c6fd418`.

Raw probes reuse existing libraries without rebuilding the CLI. Rust's split
metadata requires both `.rlib` and `.rmeta` extern paths and the repository's
`nightly-2026-09-29` toolchain. Failed exploratory probe-build receipts are kept;
the successful request is `profile-probe-build-complete-extern.toml`. No failed
sample is included in any performance result.

### Provisional cleanup and remaining integration

All owned writers finished. Both scoped passes succeeded using the retained
optimized reader:

```sh
/home/jim/sm-decomp-performance-tools-proof/profile-verification/tools/sm-pipeline cleanup --root /home/jim/sm-decomp-performance-tools-proof --apply
/home/jim/sm-decomp-performance-tools-proof/profile-verification/tools/sm-pipeline cleanup --root /home/jim/sm-decomp-performance-tools-proof --deduplicate --apply
```

`logs/cleanup-provisional-apply.toml` reports zero eligible intermediate bytes;
`logs/cleanup-provisional-deduplicate.toml` reports **25,165,824 bytes** reclaimed
and **324 snapshots** checked. The isolated tamper control's disposable Git
metadata was removed (98,504 logical bytes), preserving its source, copied images
and diagnostic record. The nine retained ordinary family runs then independently
authenticated again with the optimized reader; exact requests/results are in
`retention/`. Private cache seeds, exact source snapshots, selected tools/debug
companions and probe readers remain retained. No shared cache was cleared.

This completes provisional source/profile controls, not installation acceptance.
The owned worktree and exclusive Cargo target remain needed for the final source
build. Root must merge 07/08 and its Ceres/Native test repairs, provide the exact
combined source, and coordinate final target/worktree retirement. The selected
final pair and reviewable candidate under
`/home/jim/sm-decomp-performance-installation-candidate` must use that source.
The active production registry, owner generations, original claims/policies,
integration baseline and held pause blocker remain unchanged.

### Private final candidate preparation checkpoint

Root now builds and supplies the final retained tools from the complete source
freeze. The final assembly was `6509381` at the resumed planning checkpoint;
07 and Native controls were still pending, so that revision was not bound as a
final candidate. The candidate root is
`/home/jim/sm-decomp-performance-installation-candidate`. Its
`transition-review.md`, `transition-inputs.json` and
`isolated-preparation-plan.md` record the concrete deployment seam, exact
preserved authority and final request/inspection requirements.

All 19 supplied production/original-lineage/template inputs matched their pins
and were copied and checked independently. The retained original controller's
read-only production status and host inspection passed. The host remains the
same stopped run/attempt/policy, retaining 102,220 bytes with completed cleanup,
authentication and worker retirement. A read-only status using the identical
production request at a copied private path refused with exactly
`production already activated a different checked request`. Production and copy
hashes remained unchanged afterward. Both candidate-input cleanup passes
reported zero reclaimable bytes; that checkpoint occupied 344,276 logical bytes,
with 95,470,952,448 bytes available. New source-review copies were added later
and require the final scoped cleanup pass after candidate preparation.

Existing machinery does not supply a tools-only Managed successor under the
preserved original policy. Initial production preparation requires current main
HEAD equal the old `a50a13e3...` integration baseline, and a different checked
production request cannot replace the active registry. The supported Original
continuation requires separately reviewed acceptance contracts, resource bounds
and source baselines; it must not be invoked as a tool replacement. Managed
activation is therefore an explicit unfulfilled prerequisite requiring fresh
policy review and the original-acceptance adapter work. No new authority or
ad hoc migration is implemented by this ticket.

The independently preparable Installation uses the unchanged isolated
enemy-jump-instructions template: two routines, all 16 Fixture pairs, original
compiler/Fixture limits and original policy budgets. All 44 historical frozen
source files authenticated at their existing clean
`f69ae88d96f66f0da5e2d2a78354ccb8db7fe8b9` source checkout. Source inventory,
catalog, original provenance verdict, ROM and compiler hashes also matched.
The 44 files plus inventory/catalog/verdict review copies are retained below
the private candidate. **The final assembly is tool source only**; the template
retains its historical gameplay/source revision and paths. The supported
`translation-launch --request ... prepare` and installed `... inspect` commands
invoke no model or gate. `requests/isolated-launch.template.json` remains
explicitly incomplete (`helper: null`) until root supplies final tool References.

The proof-reference audit found four absolute source paths below this worker's
worktree: `crates/pipeline/tests/startup.rs`,
`crates/pipeline/tests/tool_profiles.rs`,
`crates/translation-execution/tests/confinement_performance.rs` and
`crates/translation-execution/tests/confinement_runtime.rs`. Their exact copies
and SHA-256 manifests are in candidate `retention/worktree-source/` and
`retention/worktree-source.expected.sha256`. Root should restore these small
files at their original paths after retiring the integrated Git worktree,
without its Git metadata, Cargo target or other build outputs. The nine family
results authenticate against their already retained `family-pilot/source/` and
`family-profile-comparison/source/` snapshots. Existing readers, debug files,
probe executables, original provenance graphs and private evidence dependencies
remain required. Final authentication after retirement is still pending.
