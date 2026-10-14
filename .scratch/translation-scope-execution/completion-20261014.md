# Deterministic Rust executor completion

Status: implementation proved; production switch pending
Owner: `01a136a0-49bb-7690-a360-48def3c2c09b`
Worktree: `/home/jim/sm-decomp-translation-execution`
Branch: `architecture/translation-scope-execution-20261013`
Verified Rust source: `f845516b975c3bfc413800c6bed9e329696a79e6`
Proof/documentation checkpoint: `9dd4361`

For installed supported Translation scopes, Rust owns preparation, restricted
rewriting, source application, candidate and merged gates, integration,
publication, cleanup and recovery. The worker receives an immutable Rewrite
capsule and returns structured Source replacements. Verification policy and
authority stay with the controller. Ordinary resume creates no new evaluation.
Tickets 02–07 are complete; [ticket 08](issues/08-production-switch.md) remains open.

## Retained proofs and affected checks

The original [actual-C checkpoint](supervisor-native/actual-c-02-checkpoint.json)
and [managed checkpoint](managed-gates/actual-c-managed-11-checkpoint.json) retain
exact Installation, Completion, measurement, reader and cleanup References.
[The final reference list](verification/completion-f845516.references.json)
binds 47 distinct retained files to the verified source. Earlier infrastructure,
retry/repair, failure/accounting, confinement and recovery proofs remain in
[checkpoint 27b8 references](verification/checkpoint-27b8b0a.references.json).
Completed broad/native/linker matrices were not repeated.

| Proof | Domain | Result and measurements |
| --- | --- | --- |
| Original native actual-c-02 | Two enemy-jump routines; 16 actual-C Fixture pairs across four gates | Passed; 255.393-second loop, 484.17-second full test; 34 gate producers, 102 reused references; zero new resume evidence |
| Managed actual-c-managed-11 | Disposable Observed/Helper diagnostic; two Fixture pairs per gate across four gates | Passed; 103.420-second loop, 601.65-second full test; six gate producers, 18 reused references out of 24; 14 operation receipts |
| Managed recovery | Original claim/job/worker/attempt, source CAS, normal submission and derived integration, three distinct progress stages | Passed; both queue entries complete; exact pending promoted once with zero new producer evidence |
| Pending and tamper refusal | Partial, changed, symlinked and hardlinked pending; host source and retained acceptance edits | Passed; refused pending preserved; original owner and queue unchanged |
| Affected source checks | Two real ticket-publishing regressions; migration; admitted progress; formatting | Passed: 2 ticket tests, 11 migration tests with two ignored diagnostics, one progress test, all three affected Rust files formatted |
| Final build and Clippy | Locked/offline current pipeline CLI and test target | Passed; CLI build 15.12 seconds; Clippy 34.65 seconds with established shared exceptions |

Clippy used `-D clippy::all` and the existing allowances for `filter_next`,
`collapsible_if`, `chunks_exact_to_as_chunks`, `too_many_arguments`,
`large_enum_variant`, `needless_borrows_for_generic_args` and `op_ref`.
Existing Rust dead-code warnings remain. The optional 176-family timing failure
(18.97 seconds against 10 seconds) remains retained; no speedup is claimed.
Historical sandbox EACCES remains unproved and distinct from the reproduced,
fixed descriptor-quota bypass.

The managed proof is a disposable diagnostic with a synthetic stopped original
worker observation. It grants no production liveness, shipped gameplay,
original-ROM parity or Live replay acceptance. Original-ROM parity and Live
replay were also not tested by the original native proof.

## Cleanup and retained authentication

Both ordinary and deduplicating cleanup passes passed for managed 09/10/11,
reclaiming zero eligible bytes and checking 12 source snapshots per root. Both
final verification-directory passes passed with zero eligible bytes. Earlier
native and preparation cleanup receipts remain valid. Failed proofs and red/green
diagnostics, including the refused unpadded cleanup path, remain retained.

Managed 11 retained 27,249,752 finalization bytes and 1,203,738 publication bytes;
its run measured 27,480,037 logical bytes. The supervisor retired its producer
checkouts and reclaimed 7,666,300 logical bytes before publication.

The reviewed Rust [disposal plan](verification/disposable-retirement-plan-final-01.jsonl)
and [apply receipt](verification/disposable-retirement-apply-final-01.jsonl)
retired 23 closed disposable targets: 5,195,113,120 logical bytes and
4,825,128,960 allocated bytes. A second measured
[helper retirement](verification/helper-retirement-apply-final-01.jsonl)
removed the four remaining helper source/binary files: 9,656,336 logical bytes
and 9,666,560 allocated bytes. Combined allocated reclamation is 4,834,795,520
bytes. Final observed available space is 191,088,582,656 bytes. These receipts
distinguish logical extent, allocated extent and available space.

Both proofs authenticated after disposable target retirement without producers:
native actual-c-02 in 27.31 seconds and managed 11 in 41.98 seconds. Managed 11 also
authenticated before disposal in 36.63 seconds. No verification process remains.
The exact readers are retained and must not be overwritten:

- `supervisor-native/tools/sm-supervisor-retained-reader-02`:
  SHA-256 `494e790a810e28ade5db9999fee438b43deb641bb53020e8f1a4c2f8133f3d2f`, 236,919,608 bytes.
- `managed-gates/actual-c-managed-11/installed/controller`:
  SHA-256 `cde6329250ae3657385aa9a887ccdb6fc881a3329f4495757449e37c9bd4ecb8`, 240,702,016 bytes.

Keep sealed journals, operation results, source/revision archives, acceptance
References, original-owner snapshots and every referenced private input.
`/tmp/sm-complete-supervisor-976030-0` is protected and remains present.
The native Installation's historical disposable controller path is unchanged;
historical authentication uses its exact retained reader. Managed 08/09/10/11
keep their original controller copies. ROMs, binaries, bulk evidence and build
trees are excluded from Git checkpoints.

## Production boundary and next action

Final read-only [authority status 06](verification/recovery-authority-status-06.stdout.toml)
is byte-identical to status 05. Main status is also byte-identical and main HEAD
remains `c2cae9ac145f9343cd345e6f978bff0774510b56`. The
[four production hashes](verification/recovery-production-hashes-06.sha256)
match the saved baseline. Registered integrator remains `ceres-20261013`,
generation 2, session `01a13592-d02b-7143-8489-a15b03f19da2`.
This effort made no production source, queue, authority or progress update.

The [three-original Ceres preparation](production-preparation/preparation-checkpoint-27b8b0a.md)
remains `validated-with-blockers`, activated=false. Its original source,
claims, decisions and evidence are unchanged. Ticket 08 still requires reviewed
per-original installations/policies, supported Submitted/linked Integrate
sequencing, catalog and canonical-queue preparation, fresh actual-owner
reconciliation and the separately agreed switch decision. Preparation must be
concrete before requesting activation approval.

Continue from ticket 08's exact retained package and current ownership. Retained
proof inspection uses the readers above and creates no new evaluation. Do not
rerun the completed producer proofs, rewrite an old Installation, replace a held
claim or recreate disposable targets solely for inspection.
