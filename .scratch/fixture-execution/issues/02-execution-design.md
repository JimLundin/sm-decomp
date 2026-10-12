# Decide fixture execution module depth and isolation

Status: resolved
Blocked by: 01
Triage: needs-info
Type: grilling

## Settled constraints

- Scope: offline family checks plus live replay iteration speed.
- Include persistent-worker investigation, without assuming complete reset.
- Experiments approximately 60 seconds each; complete affected-family validation
  after selecting and implementing a change.
- Rust tools, orchestration and tests; preserve evidence contracts.
- Independent fixture chains may overlap. Preserve predecessor execution order
  and canonical report order.
- Investigate a preloaded helper that forks a fresh isolated child per fixture,
  reusing loaded data instead of loading it repeatedly.
- A failed fixture blocks all its descendants; independent chains continue.

## Accepted design

- Prototype ordinary/projectile execution first; extend callback/PLM after
  demonstrating savings and isolation.
- Preserve per-case state/results/host/callback captures/stdout/stderr and
  command/timing identities; aggregate metrics and record blocked descendants.

The concrete proposal is in [design.md](../design.md). Performance selection
will use complete bounded wall time with exact state equality and isolation
controls; no numerical improvement is assumed from retained receipts.

Further decisions about the module interface, adapters, evidence, rollout and
tests follow these prerequisites. Implementation is not started.

## Comments

Investigation is guided by the user-invoked improve-codebase-architecture skill
and `/home/jim/.agents/skills/grilling/SKILL.md`. The latter requires rounds of
decisions before implementing the selected design. Source facts are in ticket01.

User accepted independent-chain concurrency, preloaded fork-per-fixture helper
investigation and blocking all descendants after any predecessor failure.

User accepted the remaining scope/evidence proposals and confirmed shared
understanding with "accept". Bounded prototype execution is authorized.

## Completion evidence

Design and source investigation are recorded in `../design.md` and ticket01.
No verification run was created by this design round; cleanup scope is empty.
