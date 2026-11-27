# The clean-room process

How one domain of the game goes from the original to a model on `main`. Each domain is rebuilt in
**rounds**: a spec, a model built from that spec alone, and proof that the model matches the real
game. This file is the single source of truth for the order of a round, who does each step, and
who sees what. The rules it relies on live elsewhere, and are linked where they are used:
- [MODEL §5](../architecture/MODEL.md#5-how-the-work-is-planned): what a leak is, and the spec
  template;
- [DESIGN](DESIGN.md): the coding standards;
- [MODEL §4](../architecture/MODEL.md#4-size-the-score-its-gates-and-its-lints) and §1
  decisions 7 and 19: the score and the budget, a guideline;
  [size-budget](../architecture/size-budget.md), how spec units are counted.

## Replay transition

[MODEL §1 decisions 32 and 33](../architecture/MODEL.md#1-the-goal-and-what-is-exact) are the
source of truth for indexed production replay and removal policy. The one-time replacement
of previous acceptance definitions is recorded in `scenarios/replay/transition.toml`, with
exact prior claims in `scenarios/replay/prior-claims.toml`.

A replacement of production or acceptance requires a bounded work plan and independent review of:

- the inventory of previous gameplay owners, their state and fallback routes, and the reviewed
  content, codec, exact encoding, scheduling and rendering services to retain;
- the indexed replay cases and their behaviour witnesses, declared scopes and dependency
  contracts, showing that scoped replay calls the same production model and adapter entries
  as integrated replay and carries owned State without oracle next-state feedback;
- the old-code-dependent results to retire, the independent records and witnesses to preserve,
  and replacement coverage for each retired claim, including cross-domain contracts;
- the proposed cutover of acceptance checks and ratchets, with explicit before/after claims and
  any coverage loss identified for review rather than hidden by deleting a check.

Current indexed requirements remain in force until an explicit replacement is reviewed and
landed. Its transition records which claims remain, which are replaced or retired and why,
and the measured replacement proof. Scoped cases
prove only their stated scope; full-engine and milestone claims still need integrated replay.
Vectors, unit tests and localisation checks remain useful development evidence. Required
missing behaviour is reported as `Unsupported`, so removing an old owner cannot silently turn
its responsibility into a successful no-op.

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
| integrator | wires the model into the engine, wires its indexed replay witnesses, deletes the old code | [integrator](../../.agents/skills/integrator/SKILL.md) |
| engineer | changes tools, CI or docs outside a round, or reviews an assigned architecture question | [engineering task routes](../../AGENTS.md#start-by-task) |

Every role except the confined two (implementer, blind reviewer) also follows the
[lane](../../.agents/skills/lane/SKILL.md) skill for its worktree, commits, proof and report.

Use `gpt-6.2-sol` for lane sub-agents unless Jim selects another model for the task.

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
7. **Import** (root): `import.sh` first runs `code_check.sh --import`, which refuses a variant
   import (L4) even in a crate whose older ones still only warn on the trunk. Then it runs
   `scan.sh` on each file the commits add or change, and refuses a hit that file didn't hold at
   the export's start; the trunk's own hits don't block. It runs the mirror scan
   (`scan.sh --mirror`) on the model against the spec and decode notes on the lane branch, and
   refuses a non-private function the export's start didn't have that opens with an entry whose
   source is one routine. It applies the commits to a lane
   branch, takes the branch back if the export's crates break the capability rule (clippy's
   forbidden lints), then runs both clippy configurations of `tools/lane/gate.sh` over every clean
   crate in the workspace: an export holds one domain's model and `types`, so a change to
   `types` can break a crate its implementer never saw. Done when the import passes. A
   refusal goes back to the same implementer's export as a short follow-up session: the brief
   quotes each finding (file, line, the rule it breaks) in the coding standards' words, and the
   import runs again. A scan hit that is a word a player uses may instead become a reviewed
   entry in `tools/cleanroom/scan_allow.txt`, with its reason, before the import runs again.
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
     engine's (MODEL §1 decision 21). Complete canonical model pins have a shape check
     (decision 29); unavailable durable codecs are explicit, never partial saved state;
   - indexed cases compare the actual production entry with independent records, with carried
     State, declared observations and inputs, null evidence and available complete continuations;
   - the old code it replaces is deleted;
   - `tools/lane/gate.sh` passes;
   - the complete reviewed replay baseline holds, including exact expected Unsupported boundaries.

   Missing neighbours can be independently observed inputs of a scoped production case, with
   their ownership and timing explicitly unproved. Required missing integrated producers remain
   Unsupported and are named in the domain's gaps. Previous gameplay never supplies them.
10. **Round gate** (adversary), in two parts so that review stays off the critical path. The spec-and-model part starts at step 7's import, alongside
    integration: the spec delta, the model's leaks and shape, and the blind review when it ran.
    The integration part reads the landing PR's diff while its CI runs. A finding that only
    corrects wording (stale docs, a gaps line, a ledger record's description, a missing limit) is
    fixed by the root on the landing branch and re-read by the adversary in the same pass; a
    finding that changes what a spec rule says, a vector, the model or a replay outcome goes to
    its owner. The two parts together read the leaks, the spec delta
    when step 3 didn't gate it, replay provider independence, carried production State, case
    outcomes and first differences against `main`, and the blind review. It says whether the next round needs a
    blind review. Done on a verdict of MERGE. Findings go to the integrator (replay provider, adapter),
    or into the next round's queue (model shape, spec gaps). Independent review excludes size
    measurements. After the reports, the root completes the coordinator size assessment and
    [Code Health review](#coordinator-code-health-review) below.
    Size does not change the adversary's verdict; every non-size gate remains mandatory.
11. **Land** (root), as **one commit**: the spec, the import and the integration of the round
    together. On the round's branch the root adds one to the model's `rounds`
    in [`docs/architecture/domains.toml`](../architecture/domains.toml) (and moves a plan
    domain's `status` when its work starts or ends), deletes the crate's line from
    `tools/cleanroom/variant_import_warn.txt` once the round left it no variant import (the
    trunk's check refuses a listed crate with none), and regenerates the README
    (`tools report --write`); the integration already carries `played.toml` and the README for
    any played run it moved, and the root reviews those figures. It rebases the branch onto `origin/main`, re-gates
    it, and runs `tools/lane/squash.sh SUBJECT BRANCH` in the landing worktree: one commit with
    the branch's tree on `origin/main`, its subject the root's, its body each part under its role
    (spec, model, integration) with its own body indented under it, every `Role:` and
    `Co-Authored-By:` line kept once, at the end. The import's
    checks ran on the branch's own commits at step 7; the round's branch keeps them. The squashed
    commit lands through `land.sh`, a fast-forward. Quality and verification run independently;
    both must pass before landing.
12. **Queue** (root). Every gap goes to the observer, and every shape finding to the next brief.
    So does every item two blind reads name, a zero-cost one (a name, a wrong doc, an
    unreachable arm) included.
    The domain is done when its required indexed witnesses reach their declared ends, its
    required integration is proven, and the queue holds nothing that changes behaviour.

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

## Coordinator Code Health review

After independent source review, the root runs the installed CodeScene CLI on the supported
source files changed by the round or engineering task, including new modules and tests. Compare
the reviewed commit with its fixed base; for moves and extractions, account for both the old
file and every replacement so a split cannot hide a finding. Review clean models, adapters,
engine code and development tools separately. Vendor code and deliberate checker fixtures are
reported as exclusions, not improvement targets.

Use `cs version` to record the analyzer version and `cs review --help` for its supported input
forms. `cs review COMMIT:./path/to/file.rs` measures a committed file; `cs review --file-list
PATHS --output REPORT` writes a human-readable report for a newline-separated path list.
Keep reports outside the repository and confined exports. Machine-readable null scores mean
"no scorable code" and are excluded from averages.

The step is complete when the coordination log and landing/session review record:

- the base and reviewed hashes, CLI version, command, exit status and file coverage, with
  unsupported, unscorable, deleted and excluded files identified;
- each changed file's before/after score and significant findings, with new/deleted files
  shown explicitly; any aggregate names its population and weighting;
- a disposition for each material finding: addressed and rechecked, queued as a concrete
  shape outcome, or retained with the behaviour or contract that justifies it.

Code Health is maintainability feedback, not proof of exactness. Decisions retain the source
review's judgement and the existing mandatory gates. If the CLI is unavailable, authentication
is missing or analysis fails, record the missing measurement and its cause explicitly; a
missing result is never a clean result. The coordinator keeps metric reports and targets out
of implementer and independent-reviewer material, and relays only concrete shape findings.

## What can run at once

**A manageable amount.** The root runs as many lanes at
once as the machine and CI hold: disk for each building lane's own `target/` (about 5 GB) with
room to spare, and GitHub's limit of 20 concurrent jobs. At most one round is in flight per
model, so a domain with two models may have one round on each. The root names each lane to Jim
when it starts, and reports its result when it ends. Within that limit, the dependencies below
decide what may overlap.

**A backlog item is always in flight** (Jim, 2026-11-24). Beside the rounds, at least one lane
works a `status:backlog` work issue until none is left, and the items that shorten a round's
cycle time go first: the time from an issue's opening to its landing, most of which is fix loops
after reviews and waiting on a shared file, not CI. A standing backlog lane on workflow cleanup
and landing time runs beside the rounds at all times (Jim, 2026-11-25): it works one item at a
time, each on its own branch, and the root starts its next item as soon as one lands. Each item
lands with independent review, like other work.

**Parallel lanes don't share files.** A rebase after main has moved costs a
rebuild and a re-proof on this machine, which can wipe out the time running in parallel saved.
So before a lane starts, its brief lists the files it will change, and lanes running at once
change disjoint files. A file that most work passes through belongs to one lane at a time:
`tools/mutants.toml` and STATUS. Other lanes don't edit them; they hand their entries (a planted
fault, a STATUS paragraph) to the root, who adds them when the lane lands. Each owner's part of
the played loop is its own file in `production/played/` and `replay/played/`, and belongs to the
lane that integrates that owner. The two loop files, the order of owners in a frame
(`crates/game/src/production/played.rs`) and the order its runner compares them in
(`crates/verify/src/replay/played.rs`), take a few lines from each owner's lane: its State field,
its faults, and its call in the order. So does the played tests' shared fixture
(`production/played/tests.rs`), with the field's starting value; each owner's tests are in its
own module. The root settles the order of those lines at landing.
When two pieces of work need the same file, they run one after the other in a
single lane, or the second starts from the first's branch and lands after it. The lane with the
most work in flight lands first, and shorter lanes start from it. The generated README is
regenerated at landing, never merged by hand.

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
| root | none | it alone briefs, imports and lands; one landing at a time, of one change or a batch of reviewed ones |

A domain's spec can start before a domain it reads from has landed. Its own round then waits
until that domain has landed, so the interface it reads is fixed.

## Work items

The milestone issue is the umbrella for its measured goal and remaining dependency order.
Before starting a lane or round, the root creates or selects a linked issue for one bounded
outcome. Its title says what becomes possible, rather than naming only a domain, round number
or role. A research or contract issue may finish with reviewed evidence; an implementation
issue finishes with the resulting behavior and its proof. The issue states which kind it is.

Each work issue contains:

- **Outcome and scope:** the concrete change, its current boundary and what remains outside it.
- **Owner and stage:** the responsible role, the active branch or PR when one exists, and the
  next action. At transfer the root updates these fields from the repository
  [handoff](../../.agents/skills/handoff/SKILL.md).
- **Dependencies:** links to prerequisite issues and the specific input or decision each must
  supply. A missing contract becomes a linked issue before the lane broadens its scope.
- **Done when:** observable acceptance conditions and the required checks. State the expected
  change in the relevant boundary or comparison; if the work supplies a prerequisite without
  increasing played frames, say so.
- **Evidence:** the landed commit and PR for code, the reviewed result for research, applicable
  test/CI receipts, and actual versus intended results, including unchanged figures. For a
  delivered feature, link its [feature architecture review](#feature-architecture-review).

Open work uses one current-state label: `status:backlog` for work awaiting scope or priority,
`status:ready` when its prerequisites and acceptance conditions are clear, `status:in-progress`
while an owner works on it, `status:blocked` with the exact blocker and unblock condition, or
`status:in-review` while its result awaits required review, CI or landing. Issue bodies carry
the detail; labels do not replace it. Update at ownership, scope or state changes rather than
posting each tool action. Close completed issues only when their acceptance evidence holds;
remove their open-state label. Close superseded work as not planned with a replacement link,
not as a completed outcome.

The milestone issue links the child issues in dependency order and distinguishes completed
prerequisites, active work, ready work and blockers. Native sub-issue links may supplement the
written dependency list. Keep the milestone's actual played-frame and stream figures visible:
the fraction of closed issues is not the fraction of the game reproduced. Session issues
select work IDs, record owner/stage and intended outcomes, and link the final results. A
missed outcome names its cause and next work item; it does not silently reset the same goal.

### Cleanup at lane end

The root assigns one GPT-6.2 Sol cleanup agent for the session. Every lane ends with a
cleanup handoff, including lanes that hand work to a successor or stop without a PR.
List the lane's local and remote branch names and tips, worktree paths and build output.
Release each item as soon as its last user is done. For anything still needed, name the
job, PR or successor that uses it. A successor takes responsibility for releasing what it inherits.

The root forwards each handoff to the cleanup agent. The agent removes released worktrees
and build outputs even when review still needs the published branch. Before deleting a
branch, check open PRs, stacked work and the current tip on the remote. For a superseded
or abandoned branch with unmerged commits, verify a recovery ref or archive first.
Keep the default checkout and shared build seed. Leave local edits, untracked work and
unexpected ignored files with their owner.

Immediately before running `tools/lane/worktree.sh remove`, check the released checkout's
branch, tip and status, including untracked and ignored files. Confirm no live job or
successor uses that path. The helper removes the checkout and its build output.
Before deleting a local ref, check `git worktree list --porcelain` again. Keep the ref
if any registered worktree has it checked out. Delete a released local ref with
`git update-ref -d <ref> <expected-tip>` and its remote counterpart with
`git push --force-with-lease=<ref>:<expected-tip> origin :<ref>`. Verify removal against
`git worktree list`, local refs and `git ls-remote --heads origin`. Record removals and
remaining consumers in the coordination log. Revisit retained items when their job or
successor finishes. At session end, reconcile the live remote branch list with those receipts.

### Cleanup after landing

The root sends the cleanup agent the PR, landed commit, source and landing branch tips,
and any remaining worktrees. Fetch the default branch and verify that it contains the
landed commit. For a squash or cherry-pick, match the reviewed source tree or patch to
what landed. Finish the [lane cleanup](#cleanup-at-lane-end) for every released item.
Other lanes can keep working while cleanup runs.

## Feature architecture review

At the end of every feature, after delivery and any planned trial, the root reviews the
feature before closing its work issue. For a feature delivered across several rounds or PRs,
use the issue that owns the complete feature. Pin the delivered commit and acceptance
receipts. Scope the review to the feature, its callers and the responsibilities it touches.

Use the [project architecture review](../../.agents/skills/architecture-review/SKILL.md) to
trace the feature from a real caller through its owners and its proof. Give a fresh reviewer
the same revision, scope and evidence under [Who sees what](#who-sees-what). Have them run
Matt Pocock's `improve-codebase-architecture` skill before sharing the project findings.

The root compares the two reports on the feature issue. Cite where they agree, what either
pass added, and how the source or trial resolves any disagreement. Record a disposition for
each material finding, with an owner and a distinguishing check for follow-up work. When
both passes recommend keeping the existing design, record the reasoning and close the review.

## Working hours

Work is scheduled in one session a night, from 20:00 to 08:00 local time, with its planning and
review included. Each session has three parts:

1. **Planning.** The root reads the last session's review, `tools report` and the queue, chooses
   the session's rounds and lanes within the limits above, and writes the plan down before any lane
   starts, as a session issue selecting the bounded [work items](#work-items) (MODEL §1
   decisions 27 and 31). Work goes only to the lowest rung of the ladder
   not yet reached (decisions 18 and 24): the items it still needs, then the indexed replay cases' first
   differences on its rows. A round or lane outside the rung's needs list doesn't start; the
   planted faults and the gain over null (decisions 24 and 25) run alongside.
2. **Implementation.** Rounds and lanes run the plan, under the loop below.
3. **Review.** New scope stops in the last part of the session. The root completes the work
   already in flight under the rule below and writes the session's review on the plan's issue:
   what landed, the figures before and after, what the gates found, and what the next session
   should take up, including the [Code Health review](#coordinator-code-health-review) of the
   session's changes, and the behaviour share of its commit lines (MODEL §1 decision 36): the
   lines added and removed under `crates/clean/` and the played loop
   (`crates/game/src/production/played*`), against all other lines under `crates/` and `tools/`,
   from `git diff --numstat` over the session's range. Update each selected work issue with its actual outcome, evidence and
   remaining dependency; the session review links those issues.

**Finish active work.** The scheduled shift end stops new scope, not completion. Every task or
round already started continues beyond scheduled hours through necessary implementation and
corrections, required reviews and checks, passing CI, landing, reporting and cleanup. Later role
sessions needed to finish that round may start; a new round or unrelated follow-up waits for its
next permitted start. The existing lane limits, clean-room roles, mandatory proof and root-only
landing still apply. An explicit stop or pause from Jim takes precedence. A real external blocker
is reported with the unfinished work and concrete next step in a handoff; the work stays pending
and resumes when unblocked. Shift end alone is never a reason to leave work pending.
At session end the cleanup agent reports the remaining branches and worktrees as described
under [Cleanup at lane end](#cleanup-at-lane-end).

Outside working hours new rounds wait for the nightly session. New work goes to the project's
structure: tooling, movie captures, optimisations, and long computations that don't fit a session, within
the rung's needs (decision 24). Independent short tasks may run in parallel within the
lane limits above, supervised by the root (decision 28). Each task starts fresh, finishes
with a handoff and lands like any other change: reviewed, then landed by the root alone or in a
batch with other reviewed changes ([landing](../../.agents/skills/root/SKILL.md#landing)).

## The loop

- **Cut rounds.** The root applies MODEL §1 decision 19 after independent review and records
  whether the expected improvement warrants a round with no new behaviour. The brief carries
  concrete shape findings, without size numbers or targets. Findings that do not justify a cut
  accompany the domain's next behaviour round.

- A vector that won't pass, and that the spec doesn't explain, is a **gap**. The implementer
  records it, and the observer answers it in the spec. The model is never tuned to a record.
- A finding is fixed by its owner: the observer owns spec and vectors, the implementer the clean
  crates, the integrator the adapter and replay provider, the root briefs and landing.
