# Offline family verification

This crate owns complete actual-C Offline family verification: Fixture
declarations and their binary codec, catalog discovery, reviewed recipes,
compiler inputs, scheduling, state comparison and retained evidence admission.
The same private recipe descriptions drive ordinary execution, confined
execution and authentication of retained producer commands.

`OfflineFamilies` accepts trusted `HostInputs` from the application. Ordinary
callers use `verify`, `verify_registered` and `authenticate`. Registered
verification owns preparation and authentication before reuse; batch callers
only select families and aggregate results. An `AuthenticatedFamily` carries
the declaration authenticated with its evidence, so Live consumers do not
reopen an unauthenticated manifest.

Native callers declare a complete registered domain, then prepare an opaque
`PreparedNative`. Preparation checks frozen inputs and asks the application's
`ScopeAdmission` adapter about Translation approval. The handle owns resource
calculations, producer ownership, indexed receipts, verification and retained
authentication. Reading completed evidence requires no compiler or helper
launch. Serialized reports remain data; admission returns a distinct
authenticated value, including authenticated rejected Native outcomes.

`fixtures` exposes Fixture values and the current `SMFIXT01` codec. `catalog`
exposes declarations and profiles. `authoring` exposes explicit wrapper,
dependency-recording and fixed Fixture-worker preparation requests. Recipe
dispatch, compiler and Fixture commands, scheduling and operation traversal
remain private.

The crate depends on `decomp-source` for pinned Recovered C identities and
operation-local source reuse, `tool-execution` for generic build/process/input
operations, and `translation-execution` for confined producer execution and
receipts. It has no dependency on `pipeline`. CLI composition, Translation
approval, installed-controller identity, Live orchestration and cleanup stay
with the application.

`IMPLEMENTATION_SOURCES` supplies repository-relative compiled source stamps.
Verification adds these and the lower owners' stamps to the host's conservative
source declaration. Ordinary admission authenticates retained producer
provenance without requiring equality to today's whole application executable.
Installed Translation controllers retain their separate executable fence.

The current indexed evidence formats are preserved. Precise affected-family
selection and independence from unrelated CLI source changes are deferred.

Run owner tests after building the application cleanup executable:

```sh
cargo build --locked --offline -p pipeline --bin sm-pipeline
cargo test --locked --offline -p offline-verification
```

Set `SM_PIPELINE_CLEANUP_BIN` when the cleanup executable is retained elsewhere.
Application tests own the explicit real Native proofs and final run cleanup.
