# Switch reviewed production scopes to deterministic execution

Status: resolved
Blocked by: None
Triage: ready-for-agent

The user authorized production migration on 2026-10-14 and paused other work.
The production registry is active. Production smoke, unchanged replay,
retained authentication and measured cleanup are complete. Productive execution
of the original acceptance contracts remains open under
[ticket 10](10-original-acceptance-adapters.md). See [the spec](../spec.md).

## Acceptance

Migrated work uses the trusted supervisor and restricted rewriting. Unrestricted
dispatch cannot mutate migrated attempts or invoke their evidence producers.
Original lineage and historical inspection/authentication remain valid. Activation,
rollback/recovery steps and completed cleanup have durable evidence.

## Completion evidence

### Active migration — 2026-10-14

[Activation](../production-migration/activation.stdout.toml) reports `active` for
the exact [installed request](/home/jim/sm-decomp-production-execution/production-request.json).
The production registry at
`/home/jim/sm-decomp/.scratch/translation-execution/production.json` has SHA-256
`9cdac7ad8916ce323c368b443e1435f8a2669b5b6908f0aa8c2494dca7508be9`.
Its request has SHA-256
`2d4ccbb0c343c45d397eca3ceb1f37cda655fa135f329d237db80387661fa5ae`.
The sentinel at the historical migration-fence path also makes older producer
binaries refuse unrestricted work. Both `ceres-continuation` and
`decomp-parallel` now use the installed production ownership boundary.

[Integrator transfer](../production-migration/ceres-transfer.stdout.toml) and
[dispatcher transfer](../production-migration/dispatcher-transfer.stdout.toml)
record the observed current session `01a136a0-49bb-7690-a360-48def3c2c09b`,
with `ceres-20261013` generation 3 and `decomp-run-20261013` generation 2.
Original coordinator tokens, claim attempts, owners, results, observations,
history and abbreviated `base-head` values are retained.

| Original claim | Preserved state | Production treatment |
| --- | --- | --- |
| `shared-game-host` | Submitted, attempt 1, commit `73547105d870f8b05f8c6bcf55b768c223889fdb` | First imported source in the installed plan; no model rewrite |
| `integrate-shared-game-host` | Running, attempt 1 | Held under its original identity; only the linked imported submission can execute it |
| `compact-controller` | Submitted, attempt 1, commit `223f8a1b0c7382817b1290d13c7a41f4e722d5e1` | Ordered after shared-host completion; no replacement submission |
| `pause-equipment-render-offline` | Submitted, attempt 1, commit `fa6e9cd747acabcb4568ccd2c0fbf532a9525c18` | Explicitly stopped until its complete composite adapter and predecessor exist |

The installed command is:

```sh
/home/jim/sm-decomp-production-execution/bin/sm-pipeline \
  translation-production --checkout /home/jim/sm-decomp run
```

Use the same command with `status` for inspection or `recover` for exact
activation recovery. Activation does not claim completed translation,
integration, actual-C verification, Live replay or original-ROM parity.

### Remaining acceptance requirements

The installed legacy adapters authenticate original requirements and produce a
bounded readiness stop. Their completion authentication explicitly refuses
acceptance until [ticket 10](10-original-acceptance-adapters.md) supplies the
missing confined producers:

- Shared host: all 31 original Rust controls within 60 seconds, both material
  actual-C callback matrices (10 pairs, PLM/HDMA allocation and 17 ordered VRAM
  effects), and the current conservative Affected Offline catalog. Preserve
  conservative treatment of shared source and unknown dependencies. The earlier
  176-family count is historical preparation evidence, not a future coverage ceiling.
- Compact controller: eight unit controls, two public controls, the original
  60-second causal diagnostic through frame 2240 where feasible, and retained full/compact
  capture authentication. Its successor position does not bypass a stopped host.
- Pause equipment: all 607 actual-C Fixture pairs, including auxiliary direct
  animation, plus 10 Rust tests. Plain Native-family acceptance is explicitly
  refused because it cannot express this complete original contract.
- Sequential successful publication: a later scope must authenticate prior
  queue/source changes before acquiring its next frozen policy. The current
  full-ledger/source checks remain strict; ordering alone cannot rebase them.

No acceptance adapter was weakened to make these claims appear complete.

### Verification and cleanup status

The final targeted suites retained under `production-migration/verification`
passed: five legacy readiness tests, 14 managed migration tests (two retained
diagnostic tests ignored), one conservative affected-family selection test and
one native dependency-ownership test. Managed tests cover exact imported Ready
and Running integrations, unchanged repeated completion inspection, preserved
original submission bytes, stopped-owner checks and single-executor ownership.

The first production launch exited 1 with a typed stopped shared-host attempt:
`a41caf059f04b355a6e84c45e70617153cef1fbf7405ad2da9954adb468e74df`,
attempt 1. Its diagnostic names the missing confined Cargo execution, material
callback matrices and 176 affected Offline recipes. No scope completed. The
supervisor finalization reports `cleanup_complete`, `retained_authenticatable`
and `worker_retired`, with 102,220 retained bytes and 0 reclaimed bytes.
[The production checkpoint](../production-migration/checkpoint.md) records the
operator command and retained run identity.

[The second identical command](../production-migration/run-02.stdout.toml) also
exited 1 and returned byte-identical output. All 45 production-file hashes stayed
identical after the second run: no extra attempt, model call, gate or claim was
created. The stopped predecessor left its successor unstarted.
[Direct post-cleanup inspection](../production-migration/inspect-after-cleanup.stdout.toml)
exited 0 and authenticated the same stopped attempt. All 45 production hashes
and all 58 historical model-launch hashes passed after cleanup.

Ordinary and deduplicating cleanup for the production and verification roots
exited 0, reclaiming 0 bytes. The final parent migration
[ordinary](../production-migration/migration-cleanup.stdout.toml) and
[deduplicating](../production-migration/migration-deduplicate.stdout.toml) passes
also exited 0 with 0 bytes reclaimed. The
[disposal apply receipt](../production-migration/disposal-apply.jsonl) records
4,207,966,325 logical bytes removed and 3,761,475,584 reclaimable allocated bytes.
[Disposal-helper retirement](../production-migration/cleanup-helper-retirement.txt)
removed a further 4,597,048 logical bytes and 4,599,808 allocated bytes. Combined
measured disposal is 4,212,563,373 logical bytes and 3,766,075,392 allocated bytes;
observed available space afterward is 186,157,821,952 bytes. Retained evidence
authentication passed after this removal. No cleanup obligation remains for
this migration ticket; original-work acceptance remains separately open.

The scoped [disposal helper](../production-migration/cleanup_outputs.rs) permits
only `/tmp/sm-production-migration-target-20261014`,
`/tmp/sm-production-capture-recovery-20261014` and six temporary compile-time C
copies whose hashes must match `verification/compile-inputs.sha256`.
All selected writers finished before its explicit `--apply` mode was run.

Retain `/home/jim/sm-decomp-production-execution`,
`/home/jim/sm-decomp-production-inputs`, the installed production binaries,
both original-claim preparation packages, their source snapshots, activation
records, authority-transfer evidence and every referenced historical receipt.
The architecture worktree `/home/jim/sm-decomp-translation-execution` remains
an active tools/evidence dependency of the installed request and retained proofs;
it is not an integrated gameplay implementer scheduled for disposal.
Original Submitted/Running work has not completed integration; its remaining
owner worktrees are not disposable integration outputs. In particular,
`/tmp/sm-ceres-continuation-compact-controller-1` remains registered at its exact
Submitted commit. Historical preparation worktrees (`sm-decomp-preparation-`
final-source/integration/retry-source and `sm-decomp-ticket-validator-20261013`)
also remain registered; this turn's disposal list grants no authority to remove
them. Their retained reference closures require a separate audit before any
later retirement.
Historical readers and inputs remain required, particularly
`/tmp/sm-complete-supervisor-976030-0`, `model-launch/native-02`, its
`approved-native/source/.git`, `operator-run-01`, private ROM/C/provenance inputs,
`supervisor-native/tools/sm-supervisor-retained-reader-02` and the retained
managed 08/09/10/11 controllers. Prior proof cleanup is complete and should not
be rerun or broadened into these dependencies.

### Historical preparation

[Read-only original Ceres preparation](../production-preparation/preparation-checkpoint-27b8b0a.md)
was built and separately revalidated against the complete retained native proof
and unchanged main authority/source/ledgers/progress. Original Submitted source
commits, Running linked integration source, claim tuples, owners, results,
observations and abbreviated `base-head` bytes are preserved. Identical builder
retries return the original receipt and run no new validator. Historical initial
read-budget refusal remains immutable; the same package's later bounded read-only
report passes as `validated-with-blockers` and `activated = false`.

That earlier package and [proof completion](../completion-20261014.md) describe
the state before this switch and remain immutable historical evidence. Current
activation uses the newly prepared `ceres-production-01` and
`canonical-production-01` packages. The user decision, checked role transfers,
imported-submission mechanism, ownership activation, replay authentication and
cleanup are now complete. The acceptance work in ticket 10 remains open; this
completion does not establish gameplay readiness.
