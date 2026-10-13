# Deterministic Translation scope execution

Status: implementing
Blocked by: 02, 03, 04, 05, 06, 07, 08

Implement candidate 01 from the architecture review in this isolated worktree.
The Translation scope execution module owns the complete decomp loop. Its only
agent-controlled step is producing Source replacements from a Rewrite capsule.

Worktree: `/home/jim/sm-decomp-translation-execution`
Branch: `architecture/translation-scope-execution-20261013`
Baseline: `3a8b9ea28b716d90df55f798941bba68e1df6531`
Architecture review: `/tmp/architecture-review-20261013T162355Z.html`

## Confirmed requirements

The user confirmed these decisions during the architecture review:

1. Enforce rewrite worker restrictions. It cannot invoke verification commands,
   mutate shared ledgers or access evidence directories.
2. A failed gate stops the Translation attempt with bounded diagnostics. Another
   evaluation requires changed candidate source or an explicit operator retry.
3. Workers return structured Source replacements. The trusted Rust supervisor
   applies replacements and commits approved source files.
4. Build and prove the supervisor in this worktree before an explicit production
   switch. Preserve existing claims, submitted source and retained evidence.
   Retire unrestricted dispatch for migrated work at that switch.
5. Freeze verification rules. Missing exercise provenance, Fixture coverage or
   harness dependencies stops before rewriting. Changes to verification rules
   require reviewed Rust/policy work outside the rewrite worker.
6. Enforce hard time and output budgets covering logs, snapshots, temporary
   files and retained failures. Refuse launch if required enforcement cannot be
   established. Scope-specific values come from bounded measurements.

## Selected architecture

Use a small supervisor interface backed by a durable Translation attempt journal.
Depth comes from owning policy, preparation, rewriting, gates, integration,
publication and cleanup together. Keep the queue, compiler caches and existing
evidence authenticators as internal implementation where they already earn
their depth. Native gameplay and infrastructure are real execution adapters.

The rewrite seam carries an immutable Rewrite capsule and bounded structured
Source replacements. It carries no arbitrary command, run path, evidence pointer,
authority credential, stage label or agent-supplied acceptance record. Trusted
operator actions include cancellation and explicit retry; the worker cannot
invoke them.

### Deterministic sequence

1. Resolve an operator-approved Translation scope and frozen policy. Reconcile
   durable claims and ownership. Derive all resource locations from protected
   configuration rather than caller-selected roots or worker counts.
2. Prepare isolated source and declared inputs. Check inventory agreement,
   dependencies, prescribed fixture coverage, exercise provenance and bounded
   harness readiness. A missing prerequisite records a bounded stop result.
3. Produce the Rewrite capsule. Invoke a restricted rewrite adapter with source
   text and a bounded response channel. The adapter has no host repository tool
   access. Failed restriction checks refuse the launch.
4. Validate capsule identity, original hashes, relative paths, allowed files and
   replacement byte limits. Apply and commit replacements from the supervisor.
   Verification declarations remain frozen.
5. Freeze candidate identity. Execute the prescribed focused gates, all Affected
   Offline families and required Live replay gates with protected resource roots
   and enforced budgets. Reuse only evidence admitted by the existing contracts.
6. Stop failed attempts after process teardown and bounded evidence finalization.
   Identical lost-response retries recover this stop result; they execute no gate.
7. Admit successful submissions and integrate under one fenced owner. Keep the
   submitted source identity distinct from the exact clean merged source. Resume
   a previously completed merge rather than repeating it.
8. Authenticate retained evidence, complete both cleanup passes and retire
   integrated implementer worktrees after preserving required source snapshots.
   Reauthenticate required retained evidence after cleanup.
9. Publish only achieved stages through stable queue/ticket/progress identities.
   Lost-response retries repair projections from accepted results. Completion
   requires authenticatable retained evidence and finished cleanup.

### Identity and recovery

A Translation attempt binds original claim lineage, approved scope, candidate
source, pinned actual C, declared dependency/fixture inputs, installed execution
adapter identity and frozen policy. A changed candidate or explicit retry creates
a new evaluation identity; simple resume retains the previous identity.

Persist a durable intent before each side effect and its sealed result afterward.
Recovery reconciles process start identities, source/commit ancestry and retained
observations. An ambiguous or surviving writer retains ownership. Recovering a
controller never starts duplicate work simply because a response was lost.

### Restrictions and budgets

Production controller state, policies, installed runners, verification inputs and
evidence must be outside rewrite worker capabilities. Current unrestricted
collaboration workers do not meet this contract. Restrict the actual worker
invocation, not only a command that the worker could bypass through host tools.

Candidate compilation and execution are also confined to declared inputs and
bounded outputs. Treat rewritten Rust as executable input: build scripts,
compile-time file inclusion and runtime I/O cannot reach protected ledgers,
evidence, controller binaries or undeclared inputs.

Enforce aggregate output limits during production, including stdout/stderr and
temporary files. Format maxima, periodic telemetry, post-run deduplication and
per-file size limits do not satisfy an aggregate budget. Overflow stops execution
and withholds acceptance; accepted evidence is never silently truncated. Failed
diagnostics stay within a fixed retained extent. A retention reference graph
protects accepted/historical evidence while retiring only unreferenced controller-
owned data according to the reviewed policy.

`bwrap` is installed and a simple namespace probe succeeded. Cgroup delegation
and a quota-backed output volume are not established; capability detection and
fail-closed refusal are required implementation work, not assumed deployment
properties. Do not claim strict launch until adversarial checks pass.

### Verification and performance

Preserve exact actual-C Fixture pairs, Fixture chains, exercised ownership and
ordered hardware-effect comparisons. Authenticity alone does not prescribe
coverage: map required routines/domains to registered verification declarations.
Unknown dependency effects leave the Offline family affected.

Use shared source-definition discovery for production and admission; the known
`return (...)` keyword disagreement must stop appearing as stale inventory.
Fix canonical Rust module placement and actual-C dependency preparation before
starting broad verification. These are prerequisites of the selected workflow,
not an open-ended refactor of every family.

Freeze and compute actual dependency identity once per operation where possible.
Reporting consumes accepted results. Narrow invalidation only after dependency
knowledge justifies it; retain conservative fallback. Reduce repeated full
observation reconstruction only when immutability and required trust transitions
are owned by the module. Historical readers and external tamper checks survive.

## Test surface and acceptance

The module's interface is the test surface. Use a deterministic recorded-rewrite
adapter and real local Git/process/evidence behavior; avoid a test that merely
asserts journal internals.

- Equivalent start/submit/resume after a lost response runs each gate once and
  allocates no duplicate capture.
- A failed gate produces one bounded stop result; unchanged-source resume runs
  nothing. Changed source and operator retry have separate recorded identity.
- A stale capsule, foreign path, alias/symlink or changed original hash rejects
  before modifying source.
- Worker attempts to read/mutate protected state, invoke host verification or
  redirect output are denied by execution restrictions.
- Candidate compiler/runtime attempts to access undeclared inputs are denied.
- Undeclared outputs and aggregate byte/time exhaustion terminate the writer,
  leave bounded authenticatable diagnostics and grant no acceptance.
- Missing inventory agreement, coverage, dependencies or sandbox capabilities
  stops before rewrite dispatch.
- Recovery at launch, gate, merge, publication and cleanup reconciles the exact
  existing side effect and never duplicates an ambiguous writer.
- Native/infrastructure adapters keep their distinct evidence and stage meanings.
- Frozen-source changes and tampered retained output reject at the appropriate
  trust transition. Clean source and current authority are required for admission.
- One real existing supported Translation scope traverses the complete restricted
  path against actual C, including retained authentication and measured cleanup.
- Production switch imports existing lineage without creating replacement claims,
  disables unrestricted dispatch for migrated work and preserves historical
  authentication/inspection.

No performance speedup claim is required before correctness. Measure a bounded
representative complete attempt; report gate reuse, bytes allocated/retained,
cleanup reclamation and wall time. Do not run another long exploratory matrix.

## Ownership and integration

All authored work for this effort stays in this worktree until review. Main Git,
active queue and current integration role belong to the existing background
coordinator. Do not mutate them while designing or implementing this branch.
Any integration/cutover uses current checked authority and reconciled ownership.

Tests and tools are Rust. Use exact scoped commits and isolated verification
outputs. Apply `docs/agents/run-cleanup.md` to every verification run, including
failed runs, before resolving tickets or handing implementation back.

## Completion evidence

The user confirmed the complete contract and requested implementation on 2026-10-13.
Implementation, complete restricted-path verification and final production-switch
decision remain pending. Worktree creation used direct Git after the
managed worktree operation failed to populate and inspection found no registered
managed checkout. No legacy source, claim, evidence or writer was changed.
