# Focus decomp iteration and assign independent agent work

Status: open

User selected affected Offline family selection from the architecture review at
`/tmp/architecture-review-20261012T181836Z.html`, then expanded its scope to
automatic assignment of verification, diagnosis and new gameplay translation.
Design choices Q1–Q10 and final shared understanding are confirmed by the user.
Implementation is authorized and in progress on `affected-family-assignment`.

## Settled decisions

- Improve the existing changed-since workflow and explain selected/excluded
  Offline families, changed paths, dependency matches and uncertainty.
- Include every family that cannot safely be ruled out. Retain current-source
  authentication before evidence reuse and Live replay.
- Automatically choose new gameplay translation candidates as well as
  affected-family verification and failure diagnosis jobs.
- Publish local jobs in `.scratch/` for one coordinating agent to dispatch
  through existing subagent tools; a separate agent launcher is outside scope.
- New candidates first receive bounded research establishing observed exercise,
  dependencies, Translation scope, writable paths and verification commands.
  Complete scopes can then promote automatically to implementation work.
- Prioritize dependency-ready, exercised gameplay. Inventory counts and lexical
  dependency reports remain planning facts, not acceptance or write ownership.
- Implementation agents use isolated worktrees and separate batch records.
  One integration owner serializes overlapping writable files, integrates changes
  and verifies the merged source. Read-only research can overlap freely.
- One shared assignment ledger enforces exclusive claims across worktrees.
  Interrupted-agent recovery verifies that the owner has stopped before releasing
  its claim and retains unfinished source/evidence for recovery.
- Fill available subagent slots using the existing shared verification worker
  budget. Initial research experiments are capped at approximately 60 seconds
  each; larger required integration gates follow a source freeze.
- Use a pull queue: the coordinator requests ready assignments and records
  started/results/recovery updates. Keep a read-only affected-family explanation
  for the existing batch command.
- Verify with small actual-C/Rust families, concurrent claims, checked recovery
  and bounded before/after selection measurements. Require full Live replay
  only if execution or admission contracts change during implementation.
- Tooling, orchestration and maintained tests use Rust and preserve existing
  evidence contracts and scoped cleanup.

## Agreed module behavior

The module owns selection, assignment readiness and durable work state. Existing
batch owns actual-C verification, evidence reuse and Live replay. The coordinating
agent uses existing subagent tools to launch assignments and supplies lifecycle
observations back to the queue. No Rust agent launcher is introduced.

The interface supports opening/resuming an effort, producing a read-only
selection report, requesting ready work, and recording typed updates. Ready work
includes a local ticket, scope, writable ownership, source identity, independent
batch/worktree requirements and a launch brief. Outcomes distinguish ready work,
waiting on capacity/dependencies, recovery requiring checked owner state, and
completion of the requested effort.

Stable work identity, input revision and attempt identity remain distinct. An
input change cannot let an old completion resolve new work. Scope expansion
cannot silently expand a claimed agent's writable files. Completed research may
create implementation readiness; an implementation submission still requires
integration, merged-source verification and scoped cleanup before resolution.

One authoritative ledger in the main checkout's `.scratch/` coordinates claims
across worktrees. Derived ticket copies are not independent claim authorities.
Reservations consume available slots before launching; launch failures and
coordinator interruption preserve enough state for explicit reconciliation.
The ledger never turns elapsed time into proof that an agent has stopped.

The queue first honors open, dependency-ready, unclaimed local work. It can
propose additional gameplay research from pinned source inventories and observed
exercise; it does not overwrite existing claimed/resolved tickets or infer
writable independence from compiler reads. Missing exercise/scope is research
work, not implementation readiness. Ranking explains why work is selected or
deferred; inventory counts never grant acceptance.

Selection handles tracked and untracked changes, both sides of renames,
deletions, declared/captured assets, compiler closure and discovery namespace
changes. It validates the planning facts needed for exclusions without hashing
every unrelated full-WRAM artifact. Unknown information remains affected. A
selection report cannot grant passing verification, evidence reuse or Live replay
admission; those still cross existing current-source authentication.

Batch verification continues to schedule actual fixture work through its existing
worker implementation. The assignment queue does not launch one competing batch
writer per Offline family when one batch can own that verification work.

## Implementation order

See tickets 03–07 in [map](map.md). Selection and local claim/queue implementation
can proceed in parallel behind agreed result types; translation promotion follows
the ledger, and integration combines the selector with existing batch/coordinator
entry points. The integration owner handles checks, scoped cleanup and retained
evidence authentication.

## Authorization

User confirmed the complete design and instructed implementation to proceed.

## Factual constraints

- `batch.rs:367` selects before loading prior batch evidence at `:414`.
  Any tools change or port-source change currently includes all profile families.
- Compiler closure is retained as `Implementation.rust_dependencies`, plus
  harness/source/manifest identities. It is useful planning evidence; current
  authentication still validates complete artifacts and present source bytes.
- `inputs.rs:35` enumerates all top-level Rust module filenames, then matches
  every identifier-like source word. Adding a filename can alter an unchanged
  family's generated imports, so old compiler reads alone do not prove exclusion.
- Family inputs include manifest/capture/ROM assets outside compiler closure.
  Missing, stale or incomplete closure must remain explicit uncertainty.
- Existing batch internally schedules verification. Separate batches/worktrees
  already share worker admission and compiler cache safely.
- Local tickets are Git-tracked. Independent worktree copies of `Status: claimed`
  cannot alone prevent duplicate assignment by different coordinators.

## Discussion

Design decisions Q1–Q10 recorded from the user's replies. The pull queue was
selected from three independently explored interfaces; see
[comparison](interface-comparison.md). Final confirmation received; implementation
tickets03–07 are authorized.
