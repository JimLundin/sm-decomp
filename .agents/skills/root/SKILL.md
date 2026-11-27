---
name: root
description: The sm-decomp root (coordinator) role. Run the clean-room round step by step, write briefs, export and launch the confined sessions, import, land and report to Jim. Load when you are the root.
---

# Run the rounds

You run [the round](../../../docs/cleanroom/PROCESS.md#a-round) one step at a time, and you are
the only role that talks to Jim, writes briefs and lands on `main`. You hold the design and the
barrier; every other role does one step.

## Each step

1. **Pick** the next step of the current round from [PROCESS](../../../docs/cleanroom/PROCESS.md)
   and the queue in the coordination log. Done when you can name its owner, its input (a branch
   and hash) and its done-condition.
2. **Brief** a fresh session for it (below). Done when the brief is saved under
   `~/.cleanroom/briefs/` or sent as the agent's prompt.
3. **Wait** for its report. Start other work only where [What can run at once](../../../docs/cleanroom/PROCESS.md#what-can-run-at-once) allows and within the limit Jim sets for the session.
4. **Check** the report against the step's done-condition: open the branch and run the cheap
   checks yourself (scan, doc links, the vector count, the gate line). Done when every claim you
   relay comes from output you saw. After independent review of a round or engineering task,
   complete [the Code Health review](../../../docs/cleanroom/PROCESS.md#coordinator-code-health-review).
5. **Relay** to Jim: what the step found, what you decided, and what comes next. Lead with the
   ladder (the lowest rung's integrated proof and what it still needs) and the indexed cases'
   results, with scoped and whole-movie claims kept separate; give size as one line of the round's delta, a diagnostic (MODEL §1
   decision 19), and cite `tools report` at the landed head once a change lands.
6. **Record** the step in the coordination log (`$TEMP/collab/reviews/coord-log.md`): heads,
   verdicts, and the queue so a fresh root can resume from the log alone. When another root takes over, point its [handoff](../handoff/SKILL.md)
   to that record and the first open action.

## Briefs

A brief is the whole of what a fresh session knows. Write it like this:
- **Goal**, the input branch and hash, and the skill to load.
- **Pointers** to the files worth reading, rather than retyping them.
- **The done-condition** of the step, and what to report.
- **What is already ruled**, so the session builds on it.
- **One step, a time limit and the push as the end** (outside the confined roles). A lane does one step
  of about 30 minutes: make the change, prove the narrowest test locally, push, start focused
  CI and hand back the run ids. The root watches CI in the background and briefs a fresh lane
  on a failure. Each brief states a 45-minute check-in, after which the lane hands back what it
  has: most of a longer lane's time goes to waiting on local replay tests (4–10 minutes a run).

For the two confined roles, the brief is also part of the barrier:
- **Spec words only.** Name spec entries and player-visible behaviour, and phrase findings as
  shape: duplication, a decision that could be a table row, a function doing two jobs, a name
  that misleads.
- **Shapes, not mechanisms.** A shape item names what the code should express, and leaves how
  to the implementer. Before writing one, sort each thing it names into a **value** or a
  **decision**: a table or a record holds values; a decision stays in the code, written once with
  its reason. An item that asks for a mechanism can leave only a costly encoding under the checks
  DESIGN lists. The one that recurs: asking for a keyed shape to be built in a constant. A
  table's lookup is a trait method, which can't be `const`, so a constant set leaves only a
  positional array or a macro per set. Ask that the value be held, not when it is built. A
  table the compiler checks is one of DESIGN's table forms (an exhaustive `match` from each
  member to its value, or `enum_map!` rows), so a brief may ask that a table name each member
  once and leave the form to the implementer. When a
  gate names the way out of a trap ("constant rows, each naming its member"), the brief carries
  that shape; a release alone ("need not be a constant") leaves the costly encoding open. Before
  launch, the brief's gate works through each item under the checks.
- **Coding standards and shape outcomes.** The brief follows MODEL §1 decisions 10 and 11:
  implementation and independent review stay blind to size measurements and verdicts.
- **Decide cuts after independent review** (PROCESS, "Cut rounds"; MODEL §1 decision 19).
  Keep size reports and decode Budget sections out of reviewer material. Count size after the
  reports and complete
  [the coordinator size assessment](../../../docs/cleanroom/PROCESS.md#coordinator-size-assessment)
  before landing; send shape outcomes only.
- **A cut round asks for the outcome.** Each shape finding says what the code should have less of
  ("one decision per thing she meets", "the air's case decided once"), never the device that
  would get there ("a value decided before", "an enum of its cases", "no snapshot"). A named
  device is followed even where it costs more than what it replaces: two cut rounds grew this
  way. The round's largest lever, when a gate names one, leads the brief.
- **The shape to reach, not only what to remove.** Each outcome names the shape the code ends in
  and where each answer then lives ("a new file's supplies are the table's default; Inventory
  derives Default"). An outcome stated only as a removal ("no type that only carries a borrowed
  pace", "no second way in") was read as removing what the shape needed, twice.
- **What two blind reads name goes in.** An item named by two blind reads goes into the
  domain's next brief as an outcome, a zero-cost one (a rename, a wrong doc, an unreachable arm)
  included: nothing else carries those. An item one read names is optional.
- **Interface a brief adds is named.** An outcome that adds interface (a constructor, a
  `Default`, a new public type) says so in the brief, and you check the next blind read for
  it: a trap the new interface made shows there first.
- **Scanned.** `tools/cleanroom/scan_brief.sh BRIEF` reports 0 before `launch.sh` takes it.

## The confined sessions

From a checkout of `main`:
1. `bash tools/cleanroom/canary.sh plant`.
2. `bash tools/cleanroom/export.sh DOMAIN`, adding `-c COMMIT` when the spec is an amendment
   still on its branch. It writes the domain under `$CLEANROOM_ROOT` (default `/cleanroom`), outside the home.
3. `bash tools/cleanroom/launch.sh ROOT/DOMAIN BRIEF TRANSCRIPT`, with the transcript
   under `~/.cleanroom/transcripts/`. Done on `LAUNCH OK` and `CANARY CLEAN`.
4. Read the session's report: the last `result` event of the transcript.
5. `bash tools/cleanroom/import.sh ROOT/DOMAIN lane/<domain>-rN`. Done on `IMPORT OK`:
   `code_check.sh` passes, `scan.sh` finds nothing new in the changed files, and both clippy runs
   pass over every clean crate. On `IMPORT CLIPPY FAIL`, commit a mechanical fix yourself, right
   after the import, as `[exact]` with `Role: root` and the lint it answers, so the squashed
   commit records it under your role.
   Send anything that changes an item's logic back to an implementer
   ([PROCESS](../../../docs/cleanroom/PROCESS.md#a-round), step 7).

The blind review (a domain's first round, or when the last round gate asked for one) uses the
same launch, on a code-only copy that
`bash tools/cleanroom/blind_copy.sh EXPORT DEST` makes from the round's export: the model crate,
its tests, the vector file and the [blind-reviewer](../blind-reviewer/SKILL.md) skill, without
the spec, the design notes, `CLEANROOM.md` or a test that reads them. Its brief is one line:
"Read .agents/skills/blind-reviewer/SKILL.md in this workspace with the Read tool and follow it;
it is your whole task." A confined session has no Skill tool, so every confined brief names its
skill file by path.

Confined sessions have no `cleanroom` function or other tool beyond the launch's allow-list: a
brief names the checks as CLEANROOM.md does (`cargo check --all-targets`,
`cargo clippy --all-targets -- -D warnings`, `cargo fmt --check`, with `-p <crate> -p types` to
narrow a run) and the exact vector-runner command `launch.sh` appends. Before `launch.sh`, put
the toolchain's bin directory on `PATH` (`export.sh` runs cargo) and run `scan_brief.sh`; a
`name` hit is usually an ordinary phrase that matches the lexicon, which rewording clears.

## Lane closeout

Send every final lane handoff to the [cleanup agent](../../../docs/cleanroom/PROCESS.md#cleanup-at-lane-end), including work that ends before a PR lands. Check the receipt for removed remote and local refs and worktrees. Carry retained items to the named consumer.

## Landing

Before landing a round, complete [PROCESS step 11](../../../docs/cleanroom/PROCESS.md#a-round):
update its domain registration and regenerate the README after review. Land by fast-forward
through `tools/lane/land.sh` only after CI is green: Quality, content and replay.
The commands below apply to rounds and other work.

One landing PR may carry several independently reviewed changes (Jim, 2026-11-25), each kept as
its own commits, in an order where each applies onto the one before, sharing the PR's one CI run.
A batch holds at most one round, squashed first (squash.sh needs the round's branch to hold the
landing branch's head); other work is picked after it. When that run fails:

- Find the failing change in its log. The changes before it are the green prefix; when it is the
  first, the prefix is empty and nothing lands.
- Push the prefix as a new landing branch with its own PR (never force-push the old one), whose
  body lists only its changes, and run `watch.sh` on that PR with the prefix's sha: it lands once
  the prefix's own CI run passes.
- Send the failing change back to its lane. The changes after it haven't passed CI without it:
  re-batch them onto the landed prefix in a later landing, on their own CI run.
- A cancelled run names no failing change: re-run it.

1. In the landing worktree, `git status` shows no rebase or cherry-pick in progress.
2. A round: rebase its branch onto `origin/main` and re-gate it, then, with the landing branch at
   `origin/main`, `bash tools/lane/squash.sh "<subject> [exact]" <branch>`, which makes the one
   commit PROCESS step 11 describes (a lane of one commit keeps that commit's body as its own)
   and refuses a branch that doesn't hold `HEAD`.
   Other work: `bash tools/lane/pick.sh <hash>...` in the lane's order; for a batch, each change's
   commits in turn, a round's squash made first. It stops at a conflict for you to resolve by
   hand, reading every hunk, then `git cherry-pick --continue`.
3. `bash tools/lane/gate.sh`, then push the landing branch and open a PR to `main` whose body gives
   each change's commits, its review, evidence and ratchet deltas. When a gate on this machine
   already passed the same tree over the same base with the same tools (the rebased round branch,
   or a lane based on `origin/main`), it prints `GATE REUSED` naming that pass and runs nothing,
   and the landing gate may rest on that recorded pass, a lane's included (Jim, 2026-11-25); the
   PR's CI still runs every check. `SM_GATE_FRESH=1` runs the gate anyway.
4. In the background: `bash tools/lane/watch.sh <pr> <sha>`, with the sha you pushed: it lands only
   that head, so a push GitHub hasn't shown yet never lands the old head's green run. Exit 0
   landed (via `land.sh`, fast-forward only); 3 main moved (rebase, re-gate, re-push); 4 a run
   failed or was cancelled (read its log, land the green prefix as above, hand the failure to a
   lane);
   5 timed out; 6 timed out with the PR's head still not that sha. Never loop on `land.sh` by
   hand: its exit 2 (not ready) never ends on a red run, which is its exit 4.
5. Bundles: check the lane's SHA-256 with `sha256sum`, `timeout 600 gh release upload ci-data FILE`,
   confirm its size with `gh release view ci-data`, and commit the new pin. Only encrypted bundles.
6. When the watcher reports a landing, hand its branches and worktrees to the session's
   [cleanup agent](../../../docs/cleanroom/PROCESS.md#cleanup-after-landing). Check its receipt
   in the coordination log. The root retires finished clean-room exports. A spec or import
   branch can also be retired when the successor holds its patches and its lane has released it.

## Jim

- Jim sets goals and process. Bring him only goal questions, each with its full context and your
  recommendation. Engineering choices are yours.
- A decision of Jim's goes into MODEL §1 or PROCESS in the same session, and into memory.
