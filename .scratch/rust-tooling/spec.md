# Rust project tooling

Status: resolved

The project language is Rust, including verification orchestration, fixture
generation, setup, cleanup and tests. The user explicitly requires removing
Python from the project. Rust commands that invoke Python are transitional,
not completion evidence.

Remove internal JSON usage, as requested by the user. Native interfaces use
typed Rust structures; readable configuration and small metadata use TOML;
large state and ordered traces use binary formats. JSON readers remain only
where required by external reference corpora. Backwards compatibility is not
required: remove JSON imports and old format adapters when a native TOML/binary
route covers the capability.
New formats have explicit versions and are never retroactively assigned to
old evidence. The existing dynamic JSON migration draft remains transitional.

Continue on `decomp-pipeline-performance` in `/tmp/sm-decomp-pipeline`.
Preserve the original checkout's gameplay work. Retain shared worker capacity,
content-addressed builds, per-worktree run ownership, Rust-before-C admission,
authenticated reuse and packed live evidence.

Completion requires replacing every maintained Python entry point and test,
removing project-owned Python sources, switching CI/recipes/documentation, and
running Rust verification plus cleanup. Historical sealed evidence may retain
its source snapshots; changing those would invalidate the evidence.

## Completion evidence

Python and internal JSON migration is complete under the user's narrowed refactor
scope. Implementation, verification, cleanup, retained dependencies and interrupted
workflow limits are recorded in [completion.md](evidence/completion.md).
Integration branch: `decomp-pipeline-performance` in `/tmp/sm-decomp-pipeline`.
Full gameplay readiness and new optimization increments are outside this task.

## Inventory

At e820a6b: 127 tracked tools/*.py files, 18,922 lines. Includes fixture generators,
decomp/port verification, readiness and iteration gates, emulator/movie setup,
storage studies, and tests. External downloaded source is outside the migration.

## Tickets

1. Rust cleanup and native worker/build commands.
2. Rust setup/bootstrap and hardware checks.
3. Offline family verification and batch ownership in Rust.
4. Fixture generators and specialized port adapters in Rust.
5. Packed live orchestration and authentication in Rust.
6. Decomp acceptance/jobs and readiness/iteration gates in Rust.
7. Movie/emulator utilities and remaining studies; remove Python and add CI guard.

## Active section ownership

The orchestrator integrates shared CLI, worker/build/cache interfaces, cleanup,
batch, baseline and source inventory. `family_live` owns family fixtures, offline
verification, callbacks, PLM, live, decompression and RNG. `native_utilities` owns
bootstrap, movie/emulator utilities, packed evidence and timed corpus producers.
`readiness_iteration` owns readiness gates, diagnostics, pattern proof/promotion
and bounded iteration. Agents share this worktree and coordinate shared types
before changing consumers.

All sections are resolved. Final migration checks and cleanup are recorded in
the completion evidence. Full gameplay acceptance remains a separate workflow.
