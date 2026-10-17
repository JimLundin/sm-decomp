## Agent skills

### Translation first and batch verification

Prioritize translating complete recovered-C subsystems into Rust. Use lightweight
source review and focused checks during translation; accumulate coherent batches
before expensive verification. Record translated-but-unverified source honestly.

The user authorized GitHub Actions batch verification and public-project
preparation on 2026-10-17, superseding the earlier local-only restriction. Preserve
complete acceptance gates and exact source/input evidence on hosted runners.
Keep ROMs, private captures, credentials and private evidence out of public Git
history, logs and artifacts. Publication requires the reviewed source/history
audit; missing hosted inputs remain explicit unfinished gates.

### Project language

Implement project tools, orchestration and tests in Rust. Python is permitted
for running the DLL host. Outside that launcher exception, replace legacy Python
entry points with validated Rust commands; preserve their evidence contracts.

### Code quality

Before implementation, assignment or integration review, follow
`docs/agents/code-quality.md` for coding standards, required review evidence
and ways to shorten the verification loop.

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
