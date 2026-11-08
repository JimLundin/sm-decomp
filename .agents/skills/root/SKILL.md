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
- **Coding standards, never measures.** The brief gives no score, budget, fit or size target.
- **Scanned.** `tools/cleanroom/scan_brief.sh BRIEF` reports 0 before `launch.sh` takes it.

## The confined sessions

From a checkout of `main`:
1. `bash tools/cleanroom/canary.sh plant`.
2. `bash tools/cleanroom/export.sh DOMAIN C:/cleanroom/DOMAIN`, adding `-c COMMIT` when the
   spec is an amendment still on its branch.
3. `bash tools/cleanroom/launch.sh C:/cleanroom/DOMAIN BRIEF TRANSCRIPT`, with the transcript
   under `~/.cleanroom/transcripts/`. Done on `LAUNCH OK` and `CANARY CLEAN`.
4. Read the session's report: the last `result` event of the transcript.
5. `bash tools/cleanroom/import.sh C:/cleanroom/DOMAIN lane/<domain>-rN`. Done when
   `code_check.sh` passes.

The blind review uses the same launch, on a code-only copy: the model crate, its tests, the
vector file and the [blind-reviewer](../blind-reviewer/SKILL.md) skill. Leave out the spec, the
design notes and `CLEANROOM.md`. Its brief is one line: load that skill.

## Landing

Land one change at a time, by fast-forward through `bash tools/lane/land.sh <pr>`, after the PR's
CI is green: Quality, content, free-run-gate and lockstep-ratchet. The
[lane](../lane/SKILL.md#land-root-only) skill gives the steps.

## Jim

- Jim sets goals and process. Bring him only goal questions, each with its full context and your
  recommendation. Engineering choices are yours.
- A decision of Jim's goes into MODEL §1 or PROCESS in the same session, and into memory.
