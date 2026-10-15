# Ticket 02: catalog and immutable input flow

Status: research complete; implementation not started here.

Inspected the clean frozen extraction checkout
`/home/jim/sm-decomp-offline-verifier` at
`8bf8acf610e1a94a1a3bccd5d718dae4a4244857`. Source locations below refer to
that checkout. Read the root instructions, domain rule, glossary, architecture,
validation plan and ticket 02. No applicable ADR directory exists.

## Recommendation

Add an opaque invocation-owned catalog/input snapshot in `offline-verification`
and ordinary bulk verification/admission methods that consume it. Selection gets
read-only family facts from that same snapshot. Keep freshness validation as
explicit rereads; remove repeat catalog construction and declaration decoding,
not the checks that those rereads currently provide.

Extend the existing `prepare_native_catalog` implementation with the same private
declaration cache. Preserve the opaque `PreparedNative` handle, lazy preparation,
scope approval timing, individual producer contracts and attempt authority. A
new Native scheduler is unnecessary for this ticket.

There is a meaningful distinction between the first implementation slice and
the full ticket: a shared catalog/declaration cache alone does **not** make all
compiler inputs immutable. Ordinary recipes still compile paths in the mutable
checkout. The source-view plumbing described below is needed before claiming
the ticket's coherent captured input guarantee.

## Exact current flow and redundant work

| Flow | Source locations | Present behavior |
| --- | --- | --- |
| Application catalog | `crates/pipeline/src/batch.rs:244` | `operate_inner` calls `catalog` at 246 before resolving profiles. |
| Selection catalog | `crates/pipeline/src/affected.rs:467` | `select_changed` hashes `batch.toml`, builds another catalog with its bounded reader at 479, then hashes `batch.toml` again at 485. |
| Selection freshness | `crates/pipeline/src/batch.rs:260` | After selection, 263 constructs a third catalog and compares its serialized meaning with the original. This catches profile/registration/discovered membership changes. |
| Each ordinary family | `crates/pipeline/src/batch.rs:160`, `crates/offline-verification/src/ordinary.rs:33` | Every `verify_one` constructs an `OfflineFamilies` and calls `verify_registered`; that method constructs another entire catalog at 35. It hashes the selected manifest at 56. |
| Discovery decoding | `crates/offline-verification/src/catalog.rs:36` | Reads `batch.toml`, lists `tools/port`, and fully decodes each unregistered `.smf` at 52 solely to decide dispatcher versus family. The decoded manifest is then discarded. Explicitly registered entries are not decoded during catalog construction. |
| Selection decoding | `crates/pipeline/src/affected.rs:521` | Reads and fully decodes each selected manifest at 527–529, even if discovery just decoded it. Discovery source facts have their own operation-local maps at 255. |
| Fresh ordinary verification | `crates/offline-verification/src/family.rs:667` | Reads manifest bytes and digest at 687, writes the exact run copy at 688, decodes again at 689, then normalizes by recipe. Rehashes original and retained manifest at 749 before success. |
| Ordinary post-run admission | `crates/offline-verification/src/lib.rs:58`, `crates/offline-verification/src/family.rs:790` | `verify` immediately authenticates successful evidence. Admission inventories at 801, then reads and decodes the retained manifest at 808. Current source manifest remains checked at 864–876. |
| Public admitted declaration | `crates/offline-verification/src/lib.rs:111` | `AuthenticatedFamily::from_report` reads and decodes the same admitted manifest again at 113, then independently repeats PLM normalization. Returning the already admitted declaration removes this extra decode directly. |
| Native declaration | `crates/offline-verification/src/native.rs:485` | Reads the declaration at 498; recipe registration reads/parses `batch.toml` through `recipe.rs:179`. `contract_for_declared` hashes the manifest again at 590 and 609. |
| Native preparation | `crates/offline-verification/src/native.rs:733` | Reads authenticated manifest at 745, repeats recipe registration at 747, normalizes, and checks complete coverage/frozen inputs. Reviewed-catalog provenance reads/parses its sealed catalog at 635. |
| Native reopened handle | `crates/offline-verification/src/native.rs:932` | `declaration` reopens authenticated manifest and normalizes; `open_prepared` calls it at 970. Both execution and independent authentication call `open_prepared`. |
| Native retained declaration | `crates/offline-verification/src/native.rs:1437` | Family admission is followed by another retained declaration read at 1446. Reviewed-Offline provenance similarly rereads after family admission at 675. |

The three constructions describe a `batch plan`/changed-since planning path.
Whole fresh ordinary batches additionally construct the catalog once **per
family**. Fixing this benefits full required integration as well as selection.
These call counts are source observations, not measured speedup claims.

## Small public interface

The exact names can follow the implementation style, but the required separation
is concrete:

```rust
// Public, opaque, non-deserializable; private Arc-owned implementation.
pub struct CatalogSnapshot { /* ... */ }

impl CatalogSnapshot {
    pub fn catalog(&self) -> &catalog::Catalog;
    pub fn family(&self, name: &str) -> Result<FamilyFacts>;
    pub fn validate_current(&self) -> Result<()>;
}

// Read-only projections; no private Recipe, argv or producer adapter exposed.
pub struct FamilyFacts { /* private Arc-owned fields; read-only accessors */ }
// Accessors: registered name/kind/path, exact original digest, &Manifest.

impl OfflineFamilies {
    pub fn capture_catalog(&self, request: CatalogRequest) -> Result<CatalogSnapshot>;
    pub fn verify_batch(&self, request: BatchVerification<'_>) -> Result<BatchRun>;
    pub fn authenticate_batch(&self, request: BatchAdmission<'_>) -> Result<AuthenticatedBatch>;
}
```

`CatalogRequest` binds canonical checkout, deadline, input bytes and retained
memory limits. It must work without ROM/runtime/compiler access for metadata
planning. Do not make a catalog command authenticate the entire Recovered C
source just to expose profiles.

`BatchVerification` contains the snapshot, source/root/shared capacity settings,
family jobs/case jobs and exact selected registered names with output and optional
`PreviousFamily`. It cannot supply a `Recipe`, arbitrary command or a case subset.
The returned per-family outcomes preserve passing, retained rejection and
pre-execution error distinctly, in canonical request order. `pipeline` continues
owning selection policy, its batch lock/record, Live composition and cleanup.

Publication must remain incremental. Currently `batch.rs:353–357` writes each
completed family before others finish. Use a narrow result observer or an owning
completion iterator so `pipeline` can persist each outcome; returning only a
final vector would lose current interruption/recovery behavior. The callback
can publish a report, not alter the work, requirements or producer contract.

`BatchAdmission` binds a complete selected requirement set and exact evidence
references. Reject missing/duplicate/extra bindings and registration/digest
mismatches before constructing `AuthenticatedBatch`. Its private constructor
contains admitted families, not deserialized success reports. It has no compiler,
sandbox or execution environment requirement. A fresh admission invocation owns
a new validation scope even if execution just completed in the same process.

Keep `verify_registered` as the one-family convenience entry: capture a snapshot
and delegate to the same internal batch implementation with one selected name.
Keep standalone `verify(VerifyRequest)` working for explicitly supplied manifests
that do not require catalog registration; it uses the same declaration capture
and family execution function. Likewise, `authenticate` delegates to the shared
admission implementation without requiring a catalog for a previously supported
standalone declaration.

## Private representation and bounded capture

Use one private context containing:

- Canonical checkout and complete captured `tools/port` discovery membership.
- Exact `batch.toml` bytes/digest, its parsed explicit registration catalog, and
  the discovery-expanded catalog. Preserve both views: Native registration has
  deliberately stricter alias rules than ordinary discovery.
- A map of logical manifest paths to bound original identities and shared
  captured declarations. Discovery retains its decodes; explicit declarations
  load lazily when requested.
- A content-keyed decode table for identical immutable SMF bytes, independent of
  the per-path identity map. A path binding must remain authenticated even when
  another path has the same contents.
- Separately bound normalized declarations keyed by original content plus recipe
  inputs. PLM/generated PLM/direct animation cannot reuse a normalization based
  only on the original file digest.
- Source facts and captured byte references, with explicit retained-memory
  accounting and an input-read/deadline scope.

Existing `fixture_format::decode` already validates extents and stores one
`Arc<Vec<u8>>` across all packed case spans (`fixture_format.rs:206–256`).
`Writes` is copy-on-write. Extend decoding to accept/retain that existing shared
buffer rather than storing another complete byte copy next to the manifest.
Keep an `Arc<Manifest>` for immutable original declarations; recipe-specific
normalization clones may share their packed bytes. Avoid sharing a mutable
`Manifest` between recipes.

Use per-entry once initialization/locking so two family workers cannot decode
the same uncached content twice. Do not hold one global catalog lock through
full reads, hashing or execution. A cached digest alone is not captured input:
the context must own bytes or a checked immutable content reference.

In-process callers consume owned bytes. Assets already declaring a digest may
load lazily only against that bound digest; changed bytes refuse. Initial
implementation need not change SMFIXT01, write metadata sidecars, or parse old
formats. Its monolithic file still requires a full read for authentication.
Record freshness reads separately from decode counts.

Memory and aggregate I/O are different limits. `bounded` accounts I/O but has
no resident-cache capacity abstraction. A first bounded implementation can keep
all captured declarations up to a declared memory ceiling and refuse beyond it.
Evicting and decoding the same file again would violate the stated one-decode
target; a later lazy payload representation needs to preserve decoded metadata
while revalidating payload bytes on reload. Do not claim a memory-bounded cache
merely because each file is below 64 MiB.

## Selection and freshness integration

1. `batch::operate_inner` captures the snapshot once and resolves profile/names
   from its catalog projection.
2. Add a `select_with_snapshot`/`select_until_with_snapshot` path in `affected`.
   `select_changed` receives the snapshot and consumes its `FamilyFacts` instead
   of invoking `catalog_using` and `fixtures::decode` itself.
3. Existing selection entry points used by assignment/Original scope planning
   create their own snapshot inside their bounded operation and delegate.
   `select_paths_until` must continue selecting over the entire catalog.
4. Preserve `Report.planning_inputs` with the captured catalog and manifest
   identities; preserve original source/dependency/asset identities and reasons.
5. Replace the third parsed-catalog comparison with `validate_current`: check the
   discovery namespace, exact `batch.toml` identity and all captured declarations
   used to establish catalog meaning or selection. Rehash without redecoding.
   New/deleted `.smf` entries, redirection, changed registration/profile data, or
   changed discovered dispatcher classification must not silently pass.
6. Keep `affected.rs:574–593` source/module-namespace stability behavior. Unknown
   dependencies remain affected, and budget exhaustion remains a hard error,
   rather than becoming ordinary conservative uncertainty.
7. Execution revalidates the snapshot after batch/selection-lock waits, before
   using it, and at publication. It refuses changes; it never recaptures a newer
   catalog into the same launched batch.

The current third check compares semantic catalogs. Comparing exact captured
configuration bytes plus directory membership and consulted declaration hashes
is a safe stricter implementation. If retaining semantic equivalence for comment
edits is desirable, parse only a changed small `batch.toml`; never reconstruct
every discovered manifest to do that comparison. Document that choice.

## Source abstractions to extend, not replace

`decomp-source/src/context.rs:33–80` already supplies `SourceContext`,
`shared_scope`, nested `limited`/`until`, and worker `enter`. Keep declaration
caching in `offline-verification`; `decomp-source` must not depend on its
`Manifest` or recipes. `decomp-source/src/lib.rs:49–84` already caches authenticated
`Source` facts, but every cache reuse checks the archive and every pinned file.
It is expressly not an immutable-source-byte cache.

`tool-execution/src/bounded.rs:249–305` provides coherent single-read bytes/hash,
regular-file and no-symlink checks, and before/after file-version validation.
`immutable_copy` at 345 and `immutable_bytes` are existing bounded publication
primitives. Retain all path and byte accounting when extending them.

`tool-execution::build_cache::Preparation` (`build_cache.rs:83–97`) freezes
expected source identities at `freeze` (160), rechecks after its lock wait
(253), and finishes by rechecking all observed inputs/tool/environment (325).
Keep those checks. A catalog snapshot must not turn `digest(path)` into a generic
cached result: many calls are intentional freshness checks.

For complete ticket 02 input coherence, add a private source view that separates
logical original paths from materialized captured compiler-input paths:

- `family.rs:303–350` and `inputs.rs:232–285` currently generate wrappers that
  reference the live checkout. `family.rs:450–475` snapshots/hashes candidate and
  dependencies only after compilation. Callback, PLM and Ridley have equivalent
  paths in `callback.rs:34`, `plm_spawn.rs:45`, and
  `ridley_subsystem_adapter.rs:147`.
- Capture the conservative required Rust/C/harness input set under its current
  ownership fence, materialize exact bytes into the owned execution input view,
  and keep original logical source paths in evidence. Do not make the temporary
  capture root become `Implementation.source_root` accidentally.
- Map compiler-reported dependencies back to those logical inputs; refuse an
  unbound/new input instead of silently extending the source set after launch.
  Preserve compiler dependency checks in `inputs.rs:318–393` and tool execution.
- Per-case captures and ROM must also use authenticated captured bytes. Existing
  `family.rs:75–138` is the common case-input seam; different original source
  versions cannot become different families' input sets in one accepted batch.
- Keep current-source pre-publication admission checks at `family.rs:864–876`
  and 901–915, and the Recovered C pin. A frozen computation can still be refused
  when the current requested source no longer matches it.

That change requires recipe-specific input-path plumbing, but no new build cache
or build-sharing algorithm. Content/build identity narrowing remains ticket 03/04.

**Budget pitfall:** wrapping all existing full-family reads in one default
`SourceContext::limited` will accidentally turn formerly separate family I/O
budgets into a 1 GiB whole-batch budget. `bounded::until` also caps the maximum at
8 GiB and takes the minimum of inherited budgets (bounded.rs:77–113); every
artifact inventory is charged. Use an explicit bounded preparation scope and
preserve required per-family execution/admission scopes, with shared input-memory
and preparation counters. Do not claim aggregate whole-batch output-I/O bounds
from a preparation-only counter, and do not quietly increase a Native frozen
deadline/quota. Test nested/worker accounting intentionally.

## Native constraints and minimal changes

`OfflineFamilies::prepare_native_catalog` (`native.rs:238–265`) already returns a
lazy iterator sharing `NativeCatalogInputs` (275). Extend that private context
with authenticated parsed registration/reviewed catalogs and captured declarations.
It currently shares the Recovered C source inventory, not the family catalog.

Keep `recipe::registered_entry`'s explicit-catalog alias rules
(`recipe.rs:179–207`): an explicitly registered alias path may be valid, whereas
a discovered declaration must use `tools/port/{declared.family}.smf`. Do not
replace it with a search over the expanded catalog and accidentally admit aliases.
Expose only family facts to `pipeline`; `Recipe` and `Registration` stay private.

`PreparedNative.manifest` is `serde(skip)` (`native.rs:94–97`). A serialized handle
does not carry trusted decoded bytes. Each separate `verify`/`authenticate` must
reopen and authenticate the required current declaration. Inside one operation,
reuse its authenticated owned declaration after retaining that reopen/identity
check. Never let an old handle's cache bypass `declaration`'s
`verification_sha256` check (`native.rs:932–939`).

Preparation reads the reviewed baseline, while execution reads the candidate
overlay. `translation_original.rs:669–686` explicitly enforces this distinction;
candidate variability is a separately approved set. A shared catalog cache may
share matching declaration contents, but it must not replace candidate binding
with a baseline-only checkout key. Preserve `check_candidate`, candidate digest,
and saved intent validation at `native.rs:976–1015` and 1037–1110.

`translation_original.rs:682–728` prepares, runs, authenticates and durably retains
one family at a time, stopping on a failed receipt. Do not eagerly collect its
preparation iterator, inspect later candidates, change scope approval order, or
move compiler discovery before existing retained-receipt reconciliation. Expired
saved failures must still authenticate without runtime tools (`native.rs:1049`).

Refactor the private family admission return into a report **and its authenticated
original and effective declarations**, sharing their packed bytes.
`AuthenticatedFamily::from_report`, reviewed-Offline
provenance (`native.rs:667–678`), and final Native admission (`native.rs:1437–1449`)
can consume the appropriate original/effective view instead of rereading the
retained manifest. Preserve each caller's existing normalization and comparison
semantics; the Native path applies `recipes::retained_manifest` at 1447.
Do not simply
compare the prepared manifest against itself: retained bytes must still have been
independently inventoried and authenticated in that admission scope.

## Implementation sequence and ownership

1. Add captured declaration/catalog internals and read-only facts in
   `offline-verification/src/catalog.rs`, `fixture_format.rs`, and a private
   `context.rs` if needed. Export only opaque handles/facts from `lib.rs`.
2. Route selection through the shared snapshot in `pipeline/src/affected.rs` and
   `batch.rs`; preserve fresh validation and all planning entry points.
3. Add ordinary batch facade in `ordinary.rs` or private `batch.rs`, retaining
   incremental result publication; route one-family verification through it.
4. Pass captured declarations to family execution, and return admitted
   declarations from private authentication. Update `family.rs`/`lib.rs` and
   Native duplicate readers without changing evidence formats.
5. Extend Native catalog preparation/recipe registration to consume private
   captured facts lazily, retaining separate baseline/candidate binding.
6. Complete captured compiler/asset source-view plumbing through `inputs.rs`,
   `family.rs`, `recipe.rs`, `callback.rs`, `plm_spawn.rs`, and Ridley. Extend
   bounded/source context APIs only where required by real accounting or capture
   needs. Leave build coalescing and action scheduling to later tickets.
7. Update `offline-verification/src/sources.rs` for every new implementation file
   and any application source-stamp changes. Their contents are part of the
   authenticated implementation contract.

Dependencies: ticket 01 metrics integration and final extraction integration;
rebase this exact location map on their frozen merged source before claiming.
No additional crate, production installation or new Native approval channel is
needed to implement this input-flow change. Source capture touches recipes, so
reserve those paths explicitly before assigning another implementation worker.

## Meaningful interface tests

- Through public catalog/selection and bulk verification APIs, use an explicitly
  registered family, a discovered dispatcher, and two catalog names referring to
  the same binary bytes. Compare complete selection facts and canonical run
  observations with one-family calls. Assert one decode per unique captured
  content through metrics/test support, separately counting necessary rereads.
- After capture, change profile membership, add/remove an `.smf`, change a
  discovered manifest's dispatcher meaning, redirect a manifest path, or mutate
  its bytes. Selection/execution must refuse the stale snapshot; no recapture
  into the same operation. Include a change while waiting for the batch/cache
  lock and a shared dependency change before publication.
- Exercise shared packed payloads, complete roots/continuations, captures, ROM,
  callback/PLM normalization, and recipe-specific normalization of one raw input.
  Reject missing/duplicate required batch members and genuinely unknown inputs.
- Preserve per-family durable completion when a later family fails; restart with
  fresh context and confirm accepted-result reuse needs no new producers.
- Drop every invocation cache; authenticate the batch without compiler/runtime
  tools, then mutate retained manifest/payload or current source. Fresh admission
  must detect it, including after scoped cleanup.
- Public Native prepare/verify/authenticate callers must reject a changed
  declaration after preparation, retain lazy first-failure/approval behavior,
  bind candidate overlays separately, and replay expired saved failure evidence
  without touching missing runtime tools. Keep the three explicit real Native
  gates in the extraction plan where affected; ignored tests do not run by default.
- Budget controls: expired budget before any filesystem work; aggregate manifest
  capture exceeding its limit; nested family work unable to reset its inherited
  deadline; two workers jointly exhausting one preparation/memory budget; cache
  hits still checking cancellation/time.

Existing public CLI tests to extend rather than duplicate:
`pipeline/tests/affected.rs:84–206`, `pipeline/tests/family.rs:366` (full state and
continuations), 430 (rebound-inventory tampering), 954 (stale manifest), 985
(changed-since actual C), 1081 (reuse/stale candidate), and 1107 (batch locks).
Callback and PLM public gates are `pipeline/tests/callback.rs:54` and
`pipeline/tests/plm_spawn.rs:48`. Public Native exercises are in
`pipeline/src/translation_native_tests.rs:174`, 409 and 544. There is no existing
`offline-verification/tests/public_interface.rs`; a focused `bulk.rs` integration
test is an appropriate new home for the library facet.

Read-only test expectations worth retaining include
`fixture_format.rs:408` (decoded bytes/digest share one input read; later operation
reopens), `batch.rs:423` (expired/inventory refusal before manifest reads),
`native_tests.rs:274/302` (discovered versus registered aliases), and
`native_tests.rs:465/527/532` (deadlines and expired saved failures).

## Handoff and cleanup

No worktree, build, producer, verification run, claim or source edit was created.
Only this research note was written in the authorized main `.scratch` directory.
Run cleanup is therefore not applicable to this read-only research; the frozen
checkout must remain with its independent verifier. No timing or passing
verification claim is made here.

The final read-only status check observed an independently modified
`.scratch/offline-family-architecture/checkpoint.md` in the verifier checkout;
it was untouched by this research. The checkout was clean at the initial read.
