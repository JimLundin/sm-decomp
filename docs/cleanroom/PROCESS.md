# The clean-room process

How one domain of the game goes from the original to a model on `main`. Each domain is rebuilt in
**rounds**: a spec, a model built from that spec alone, and proof that the model matches the real
game. This file is the single source of truth for the order of a round, who does each step, and
who sees what. The rules it relies on live elsewhere, and are linked where they are used:
- [MODEL §5](../architecture/MODEL.md#5-how-the-work-is-planned): what a leak is, and the spec
  template;
- [DESIGN](DESIGN.md): the coding standards;
- [MODEL §4](../architecture/MODEL.md#4-gates-and-lints): the gates and
  lints the clean crates are held to.

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
(`cargo cleanroom export`, then `cargo cleanroom run`) that holds only their column. Every other role works in the
repository.

| Material | observer | adversary | implementer | blind reviewer | integrator | root |
| --- | :-: | :-: | :-: | :-: | :-: | :-: |
| the original (bank logs, ROM, records, decode notes, `verify`'s addresses) | yes | yes | - | - | yes | yes |
| the spec and its vectors | yes | yes | yes | vectors only | yes | yes |
| the clean model's code and tests | after import | yes | writes it | yes | yes | yes |
| the old engine code | yes | yes | - | - | deletes it | yes |

Blind reviewers receive their code-only export. The implementer's material states coding
standards only, and feedback carries concrete shape findings from review.

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
     `tools/lane/gate.sh` holds every domain whose spec or vectors a branch changes to this
     rule, through `tools/cleanroom/adjusted_vectors.sh --gate`;
   - `docs/behaviours/<domain>.decode.md` holds the sources and the harness note;
   - the observer's reference model agrees with every vector;
   - `tools/cleanroom/scan.sh` finds 0 on the spec and vectors;
   - the doc links pass.
2. **Brief** (root), after 1. Done when the brief:
   - names only spec entries and shape findings;
   - passes the name scan, which `cargo cleanroom export` and `cargo cleanroom run` run before
     they take a brief.
3. **Spec gate** (adversary), after 1 and 2, on a domain's first spec and on a spec delta that
   adds a mechanism (a new table, interpreter or kind of state) or crosses into another domain
   (MODEL §1 decision 20). It gates the spec and the brief together. Done on a verdict of
   LAUNCH. LAUNCH AFTER FIXES sends numbered findings to their owners (the observer for the
   spec, the root for the brief), then a re-gate of the delta. Any other amendment skips this
   step: its scans still pass before the launch, and the round gate (step 10) reviews the spec
   delta with the code.
4. **Hold the spec** (root). A round lands whole (step 11), so the spec stays on the
   round's branch, a new domain's first spec as much as an amendment, and the round exports from
   that branch's commit. An amendment couldn't land alone anyway: it changes what the landed
   model must do, so the model's own vector test fails on it until the round implements it.
5. **Export and launch** (root): `cargo cleanroom export --commit COMMIT DOMAIN BRIEF` with the
   spec's commit on the round's branch, then the pre-round check `cargo cleanroom run --probe
   DOMAIN`, then `cargo cleanroom run DOMAIN`. The probe runs the box on the export with a
   command that writes there and tries to read the checkout, the lane folder and the home, then
   prints the mounted CLI's version. It refuses a CLI other than the one the last passing probe
   recorded beside the lane folders, and `run` refuses to start without such a probe. On a new
   CLI, read what it changes in the flags `run` passes, delete the record and probe again.
   Done on `RUN OK`. A new
   domain is named `<area>-<name>`, its area from `domains.toml`, and its first export creates its
   crate at `crates/clean/<area>_<name>`.
6. **Implement** (implementer, confined). Done when every vector passes or is listed as a gap,
   and `cleanroom test` passes: the code rules, the name scan on the changed files, fmt, clippy
   and the tests. The lexicon stays outside the box: `cleanroom test` sends the changed files to
   the relay, which runs the name scan on the host and answers each file's hit count and line
   numbers, never the words. The relay answers 100 scans a session and logs each, with the lines
   that hit, in the lane folder's `scan.log`. A session can still test words of its choosing
   against the lexicon within those 100 calls, so the root reads `scan.log` after the session
   for lines that try words rather than model code.
7. **Import** (root): `cargo cleanroom import DOMAIN` on the lane branch. It refuses an
   export whose `.git` is not the folder the export made, whose config differs from the
   exported bytes, whose info or hooks is not empty, or which holds anything but objects, refs,
   HEAD, the index, logs, the last commit message and ORIG_HEAD beside them, a history that doesn't
   start at the export's commit, and a commit that touches anything but the
   manifest and the Rust files under `src/` and `tests/` of the shared types and the model (an
   attributes file, a build script, cargo configuration or a hidden file among them). It then
   reruns the code rules on the export's head, each file read as git stores it, the name scan
   on each file the commits add or change (a hit that file didn't hold at the export's start is
   refused; the trunk's own hits don't block), and the mirror scan
   on the model against the spec and decode notes on the lane branch (a non-private function the
   export's start didn't have that opens with an entry whose source is one routine is refused).
   It applies the commits to the lane branch, takes the branch back if the export's crates break
   the capability rule (clippy's forbidden lints), then lints every clean crate in the workspace:
   an export holds one domain's model and `types`, so a change to `types` can break a crate its
   implementer never saw. Done on `IMPORT OK`. A refusal goes back to the same implementer's
   export as a short follow-up session (`cargo cleanroom run DOMAIN BRIEF`): the brief quotes
   each finding (file, line, the rule it breaks) in the coding standards' words, and the import
   runs again. A scan hit that is a word a player uses may instead become a reviewed entry in
   `tools/cleanroom/scan_allow.txt`, with its reason, before the import runs again.
   A clippy failure names the crates and leaves the patches applied:
   - a **mechanical** fix (a parameter by value instead of by reference, a derive, `assert_eq!`
     for `assert!(a == b)`, an import path; no item's logic changed) is the root's own commit,
     right after the import, titled `[exact]` and naming the lint it answers. The round gate
     checks it;
   - anything else goes back to an implementer: this domain's when its `types` change
     wasn't needed, the broken crate's next round when it was;
   - an integrator never edits a clean crate. A fix it finds it needs is a finding for the root.
8. **Blind review** (blind reviewer, confined), after 7, on a domain's first round and when the
   last round gate asked for one (MODEL §1 decision 20). The root makes a code-only export (the
   model, its tests and the vector file) of the lane branch with
   `cargo cleanroom export --review --commit BRANCH DOMAIN BRIEF` and runs the review on it with
   `cargo cleanroom run DOMAIN-review`. Done on its report. The root compares the review's "what the code
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
    or into the next round's queue (model shape, spec gaps).
11. **Land** (root): the spec, the import and the integration of the round together, each
    as its own commits. On the round's branch the root moves a plan domain's `status` in
    [`docs/architecture/domains.toml`](../architecture/domains.toml) when its work starts or
    ends, and regenerates the README
    (`tools report --write`); the integration already carries `played.toml` and the README for
    any played run it moved, and the root reviews those figures. The round then lands like any
    other branch ([landing](../../.agents/skills/root/SKILL.md#landing)). Quality and
    verification run independently; both must pass before landing.
12. **Queue** (root). Every gap goes to the observer, and every shape finding to the next brief.
    So does every item two blind reads name, a zero-cost one (a name, a wrong doc, an
    unreachable arm) included.
    The domain is done when its required indexed witnesses reach their declared ends, its
    required integration is proven, and the queue holds nothing that changes behaviour.

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
fault, a STATUS paragraph) to the root, who adds them when the lane lands. Put findings specific
to the [title, file select and options](../replay-findings/title-file-select-and-options.md)
or the [flight to Ceres](../replay-findings/flight-to-ceres.md) on the corresponding page as
part of the owner's lane.

Each owner's part of the played loop is its own file in `production/played/` and `replay/played/`,
and belongs to the lane that integrates that owner. The two loop files, the order of owners in a frame
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

1. **Planning.** The root reads the last session's review, the README's progress and the queue,
   chooses the session's rounds and lanes within the limits above, and writes the plan down before any lane
   starts, as a session issue selecting the bounded [work items](#work-items) (MODEL §1
   decisions 27 and 31). Work goes only to the lowest rung of the ladder
   not yet reached (decisions 18 and 24): the items it still needs, then the indexed replay cases' first
   differences on its rows. A round or lane outside the rung's needs list doesn't start; the
   planted faults and the gain over null (decisions 24 and 25) run alongside. A planted fault
   is killed when its case reports any difference or data failure or the replay crashes, and
   fails the check only when its case still passes. Its manifest entry names the case, never the
   cut or field where it is caught, so a fault that moves to another cut needs no change
   ([STATUS](../STATUS.md#the-models-gain-over-the-null-model)).
2. **Implementation.** Rounds and lanes run the plan, under the loop below.
3. **Review.** New scope stops in the last part of the session. The root completes the work
   already in flight under the rule below and writes the session's review on the plan's issue:
   what landed, the figures before and after, what the gates found, and what the next session
   should take up. Report the behaviour share of the session's commit lines (MODEL §1 decision 36).
   Count the lines added and removed under `crates/clean/` and the played loop
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

Outside working hours new rounds wait for the nightly session. New work goes to the project's
structure: tooling, movie captures, optimisations, and long computations that don't fit a session, within
the rung's needs (decision 24). Independent short tasks may run in parallel within the
lane limits above, supervised by the root (decision 28). Each task starts fresh, finishes
with a handoff and lands like any other change: reviewed, then landed by the root alone or in a
batch with other reviewed changes ([landing](../../.agents/skills/root/SKILL.md#landing)).

## The loop

- **Cut rounds.** The root applies MODEL §1 decision 19 after independent review and records
  whether the expected improvement warrants a round with no new behaviour. The brief carries
  concrete shape findings. Findings that do not justify a cut
  accompany the domain's next behaviour round.

- A vector that won't pass, and that the spec doesn't explain, is a **gap**. The implementer
  records it, and the observer answers it in the spec. The model is never tuned to a record.
- A finding is fixed by its owner: the observer owns spec and vectors, the implementer the clean
  crates, the integrator the adapter and replay provider, the root briefs and landing.
