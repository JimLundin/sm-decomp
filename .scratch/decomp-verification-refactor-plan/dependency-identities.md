# Dependency and result identity contract

The refactor retains the existing authorities and evidence formats. Source facts
describe syntax; they do not authenticate a compiler closure or admit a result.
An incomplete planning closure keeps its Offline family affected. Execution
rejects an unsupported captured closure rather than compiling against inputs
omitted by that discovery.

## Four different identities

| Identity | Existing owning record and checks | Permitted use |
| --- | --- | --- |
| Input/content | `InputView::File` binds a logical path, captured bytes or a streamed asset, exact SHA-256, extent and complete captured namespace. Catalog declarations additionally bind their registration and original bytes. `RustSourceFacts` retains the exact logical source path/hash and owned syntax facts. | Share immutable input bytes or parsed facts. A remembered hash, deserialized map, filename or syntax result cannot substitute for current reads and namespace validation. |
| Build computation | `tool_execution::build_cache::CacheIdentity` binds its admitted command, complete observed source closure, compiler executable/version, relevant environment, platform and Rust toolchain. The existing key lock is followed by source freshness checks; cache hits authenticate the actual artifact and seal. Candidate/generated work remains uncached unless its existing adapter proves eligibility. | Find an eligible compiled artifact. It grants no completed Fixture pair or new Native compiler producer. Generated paths, `file!`/`__FILE__`, flags and environment must remain bound by the captured-input mapping and compiler adapter. |
| Verification computation | The original/effective manifest, authenticated `Implementation` source/dependency/harness maps and binaries, pinned C `Source`, ordered complete Fixture evidence, comparison implementation and execution mode remain bound by ordinary family admission. The new private `RecipeComputation` projection also compares the current registered recipe with the already admitted dependency/ABI contract and callback adapter. | Locate potentially reusable evidence and check that it actually performed the required computation. A caller's `PreviousFamily.kind` or `FamilyEvidence.kind` is only a lookup/registration hint. Equal hashes or a matching recipe projection alone cannot admit a result. |
| Attempt/admission | The captured required family set and original registration are checked by `authenticate_batch`. Native additionally binds prepared requirements, current approved candidate paths/digest, provenance, installed Program/runtime, immutable `OperationContext`, claim/attempt/phase, reservations, deadlines and every required producer. The Translation journal remains authority. | Accept authenticated evidence for this request. An ordinary retained producer hash records its historical producer; it is not an assertion that the current reader is the same executable. Native installed-controller and producer identities remain separate mandatory checks. |

There is no new universal result key or result ledger. In particular, the small
recipe projection is an additional admission comparison, not a replacement for
the full computation identity already carried by authenticated records. Ordinary
single-family calls, ordinary batch admission and Native receipts preserve their
existing independent readback. Fresh execution policy still suppresses result
lookup; a compiled artifact hit does not satisfy a fresh-producer obligation.

The broad implementation source maps are deliberately preserved. The scanner's
source, its crate declaration and the exact root Cargo manifest/lock are included
in required verifier source stamps. A changed comparator, scanner dependency,
candidate or transitive source still invalidates retained evidence. This change
does not claim independence from unrelated application source or narrow current
frozen requirements.

## Syntax and actual inputs

`source_facts::scan` consumes exact bytes under fixed source/token/nesting/work
ceilings and the inherited deadline. It discards parser ASTs and returns ordinary
owned data suitable for sharing. Malformed or unsupported syntax is explicitly
incomplete; resource exhaustion remains an error in planning as well as execution.

Both wrapper discovery paths, affected selection and the captured Rust guard use
the same facts. Comments and ordinary string/byte/character literals produce no
module edge. Real code identifiers and macro token arguments conservatively
retain possible edges. Imports, groups, raw identifiers and rooted alias chains
are handled without evaluating cfg branches; unrooted alias cycles are incomplete.

Top-level literal relative `#[path = "...rs"] mod name;` is a real source edge.
Callers traverse, capture and validate the referenced source; path strings do not
become heuristic crate-root imports. Missing or redirected edges are rejected.
Implicit, dynamic, escaping or nested module-path resolution remains incomplete.
Actual include/environment macros retain their literal or dynamic input facts
and remain unsupported until their complete input contracts are implemented.
Unmodeled macros/attributes, runtime capabilities and foreign linkage retain
explicit uncertainty. These limits are reported, not treated as proof of an empty
dependency set.

A child using top-level `super::*` can resolve its specific builtin-shadow
uncertainty only through a complete, explicitly linked literal parent without
globs or potentially shadowing builtin imports. Raw child facts remain
incomplete outside that placement. Every `(file,parent)` placement is validated
before traversal deduplication; a successful proof for one parent cannot stand
in for a second parent. This is a narrow contextual proof, not general module
resolution or permission to discharge other unsupported inputs.

Captured facts are lazy and share a bounded memory reservation with their owned
source bytes. This cache exists only in the invocation; independent retained
admission does not trust or require it. Rust compiler dep-info still must match
captured logical/physical inputs, rejects unbound paths and `# env-dep:` entries,
and remains included in retained evidence. Pinned C, real compiler/header closure,
ROM and capture validation remain with their existing owners; Rust syntax cannot
stand in for any of those checks.

## Planner reuse and final authentication

Within a planning invocation, complete transitive root closures may be reused
under a separate bounded cache. Each family's reported memberships still consume
the original aggregate entry budget. Pending traversal visits are deduplicated
without suppressing parent-placement checks or final source/namespace validation.

Declared assets are gathered without reading their payload repeatedly. The final
pass reads each distinct asset once, compares every declaring family's required
digest and retains its coherent filesystem version. Device, inode, length, mtime
and ctime (including nanoseconds) are rechecked after final source/catalog
validation. Missing, changed or mismatched assets leave affected families
selected. Paths containing parent traversal are unresolved before lexical
normalization because intervening symlinks can change the file actually named.
This optimization preserves the original read/entry/deadline budgets and cannot
turn an unverified asset or cached hash into proof of exclusion.

## Controls and remaining acceptance

The actual-C red checkpoint `18c4501` demonstrated that a forged matching kind
hint could otherwise admit a family result for a callback registration. It
retains its exact source/readers and finished-run cleanup in the dependency proof
root. The green control also checks content hints, equivalent recipe aliases,
actual compiler transitive/path input closure, changed candidate/comparator/lock,
changed Fixture requirements and missing dependencies against independently
admitted evidence. See `dependency-facts-handoff.md` for executed results.

The original 177-family corpus now passes the required controls on combined
source `b1f8691`: documentation excludes 174 families and retains three explicit
uncertainties; actual audio mutation, rename and deletion each select exactly
the 162 known dependents plus those three. The same original byte/entry/deadline
budgets apply. Post-retirement readback authenticates every recorded input hash
using retained source and assets; the reports still carry selection-only scope.

Final ticket acceptance remains with the integrator's combined-source gates and
implementer retirement. Current Native contract/claim/attempt controls remain
applicable; this worker does not replace them or claim that ordinary evidence
establishes Native provenance.
