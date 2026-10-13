# Corrected-source readiness successor

Status: helper and corrected engine sealed; actual drill/readiness validation pending
Required actual caller: `01a13500-1ef8-73d3-9a0d-3446b6050eba`
Observer: `01a13502-c69b-7d73-8bdb-1d6acc07f12b`

This package prepares a policy only inside a private `bwrap` namespace and calls
maintained `validate-activation`. Host authority/canonical ledger remain absent;
no gameplay or production queue operation runs. The helper holds the host
assignment lock without changing its bytes and uses a separate private lock.
Root executes actual validation after the corrected fresh drill/cleanup finish.

[Predecessor](../readiness-recovery-20261013/README.md) evidence remains unchanged;
its `artifacts.sha256` SHA-256 is
`96c408f3e5422c1cd5db953f669490c2b23026d558667ce11745a3702fc143f1`.
Fresh thread inspection observed current root active/inProgress, prior preparation
and Ceres parent failed, controller/host and reporting completed. The Rust process
scan found no retained Ceres writer in cwd/argv/open descriptors. Reporting server
PID 420446 retains boot `7119fcce-5593-4526-aa9b-07127767c4ef`, start ticks 2249195.
Prior isolated drill coordinator remains active; absence of matching processes
releases no claim. Actual observations remain a tool trust boundary.

The generation-zero historical transfer retains all three exact Ceres claims,
worker identities and original coordinator `ceres-20261013`. Controller/host
submissions and running host integration remain held. It registers current root
at prospective generation 1 only inside the namespace. Original ledger SHA-256
`77fdba066af4f4c045e8e8f419997563d569ac0bd8045cef0a68ac916a126b89` and bridge
`cb537f195b4f83529f2ea7a61bd65c6e77ad215366275e8d60a7a604c4eb0ba9` authenticate.

The retained helper is identified by [helper-build.toml](helper-build.toml).
Readiness helper source differs from predecessor only in the new frozen subject/output
paths. The inspector adds retry-drill paths and omits unrelated generic main-cwd
processes. `package-readiness.rs` generated/sealed this package from fresh
observations; it runs no namespace validation. Locked/offline build passed.

## Root execution

Frozen subject: `/tmp/sm-decomp-preparation-retry-source-20261013`.
Source commit: `57f93c5ce4e6adac2dfe181b852d4aad6e0776bd`.
Source fingerprint:
`bd103af4b5d563904a3cbdbe62e971c46c6b4a4e907b60bbbb8575a2373355ec`.
Engine SHA-256 supplied by root:
`04f4a10aec71b9654cdc1c7f1aca6936f9a9bb7ce810f8232c955f3b11e7d573`.
Root created the clean frozen checkout and reports main integration `0d042e0`
with no Cargo/crates/tools delta.
[engine-build.toml](../../drill-retry/engine-build.toml) SHA-256 is
`cdf1295fdc8ad86b574866636f68966205e5c46bf2fd19649fa9a4dd2f5bdbb9`,
checkpoint `9acff17`. **PENDING:** final actual drill and readiness verdict.
Fresh drill is `drill-retry/concurrency-drill.toml`, effort `concurrency-drill-retry`.
Retain both completed retries at final merged HEAD, all eleven controls, retired
workers and post-cleanup authentication before this command. Inherit actual root
`CODEX_THREAD_ID`; every output directory is new.

```sh
/home/jim/sm-decomp/.scratch/decomp-parallel/preparation/readiness-retry-20261013/bin/decomp-readiness \
  --engine /home/jim/sm-decomp/.scratch/decomp-parallel/drill-retry/bin/sm-pipeline \
  --source-checkout /tmp/sm-decomp-preparation-retry-source-20261013 \
  --source-commit 57f93c5ce4e6adac2dfe181b852d4aad6e0776bd \
  --source-fingerprint bd103af4b5d563904a3cbdbe62e971c46c6b4a4e907b60bbbb8575a2373355ec \
  --build /home/jim/sm-decomp/.scratch/decomp-parallel/drill-retry/engine-build.toml \
  --prepare-request /home/jim/sm-decomp/.scratch/decomp-parallel/preparation/readiness-retry-20261013/prepare-request.toml \
  --drill /home/jim/sm-decomp/.scratch/decomp-parallel/drill-retry/concurrency-drill.toml \
  --output /home/jim/sm-decomp/.scratch/decomp-parallel/preparation/readiness-retry-20261013/production-validation-1
```

Passing validation must retain private policy state `prepared`, unchanged policy
bytes and host-ledger/authority preservation receipts. Failure retains operation
diagnostics and host preservation. Changed owners need another sealed successor.
Later production activation still follows the user's separate gameplay launch
instruction and fresh role/source/legacy inspection.

No namespace validation ran during package preparation. Wrong-caller and direct
inner guard checks rejected before output/namespace launch; their exit 1 diagnostics
are sealed. Both package cleanup utility passes returned 0 bytes. Cargo clean
removed 344 private build files, 53.6 MiB reported, 38,051,234 logical target bytes
measured before removal. Available filesystem space was 221,274,275,840 bytes.
The helper remains retained outside the disposable target. Root completes both
cleanup passes over actual validation output, retains exact source/readers/
evidence dependencies and authenticates before declaring preparation ready.
