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
   relay comes from output you saw.
5. **Relay** to Jim: what the step found, what you decided, and what comes next. Give every size
   as score, actual code lines and logic-equivalent lines (score ÷ 6), and cite `docs/REPORT.md`
   once a change lands.
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
  positional array or a macro per set. Ask that the value be held, not when it is built. Its
  mirror: asking that the compiler check a table's members turns the table into a `match` per
  member, which is logic. Ask that each row name its member; a test checks the set. When a
  gate names the way out of a trap ("constant rows, each naming its member"), the brief carries
  that shape; a release alone ("need not be a constant") leaves the costly encoding open. Before
  launch, the brief's gate works through each item under the checks.
- **Coding standards, never measures.** The brief gives no score, budget, fit or size target. After
  a round gate it may give the coarse verdict in words (MODEL §1 decision 11).
- **A cut round carries only measured cuts** (PROCESS, "Cut rounds"): shapes a gate prototyped and
  measured to shrink, stated as outcomes. Four samus cut rounds grew the model under shape-only
  briefs, while every cut a gate measured shrank it.
- **A cut round asks for the outcome.** Each shape finding says what the code should have less of
  ("one decision per thing she meets", "the air's case decided once"), never the device that
  would get there ("a value decided before", "an enum of its cases", "no snapshot"). A named
  device is followed even where it costs more than what it replaces: two cut rounds grew this
  way. The round's largest lever, when a gate names one, leads the brief.
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

The blind review uses the same launch, on a code-only copy: the model crate, its tests, the
vector file and the [blind-reviewer](../blind-reviewer/SKILL.md) skill. Leave out the spec, the
design notes and `CLEANROOM.md`. Its brief is one line: load that skill.

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
