# Research Map Rando compatibility

Status: open
Blocked by: 09
Triage: ready-for-agent
Type: research

After full Rust vanilla gameplay ownership, define the supported Map Rando
version, seed interface and custom-behavior contract. Follow
[the pivot](../../../docs/DECOMP_PIVOT.md); vanilla ROM table access alone does
not reproduce randomizer gameplay-code and per-seed setup patches.

## Questions to answer

- Which Map Rando version/settings and reproducible seed inputs are supported,
  and which modified code, topology/item data and door/setup behavior must become
  owned Rust rather than CPU fallback?
- Which stable vanilla seams and exact modules can host those behaviors, with
  dependency-ready disjoint child scopes and one integration owner?
- Which selected seeds, routes and custom/trick behavior prove actual seed play
  and completion, and what unsupported behavior could make a seed unbeatable?

## Acceptance

`## Answer` pins the version/seed contract, custom-behavior inventory, evidence
sources, compatibility exclusions, child implementation dependencies and concrete
acceptance gates. Add an Answer pointer to the map. Supported tricks and seed
consequences are explicit; exceptions covered by the pivot's required user
review follow that existing instruction. Research completion creates actionable
implementation work; actual Map Rando seed completion still needs its children.

## Answer

Pending research.

## Completion evidence

Append pinned source/data identities, selected seed inputs, research evidence
hashes, seam/ownership review and child ticket pointers. Complete scoped
[run cleanup](../../../docs/agents/run-cleanup.md) for any experiments and record
measured bytes/free space and retained dependencies, or record that no runs were
created before resolving.
