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
   relay comes from output you saw.
5. **Relay** to Jim: what the step found, what you decided, and what comes next. Lead with the
   ladder (the lowest rung's integrated proof and what it still needs) and the indexed cases'
   results, with scoped and whole-movie claims kept separate.
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
- **Decide cuts after independent review** (PROCESS, "Cut rounds"; MODEL §1 decision 19).
  Send shape outcomes only.
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

## The confined sessions

From a checkout of the round's branch, with the toolchain's bin directory on `PATH`:
1. `cargo cleanroom export [--commit COMMIT] DOMAIN BRIEF`, adding `--commit` when the spec is an
   amendment still on its branch. It scans the brief, writes the export to `ROOT/DOMAIN` and the
   brief to `ROOT/DOMAIN.lane/`, ROOT being `$CLEANROOM_ROOT` (default `/cleanroom`), outside the
   home. Done on `EXPORT OK`.
2. `cargo cleanroom run DOMAIN`. Done on `RUN OK`.
3. Read the session's report: the last `result` event of `ROOT/DOMAIN.lane/transcript.jsonl`.
4. `cargo cleanroom import DOMAIN` on `lane/<domain>-rN`. Done on `IMPORT OK`: the code rules
   pass, the name scan finds nothing new in the changed files, and Clippy passes over every clean
   crate. On `IMPORT CLIPPY FAIL`, commit a mechanical fix yourself, right after the import, as
   `[exact]` naming the lint it answers. Send anything that changes an item's logic back to an
   implementer ([PROCESS](../../../docs/cleanroom/PROCESS.md#a-round), step 7), as a follow-up
   session on the same export: `cargo cleanroom run DOMAIN BRIEF`.

The blind review (a domain's first round, or when the last round gate asked for one) runs on a
code-only export of the lane branch after the import:
`cargo cleanroom export --review --commit lane/<domain>-rN DOMAIN BRIEF`, then
`cargo cleanroom run DOMAIN-review`. It holds the model crate, its tests, the vector file and the
[blind-reviewer](../blind-reviewer/SKILL.md) skill, without the spec, the design notes,
`CLEANROOM.md` or a test that reads them. Its brief is one line: "Read
.agents/skills/blind-reviewer/SKILL.md in this workspace with the Read tool and follow it; it is
your whole task." A confined session has no Skill tool, so every confined brief names its skill
file by path.

A confined session has the export, the toolchain and `cleanroom test`, and nothing else: no
network but the model's API. The implementer checks with `cleanroom test`, which `CLEANROOM.md`
names. When the export or a run refuses a brief on a `name` hit, that is usually an ordinary
phrase that matches the lexicon, which rewording clears.

## Landing

Before landing a round, complete [PROCESS step 11](../../../docs/cleanroom/PROCESS.md#a-round):
update its domain registration and regenerate the README after review. A lane's bundle goes up
before its landing: check its SHA-256 with `sha256sum`, `timeout 600 gh release upload ci-data
FILE`, confirm its size with `gh release view ci-data`, and commit the new pin. Only encrypted
bundles.

One landing PR may carry several independently reviewed changes (Jim, 2026-11-25), each kept as
its own commits, in an order where each applies onto the one before, sharing the PR's one CI run.
A batch holds at most one round.

Land in the background, from the main checkout, with
`python3 tools/lane/land.py NAME LANE... --title TITLE --body-file BODY`, the lanes in landing
order and the body giving each change's commits, its review, evidence and ratchet deltas. It
lands on `main` only by fast-forward, onto a head whose CI passed: Quality, content and replay.

1. It picks each lane's commits that `origin/main` lacks onto `land/NAME`, in a worktree at
   `$TEMP/collab/land-NAME`. At a conflict it stops with exit 2. Resolve the conflict there,
   reading every hunk, run `git cherry-pick --continue`, then the same command again. It skips
   a commit that `main` already has, as the same patch, under the same author, date and subject,
   or as a pick that changes nothing.
2. It runs `tools/lane/gate.sh` there. A gate that already passed the same tree over the same
   base with the same tools, a lane's included, prints `GATE REUSED` and runs nothing, and the
   landing may rest on that pass (Jim, 2026-11-25). The PR's CI still runs every check.
3. It pushes `land/NAME`, opens its PR, retrying `gh pr create`, and stops when no PR opens. It
   waits for the CI run on the pushed head, polling through gh and fetch failures until it times
   out, then fast-forwards `main` to that head.
4. It removes the worktree, local branch and remote branch of each lane and of the landing once
   all their commits are on `main`, by hash or as the same patch. A worktree stays when it has
   uncommitted changes or ignored files other than `target/` and Python caches, and so does a
   branch with a commit `main` lacks; it names each. When `land/NAME` is already on `main`, the
   same command again only does this step.

It exits 0 when landed, 1 when a step fails, 2 at a conflict, 3 when `main` moved (land again
under a new NAME), 4 when the CI run failed or was cancelled, and 5 when it timed out.
`--dry-run` picks and gates, then names what a landing would remove, and pushes nothing.
`--after SHA` stacks a landing on an earlier one whose head is SHA: it builds on SHA and lands
only once SHA is on `main`.

When a batch's run fails:

- Find the failing change in its log. The changes before it are the green prefix; when it is the
  first, the prefix is empty and nothing lands.
- Land the prefix's lanes under a new NAME, never force-pushing the old landing branch.
- Send the failing change back to its lane. The changes after it haven't passed CI without it:
  re-batch them onto the landed prefix in a later landing, on their own CI run.
- A cancelled run names no failing change: re-run it, then the same command again.

The root retires finished clean-room exports, and removes an abandoned lane's worktree and
branches itself.

## Jim

- Jim sets goals and process. Bring him only goal questions, each with its full context and your
  recommendation. Engineering choices are yours.
- A decision of Jim's goes into MODEL §1 or PROCESS in the same session, and into memory.
