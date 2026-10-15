# Offline family migration and verification plan

Status: implemented; final verification and integration in progress

This implements the [approved design](proposal.md). All tools and tests stay
in Rust. The user approved implementation on 2026-10-15. Tickets 02–05 follow
the sequence below; [checkpoint.md](checkpoint.md) records current ownership.

## 1. Establish ownership and the two lower crates

Before source changes, follow `docs/agents/decomp-sessions.md`: inspect current
source/worktrees, production status and overlapping claims, identify the real
owners and select the permitted infrastructure scope. Keep installed policies,
production registries and other sessions' claims intact. Claim the local ticket
before implementation and use the applicable isolated-worktree procedure.

Writable groups for this stage:

- Workspace Cargo manifests/lockfile and new `crates/tool-execution/` and
  `crates/decomp-source/`.
- Shared helpers in `main.rs`, generic `builds.rs`, `build_cache.rs`,
  `bounded_inputs.rs`, `metrics.rs`, `resource_sampler.rs` and compiler
  dependency parsing in `inputs.rs`.
- Pin/source identity and inventory logic from `family.rs`, `family_model.rs`
  and `source_inventory.rs`, plus their current consumer imports.

Extract process execution and authenticated build preparation without changing
algorithms. Move bounded metadata reading intact. Separate its typed source cache
into the source context before creating a lower-to-higher dependency. The source
crate owns the existing serialized `Source` shape and private lexical analysis.

Adapt all existing source consumers to typed source operations. Put private
document helpers beside application consumers; remove unrelated helper imports
from `family`. Main supplies producer/configuration values instead of owning
reusable leases, hashes or atomic publication.

Completion: both new libraries build independently; existing compiler/cache,
source freshness, bounded-input and process behavior remains covered. No lower
crate imports application or Offline family code.

## 2. Extract the complete Offline family module

Writable groups for this stage:

- New `crates/offline-verification/` and its Cargo registration.
- Fixture model/codec/payloads, catalog declarations, family verification,
  callback/PLM/Ridley recipes and their maintained C/Rust templates.
- Fixture scheduler and workers, family inputs/wrapper generation, Native
  preparation/execution/admission and indexed operation storage.
- Recipe-specific build/invocation/output mapping currently in `builds.rs`.

Introduce the ordinary verification/admission interface and opaque Native
prepared handle. Move ordinary and Native code into the same owner, then replace
duplicated recipe dispatch, compiler-output naming and invocation reconstruction
with one private description per existing recipe. That description drives
ordinary execution, confined bindings and later evidence authentication.

Expose current Fixture authoring values/codec and catalog facts explicitly.
Keep recipe-specific execution and producer-operation traversal private. Preserve
the scheduler/cache implementation and all current comparison/receipt semantics.

Completion: ordinary and Native paths use the same recipe owner; no temporary
forwarding modules or wildcard exports recreate the previous interface.

## 3. Migrate callers and source declarations

Writable groups for this stage:

- `pipeline` CLI routing, `batch.rs`, `affected.rs`, `fixture_cli.rs`, Fixture
  authoring callers, Live predecessor admission and source-only consumers.
- Native application bridge and its Translation scope approval adapter;
  preparation/managed execution/Original callers that currently inspect verifier
  internals or reconstruct reservations.
- Ordinary/Native required-source inventories, retained source-copy lists,
  `include_str!`/template paths and test workspace constructors.
- Relevant crate READMEs and existing verification documentation.

Batch retains selection, scheduling, aggregation and batch cleanup. It delegates
family evaluation and current authentication-before-reuse behavior. Live consumes
an authenticated declaration while retaining its own route/composition checks.

The trusted Native bridge retains current scope-validation timing, exact frozen
contract authorization, installed-controller identity and Translation stage
publication. It delegates recipes, operation evidence and resource calculations.

Update every moved source path in declarations, prepared snapshots and checks.
Retain the conservative host-source superset. Extend conservative affected-family
rules to the three new crate paths. Update the stale historical-format language
in `crates/pipeline/README.md` to match `docs/FIXTURE_FORMAT.md` and the resolved
current-only policy.

Completion: pipeline callers no longer construct Fixture compiler/command
recipes, enumerate Native operation chunks or import family parsing helpers.
Lower Cargo dependencies cannot reach the application. CLI protocol and
conservative selection remain unchanged.

## 4. Verify current behavior and complete cleanup

Use the shared pipeline root and worker budget required by the session runbook.
Record source identity, commands, outcomes and run locations under each ticket.
Do not report a passing default test suite as proof that ignored Native gates ran.

### Build and interface checks

- Run formatting checks, locked/offline workspace compilation and the affected
  library/application test suites after caller migration.
- Inspect the Cargo dependency graph and public exports. Compile each new
  library through its declared dependencies; keep internal recipe/scheduler and
  receipt modules private. Do not add tests that merely mirror file layout.
- Migrate behavioral tests to the owning public interface where appropriate.
  Keep CLI tests that exercise argument/protocol behavior. Remove superseded
  tests of forwarding helpers once equivalent behavior is covered.
- Keep `tests/no_python.rs` and maintained-configuration checks passing.

Representative commands after the packages exist:

```sh
cargo fmt --all -- --check
cargo check --locked --offline --workspace
cargo test --locked --offline -p tool-execution -p decomp-source -p offline-verification -p pipeline
```

### Behavioral gates

| Concern | Existing coverage to preserve/migrate |
| --- | --- |
| Actual-C Fixture pairs, full state and continuations | `tests/family.rs`: `binary_roots_and_continuations_preserve_full_state_and_compact_evidence` and packed-word/pointer-bank controls |
| Current admission and tampering | Rebound-inventory tampering, stale manifest/candidate and Live predecessor controls in `tests/family.rs` |
| Execution order, chains, capacity and cancellation | Rust rejection before C; independent/blocked chains; idle capacity; pristine fork state; descendant cancellation in `tests/family.rs` |
| Adapter-specific comparison | `tests/callback.rs`, `tests/plm_spawn.rs`, recipe/ABI controls and existing Ridley/dispatcher tests |
| Build cache and source changes while waiting | `tests/build_preparation.rs`, current `inputs.rs` discovery/publication tests and bounded-input tests |
| Fixture authoring and ordering | `tests/fixture.rs` and current SMFIXT01 round-trip/validation coverage |
| Conservative affected families | `tests/affected.rs`; include all new crate paths in the current conservative harness-change case |
| Native receipts and ownership | Indexed-attribution corruption, incomplete/redirected evidence, candidate ownership, coverage, deadlines and retry controls currently in `translation_native.rs` and managed-execution tests |
| Application integration | Current batch reuse/locking, scope-approval, Translation integration and Live admission tests affected by changed calls |

### Explicit Native proofs

Run the existing ignored proofs under the refactored owner with their required
private source/ROM and installed confinement test prerequisites:

- `complete_current_catalog_has_a_frozen_native_recipe`: complete current
  declaration/recipe coverage; this gate does not launch producers.
- `real_reviewed_recipes_replay_and_authenticate`: representative real confined
  recipes, ABI controls and authentication across the moved recipe variants.
- `full_real_family_retains_authenticates_and_reuses_every_producer`: real
  compiler/actual-C execution, retained producer evidence and retry without
  relaunch.

Preserve the expired-deadline-before-access and saved-failure-without-runtime
controls. Readiness must still run the full prescription. Use fresh proof roots
and the current test configuration; do not modify installed production policy
to make a gate pass. Record any missing prerequisite explicitly and leave the
corresponding verification stage incomplete until it is resolved.

No CLI-only independence experiment, historical-format gate or performance
campaign is part of this refactor. Investigate regressions if the existing
behavior checks expose them.

### Cleanup and completion evidence

After writers finish, use the retained pipeline executable for both scoped
cleanup passes, then authenticate retained ordinary/Native evidence. Follow
`docs/agents/run-cleanup.md` for exact supported commands and retention rules.
Use `live-authenticate` after cleanup for any Live evidence produced by the
affected integration gates.

Record bytes reclaimed, free disk space, retained source/evidence dependencies,
and authentication outcomes, including failed-run diagnostics. Remove integrated
implementer worktrees and disposable build output after preserving required
source snapshots. Reauthenticate after relocation/worktree removal.

Resolve implementation tickets only after their checks and cleanup succeed.
Record the final source/commit and integration evidence in the spec. The final
result is all four owners with their stated responsibilities and both execution
paths verified under the current evidence contract.
