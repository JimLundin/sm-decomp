# Candidate 01 design tree

## Settled

- Candidate: Offline family verification ownership.
- Domain language: root GLOSSARY.md. Use Offline family, Fixture pair,
  Fixture chain and Affected Offline family. Recovered C source is now defined
  for the shared source authentication/analysis owner.
- Current-only verification policy is already established; compatibility with
  retired encodings/results is not a question for this design.

## Round 1 — answered

### Q1: First deliverable

User decision: reorganize first. Proof of independence from unrelated CLI edits
is not required for this pass; precise selection and evidence reuse are deferred.

### Q2: Packaging

User decision: choose the most clearly separated and well-structured design.
Packaging is delegated to the assistant. A separate executable is not
intrinsically necessary for ordinary family admission to survive CLI-only
changes, but that proof is not this pass's objective.

## Downstream decisions

Three materially different interfaces and ownership arrangements have been
compared under the codebase-design skill. The [recommendation](proposal.md)
chooses three focused libraries with a small ordinary interface and an opaque
Native prepared handle. The [migration plan](implementation.md) specifies shared
execution/source ownership, caller changes and current-behavior verification.
Packaging is decided through delegated engineering judgment; it is not an open
question for the user.

## Source facts

- `inputs.rs:401–406` records the producer executable identity in
  pipeline-inputs.toml. `family.rs:1222–1227` authenticates retained artifact
  bytes. It does not compare that recorded value with the current executable.
- `family.rs:1321–1336` checks current implementation dependencies/harness
  sources and currently includes main.rs.
- `translation_execution.rs:429` separately checks the installed controller
  executable against controller.sha256; production preparation seals that
  binary at `translation_production_prepare.rs:379/392`.
- `docs/FIXTURE_FORMAT.md:95–115` requires current indexed attribution and
  explicit candidate ownership. Resolved ticket 04 records the user's explicit
  rejection of historical compatibility work.

No implementation, verification run or queue mutation has started. Documentation
includes the proposed design and a definition of Recovered C source.

## Completed source refresh

The explorer confirmed candidate 01 at `7702289`. The following are facts to
carry into the next design round, independent of the pending choices:

- Planning and evidence admission are separate: `affected.rs:237/550` selects
  every family for any pipeline path; ordinary admission checks the sealed
  verifier/harness dependencies in `family.rs:1321/1332`.
- Native verification also declares main.rs and other common harness files at
  `translation_native.rs:359`, freezes them at `:468`, and checks them at `:782`.
  Full family independence therefore requires ordinary and native inventories
  to be treated together.
- Reusable helpers currently live at the binary root. The Fixture scheduler
  calls root leases (`fixture_execution.rs:198`); build preparation calls root
  executable handling (`build_cache.rs:2`). These need an explicit implementation
  owner outside CLI routing even if no crate is added.
- `family.rs` also contains parsing helpers used by unrelated callers, e.g.
  `readiness.rs:60`; exporting the whole file would preserve a shallow interface.
- Source inventory parsing and family authentication depend on each other
  (`family.rs:835/868`, `source_inventory.rs:175`). Separate lexical parsing
  ownership from inventory orchestration when shaping the module.
- Recipe knowledge spans adapters (`callback.rs:470`, `plm_spawn.rs:292`),
  positional input/output mapping (`builds.rs:79`), and native construction and
  authentication (`translation_native_recipe.rs:308/333/419`). Consolidation
  must cover ordinary and confined Fixture execution.
- `crates/pipeline/README.md:31/91` is stale about preserving historical formats;
  the current format document and resolved user-directed ticket take precedence.

## Deferred proof — outside this refactor

Produce current-format family evidence with the refactored verifier, then
rebuild with only a proved-independent CLI/help edit. Family planning should
exclude it and both ordinary and native evidence checks should retain validity.
Changes to verifier/adapter inputs must still select affected families and reject
stale admission. Unknown dependencies remain affected. Pre-refactor receipts
sealed against the broad dependency list need no compatibility shim.

## Current behavior to preserve

Preserve existing behavioral tests for actual-C results, Fixture chains, shared
capacity, tampering, source identity, current indexed native attribution,
writable ownership, confinement and retry without relaunch. The installed
Translation controller's own binary fence remains a separate acceptance rule.

## Design comparison — completed

- A: three verification operations with separate execution and source crates.
  Strongest enforced dependency direction; ordinary preparation is too visible.
- B: immutable plans, execution sessions and two sealed execution adapters.
  Useful Native/Live concepts; too much public session machinery for ordinary use.
- C: private Offline family module with a simple application session.
  Best common caller; weaker enforcement of dependency direction in one crate.

Choose A's ownership split, C's ordinary ergonomics and B's authenticated
declaration/prepared-state ideas. The selected packages are
`offline-verification`, `tool-execution` and `decomp-source`. Keep execution
adapters private; preserve the narrow caller-owned Translation scope approval
adapter and existing validation timing.

The generic bounded-input module currently caches `family_model::Source`.
Separate that typed cache into the source context while preserving its shared
operation lifetime and budget. This is necessary to make the new dependency
direction real, rather than exporting family helpers from another directory.

## Final checkpoint — approved

The user reviewed the rendered architecture and replied "i approve. go ahead"
on 2026-10-15. The design frontier is empty and implementation is authorized.
Routine packaging/interface choices and reorganization before independence
proofs remain settled. Do not repeat the confirmation or packaging questions.
