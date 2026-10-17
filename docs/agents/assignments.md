# Assign decomp work

Use this workflow when coordinating affected Offline verification, failure
diagnosis, or new gameplay translation across subagents. Rust publishes and
claims work; the coordinating agent launches it through existing subagent tools.

For implementation assignments and integration review, include the standards
and review evidence required by [code quality](code-quality.md).

## Plan and claim

1. Inspect `sm-pipeline batch plan --profile offline-all --changed-since BASE`.
   It explains selected and excluded families without creating a run or authenticating
   retained evidence. Unknown inputs remain affected. The same selector drives
   `batch verify --changed-since BASE`.
2. Inspect `sm-pipeline assign plan --changed-since BASE --coordinator TOKEN`.
   Use one persistent coordinator token for the effort. Add `--no-translations`
   for verification alone; `--batch-result BATCH/batch.toml` proposes diagnoses.
   Native exercise receipts and ready local tickets propose bounded translation
   research. Historical counters remain research inputs.
3. Request `assign next --changed-since BASE --coordinator TOKEN --available-slots N`.
   Repeat for each free subagent slot, inspecting each returned assignment.
   Reservations consume the fixed `--capacity` (default 3) before launch.
   The authoritative ledger and generated Markdown jobs live in the main
   checkout's `.scratch/EFFORT/`, shared by every worktree and coordinator.
   Use `--effort` to select the effort and keep the same capacity when resuming it.
4. For an implementation assignment, create exactly its returned branch/worktree
   with `git worktree add -b BRANCH WORKTREE HEAD`. Use its independent batch path
   and exact writable files. Give every agent the same `SM_PIPELINE_ROOT` and
   `SM_PIPELINE_WORKERS`; batch/fixture workers already share that worker budget.
   Verification gets one batch job covering its selected families.
5. Launch the assignment through the existing collaboration tools, then record
   its returned agent identity. A launch that definitely failed before starting
   uses `launch-failed`; an ambiguous launch stays reserved for reconciliation.

Commands also accept a complete TOML request on stdin. Pass the shared worker
settings in `pipeline_root` and `workers` for stdin requests; flag commands inherit
`SM_PIPELINE_ROOT` and `SM_PIPELINE_WORKERS` and accept explicit overrides.
`assign record --update
UPDATE.toml --coordinator TOKEN` reads a typed update. For example:

```toml
event = "started"
agent = "/root/assigned_agent"

[claim]
id = "RETURNED_JOB_ID"
revision = "RETURNED_REVISION"
attempt = 1
coordinator = "TOKEN"
```

## Report results and integrate

Record `event="finished"`, its exact `claim`, and a `[result]` table whose
`result` field is `verified`, `diagnosed`, `researched`, `submitted`, `integrated`
or `failed`. Evidence is an array of `{path, sha256}` artifacts with absolute
paths. Use the native typed schema in `assignment.rs` for each variant.

For verification, supply the assignment's completed `batch.toml`. The coordinator
authenticates every selected actual-C family using the existing current-source
contract and writes its own authentication receipt. Caller authentication fields
are placeholders replaced by validated receipts. The coordinator applies both
cleanup passes and reauthenticates before completing verification. A failed batch
produces diagnosis jobs, cleans finished runs and preserves its evidence.

Research publishes `translation-scope-v1` TOML (schema: `translation::Scope`).
It identifies observed routines, pinned inventory/exercise receipts, completed
dependency tickets, reviewed dependency evidence, current input digests, exact
writable files and concrete verification argv. Initial commands are capped at
60 seconds; integration gates at 3,600 seconds. Complete facts promote automatically;
missing facts return a research job explaining the gap. Research for an existing
task preserves that task until implementation, merged verification and cleanup.
Counter receipts establish planning provenance; they grant no gameplay acceptance.
Read [translation.rs](../../crates/pipeline/src/translation.rs) for the native
scope fields and permitted verification commands. Native gates must produce a
verification receipt; Cargo test gates must report at least one executed test.

Submission requires an exact commit based on its recorded baseline, confined to
the approved writable files, in the assigned clean worktree. The coordinator runs
the scope's bounded gates there. Submission creates integration work, retaining
writable ownership. One integration owner runs one integration at a time; overlapping
implementation waits while independent scopes and read-only research can proceed.

The integration owner merges the submitted commit, freezes source and supplies
passing batch evidence covering all affected families. Required native gates can
also supply their batch evidence directly to result validation.
Remove the integrated worktree after preserving any sealed source dependencies.
The coordinator checks commit ancestry, executes the required gates on merged
source, authenticates family evidence, applies both scoped cleanup passes and
reauthenticates. Only this completion resolves the implementation task.
Preserve the ledger's result artifacts. Retrying the exact finished update reuses
the sealed coordinator result, including after implementer worktree removal.

## Recover an interrupted owner

Use `assign status` to inspect durable claims and pending revisions. Claims have
no expiry. Inspect the named agent through the collaboration tools and stop it
if needed. Preserve its source, logs and evidence. Publish a sealed TOML observation
with `agent`, `coordinator`, `status` (`completed`, `failed`, `interrupted`, or
`not-launched`) and the exact `[claim]`. Record `event="owner-stopped"` with
`observation={path,sha256}` under the same persistent coordinator token.

The queue checks the observation against the stored owner and attempt, then permits
a new claim. Wall time alone never releases ownership. A changed revision cannot
expand a running agent's scope or let an older result complete newer work. An empty
selection is reported as `not-required`; it grants no verification acceptance.
