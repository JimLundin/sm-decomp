---
name: handoff
description: Transfer or resume an sm-decomp repository lane at an ownership or context change, or prepare its final handoff to the root.
---

Write the repository-side handoff for the next owner's task. Save it at `$TEMP/collab/<lane>/HANDOFF.md` in at most 60 lines. For a final report, send the path with the result. Check that the recipient can read both the note and every linked artifact under the [role visibility rules](../../../docs/cleanroom/PROCESS.md#who-sees-what). A confined recipient receives the root's approved brief through the export procedure.

- **State** — Give the issue, stage, worktree, branch, fixed base and observed tip. Identify any uncommitted files and where they live. Link the receipt for each material claim to the revision it checked. Describe remaining proof and the reason it is pending.
- **Next** — Name the next owner and first concrete action. Carry forward user decisions that affect this assignment, including their scope. Give the root the tracker update it needs.
- **How** — Point to the applicable skill or role, durable artifacts and any successful command the recipient needs to repeat.
- **Traps** — Record a failure that changes the next attempt, including the reproduction and its boundary. Keep sensitive values and original game bytes out of reports.
- **Cleanup** — At final handoff, finish or stop your own jobs. List the local and remote branches with their tips, worktree paths and disk use, and build outputs. Mark each released item or name its continuing owner and use. Preserve uncommitted work. The root routes releases to [lane cleanup](../../../docs/cleanroom/PROCESS.md#cleanup-at-lane-end).

On resume, compare the recorded worktree status, base and tip with the checkout and the current work issue. Reuse evidence for the revision it covers. If the checkout or outstanding edits differ from the recorded state, identify the changes and rerun affected proof. Continue at the recorded action or update it to reflect newer user direction.
