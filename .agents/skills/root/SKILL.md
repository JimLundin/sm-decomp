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
3. **Wait** for its report. Start other work only where [What can run at once](../../../docs/cleanroom/PROCESS.md#what-can-run-at-once) allows, within its limit of three lanes.
4. **Check** the report against the step's done-condition: open the branch and run the cheap
   checks yourself (scan, doc links, the vector count, the gate line). Done when every claim you
   relay comes from output you saw. After independent review of a round or engineering task,
   complete [the Code Health review](../../../docs/cleanroom/PROCESS.md#coordinator-code-health-review).
5. **Relay** to Jim: what the step found, what you decided, and what comes next. Lead with the
   ladder (the lowest rung's frames played alone and what it still needs) and the free run's
   held frames per row; give size as one line of the round's delta, a diagnostic (MODEL §1
   decision 19), and cite `docs/REPORT.md` once a change lands.
6. **Record** the step in the coordination log (`$TEMP/collab/reviews/coord-log.md`): heads,
   verdicts, and the queue. Done when a fresh root could resume from the log alone.

## Briefs

A brief is the whole of what a fresh session knows. Write it like this:
- **Goal**, the input branch and hash, and the skill to load.
- **Pointers** to the files worth reading, rather than retyping them.
- **The done-condition** of the step, and what to report.
- **What is already ruled**, so the session builds on it.

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
2. `bash tools/cleanroom/export.sh DOMAIN C:/cleanroom/DOMAIN`, adding `-c COMMIT` when the
   spec is an amendment still on its branch.
3. `bash tools/cleanroom/launch.sh C:/cleanroom/DOMAIN BRIEF TRANSCRIPT`, with the transcript
   under `~/.cleanroom/transcripts/`. Done on `LAUNCH OK` and `CANARY CLEAN`.
4. Read the session's report: the last `result` event of the transcript.
5. `bash tools/cleanroom/import.sh C:/cleanroom/DOMAIN lane/<domain>-rN`. Done on `IMPORT OK`:
   `code_check.sh` passes and both clippy runs pass over every clean crate. On `IMPORT CLIPPY
   FAIL`, commit a mechanical fix yourself, right after the import, as `[exact]` with
   `Role: root` and the lint it answers, so the squashed commit records it under your role.
   Send anything that changes an item's logic back to an implementer
   ([PROCESS](../../../docs/cleanroom/PROCESS.md#a-round), step 7).

The blind review (a domain's first round, or when the last round gate asked for one) uses the
same launch, on a code-only copy that
`bash tools/cleanroom/blind_copy.sh EXPORT DEST` makes from the round's export: the model crate,
its tests, the vector file and the [blind-reviewer](../blind-reviewer/SKILL.md) skill, without
the spec, the design notes, `CLEANROOM.md` or a test that reads them. Its brief is one line: load
that skill.

## Landing

A round lands as **one commit** on `main` (Jim, 2026-11-10), its spec, model and integration
together; nothing of a round lands before it ([PROCESS](../../../docs/cleanroom/PROCESS.md#a-round),
steps 4 and 11). Before the squash, on the round's branch: add one to the model's `rounds` in
`docs/architecture/domains.toml`, move a plan domain's `status` if its work started or ended, and
regenerate REPORT and the README's status. Then squash it with `tools/lane/squash.sh SUBJECT
BRANCH`: you write the subject, which says what is true after the round and ends with its kind.
Land one change at a time, by fast-forward through `bash tools/lane/land.sh <pr>`, after the PR's
CI is green: Quality, content, free-run-gate and lockstep-ratchet. The
[lane](../lane/SKILL.md#land-root-only) skill gives the steps.

## Jim

- Jim sets goals and process. Bring him only goal questions, each with its full context and your
  recommendation. Engineering choices are yours.
- A decision of Jim's goes into MODEL §1 or PROCESS in the same session, and into memory.
