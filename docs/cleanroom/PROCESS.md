# The clean-room process

How one domain of the game goes from the original to a model on `main`. Each domain is rebuilt in
**rounds**: a spec, a model built from that spec alone, and proof that the model matches the real
game. This file is the single source of truth for the order of a round, who does each step, and
who sees what. The rules it relies on live elsewhere, and are linked where they are used:
- [MODEL §5](../architecture/MODEL.md#5-how-the-work-is-planned): what a leak is, and the spec
  template;
- [DESIGN](DESIGN.md): the coding standards;
- [MODEL §4](../architecture/MODEL.md#4-the-reference-domain-samuss-motion) and §1 decision 7: the
  score and the budget, a diagnostic since decision 19; [size-budget](../architecture/size-budget.md),
  the study behind them.

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
| the score, budget, k, fit, old-code baselines and size verdicts | yes | - | - | - | yes | after review |
| the old engine code | yes | yes | - | - | deletes it | yes |

Independent review judges behaviour and code before the root counts size (MODEL §1 decisions
10 and 11). The root provides adversaries with review copies of decode notes and evidence that
preserve source citations and harness notes, and omit size material wherever it appears,
including Budget sections, reports and size verdicts; their briefs assign no size assessment.
Blind reviewers receive their code-only export. Reviewers return their independent findings
without receiving size material or assessments. The implementer's material states coding standards only;
feedback carries concrete shape findings from review, with size numbers and coarse verdicts
withheld. The root keeps the subsequent assessment in the coordination log and landing report.

## A round

Each step names its owner, what it waits for, and its done-condition.

```
1 spec ─ 2 brief ─ 3 spec gate ─ 4 hold spec ─ 5 launch ─ 6 implement ─ 7 import ─┬─ 8 blind review ─┬─ 10 round gate ─ 11 land ─ 12 queue
                                                                                  └─ 9 integrate ────┘
```

1. **Spec** (observer). Done when:
   - `docs/behaviours/<domain>.md` states every rule in arm form;
   - `scenarios/vectors/<domain>.toml` holds recorded and constructed vectors;
   - each entry whose switch is `Parameter` has at least one vector at a value other than the
     reference one: a vector that names the entry and overrides the reference rules (the
     vector's `rules` table), so adjusting the behaviour is proven, not assumed.
     `tools/cleanroom/adjusted_vectors.sh --gate <domain>` passes. A domain specced before this
     rule meets it at its next spec round; until then the script's listing is advisory;
   - `docs/behaviours/<domain>.decode.md` holds the sources, the harness note and the Budget
     section;
   - the observer's reference model agrees with every vector;
   - `tools/cleanroom/scan.sh` finds 0 on the spec and vectors;
   - the doc links pass.
2. **Brief** (root), after 1. Done when the brief:
   - names only spec entries and shape findings;
   - carries the runner command;
   - passes `tools/cleanroom/scan_brief.sh` with 0.
3. **Spec gate** (adversary), after 1 and 2, on a domain's first spec and on a spec delta that
   adds a mechanism (a new table, interpreter or kind of state) or crosses into another domain
   (MODEL §1 decision 20). It gates the spec and the brief together, and refuses a spec whose
   `Parameter` entries lack an adjusted vector (`adjusted_vectors.sh --gate`). Done on a
   verdict of LAUNCH. LAUNCH AFTER FIXES sends numbered findings to their owners (the observer
   for the spec, the root for the brief), then a re-gate of the delta. Any other amendment
   skips this step: its scans and `adjusted_vectors.sh --gate` still pass before the launch,
   and the round gate (step 10) reviews the spec delta with the code.
4. **Hold the spec** (root). A round lands as one commit (step 11), so the spec stays on the
   round's branch, a new domain's first spec as much as an amendment, and the round exports from
   that branch's commit. An amendment couldn't land alone anyway: it changes what the landed
   model must do, so the model's own vector test fails on it until the round implements it.
5. **Export and launch** (root): `canary.sh plant`, `export.sh -c COMMIT` with the spec's commit
   on the round's branch, then `launch.sh`. Done on `LAUNCH OK` with a clean canary check. A new
   domain is named `<area>-<name>`, its area from `domains.toml`, and its first export creates its
   crate at `crates/clean/<area>_<name>`.
6. **Implement** (implementer, confined). Done when every vector passes or is listed as a gap,
   and check, clippy and fmt are clean.
7. **Import** (root): `import.sh` runs `code_check.sh --import`, which refuses a variant import
   (L4) even in a crate whose older ones still only warn on the trunk, applies the commits to a
   lane branch, then runs both clippy configurations of `tools/lane/gate.sh` over every clean
   crate in the workspace: an export holds one domain's model and `types`, so a change to
   `types` can break a crate its implementer never saw. Done when the import passes. A
   refusal goes back to the same implementer's export as a short follow-up session: the brief
   quotes each finding (file, line, the rule it breaks) in the coding standards' words, and the
   import runs again.
   A clippy failure names the crates and leaves the patches applied:
   - a **mechanical** fix (a parameter by value instead of by reference, a derive, `assert_eq!`
     for `assert!(a == b)`, an import path; no item's logic changed) is the root's own commit,
     right after the import, titled `[exact]` with `Role: root` and the lint it answers. The
     round's squashed commit lists it under the root, and the round gate checks it;
   - anything else goes back to an implementer: this domain's when its `types` change
     wasn't needed, the broken crate's next round when it was;
   - an integrator never edits a clean crate. A fix it finds it needs is a finding for the root.
8. **Blind review** (blind reviewer, confined), after 7, on a domain's first round and when the
   last round gate asked for one (MODEL §1 decision 20). The root makes a code-only copy (the
   model, its tests and the vector file) with `tools/cleanroom/blind_copy.sh EXPORT DEST` and
   launches the review on it. Done on its report. The root compares the review's "what the code
   does" with the spec: where the reviewer misread the code, the code doesn't explain itself.
9. **Integrate** (integrator), after 7. Done when:
   - the engine runs the model through an adapter, which holds the model's State as the
     engine's (MODEL §1 decision 21) for a new domain, and moves toward it for an old one: a
     `canonical::model::Model<P>` in the engine's State, `P` pinning the State's shape, with a
     test that `canonical::model::pin_holds::<P>()` (decision 29);
   - the honest harness compares it with the records, and its floors are committed;
   - the domain free run (`every_domain_runs_free_against_the_cache`) holds its floors in
     `crates/verify/src/harness/free_floors.txt`; a raise is committed with the round;
   - the old code it replaces is deleted;
   - `tools/lane/gate.sh` passes;
   - the free-run gate's green pairs and the lockstep ratchet hold.

   Where the engine can run the model exactly only once another domain gives it inputs that
   domain's model doesn't yet give, the round lands with the harness alone. The domain's gaps
   file names the missing inputs, and LEGACY lists what the old code lacks. The engine wiring,
   and the deletion of the old code, then land with the round that supplies those inputs.
10. **Round gate** (adversary), after 8 (when it ran) and 9. It reads the leaks, the spec delta
    when step 3 didn't gate it, the harness's honesty, the free run's held frames and first
    divergences against `main`, and the blind review. It says whether the next round needs a
    blind review. Done on a verdict of MERGE. Findings go to the integrator (harness, adapter),
    or into the next round's queue (model shape, spec gaps). Independent review excludes size
    measurements. After the reports, the root completes the coordinator size assessment below.
    Size does not change the adversary's verdict; every non-size gate remains mandatory.
11. **Land** (root), as **one commit**: the spec, the import and the integration of the round
    together (Jim, 2026-11-10). On the round's branch the root adds one to the model's `rounds`
    in [`docs/architecture/domains.toml`](../architecture/domains.toml) (and moves a plan
    domain's `status` when its work starts or ends), deletes the crate's line from
    `tools/cleanroom/variant_import_warn.txt` once the round left it no variant import (the
    trunk's check refuses a listed crate with none), and regenerates `docs/REPORT.md` and the
    README's status (`tools report --write`). It rebases the branch onto `origin/main`, re-gates
    it, and runs `tools/lane/squash.sh SUBJECT BRANCH` in the landing worktree: one commit with
    the branch's tree on `origin/main`, its subject the root's, its body each part under its role
    (spec, model, integration) with its own body indented under it, every `Role:` and
    `Co-Authored-By:` line kept once, at the end. The import's
    checks ran on the branch's own commits at step 7; the round's branch keeps them. The squashed
    commit lands through `land.sh`, a fast-forward.
12. **Queue** (root). Every gap goes to the observer, and every shape finding to the next brief.
    So does every item two blind reads name, a zero-cost one (a name, a wrong doc, an
    unreachable arm) included.
    The domain is done when its free run holds every row it runs in to the row's end or to a
    named mask, and the queue holds nothing that changes behaviour.

## Coordinator size assessment

After independent review (step 10), and before landing, the root runs the official counters
(`tools spec-units`, `tools size`) and checks the decode notes' Budget section. It records the
score, budget, fit and old-code baseline, and the round's delta for model, bridge and code moved
into test support separately. The counters, weights and k are unchanged (MODEL §1 decision 19).

The root records **cut** or **proceed**, with a rationale relating the behaviour scope, code
quality and independent findings to whether a cut would improve the model's fit, considering
its cost and risk. Exceeding the old-code guideline alone fails no gate and starts no cut.
Proceeding above it needs an explicit reason in the coordination log and landing report.
A cut uses concrete review findings under "The loop"; its brief carries those shapes, with all
size material withheld. It aims to improve the model, rather than shave a number.

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
| implementer | any step | `crates/clean/types` is shared: two implementers in flight at once each add only new items there, and the second import rebases on the first |
| blind reviewer | any step | none |
| integrator | integration or gate | both touch the engine and `crates/verify`: the second rebases on the first, and re-proves the gates |
| engineer | any step | its files lie outside every running round |
| root | none | it alone briefs, imports and lands; landing is one change at a time |

A domain's spec can start before a domain it reads from has landed. Its own round then waits
until that domain has landed, so the interface it reads is fixed.

## Working hours

Work runs in one session a night (Jim, 2026-11-11), from 20:00 to at most 08:00 local time, 10 to
12 hours with its planning and review included. Each session has three parts:

1. **Planning.** The root reads the last session's review, REPORT and the queue, chooses the
   session's rounds and lanes within the limits above, and writes the plan down before any lane
   starts, as an issue (MODEL §1 decision 27). Work goes only to the lowest rung of the ladder
   not yet reached (decisions 18 and 24): the items it still needs, then the free run's first
   divergences on its rows. A round or lane outside the rung's needs list doesn't start; the
   planted faults and the gain over null (decisions 24 and 25) run alongside.
2. **Implementation.** The loop below runs the plan: rounds and lanes start, land and report as
   they do today.
3. **Review.** No new lane starts in the last part of the session. The lanes in flight finish
   and land, or stop with a handoff. The root writes the session's review on the plan's issue:
   what landed, the figures before and after, what the gates found, and what the next session
   should take up.

Outside working hours no round runs (Jim, 2026-11-12). The time goes to the project's structure:
tooling, movie captures, optimisations, and long computations that don't fit a session, all within
the rung's needs (decision 24). Independent short tasks may run in parallel within the
three-lane limit, supervised by the root (Jim, 2026-11-13). Each task starts fresh, finishes
with a handoff and lands like any other change; the root reviews and lands one change at a time.

## The loop

- **Cut rounds.** The root applies MODEL §1 decision 19 after independent review and records
  whether the expected improvement warrants a round with no new behaviour. The brief carries
  concrete shape findings, without size numbers or targets. Findings that do not justify a cut
  accompany the domain's next behaviour round.

- A vector that won't pass, and that the spec doesn't explain, is a **gap**. The implementer
  records it, and the observer answers it in the spec. The model is never tuned to a record.
- A finding is fixed by its owner: the observer owns spec and vectors, the implementer the clean
  crates, the integrator the adapter and harness, the root briefs and landing.
- Old code and documents kept only for comparison are deleted. A comparison uses git history
  (MODEL §1 decision 8).
