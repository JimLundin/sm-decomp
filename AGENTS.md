## Agent skills

### Project language

Implement project tools, orchestration and tests in Rust. The migration target
is a project with no Python source or runtime dependency. Replace legacy Python
entry points with validated Rust commands; preserve their evidence contracts.

### Issue tracker

For specs, tickets and work status, use local Markdown in `.scratch/` following
`docs/agents/issue-tracker.md`.

### Triage labels

For triage, use the default role names defined in `docs/agents/triage-labels.md`.

### Domain docs

Before codebase exploration, follow the single-context reading rules in
`docs/agents/domain.md`.

### Run cleanup

After verification runs and before closing work or handing it back, follow
`docs/agents/run-cleanup.md`. Cleanup is part of completion.

### Translation progress

For gameplay translation, assignment or integration, follow
`docs/agents/translation-progress.md`. Assign whole subsystems and publish each
stage to the shared inventory before handing work back.

### Parallel sessions

For decomp session startup, concurrent sessions or crash recovery, follow
`docs/agents/decomp-sessions.md`.
