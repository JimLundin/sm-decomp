# The clean-room process

How one domain of the game goes from the original to a model on `main`. Each domain is rebuilt in
**rounds**: a spec, a model built from that spec alone, and proof that the model matches the real
game. This file is the single source of truth for the order of a round, who does each step, and
who sees what. The rules it relies on live elsewhere, and are linked where they are used:
- [MODEL §5](../architecture/MODEL.md#5-how-the-work-is-planned): what a leak is, and the spec
  template;
- [DESIGN](DESIGN.md): the coding standards;
- [size-budget](../architecture/size-budget.md): the score and the budget.

## Roles

Each role is its own session with its own skill, and is never combined with another role in one
session. The root starts each session with a brief.

| Role | Does | Skill |
| --- | --- | --- |
| root | runs the round: briefs, exports, launches, imports, lands, reports to Jim | [root](../../.agents/skills/root/SKILL.md) |
| observer | reads the original; writes the spec, the vectors, the decode notes and the reference model; answers gaps | [observer](../../.agents/skills/observer/SKILL.md) |
| adversary | gates a spec before a round, and a round before it lands | [adversary](../../.agents/skills/adversary/SKILL.md) |
| implementer | builds the model from the spec alone, confined to an export | [implementer](../../.agents/skills/implementer/SKILL.md) |
| blind reviewer | reads the model's code cold, confined, and reviews it as code | [blind-reviewer](../../.agents/skills/blind-reviewer/SKILL.md) |
| integrator | wires the model into the engine, writes its honest harness, deletes the old code | [integrator](../../.agents/skills/integrator/SKILL.md) |
| engineer | builds tools, checks and CI outside a round | [lane](../../.agents/skills/lane/SKILL.md) |

Every role except the confined two (implementer, blind reviewer) also follows the
[lane](../../.agents/skills/lane/SKILL.md) skill for its worktree, commits, proof and report.

## Who sees what

The **barrier** is the reason the clean room exists: the model is shaped by what a player sees,
not by the original's code. The implementer and the blind reviewer work in a confined export
(`tools/cleanroom/launch.sh`) that holds only their column. Every other role works in the
repository.

| Material | observer | adversary | implementer | blind reviewer | integrator | root |
| --- | :-: | :-: | :-: | :-: | :-: | :-: |
| the original (bank logs, ROM, records, decode notes, `verify`'s addresses) | yes | yes | - | - | yes | yes |
| the spec and its vectors | yes | yes | yes | vectors only | yes | yes |
| the clean model's code and tests | after import | yes | writes it | yes | yes | yes |
| the score, budget, k, fit, old-code baselines | yes | yes | - | - | yes | yes |
| the old engine code | yes | yes | - | - | deletes it | yes |

The score is an **evaluation** tool: the root and the adversary use it to compare spec and code
after a round. The implementer's goal is to model the spec well, and its material states coding
standards only. A size finding reaches the implementer as a **shape** finding: duplication, a
decision that could be a table row, a function doing two jobs. It never arrives as a number.
After a round gate, the next brief may also give a **coarse verdict** in words: that the model is
much larger than the behaviour needs, and in which files and items (MODEL §1 decision 11). No
score, budget, ratio or target goes with it.

## A round

Each step names its owner, what it waits for, and its done-condition.

```
1 spec ─ 2 brief ─ 3 spec gate ─ 4 land spec ─ 5 launch ─ 6 implement ─ 7 import ─┬─ 8 blind review ─┬─ 10 round gate ─ 11 land ─ 12 queue
                                                                                  └─ 9 integrate ────┘
```

1. **Spec** (observer). Done when:
   - `docs/behaviours/<domain>.md` states every rule in arm form;
   - `scenarios/vectors/<domain>.toml` holds recorded and constructed vectors;
   - `docs/behaviours/<domain>.decode.md` holds the sources, the harness note and the Budget
     section;
   - the observer's reference model agrees with every vector;
   - `tools/cleanroom/scan.sh` finds 0 on the spec and vectors;
   - the doc links pass.
2. **Brief** (root), after 1. Done when the brief:
   - names only spec entries and shape findings;
   - carries the runner command;
   - passes `tools/cleanroom/scan_brief.sh` with 0.
3. **Spec gate** (adversary), after 1 and 2: it gates the spec and the brief together. Done on a
   verdict of LAUNCH. LAUNCH AFTER FIXES sends numbered findings to their owners (the observer
   for the spec, the root for the brief), then a re-gate of the delta.
4. **Land the spec** (root), through `tools/lane/land.sh`, when it is a new domain's first spec.
   An amended spec changes what the landed model must do, so the model's own vector test fails
   on it until the round implements it. An amendment therefore stays on its branch, the round
   exports from that branch's commit, and the spec lands with the round (step 11).
5. **Export and launch** (root): `canary.sh plant`, `export.sh` (with `-c COMMIT` for an
   amendment's branch), then `launch.sh`. Done on `LAUNCH OK` with a clean canary check.
6. **Implement** (implementer, confined). Done when every vector passes or is listed as a gap,
   and check, clippy and fmt are clean.
7. **Import** (root): `import.sh` runs `code_check.sh` and applies the commits to a lane branch.
   Done when the import passes. A refusal goes back to the same implementer's export as a short
   follow-up session: the brief quotes each finding (file, line, the rule it breaks) in the
   coding standards' words, and the import runs again.
8. **Blind review** (blind reviewer, confined), after 7. The root makes a code-only copy (the
   model, its tests and the vector file) and launches the review. Done on its report. The root
   compares the review's "what the code does" with the spec: where the reviewer misread the
   code, the code doesn't explain itself.
9. **Integrate** (integrator), after 7. Done when:
   - the engine runs the model through an adapter;
   - the honest harness compares it with the records, and its floors are committed;
   - the old code it replaces is deleted;
   - `tools/lane/gate.sh` passes;
   - the free-run gate's green pairs and the lockstep ratchet hold.
10. **Round gate** (adversary), after 8 and 9. It reads the leaks, the harness's honesty, the
    score against the budget and the blind review. Done on a verdict of MERGE. Findings go to
    the integrator (harness, adapter), or into the next round's queue (model shape, spec gaps).
11. **Land** (root), through `land.sh`: the spec amendment, the import and the integration
    together. `docs/REPORT.md` is regenerated in the same change.
12. **Queue** (root). Every gap goes to the observer, and every shape finding to the next brief.
    The domain is done when the queue holds nothing that changes behaviour or shape enough to
    be worth a round.

## What can run at once

**A manageable amount** (Jim, 2026-11-08). At most three lanes run at once, and at most one round
is in flight per domain, so every lane stays observable while the process is still changing. The
root names each lane to Jim when it starts, and reports its result when it ends. Within that
limit, the dependencies below decide what may overlap.

**Within a round:** steps 8 and 9 run together. Both need only the import; the blind reviewer
writes nothing, and the integrator works outside the clean crates.

**Across rounds of one domain:** the observer answers round N's gaps (step 1 of round N+1) as
soon as step 6 reports them, alongside N's steps 7 to 10. Round N+1's export (step 5) waits for
N to land, so the implementer starts from the landed model.

**Across domains:**

| Role | Runs alongside another domain's | Constraint |
| --- | --- | --- |
| observer | any step | reads the original and writes its own domain's files |
| adversary | any step | one gate per subject |
| implementer | any step | `crates/clean_types` is shared: two implementers in flight at once each add only new items there, and the second import rebases on the first |
| blind reviewer | any step | none |
| integrator | integration or gate | both touch the engine and `crates/verify`: the second rebases on the first, and re-proves the gates |
| engineer | any step | its files lie outside every running round |
| root | none | it alone briefs, imports and lands; landing is one change at a time |

A domain's spec can start before a domain it reads from has landed. Its own round then waits
until that domain has landed, so the interface it reads is fixed.

## The loop

- **Cut rounds.** When a round gate's coarse verdict says the model is much larger than its
  behaviour needs, the next round of that domain is a cut round: its brief carries the verdict
  and the gate's shape findings, and asks for no new behaviour. Cut rounds repeat until the
  gate finds nothing that changes the shape enough to be worth a round.

- A vector that won't pass, and that the spec doesn't explain, is a **gap**. The implementer
  records it, and the observer answers it in the spec. The model is never tuned to a record.
- A finding is fixed by its owner: the observer owns spec and vectors, the implementer the clean
  crates, the integrator the adapter and harness, the root briefs and landing.
- Old code and documents kept only for comparison are deleted. A comparison uses git history
  (MODEL §1 decision 8).
