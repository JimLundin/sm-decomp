# Proposed gameplay swarm launch handoff

Status: proposal; root adds actual validation and cleanup receipts before adoption
Blocked by: 04 and the user's separate gameplay launch instruction

Preparation establishes the launch workflow and retains its evidence. Production
authority and the canonical ledger stay absent until the separately requested
launch. The next session first reads [AGENTS.md](../../../AGENTS.md),
[domain rules](../../../docs/agents/domain.md), [GLOSSARY.md](../../../GLOSSARY.md),
relevant ADRs if present, then the [session runbook](../../../docs/agents/decomp-sessions.md),
[spec](../spec.md), [map](../map.md), [checkpoint](../checkpoint-20261013.md),
[preparation status](status.md), [final readiness](readiness-final/README.md),
[fresh drill handoff](../drill-final/handoff.md),
[catalog](../catalog/subsystem-catalog.md) and [later lanes](../catalog/later-lanes.md).
Read [assignments](../../../docs/agents/assignments.md),
[translation progress](../../../docs/agents/translation-progress.md) and
[run cleanup](../../../docs/agents/run-cleanup.md) before dispatch or integration.

## Retained preparation subject

The corrected execution and validator use one frozen source and executable:

| Identity | Retained value |
| --- | --- |
| Frozen source | `/tmp/sm-decomp-preparation-final-source-20261013` |
| Full commit | `2d59536c059a1ca9c31d9278bff2d9222eafa56c` |
| Source fingerprint | `e5e412053f6dbe2cbadaba077192e885684fbb9f2699cafeaa6acd694879d78d` |
| Engine | `.scratch/decomp-parallel/drill-final/bin/sm-pipeline` |
| Engine SHA-256 | `df6eb4dc3a9fd69e5d5923b5c22f34dea7b434cd7ee87f1b69085cd9e9431c84` |
| Build receipt | [engine-build.toml](../drill-final/engine-build.toml) |

Root must attach the final `drill-final/concurrency-drill.toml`, its sealed
evidence manifest, passing maintained validation, and readiness-final's actual
namespace validation/host-preservation receipts before declaring preparation
ready. These final artifacts were still pending when this proposal was written.
The ready helper README or a passing unit suite alone supplies no actual drill
or production readiness verdict. Preserve the old `drill/` and historical frozen
subjects as evidence of their own source epochs.

Use relative repository pointers for documents and manifests. Typed requests and
sealed evidence retain their required absolute paths. Recovery on another machine
must restore those source paths, executable/readers, retained evidence store,
subject pointers and private ROM/movie inputs; a Git clone alone is insufficient.

## Launch procedure

1. Inspect the checkout, worktrees, current HEAD and retained seals. Recheck
   `engine-build.toml`, the final drill manifest/evidence, readiness-final
   manifests and checkpoint archive. Read Ceres `assign status` with effort
   `ceres-continuation`, token `ceres-20261013`, capacity `3`; inspect every other
   existing ledger and claimed ticket before selecting writable paths. All
   production sessions share `SM_PIPELINE_ROOT=/home/jim/sm-decomp/out/pipeline`
   and `SM_PIPELINE_WORKERS=4`. Assignment capacity and process workers are
   separate budgets.
2. Inspect each exact real session/thread and its child/process start identity
   through the available tools. Preserve live or ambiguous claims. Record who
   owns main Git/index, each integration role and retained reporting processes.
   Names such as `/root` and elapsed time cannot establish liveness. The final
   readiness observation records root `01a134cd-dafb-7e50-8284-f285a905e05b` as
   prospective integrator; later sessions must establish their own current
   identities and checked transfers rather than reuse that observation as proof.
3. Prepare fresh typed production requests derived from the final reviewed
   readiness policy, exact bridge and passing fresh drill. The readiness helper
   prepares and validates only inside a private namespace; it leaves host policy
   absent and canonical authority inactive. For host activation, the observed
   integrator calls maintained `decomp-authority prepare`, records actual role
   registrations/checked transfers, then `validate-activation` and `activate`
   against the exact retained frozen source/drill. Bind the calling real
   `session_id`, current generation and seals; never override `CODEX_THREAD_ID`.
   Register distinct persistent dispatcher tokens and one explicit integrator.
   `activate` changes policy state; create the canonical ledger only through
   subsequent admitted queue operations, never by copying legacy jobs.
4. Confirm policy is active and returned roles/generations match inspected
   owners. Preserve the [v2 legacy bridge](../reconciliation/legacy-owner-bridge-v2.toml),
   SHA-256 `cb537f195b4f83529f2ea7a61bd65c6e77ad215366275e8d60a7a604c4eb0ba9`.
   Before any legacy mutation, recheck the original Ceres ledger seal
   `77fdba066af4f4c045e8e8f419997563d569ac0bd8045cef0a68ac916a126b89`
   against the final preparation receipt. Any change requires reconciliation,
   not blind replay of old request bytes. All original claim tuples, submissions,
   attempts, tokens and acceptance gates remain authoritative. Exact legacy
   infrastructure envelopes may authorize only original requests sealed as
   requests in the bridge; receipt artifacts cannot stand in for requests.
5. Select the first open, unblocked, unclaimed ticket by number and inspect its
   permitted route. Reserve one assignment per available subagent slot; give
   each actual child the exact returned claim, pinned baseline, writable files,
   isolated worktree, run path, scope and gates. Persist the returned child UUID
   in typed Start and checkpoint before/after gates. A surviving child keeps its
   old claim; a checked transfer/adoption may change its parent role. A stopped
   submitter retains submission ownership until authenticated integration.

Mutations use current real role credentials. `assign plan` and status remain
read-only; `--no-translations` suppresses new discovery but does not exclude
already queued work. A changed mutation/admission implementation requires a
fresh actual drill; validator-only/docs changes need explicit source-equivalence
review. Activation validation binds the retained source checkout's exact path,
HEAD and fingerprint: do not silently substitute current main HEAD or a relocated
source checkout.

## First dependency-ready work

After 04 is resolved, [11](../issues/11-finish-held-ceres-integration.md) is the
first continuation. It tracks the existing Ceres claims and creates no new
implementation ownership. Reconcile and finish
`(integrate-shared-game-host, 48656a2-host-v1-1, 1, ceres-20261013)` first.
Host source `73547105d870f8b05f8c6bcf55b768c223889fdb` is already merged by
`e8c9c3e10ac71bf0377dc00d7de5764a4263b445`; avoid a duplicate merge. Then
reserve the existing ready `integrate-compact-controller` linked to submitted
`(compact-controller, 48656a2-compact-v1, 1, ceres-20261013)` at exact source
`223f8a1b0c7382817b1290d13c7a41f4e722d5e1`. Continue original source/gates under
the checked Ceres integrator; preserve source/readers before worktree retirement.

[12](../issues/12-ceres-nmi-poll-continuation.md) follows 11 under ticket 78.
Research the extra poll at ordinal 7,741/frame 8,508; publish an Answer and a
reviewed correction task/scope before any source mutation. Its pipeline/observer/
C-template envelope is infrastructure; the custom
[holding TOML](../catalog/ceres-continuation-holding.toml) is unsupported as
`assign --scope` input.

Read-only whole-subsystem research [13](../issues/13-host-hardware-lifecycle-scope.md)
and [14](../issues/14-samus-frame-scope.md) can occupy the next free slots while
11 integrates; [15](../issues/15-door-room-plm-scope.md) and 17–20 follow in
ticket order. Their implementation readiness is incomplete. The native promoter
currently accepts bank86 projectile exercise provenance and coherent 1–32
routine scopes only. NMI/host, Samus, doors, audio/UI, SRAM and credits require
supported matching whole-root evidence or an explicit reviewed Rust exercise/
hardware-contract extension task. Callee counters and historical leaf receipts
do not prove a scheduled caller. Research completion creates exact implementation
children and adds them to milestone blockers.

[16](../issues/16-continue-held-whole-ridley.md) continues held ticket 141 when
route dependencies and user steering permit, preserving its original formal
gates. Its broad source and Ceres diagnostic evidence do not establish real-boss
or full progression domains. Reuse accepted player/projectile/trail/camera/PLM/
enemy/audio leaves; research dedicated complete boundaries and exact disjoint
files. The integrator retains shared lib/state/ROM, CLI/catalog/ABI/observer/host
wiring, inventory and main Git/index ownership.

Initial reviewed experiments remain at most 60 seconds; integration gates at
most 3,600 seconds. Rust project tools/tests/orchestration remain the migration
target. One integrator freezes merged source, verifies every affected Offline
family conservatively, runs required material actual-C/live/original-ROM gates,
publishes each achieved progress stage, completes both cleanup passes, preserves
source/store/readers/private inputs, retires integrated worktrees and
reauthenticates before durable completion. Ceres escape requires ordinary reset
and observed `32 → 33 → 34`; later milestones retain their separate route,
save/load, credits and zero gameplay C/CPU fallback requirements.

## Documentation inconsistencies for root to reconcile

- `preparation/status.md` still names historical c8799f3 execution, separate
  validator and the failed previous coordinator. Operational final-source,
  role, fresh drill and readiness receipts should replace those current claims
  while their history remains linked.
- Spec's capability table and manual-handoff wording describe pre-02/03 limits;
  maintained cross-effort fencing and typed Start/Checkpoint/Resume/Adopt now
  exist. Its checkpoint tables are historical source/claim observations.
- Catalog source/owner/Git/registry hashes are dated research observations.
  Reinspect at launch. Catalog and later-lanes differ on prospective dedicated
  module names (`door_transition`/`room_transition`, `persistence`/`save`); each
  research Answer selects one exact reviewed envelope before claim creation.
- Old cutover activation template contains an old checkout/session and a
  placeholder drill seal. It cannot authorize launch. Final readiness prepare
  request is also bound to a particular caller and private validation; fresh
  production requests need current real observations and exact retained seals.

This proposal only read repository documents/source and wrote this file. It
created no fixture, replay, build, worktree, queue mutation or disposable run;
no-run cleanup reclaimed 0 bytes. Root owns integration of the proposal and
final status/issue updates after actual validation.
