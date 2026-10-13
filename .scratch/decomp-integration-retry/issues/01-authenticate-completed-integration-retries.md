# Authenticate completed integration retries after later merges

Status: resolved
Blocked by: None
Triage: ready-for-agent
Type: task
Owner: `/root/drill_review`
Coordinator: `01a13559-e517-72d1-88b5-e554fed61da4`
Observed source: `2d59536c059a1ca9c31d9278bff2d9222eafa56c`

An exact completed infrastructure integration retry should return its existing
authenticated admission without rerunning gates. The current cache checks the
shared checkout's present HEAD/fingerprint against A's original accepted source.
After legitimate disjoint B integration advances HEAD, A's retry rejects even
though A/B integration claims are both complete and their sealed results remain
authenticatable. B's newest-HEAD retry succeeds.

## Cause evidence

Actual isolated drill at
`/tmp/sm-decomp-concurrency-drill-final-20261013`, engine source
`2d59536c059a1ca9c31d9278bff2d9222eafa56c`, retained binary SHA-256
`df6eb4dc3a9fd69e5d5923b5c22f34dea7b434cd7ee87f1b69085cd9e9431c84`.
Actual B session `01a134dc-6c44-78d1-8168-fc21f3499b55` owns current
`drill-integrator` generation2.

- A exact retry execution:
  `.scratch/drill-control/receipts/integrated-exact-retry-481780-1791891128704929679.execution.toml`
  inside that fixture; SHA-256
  `8597d91cd1591ce630f565b0a95be3f3e7753503189451080cea5ceacaa1e45c`,
  exit1, stderr `Error: cached infrastructure source changed`.
- Its sealed request SHA-256 is
  `18978d52b8a19bf8d1788c5b7382554f61e3c2487968dd726ad09dc5964e983d`.
  Claim is `(integrate-lane-a,
  5eae3f5aab7d9ff316e3ad9068571ad92866bf3a5a382fbacea4b6b10c5435d4-2,
  1, drill-integrator)`, exact submission `(lane-a,
  5eae3f5aab7d9ff316e3ad9068571ad92866bf3a5a382fbacea4b6b10c5435d4,
  2, drill-a)`.
- A saved infrastructure-result publication records source commit
  `682e7e8c78026e35ca32739f1c77de3e14c37a7f`, fingerprint
  `cc0a62da36459475ce6ff6acf1ff22c8fba1556207405027f60cc83f0bfd42d4`.
  After completed B integration, shared HEAD is
  `1381537d506a21c392401acebe0fe94fcb50d331`; the ledger records both
  `integrate-lane-a` and `integrate-lane-b` complete.
- B exact retry execution:
  `.scratch/drill-control/receipts/integrated-exact-retry-482343-1791891179651521531.execution.toml`;
  SHA-256 `eba7ef59a3b132f75abbbf2028dd474d509634cf3007b532c40a0ea2fa45651b`,
  exit0 at unchanged newest HEAD.

`crates/pipeline/src/session.rs` cached `ResultPublication` path rechecks sealed
admitted artifacts, then requires checkout clean, current HEAD equal saved
`source_commit` and current fingerprint equal saved `source_fingerprint` whenever
`.git` exists. Integration uses the shared checkout, so legitimate later accepted
merges make this historical completed replay fail. These exact receipts and
diagnostics remain historical; do not roll the shared source back to obtain a
passing retry.

## Work and acceptance

Use the worker's isolated approved source/test paths. Add a meaningful Rust
regression reproducing A completion, disjoint B completion and subsequent A
exact replay with retired implementer worktrees. Preserve red/green receipts.

Completed A replay must return the exact original admitted result and original
sealed gate/authentication/cleanup identity after B advances shared HEAD. It runs
no gate again, creates no new acceptance, and changes no original publication.
Authenticate the completed historical source and exact integrated/submission
lineage using retained sealed evidence; current source ancestry alone does not
prove historical acceptance.

Keep current real integrator role/generation and exact claim/revision/attempt/
request/scope checks on every retry. Reject changed request bytes, stale role or
attempt, changed/missing seals, conflicting completed lineage and in-progress
source drift. Historical replay acceptance must not relax source checks for a
running or unpublished integration. Run focused locked/offline maintained
session/assignment/authority tests and record limitations.

This changes mutation/admission behavior. Root must freeze the new exact
source/path/fingerprint/binary and run a fresh actual two-independent-session
drill: two serial authenticated integrations, both A/B completed exact retries
after final merged HEAD and implementer retirement, no gate reruns, all eleven
controls, current-root private readiness validation and post-cleanup
authentication. Older `2d59536` drill manifests cannot validate the new subject.
Preparation remains pending until these required gates pass.

## Completion evidence

Scoped Rust commit `57f93c5ce4e6adac2dfe181b852d4aad6e0776bd`, integrated as
`0d042e0`, retains red/probe failures and green/tamper/dirty/stale/ancestry controls
in [worker handoff](../handoff.toml) and worker source/evidence manifests.
Assignment 23, session 17 and authority 11 controls passed; formatting was clean.
Frozen source/build identity is in
[engine-build.toml](../../decomp-parallel/drill-retry/engine-build.toml), SHA-256
`cdf1295fdc8ad86b574866636f68966205e5c46bf2fd19649fa9a4dd2f5bdbb9`.

Fresh actual [drill](../../decomp-parallel/drill-retry/concurrency-drill.toml),
SHA-256 `63912c67e38680fd163d09126ba285f8ce26da8a2445c801f489f8971b74751f`, passed both serial integrations and exact completed
A/B replays after final B HEAD/worktree retirement with unchanged original
publications/gates and zero reruns. Retry proof `.scratch/decomp-parallel/drill-retry/post-b-retry.toml`, SHA-256
`f8ab8db23c0e795f574b4c59be8cadfc5b3349c5300a77070152cd249030a954`. Current-root private readiness passed and preserved
host authority/ledger absence; [ticket04](../../decomp-parallel/issues/04-concurrency-and-crash-recovery-drill.md)
retains final role/manifest/host-preservation evidence.

Both cleanup passes and worker worktree/target retirement completed; source,
binary/readers and sealed red/green/drill evidence remain retained/authenticatable.
Cleanup `.scratch/decomp-integration-retry/integrator-cleanup.md`, reclaimed `1,909,161,087`
bytes; final free space `221,230,100,480` bytes. Post-cleanup
authentication `.scratch/decomp-parallel/drill-retry/metadata-closure-20261013/independent-authentication.json`; root checkpoint
`.scratch/decomp-parallel/preparation/final-checkpoint-20261013.md`.

## Comments

2026-10-13: Created from the actual completed-A retry failure. Root delegates
the source correction to `/root/drill_review`; this ticket creates no gameplay
claim or production authority mutation. The earlier preparation documentation
proposal remains pending and historical drill evidence remains unchanged.
