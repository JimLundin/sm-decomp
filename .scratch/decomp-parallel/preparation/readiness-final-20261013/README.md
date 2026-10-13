# Replacement-root final readiness

Status: successor prepared, helpers verified and cleanup complete; actual root validation pending
Required actual caller: `01a13554-bc0d-7662-9378-28eecfa6122f`
Observer: `01a13555-fca7-7893-9b16-c9355ad3f516`

The crashed root's [original successor](../readiness-retry-20261013/README.md)
and manifest remain immutable. Original manifest SHA-256 is
`76e5c8541c32af2c1cd0f00bf43d9ec8f2f92b05bb6d53692a369228b0b8cd6b`;
all its listed artifacts authenticate. Its documented `production-validation-1`
directory is absent. Fresh direct thread inspection observes previous root
`01a13500-1ef8-73d3-9a0d-3446b6050eba` in `systemError` with a failed turn,
current root active/inProgress, and legacy Ceres parent failed. Both submitted
implementers and the reporting owner have completed turns. Fresh Rust `/proc`
inspection observes no Ceres path writer. Reporting server PID 420446 retains
boot `7119fcce-5593-4526-aa9b-07127767c4ef` and start ticks 2249195.

The generation-zero historical transfer retains all three exact Ceres claims,
worker identities and original coordinator `ceres-20261013`. Submitted controller
and host work and running host integration remain held. Current root receives
prospective generation 1 only inside the private readiness namespace. Host
production authority and canonical ledger remain absent. Original ledger
SHA-256 `77fdba066af4f4c045e8e8f419997563d569ac0bd8045cef0a68ac916a126b89`
and bridge `cb537f195b4f83529f2ea7a61bd65c6e77ad215366275e8d60a7a604c4eb0ba9`
remain unchanged. Observations are a tool trust boundary, not platform attestation.

The Rust helper uses the same frozen engine/source epoch. Its successor changes
require the actual replacement root, direct final drill retention of the sealed
post-publication interruption proof, and unchanged proof bytes during validation.
It binds that proof's final SHA-256 at actual root execution. Its unit regression
rejects a path/hash split across separate evidence entries. Locked/offline build
and test passed; wrong-caller and direct-inner controls rejected before output
creation or namespace launch. [helper-build.toml](helper-build.toml) seals source,
manifest, lock and retained binaries. [package-readiness.rs](package-readiness.rs)
provides preparation, controls, cleanup and immutable sealing commands.

## Actual root execution

Wait for the recovered drill worker's final stable receipt, regenerated actual
drill/execution manifests, retired worktrees and post-cleanup authentication.
Review the sealed supplemental proof at the exact path below. Recheck actual
owners/processes and inherit the root's real `CODEX_THREAD_ID` unchanged.
Authenticate this package with `sha256sum -c artifacts.sha256` from its directory.

```sh
/home/jim/sm-decomp/.scratch/decomp-parallel/preparation/readiness-final-20261013/bin/decomp-readiness \
  --engine /home/jim/sm-decomp/.scratch/decomp-parallel/drill-retry/bin/sm-pipeline \
  --source-checkout /tmp/sm-decomp-preparation-retry-source-20261013 \
  --source-commit 57f93c5ce4e6adac2dfe181b852d4aad6e0776bd \
  --source-fingerprint bd103af4b5d563904a3cbdbe62e971c46c6b4a4e907b60bbbb8575a2373355ec \
  --build /home/jim/sm-decomp/.scratch/decomp-parallel/drill-retry/engine-build.toml \
  --prepare-request /home/jim/sm-decomp/.scratch/decomp-parallel/preparation/readiness-final-20261013/prepare-request.toml \
  --drill /home/jim/sm-decomp/.scratch/decomp-parallel/drill-retry/concurrency-drill.toml \
  --post-publication-proof /home/jim/sm-decomp/.scratch/decomp-parallel/drill-retry/post-publication-interruption.toml \
  --output /home/jim/sm-decomp/.scratch/decomp-parallel/preparation/readiness-final-20261013/production-validation-1
```

Required engine SHA-256 is
`04f4a10aec71b9654cdc1c7f1aca6936f9a9bb7ce810f8232c955f3b11e7d573`.
Passing validation retains private policy state `prepared`, unchanged policy bytes,
unchanged host ledger seals/bridge/source/engine/drill/proof, and absent host
production authority/canonical ledger. All outputs and failed diagnostics are sealed
under the new output directory. Root completes both cleanup passes over that output,
authenticates its manifest and referenced retained inputs, then writes a separately
sealed completion receipt beside this immutable package. New observations require
another successor; original manifests and README bytes stay unchanged.

Both scoped package cleanup passes returned 0 bytes. Cargo clean removed 451 files,
79.0 MiB reported, with 64,644,986 logical target bytes measured before clean.
Available filesystem space was 221,312,856,064 bytes. The small bootstrap build
binary was additionally removed; source and all retained helper/readers/evidence
dependencies remain. No namespace validation ran during package preparation.

Production activation and gameplay launch require the user's separate instruction
and fresh exact owner/source/legacy observations from the final launch handoff.
