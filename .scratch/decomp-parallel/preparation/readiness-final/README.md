# Final production readiness successor

Status: READY for root execution after fresh engine and drill seals are supplied
Owner: actual root `01a134cd-dafb-7e50-8284-f285a905e05b`
Observer: `01a134cf-a6ce-7523-9067-d8c00988129e`
Observed: 2026-10-13 10:34:54 UTC

The dependency-free Rust helper prepares a policy in a private `bwrap` mount
namespace and calls maintained `validate-activation`. Host production authority
and canonical ledger remain absent. Every host path is mounted read-only except
the new output directory and a private assignment lock. The outer helper holds
the real assignment lock without changing its bytes. Namespace identity and the
private tmpfs are checked before preparation. No gameplay or claim operation runs.

This successor preserves the original bridge and Ceres ledger seals. The previous
readiness and current-owner packages remain immutable. Six direct thread
inspections record the current root active/inProgress; previous preparation root
and Ceres parent systemError/failed; controller idle/completed; host
notLoaded/completed; reporting idle/completed. Fresh process inspection found no
retained Ceres path writer in cwd, argv or open descriptors. Reporting server
PID420446 retains boot `7119fcce-5593-4526-aa9b-07127767c4ef` and start ticks2249195.
Thread/process observations are a tool inspection trust boundary.

The generation-zero historical Ceres transfer accounts for all three exact held
claims and their original worker identities. It binds the current root at
prospective generation1 only inside the private policy view. Both submitted jobs
and running host integration remain held. No production owner-stop, completion,
release, registration or cutover occurs. Preparation root recovery is separately
sealed in the ownership record; the previous preparation policy never existed on
the host.

The reviewed new frozen subject is
`/tmp/sm-decomp-preparation-final-source-20261013`. Its commit/fingerprint and
binary/build fields are supplied at execution from root's exact retained seals.
The new engine contains both execution and validator source. The helper checks
Git HEAD and build identity, then the maintained validator recomputes the frozen
subject fingerprint and authenticates actual fresh drill evidence. No source or
binary digest has been invented. The retained build must have top-level fields
`source_checkout`, `source_commit`, `source_fingerprint`, `binary_path` and
`binary_sha256` agreeing exactly with the arguments.

Prospective root execution after supplying the sealed commit/fingerprint:

```sh
/tmp/sm-production-readiness-final-helper-20261013 \
  --engine /home/jim/sm-decomp/.scratch/decomp-parallel/drill-final/bin/sm-pipeline \
  --source-checkout /tmp/sm-decomp-preparation-final-source-20261013 \
  --source-commit "$DECOMP_FINAL_COMMIT" \
  --source-fingerprint "$DECOMP_FINAL_FINGERPRINT" \
  --build /home/jim/sm-decomp/.scratch/decomp-parallel/drill-final/engine-build.toml \
  --prepare-request /home/jim/sm-decomp/.scratch/decomp-parallel/preparation/readiness-final/prepare-request.toml \
  --drill /home/jim/sm-decomp/.scratch/decomp-parallel/drill-final/concurrency-drill.toml \
  --output /home/jim/sm-decomp/.scratch/decomp-parallel/preparation/readiness-final/production-validation-1
```

The actual caller must inherit root's `CODEX_THREAD_ID`; it must never be
overridden. Each output is new. Successful validation leaves private policy
state `prepared`, preserves its bytes and seals unchanged host ledger hashes.
Failed namespace runs retain and seal their operation diagnostics and host
preservation receipts. Fresh owner changes need another sealed successor.

Both Rust helpers compiled with `rustc 1.101.0-nightly`. Wrong-caller execution
rejected before any filesystem output, and direct inner execution rejected the
missing private namespace proof. Their stdout/stderr diagnostics are retained.
These checks are helper checks, not actual readiness validation.

No verification fixture or run/build tree was produced here. The disposable
process-inspector binary was removed after inspection, reclaiming 4,810,208 bytes.
The compiled readiness helper remains at
`/tmp/sm-production-readiness-final-helper-20261013`, size5,019,608 bytes, for
root's actual execution and later removal. Root completes both utility cleanup
passes over actual readiness output, reauthenticates retained manifests and
records final measured free space before closure. Available filesystem space
after inspector removal:221,048,549,376 bytes.

