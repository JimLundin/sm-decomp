# Pipeline speed, parallel work and simulation confidence

Status: resolved

Document opportunities to speed up future decomp verification, recommend a
parallel implementation and verification setup, audit reference provenance and
assess simulation stability and accuracy against the retained evidence.

Scope: read-only investigation in the decomp checkout plus local Markdown
findings. Preserve the optimization worktree and branch. No pipeline source
changes, new replay runs or optimization experiments are authorized by this
review.

Findings: `pipeline-review.md` and `reference-and-accuracy.md`.

## Completion evidence

Completed on `decomp-port-integration`, 2026-10-11. The reports cite primary
runner/source code, pinned upstream sources, official movie-format/submission
material and retained verification/provenance records. Recommendations separate
existing capabilities, measured costs, proposed changes and unproved accuracy.
Local citations and whitespace checks pass; no new simulation/build/evidence
run or disposable worktree was created. See the research ticket for cleanup.
