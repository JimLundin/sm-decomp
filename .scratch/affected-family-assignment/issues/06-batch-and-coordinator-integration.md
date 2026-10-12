# Integrate selection and assignment with existing execution

Status: open
Blocked by: 03, 04, 05
Triage: ready-for-agent
Type: task

## Required work

Use the selector in the existing changed-since batch path and expose its
read-only explanation. Add coordinator-facing queue commands using native typed
requests/results. Existing subagent tools launch returned briefs; introduce no
agent launcher. Batch remains the sole record owner and retains its scheduling,
authentication and fixture execution. Return selected names explicitly so callers
can distinguish invocation work from historical batch records.

Document planning, claim/result/recovery and isolated worktree usage in project
docs. Demonstrate a queue cycle with research promotion, independent implementation
assignments, overlap deferral and integration ordering. Changes to agent-facing
instructions require the writing-for-agents skill.

## Comments

Integration owner coordinates shared main.rs/batch.rs edits and source freeze.
