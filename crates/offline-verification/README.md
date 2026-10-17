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

Batch callers capture one `CatalogSnapshot` and can capture `BatchInputs` before
an application lock wait. `verify_batch` owns lazy declaration decoding, captured
source facts, compatible compiler preparation, ready Fixture work and complete
per-family admission. `authenticate_batch` checks exact required membership and
starts independent evidence reads. Current-source checks reject a changed input
instead of mixing revisions. The private work graph is an advisory projection
over existing work; callers cannot submit arbitrary tasks or use it as a journal.

Ordinary input capture materializes the complete selected input set, then
authenticates current source and materialized compiler bytes. Each phase has a
1-GiB I/O ceiling and both share one absolute deadline. Without an enclosing
scope, their combined ceiling is 2 GiB. A caller's existing byte/deadline scope
still covers both phases and can impose a stricter aggregate limit. The separate
1-GiB declaration resident limit and Native limits are unchanged. Generated PLM
capture binds the complete trusted generator's inputs and the raw registration;
its obsolete placeholder ROM is not an effective input.

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

The current indexed evidence formats are preserved. A bounded reader shares
authenticated immutable payload bytes inside one admission; later admission
opens and authenticates them again. Syntax-aware dependency facts support
conservative affected-family selection. Incomplete facts stay affected, and
installed-controller identity remains separate from computation identity.

Run owner tests after building the application cleanup executable:

```sh
cargo build --profile verification --locked --offline -p pipeline --bin sm-pipeline
SM_PIPELINE_CLEANUP_BIN="$PWD/target/verification/sm-pipeline" cargo test --profile verification --locked --offline -p offline-verification
```

Set `SM_PIPELINE_CLEANUP_BIN` when the cleanup executable is retained elsewhere.
Application tests own the explicit real Native proofs and final run cleanup.
