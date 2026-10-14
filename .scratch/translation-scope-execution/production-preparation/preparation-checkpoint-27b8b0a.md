# Original Ceres migration preparation checkpoint

Status: read-only preparation validated with blockers; production switch pending
Owner: `01a136a0-49bb-7690-a360-48def3c2c09b`
Worktree: `/home/jim/sm-decomp-translation-execution`
Source checkpoint: `27b8b0ac78080d9ccc953956e1829bf0571efbe2`

The Rust builder captured the three original Ceres claims into
[ceres-actual-c-02/package/migration.json](ceres-actual-c-02/package/migration.json).
The original Job, Claim, state, agent, result, observations and abbreviated
`base-head = "48656a2"` remain intact. Submitted source is the original
`223f8a1b0c7382817b1290d13c7a41f4e722d5e1` controller and
`73547105d870f8b05f8c6bcf55b768c223889fdb` host commits. Running linked integration
uses the explicitly supplied main HEAD
`c2cae9ac145f9343cd345e6f978bff0774510b56`. Required writable source snapshots are
small regular Git blobs, with no Git metadata.

The retained build receipt is `prepared-with-blockers`, `activated = false`,
281,128 referenced bytes and 52 referenced files. The initial validator correctly
refused when full proof authentication followed by the production catalog exceeded
the generic 1 GiB logical read budget. That historical `validation.json` and
`build-receipt.json` remain unchanged. Identical retries before and after the
bounded-read fix return the same receipt bytes and execute no new validator.

A separate later [read-only report](ceres-actual-c-02.validation-final-02.stdout.toml)
passes as `validated-with-blockers`. It authenticates all three exact original
lineages and stopped-owner observations, current generation2 authority and the
complete retained actual-C supervisor proof. The report describes 46 direct
references / 250,813 bytes separately from 60,107,351 proof-finalization bytes.
[Revalidation record](ceres-actual-c-02.revalidation.json) binds this report to the
unchanged migration and historical refusal.

Production preparation has an explicit 300-second/8-GiB logical IO budget.
Nested reads retain the earlier deadline and aggregate usage; unique migration
reference closure stays at 1 GiB. This is a read-only authentication allowance,
not a producer output limit. The red regression reproduces the real package
failure in 21.51 seconds; the same interface passes after the fix in 161.17
seconds and proves unchanged authority, ledger, registry, package decision and
retained producer records. Logs are `verification/migration-read-budget-{red,green}-01`.

The package remains unsuitable for activation:

- None of the three originals has an exact reviewed production installation or
  policy. Submitted and linked Integrate lineage cannot be rewritten as a new
  Running native Implement claim.
- Shared host Submitted/Integrate claims retain overlapping held ownership;
  managed sequencing for this pair is unsupported.
- The catalog has specialized native recipes and families outside the closed
  enemy-jump-instructions proof. A proof for one isolated scope cannot approve
  another family or original claim.
- Canonical queue work, including `pause-equipment-render`, remains outside this
  Ceres package. Current actual owners/processes must be reinspected at a later
  switch; retained observations are historical.
- Production activation requires the separate explicit user switch decision
  agreed in the spec, current checked authority and exact reviewed preparations.

No production lock, owner-stopped queue event, claim, authority, source/index or
progress update was written. The builder mutates only its private package root.
Main HEAD and the four production hashes recorded in the recovery handoff remain
unchanged. Completion cleanup belongs to the root checkpoint after all writers
finish; package snapshots, observations, immutable build/retry decisions, proof
source, reader and private source/ROM dependencies must remain available.
