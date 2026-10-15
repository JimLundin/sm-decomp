# Offline family verification ownership

Status: implementation

The user selected candidate 01 from the architecture review on 2026-10-15.
The aim is a deep Offline family module with clear ownership of Fixture
execution, adapter recipes and the source/evidence contract. The preceding
review is recorded in `.scratch/architecture-scope-review-20261014/`.

Follow the selected architecture skill's grilling process. Record the design
tree and decisions before implementation. All implementation and tests will be
Rust. Design work does not authorize changes to other sessions' source or
installed production policies.

## Established constraints

- Preserve actual-C Fixture pair/Fixture chain comparison, source identity,
  tampering rejection, execution limits, cleanup and current receipt checks.
- Incomplete dependency knowledge leaves an Offline family affected.
- Current indexed verification formats only. The user's recorded instruction
  rejects historical compatibility work; do not add old-format readers or
  migration tools. See `../fixture-format-performance/issues/04-current-verification-only.md`
  and `docs/FIXTURE_FORMAT.md`.
- Keep the existing build preparation/cache and Fixture scheduler as deep
  implementation modules. This effort concerns their verification contract,
  not a separate redesign of execution scheduling or process accounting.
- Ordinary family admission authenticates recorded producer provenance but
  does not compare it with the current whole sm-pipeline executable hash.
  Installed Translation controllers separately require their pinned executable
  identity; do not promise controller reuse after changing that executable.

## User decisions

- Structure first. The user rejected making proof of CLI/family independence
  part of this first refactor. Precise affected-family selection and evidence
  reuse across unrelated CLI changes are deferred; keep conservative behavior.
- Choose the arrangement with the clearest separation and structure. The user
  delegated the packaging decision; no preference for keeping today's crate
  count or avoiding a dedicated verification crate should be assumed.
- Compare concrete designs, recommend the strongest ownership split and record
  its dependency direction and current-behavior checks before implementation.

## Work

- [Design tree](design.md).
- [Recommended ownership and interface](proposal.md).
- [Migration, behavioral gates and cleanup plan](implementation.md).
- [Design ticket](issues/01-design.md).
- Baseline inspected: `7702289` on `main`; no tracked source changes at startup.

## Proposed result

Add `offline-verification`, `tool-execution` and `decomp-source` as focused
libraries. Keep application composition and Translation approval in `pipeline`.
Use the simplest ordinary verification call and an opaque prepared Native handle,
with shared private recipe ownership and independent retained admission.

Three parallel designs have been compared. The user approved the displayed
architecture on 2026-10-15 and instructed implementation to proceed. The existing
Recovered C source concept is now named in the root glossary. Follow
[checkpoint.md](checkpoint.md) and tickets 02–05 for implementation and checks.
