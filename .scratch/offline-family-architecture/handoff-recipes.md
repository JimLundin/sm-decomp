# Shared Offline family recipe owner

Status: implemented and library compilation passed; consumer integration remains
with the Offline verifier and Native owners.

- Owner: `/root/gameplay_architecture`
- Worktree: `/home/jim/sm-decomp-offline-recipes`
- Branch: `architecture/offline-recipes-20261015`
- Initial baseline: `446dd64`
- Scaffold: `988ae73`, cherry-picked locally as `09c6234`

## Scope

- Added private `recipe.rs` and its private `recipe_commands.rs` child module.
- Added `mod recipe;` in `lib.rs` and both files to
  `sources::IMPLEMENTATION_SOURCES`, as authorized by the integrator.
- No pipeline consumer, adapter, build execution, manifest or queue edits.

`Recipe` is `pub(crate)`. Its existing internally tagged serde representation,
variant names and fields are unchanged. The owner does not depend on Native
contracts, task lookups, prepared handles or application scheduling.

The original registration rules, explicit catalog aliases, supplementary C
providers, complete generated PLM comparison, complete 360-case direct animation
domain, asset authentication and captured Ridley dependency exception are kept.
Ridley retains the original 8,192-file ceiling; other recipes retain 4,096.

## Consumer integration

Use only `mod recipe;` at the crate root. `recipe.rs` declares the command file
as its private `commands` child using `#[path = "recipe_commands.rs"]`. A second
crate-root `mod recipe_commands;` declaration is unnecessary and incorrect.

The common entry points are:

```rust
recipe::for_adapter(adapter: &str, manifest: &Manifest) -> Result<Option<Recipe>>
recipe::for_family(checkout: &Path, path: &Path, declared: &Manifest)
    -> Result<Option<Recipe>>
recipe::registration(checkout, manifest: &Asset, family: &str, recipe, original)
    -> Result<Registration>
recipe::build(recipe, manifest, request, checkout, pinned_source, run,
    execution: Option<&builds::StrictExecution>) -> Result<Implementation>
recipe::fixture(recipe, manifest, routine, fixture, index, run,
    execution: FixtureExecution<'_>) -> Result<CaseRecord>
recipe::verdict_fields(recipe, manifest, verdict: &mut Verdict) -> Result<()>
```

`FixtureExecution` admits either `Ordinary { request, worker }` or
`Confined(&StrictExecution)`. Root's scaffold already exposes `family::build_with`
and `family::case` within the crate. Keep `inputs::with_host` around owning public
operations; the recipe/build methods have no host-input parameter.

Ordinary adapter selection preserves ordinary manifest behavior and does not
require a Native catalog registration. Native retains its registration and
preparation checks and passes the resolved recipe to the same owner.

Native's remaining variant-specific readback uses:

```rust
recipe::retained_manifest(manifest: Manifest, recipe: Option<&Recipe>)
    -> Result<Manifest>
recipe::authenticate_fixture_artifacts(recipe, directory, label) -> Result<()>
```

The first normalizes only retained PLM declarations. The second authenticates
packed callback snapshots only for callback, PowerBomb and GameHost recipes;
invoke it after authenticating a passed noncompiler operation.

## Shared command contracts

```rust
recipe::rust_compiler_command(recipe, run) -> CompilerCommand
recipe::c_compiler_command(recipe, manifest, source, run,
    c_units: &[PathBuf], fork: bool) -> Result<CompilerCommand>
recipe::compiler_files(recipe, rust: bool)
    -> (Vec<&'static str>, Vec<&'static str>, bool)
```

`CompilerCommand` contains `command`, `generated`, `outputs` and `full_c`.
Pass its file description unchanged to the existing confined compiler mapping.
The common builders perform no filesystem mutation. Actual adapters still emit
their wrapper/oracle and instrumented C contents and authenticate those sources.
They must pass the trusted **original sorted C units**, before replacing unit
paths with generated seam paths. The builder owns those replacements, all flags,
link-wrap ordering, output names and ordinary family fork-worker linkage.

```rust
recipe::fixture_command(recipe, manifest, selection, run, label)
    -> Result<FixtureCommand>
recipe::fixture_program(recipe, manifest, selection, run, label, helper)
    -> Result<confinement::Program>
recipe::confined_fixture_program(helper, command, kind)
    -> Result<confinement::Program>
recipe::fixture_output_path(run, label, name) -> PathBuf
```

`FixtureSelection` is either `Case { routine, fixture, index }` or `AbiControls`.
`FixtureCommand` contains `command` and `kind`. `FixtureKind` preserves all six
old `StrictFixture` cases. The positional input/output map, callback packed-output
handling, accepted argument counts and output restoration paths moved intact.
Native must authenticate its selected case directory before constructing the
selection. The common code does not resolve a Native task lookup.

Root can remove the old `StrictFixture` enum and positional mapping from
`builds.rs`, using a crate-private alias or the common names directly. Keep its
generic compiler confinement/runtime and execution ownership there.

Plain, Ridley and ABI execution should call these command builders. Callback
and PLM construction already share their pure `fixture_command` helpers, which
the recipe builder calls. Do not replace those helpers with calls back into
`recipe::fixture_command` without moving their bodies, which would recurse.

## Source sealing

The conservative compiled-source declaration includes both new files. Recipe
harness paths use the moved Offline adapter/template paths. The Ridley authoring
generator intentionally remains sealed at
`crates/pipeline/src/ridley_subsystem_fixtures.rs`; it is not imported as Rust
code by the new owner. Root supplies that host source stamp.

## Completion evidence

- `rustfmt --edition 2024 --check` passed for `recipe.rs`,
  `recipe_commands.rs`, `lib.rs` and `sources.rs`.
- `git diff --check` passed.
- `SM_PIPELINE_ROOT=/home/jim/sm-decomp/out/pipeline SM_PIPELINE_WORKERS=4 cargo
  check --locked --offline -p offline-verification --lib -j 4` passed in 32.73 s.
  The compiler reported expected unused/dead-code warnings while ordinary and
  Native consumers still await integration; no type errors occurred.
- The first check stopped before compilation because this new worktree lacked
  its ignored `deps/rustysnes` tree. A temporary `deps` symlink to the existing
  main dependency tree allowed the successful check.
- Reviewed compiler argv, generated/output descriptions, Fixture/ABI argv and
  confined positional mappings against the original adapters, Native recipe and
  build implementation. This is source review, not runtime proof.
- No actual-C run, ignored Native proof, evidence gate or gameplay test was run.
  Root must run the approved integrated verification before ticket closure.
- After the compiler exited, `du -sb target` measured 270,406,488 bytes.
  `cargo clean --locked --offline --target-dir
  /home/jim/sm-decomp-offline-recipes/target` removed 1,109 files (260.0 MiB
  reported). `unlink deps` removed the temporary dependency link.
- No run directories or retained evidence dependencies were created. Available
  disk space after cleanup was 156,374,896,640 bytes (`df -B1`). Root remains
  responsible for removing this implementer worktree after integration.
