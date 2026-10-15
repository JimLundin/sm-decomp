# Native verification after Offline family extraction

Status: corrected source frozen; final Native verification in progress
Owner: integration owner, ticket [05](issues/05-verification-and-cleanup.md)
Audited: 2026-10-15, source baseline `77022896e6e92abe69c27c1c0f9786b838d63ba9`

This is the Native portion of the [approved implementation plan](implementation.md).
It records the existing gates and the harness migration they need. The audit
read source and local prerequisite metadata/hashes only. It did not build, run
producers, change installed policy, or create verification artifacts.

The harness migration and PLM wrapper/accounting correction are frozen at
`4a736587af557de9e44b3126ff08f6d6d11beee8`. The requirements below remain the
gate contract; [the checkpoint](checkpoint.md) records current execution and
evidence. Current proof parent:
`/home/jim/sm-decomp-offline-architecture-proofs/plm-fix-20261015`.
The earlier `15d5a7e-20261015` source, readers and partial recipe results remain
historical evidence; they are not final proof of the corrected source.

Use frozen integrated source for the actual gates. All new harness code remains
Rust. Preserve the current indexed evidence formats, complete declared domains,
bounded accounting, producer ownership, retry and authentication semantics.
Scope approval stays in the pipeline application. Keep its broad source
inventory and the separate installed-controller executable fence. All three
new crate paths retain conservative affected-family selection. A CLI-only
independence experiment and catalog-wide real execution are outside this gate.

## Prerequisites checked

These observations establish availability, not successful confinement or current
Native acceptance. Recheck identities when freezing the integrated source.

| Input | Observed location and state |
| --- | --- |
| Pinned actual-C tree | `/home/jim/sm-decomp/deps/snesrev-sm`; `LICENSE.txt` exists, 2,597 bytes |
| Pinned archive | `/home/jim/sm-decomp/deps/downloads/snesrev-sm.tar.gz`, 1,031,954 bytes; revision `578f90b3cc49557bb70060ad033bb90b8cf8ac50` in `tools/dependencies.toml` |
| Private ROM | `/home/jim/sm-decomp/rom/sm.sfc`, 3,145,728 bytes; needed by complete catalog/generated PLM preparation |
| Source inventory | `tools/port/source-inventory.toml`, 3,256,608 bytes |
| Requirements | `.scratch/translation-scope-execution/issues/10-original-acceptance-adapters.md`, 6,947 bytes |
| Direct-animation plan | `.scratch/decomp-coverage/renderer/direct-animation.toml`, 441 bytes |
| Rust compiler | `/home/jim/.rustup/toolchains/nightly-2026-09-29-x86_64-unknown-linux-gnu/bin/rustc`, executable |
| C compiler | `/usr/bin/x86_64-linux-gnu-gcc-9`, executable |
| Namespace helper dependency | `/usr/bin/bwrap`, executable |
| Shared process capacity | `/home/jim/sm-decomp/out/pipeline/workers/capacity.toml` records `workers = 4` |

Audited SHA-256 values:

```text
deps/downloads/snesrev-sm.tar.gz
69c78831351112887575f852dadf0281dd5c23cef890b07a268ae6cf60aa9193

rom/sm.sfc
12b77c4bc9c1832cee8881244659065ee1d84c70c3d29e6eaf92e6798cc2ca72
```

A separately retained test helper exists at
`/home/jim/sm-decomp-performance-20261014/confinement/tools/sm-translation-sandbox-v7-opt1`,
SHA-256 `fb403f0445d10cb3ce75fd2f928e6bedea27c4b9a1f5de8d445527252dcf2351`.
Prefer a freshly built, separately retained current helper for the integrated
gate; record its exact source and hash. Never silently take the recipe test's
default production helper at
`/home/jim/sm-decomp/out/translation-production/sm-translation-sandbox` or replace
that installed executable. Its audited hash is
`00a9853ea1668145c99972ad3c62c01ffd25b6ec5ea109b6792eb7d5b629132e`.

The captured Ridley declaration additionally retains absolute dependencies in
the existing Original evidence tree. Its exact event input is
`/home/jim/sm-decomp/out/port/ridley-subsystem/recovery-inventory/1791858734336387661-2/ridley-subsystem-events.jsonl`
(71,595,494 bytes). The source requires SHA-256
`f1f286cae8090e5f23f8e1e9b0a20fcc4de41d91766f349c9a0be7518eec1f35`;
this audit checked that file's extent but did not hash it. The existing bounded
reader validates that exact exception during preparation. Preserve every
declared capture dependency at its authenticated path; the event alone is not
the complete captured input closure.

At the prerequisite check the shared filesystem had 131,832,672,256 free bytes.
Recheck before execution and record final free space. Output reservations are
logical bounds; deduplication does not reduce the reservation charged to a gate.
No confinement capability probe ran. The host exposes
`user.max_user_namespaces = 47816` and `ptrace_scope = 1`; absence of the
distribution-specific `unprivileged_userns_clone` sysctl is not a failed probe.

## Harness migration (implemented)

Proof entry points retain their exact names under `translation_native::tests`.
They now live in `crates/pipeline/src/translation_native_tests.rs`; the source
anchors below describe their locations at the original audit baseline:

| Exact current test name | Source anchor | Purpose |
| --- | --- | --- |
| `translation_native::tests::complete_current_catalog_has_a_frozen_native_recipe` | line 2189 | Freeze every current catalog declaration and the direct-animation auxiliary |
| `translation_native::tests::real_reviewed_recipes_replay_and_authenticate` | line 1981 | Complete selected real recipes, unchanged retry, cleanup and authentication |
| `translation_native::tests::full_real_family_retains_authenticates_and_reuses_every_producer` | line 2603 | Complete actual-C family plus confinement, attribution and reservation controls |

Their package/module names can change during extraction. Discover and record
the exact migrated test targets with `cargo test ... -- --list` or the retained
test executable's `--list`; require exactly one selected test per proof. A
zero-test exit or passing default suite does not satisfy an ignored gate.

Required changes:

- Replace the recipe proof's flat copies of `crates/pipeline/src` and
  `crates/port/src` with the complete owned source closure across the new
  crates, including nested modules and `.rs.in`/`.c.in` templates. Keep the
  application host source superset, including `main.rs`, batch and cleanup.
- Change `compiled_sources()` and the full proof's destination mapping from
  pipeline leaf names to workspace-relative paths. Migrate the Native
  `HARNESS`, recipe `harness()` paths and compiled-source checks in `inputs.rs`
  together. Preserve `pipeline-inputs.toml` producer identity and stale-source
  rejection; source relocation must not weaken those checks.
- Use the new source owner for inventory generation. Keep Translation scope
  authorization and application cleanup in pipeline test orchestration; do
  not create verifier-to-pipeline dependencies to keep old tests compiling.
- Preserve indexed operation storage tests inside its owner. Exercise public
  preparation, execution and authentication from application tests without
  exposing operation traversal through the opaque prepared handle.
- Preserve the catalog registration alias `instructions` / `enemy-instructions`,
  generated PLM validation against the full generator, direct-animation
  handling and the captured Ridley declaration/streaming exception.
- Extend the full real proof to clean every completed output and authenticate
  every final positive/rejected Native receipt. Add a retained read-only test
  mode using the new prepared authentication interface, described below.

The current full real test has no source override: `current_checkout()` derives
the workspace root from compile-time `CARGO_MANIFEST_DIR`. Build it from the
frozen source or add an explicit test source parameter during migration. Its
read-only authentication mode must use retained paths rather than require the
implementer worktree to still exist.

## Source and executable retention

Use a fresh persistent parent such as
`/home/jim/sm-decomp-offline-architecture-proofs/<frozen-source-id>/`, outside
implementer worktrees. Record the exact integrated commit, any source overlay,
source hashes, toolchain paths, commands, exit status and test counts there.
Keep build output separate from retained `tools/`, source snapshots and proofs.

Create a fresh complete catalog source snapshot before the catalog probe. It
must include the integrated tracked source/declarations plus private ROM and
all declared external dependencies. The current inventory, requirements and
direct-animation plan are tracked, but still need inclusion at the exact paths
the harness reads. A Git-only copy omits the ROM and does not itself preserve
the absolute captured dependencies.

The catalog probe calls `prepare_generated_plm` and writes
`.scratch/translation-native/plm-instructions` in its supplied source checkout.
Therefore set `SM_NATIVE_CATALOG_SOURCE` to that fresh retained snapshot,
never the shared checkout. Record the final generated source closure. The
recipe probe also generates PLM inputs, within its own retained `source/`.
Use a fresh proof root after any source change; existing immutable files must
not be overwritten or reused as if they described the new source.

Build current optimized test readers, `sm-pipeline` for cleanup and a separate
`sm-translation-sandbox` once, with an isolated `CARGO_TARGET_DIR` and `-j 1`.
Retain the exact executables under `tools/` with SHA-256 values. Current build
commands, before test-owner migration, are:

```sh
cargo test --locked --offline --release -j 1 -p pipeline --bin sm-pipeline --no-run
cargo build --locked --offline --release -j 1 -p pipeline --bin sm-pipeline
cargo build --locked --offline --release -j 1 -p translation-execution --bin sm-translation-sandbox
```

Use the migrated owner for the test build. Identify the emitted test
executable from Cargo output; do not select a stale binary by a wildcard.
Directly invoking that retained executable keeps proof execution distinct
from build time. `/usr/bin/time -v -o <proof>.time` can record elapsed time,
peak RSS and exit status, with test stdout/stderr retained in `<proof>.log`.
This is execution evidence, not a new performance campaign.

Every execution uses:

```sh
export SM_PIPELINE_ROOT=/home/jim/sm-decomp/out/pipeline
export SM_PIPELINE_WORKERS=4
export SM_NATIVE_FIXTURE_JOBS=4
unset SM_NATIVE_PERFORMANCE_ONLY
```

Keep the shared capacity at four. If other authorized verification is active,
coordinate scheduling or reduce this proof's Fixture jobs within that capacity;
do not allocate another pool to evade it. Run these expensive proofs serially.

## Gate 1: complete current catalog

Set both variables to fresh persistent paths:

```sh
export SM_NATIVE_CATALOG_PROOF=<proof-parent>/catalog
export SM_NATIVE_CATALOG_SOURCE=<proof-parent>/catalog-source
cargo test --locked --offline --release -j 1 -p pipeline --bin sm-pipeline \
  translation_native::tests::complete_current_catalog_has_a_frozen_native_recipe \
  -- --exact --ignored --nocapture --test-threads=1
```

For the retained reader, the equivalent invocation is
`<reader> --exact <migrated-test-name> --ignored --nocapture --test-threads=1`
under the same environment. Replace the package/filter above after migration.

The current test has one 600-second preparation deadline and uses 8 GiB
bounded-read scopes; it never launches a compiler or Fixture. It prepares every
entry in the dynamically discovered current catalog, retaining `catalog.toml`,
one `<catalog-key>.toml` prepared record and incremental `summary.toml`.
Every catalog entry must be represented and accepted. Do not hardcode the
catalog size or remove discovered declarations to pass the probe.

The additional `pause-direct-animation` contract must retain exactly 360 pairs,
`pause-direct-animation.toml` and `auxiliary-summary.toml`. Generated PLM
must retain its complete generator output. Both Ridley declarations remain in
catalog coverage. Historical results of 177 entries / 150,204 pairs plus 360
auxiliary pairs describe older evidence only; report the new frozen catalog's
actual totals. This test does not automatically perform cleanup.

## Gate 2: complete real recipe representatives

The default recipe selection covers only the two GameHost declarations.
Specify the full selection explicitly. This minimum matrix covers every
current recipe variant, with additional support-provider branches where the
code supplies different C inputs:

| Native recipe | Complete declaration to execute | Required coverage |
| --- | --- | --- |
| Plain / no recipe | `enemy-jump-instructions` in gate 3 | All 16 Fixture pairs, two routines |
| Dispatcher | `projectile-script-leaves` | Whole current registered manifest |
| Callback | `collision` | Whole current registered manifest and callback comparison |
| PowerBomb | `power_bomb` | Whole current registered manifest |
| GameHost | `game-host-projectile`, `game-host-room-scroll` | Both material callback controls; seven and three pairs respectively |
| Plm | `plm-spawn` | Whole normalized declaration, including ABI controls |
| Ridley | `ridley-subsystem-captured` | Whole current captured declaration and all original dependencies, including ABI controls |
| GeneratedPlm | `plm-instructions` | Complete generated domain and ROM validation |
| DirectAnimation | `pause-direct-animation` | Complete 360-pair auxiliary from the retained plan |
| SupplementedC | `animated-tile-frames`, `animated-tile-spawns`, `ceres-pad` | Whole declarations; each distinct support-provider mapping |

For Ridley this selects an existing complete declaration, not a sliced fixture
sample. The current regression explicitly maps it to `NativeRecipe::Ridley`.
Its historically measured domain is 670 pairs / 1,343 producers. Compare the
new result with the current declaration, not just those historical numbers.
The larger `ridley-subsystem` declaration is still frozen by gate 1; it is
another complete option for the same recipe if the integration owner chooses
it. Do not replace either declaration with a truncated manifest.

Current command and required environment:

```sh
export SM_NATIVE_RECIPE_PROOF=<proof-parent>/recipes
export SM_NATIVE_RECIPE_SOURCE=<proof-parent>/frozen-source
export SM_NATIVE_RECIPE_REQUIREMENTS=<proof-parent>/frozen-source/.scratch/translation-scope-execution/issues/10-original-acceptance-adapters.md
export SM_NATIVE_RECIPE_HELPER=<proof-parent>/tools/sm-translation-sandbox
export SM_NATIVE_RECIPE_COMPILER_SECONDS=180
export SM_NATIVE_RECIPE_FAMILIES=projectile-script-leaves,collision,power_bomb,game-host-projectile,game-host-room-scroll,plm-spawn,ridley-subsystem-captured,plm-instructions,pause-direct-animation,animated-tile-frames,animated-tile-spawns,ceres-pad
unset SM_NATIVE_PERFORMANCE_ONLY
cargo test --locked --offline --release -j 1 -p pipeline --bin sm-pipeline \
  translation_native::tests::real_reviewed_recipes_replay_and_authenticate \
  -- --exact --ignored --nocapture --test-threads=1
```

Use migrated test names/owner or the retained reader as above. The pinned C,
Rust compiler, C compiler, shared pool and four-worker capacity are currently
hardcoded to the audited paths. The source override controls snapshot input;
it does not change those runtime paths.

Each family has a 600-second gate and 64 GiB aggregate output allowance. Shared
preparation and each family preparation use 600-second / 8 GiB contexts.
Compiler limits are 180 seconds, 128 MiB writable, 4 KiB stdout, 256 KiB
stderr and no stdin. Fixture limits are five seconds, 16 MiB writable,
4 KiB per output stream and no stdin. Preserve these values; a failure is
not authorization to increase them or skip coverage.

For every selected family retain:

- Root `catalog.toml`, `requirements.md`, `source/` and final `summary.toml`.
- `<family>/prepared.toml`, `run/native-intent.toml` and
  `run/native-receipt.toml`.
- `run/native-operations/index.json` and every authenticated indexed chunk;
  `<family>/operations` with all confined producer results and dependencies.
- `run/family/verdict.toml`, the full `manifest.smf`, artifacts, source and
  binary inventory, comparisons and Fixture outputs.
- `<family>/cleanup.toml` and `cleanup-deduplicated.toml` plus post-cleanup
  Native authentication results.

The proof must authenticate before retry, preserve the identical receipt and
producer-result modification times on retry, perform both cleanup passes and
authenticate afterward. It currently does these checks. Keep them when moving
the harness. The room-scroll result checks all three cases have 17 callbacks;
projectile checks seven cases, two callbacks in each of the first two cases and
at least one in the third. Preserve these material-effect assertions.

A historical captured Ridley run completed 670 Fixture pairs but failed
finalization before publishing a Native receipt. Later bounded-tree changes
and read-only historical audits do not establish new Native acceptance. The
new real gate must publish and authenticate its own receipt. Preserve current
indexed operations and declared-tree limits; investigate any failure without
silently enlarging generic read or execution budgets.

## Gate 3: real family and negative controls

Current command and environment:

```sh
export SM_NATIVE_PROOF_ROOT=<proof-parent>
export SM_NATIVE_PROOF_NAME=full-real
export SM_NATIVE_SANDBOX=<proof-parent>/tools/sm-translation-sandbox
export SM_NATIVE_RUSTC=/home/jim/.rustup/toolchains/nightly-2026-09-29-x86_64-unknown-linux-gnu/bin/rustc
unset SM_NATIVE_PERFORMANCE_ONLY
cargo test --locked --offline --release -j 1 -p pipeline --bin sm-pipeline \
  translation_native::tests::full_real_family_retains_authenticates_and_reuses_every_producer \
  -- --exact --ignored --nocapture --test-threads=1
```

`SM_NATIVE_PERFORMANCE_ONLY` must be absent, not set to `0`: its presence
returns before the compiler/runtime/reservation negative controls. The proof
uses the full `enemy-jump-instructions` declaration and asserts 16 pairs, two
routines and 34 producer operations. It must additionally prove:

- Missing coverage, a missing canonical `memory.rs` dependency and changed
  provenance are rejected during preparation.
- Identical replay retains every original producer result and receipt.
- Tampering with a recorded producer invocation is rejected, then original
  bytes are restored and successfully authenticated.
- A candidate `include_str!` of protected `compiler-secret.txt` is rejected;
  unchanged failure replay reuses the same rejected receipt.
- Runtime parent read/write, sibling C-state read and `/program-c` read are
  denied while the complete family still passes and parent contents stay intact.
- An aggregate allowance of parent reservation plus 4,096 bytes rejects
  before launching producers, with operation count zero.

Per evaluation the gate is 180 seconds with 1 GiB aggregate output. Compiler
limits are 60 seconds, 128 MiB writable, 4 KiB stdout, 256 KiB stderr, no
stdin. Fixture limits are five seconds, 512 KiB writable, 4 KiB stdout/stderr,
no stdin. Authentication gets its existing 60-second / 8 GiB scope.

Retain `full-real/prepared.toml`, `source-inventory.toml`, `catalog.toml`,
`requirements.md`, `source/`, `operations/`, `compile-denial-source/`,
`runtime-denial-source/`, protected diagnostic files and each of `run/`,
`compile-denial/`, `runtime-denial/`, `reservation-denial/`. Each completed
output retains its intent, receipt, indexed attribution, verdict and artifacts.
The migrated test performs both cleanup passes and retained authentication
after the reservation assertion.

Preserve these nonignored regressions under their new owner as well:

```text
translation_native::tests::expired_outer_deadline_refuses_before_accessing_or_creating_any_source
translation_native::tests::expired_saved_failure_replays_authentication_without_runtime_or_dispatch
translation_native::tests::discovered_captured_ridley_keeps_its_original_callback_recipe
translation_native::tests::registered_catalog_alias_preserves_declared_family_and_coverage
```

Readiness must still execute the complete prescription; a compile-only result
does not cover the existing `check_ready` behavior.

## Read-only authentication after cleanup and worktree retirement

The migrated Rust harness provides
`translation_native::tests::retained_native_proofs_authenticate_without_execution`.
Select exactly that ignored test and set `SM_NATIVE_AUTHENTICATION_MANIFEST` to
the retained proof's `retained-authentication.toml`. It uses the prepared handle's
existing authentication interface without preparation, execution or cleanup.
Ordinary `family-authenticate` omits confined producer attribution and cannot
satisfy this gate. Rerunning the full real proof is also unsuitable: it rebuilds
its negative source variants and executes controls. This reader is application
test orchestration, with no new production session or receipt interface.

At proof creation, retain a small authentication-input manifest containing, for
each case, its prepared-record path, exact candidate checkout, Native receipt
path and expected passed/rejected outcome. Include all selected recipe outputs
and these full-proof combinations:

| Output | Prepared record | Candidate source | Expected result |
| --- | --- | --- | --- |
| `full-real/run` | `full-real/prepared.toml` | `full-real/source` | Passed |
| `full-real/compile-denial` | same | `full-real/compile-denial-source` | Rejected with retained compiler denial |
| `full-real/runtime-denial` | same | `full-real/runtime-denial-source` | Passed with runtime denial assertions |
| `full-real/reservation-denial` | same | `full-real/source` | Rejected with reservation exhaustion and no producers |

The reader loads retained prepared/receipt records and invokes Native
authentication for each exact candidate path. It asserts original outcomes
and complete coverage/producer evidence through the owning interface. It must
not call preparation, generation, execution, retry, cleanup, or recreate source
variants. Keep storage-specific traversal and tamper tests in the verifier
owner. Record the concrete migrated reader mode, arguments/environment and
test count in ticket 05 before treating it as a runnable completion check.

Authenticate after both cleanup passes and again after integrated worktree or
disposable-build removal. Retain source at the authenticated paths so those
operations do not require rewriting sealed contracts. The exact current reader
and required source, C archive/tree, ROM, capture inputs, provenance files,
operation indexes/chunks and producer results must survive retirement.
Read-only authentication should preserve receipt/result hashes and modification
times and require no live compiler/helper launch. The expired saved-failure
regression additionally exercises absent runtime/helper paths. Historical
retained readers may explain old evidence, but cannot validate this extraction.

## Cleanup and completion record

Follow [run-cleanup.md](../../docs/agents/run-cleanup.md), including failed or
interrupted proofs. Wait for all writers before cleanup. Use the separately
retained current pipeline executable and scope both passes to this work:

```sh
<retained-sm-pipeline> cleanup --root <finished-proof-root> --apply
<retained-sm-pipeline> cleanup --root <finished-proof-root> --deduplicate --apply
```

The recipe harness already records both passes for each `run/`; inspect those
results and cover any remaining proof directories or failed partial outputs.
Do not repeat real producers just to obtain a cleanup check. Preserve indexed
Native evidence, source snapshots, exact tools and diagnostic failures.
Remove only the integrated implementer worktrees and identified disposable
build outputs owned by this effort, after retaining their referenced source.
The integration owner handles that removal and final reauthentication.

Ticket 05 completion evidence must identify the integrated source, exact
commands/test names, nonzero selected test counts, explicit recipe selection,
catalog/Fixture totals, successful positive and expected-negative controls,
retained tool/source hashes, cleanup outputs, bytes reclaimed, remaining free
space and Native authentication after cleanup and retirement. A gate with a
missing prerequisite, rejected recipe or unauthenticated receipt remains
incomplete and retains its diagnostic evidence.

This read-only audit produced only this plan. It created no run directories or
build outputs, so there is no verification-run cleanup to perform for the audit.
