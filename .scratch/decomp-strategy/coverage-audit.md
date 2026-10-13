# Reconciled translation coverage

Status: complete
Date: 2026-10-13
Owner: /root/coverage_finish

The report now includes maintained source mappings, substantiated retained
fixture coverage, and explicitly measured live execution. Earlier published
totals omitted source-only implementations and much of the historical evidence.
The audit grants each stage independently and retains bounded domains.

| Stage | Unique native definitions | C body lines | Interpretation |
| --- | ---: | ---: | --- |
| Translated | 1,460 | 16,498 | Reviewed maintained executable Rust correspondence, including eight inlined Ridley helpers |
| Verified | 924 | 10,074 | Authenticated retained actual-C fixture domains; current and historical contexts overlap |
| Integrated | 38 | 775 | Explicitly recorded merged gameplay roots accepted in the current bounded integration gate |
| Live exercised | 121 | 1,642 | 95 historical native roots and 26 current composed/trail roots or measured internal implementations |
| Translating | 220 | 3,129 | Retained assignment scope; this is an independent historical claim, not remaining work subtracted from translation |

The pinned inventory contains 6,053 definitions and 75,229 body lines. Fixture
declarations name 834 native definitions / 9,079 body lines across 175 maintained
manifest declarations. Declaration membership and body-line percentages are
inventory measures; they do not estimate playable-game or engineering completion.

## Evidence contexts

| Context | Translated | Verified | Integrated | Live exercised |
| --- | ---: | ---: | ---: | ---: |
| Checkout at observation | 1,460 | 38 | 38 | 26 |
| Retained historical source | 911 | 911 | 0 | 106 |
| Unspecified / legacy | 34 | 34 | 34 | 0 |

Context totals overlap. The eleven composed internal live symbols have both the
integration owner's checkout-at-observation record and a conservative frozen
source record. Their overlap adds eleven definitions once to the combined total.
Legacy context is preserved; new context records never edit old IDs or hashes.
Checkout at observation describes gameplay checked at that observation, while
its exact historical harness remains frozen. It does not authenticate later edits.

The final snapshot contains 256 observations, 16 stale observations, one
superseded observation and 165 registry notices. Stale and superseded claims are
excluded from counts but remain visible in routine details. Four stale claims
use an older source inventory; one earlier trail claim references a subsequently
edited report; eleven early synthetic mappings referenced the audit JSON while
it was still being finalized. Fresh independent observations recover each of
these scopes. The frozen mapping sibling and new IDs preserve old history.
Both reviewed Ridley correction records explicitly supersede the older source
claim, without deleting it or allowing it to revive if a replacement later stales.

## What was reconciled

The maintained source audit reviewed all 44 Rust port modules / 19,453 lines.
It mapped 1,452 unique owned native implementations / 16,425 C body lines;
all 834 explicitly declared native roots have an owned mapping. Another 618
mapped definitions have no explicit native manifest declaration. Eight Ridley
helpers are implemented within reviewed native callers and add eight mappings
/ 73 body lines. `Ridley_Func_48` has the sole caller's 4-by-4 specialization.
These source correspondences grant translation only.

The initial metadata catalog scanned 533 retained verdicts, including 360
`offline-verified` receipts. Metadata alone initially identified only 611 direct
roots. The read-only artifact audit checked 122 selected/cumulative runs,
authenticated 118, and recovered 823 distinct direct native fixture roots.
Accepted runs declare 1,274,545 sealed artifacts and 117,380 fixture pairs
(domains may repeat); hard-link-aware hashing read 1,087,986 distinct files.
Older candidates sealed as dependency snapshots were included after comparison
with their explicit module hashes, avoiding the metadata false negatives.

Synthetic fixture dispatch was reviewed through native function defines,
initializer callback addresses and actual retained ROM words/writes. A script
pointer is never treated as an opcode address. The frozen audit contains 128
manifest/native records for 93 native symbols. Four additional sparks families
were independently rehashed read-only (34,522 distinct files) and passed;
their retained dispatch maps resolve two native initializers/preinstructions.
The direct-plus-synthetic historical union is 911 native verified definitions.
The current 38 explicitly checked roots add thirteen distinct definitions to
that union, producing 924 overall.

The live audit examined 26 older mixed verdicts and authenticated sixteen with
95 distinct native Rust roots. Both mixed subjects must have positive native
Rust counters, exclusive Rust ownership and matching state/effects with zero
CPU fallback. Fifteen unpacked runs had their sealed subject artifacts rehashed
read-only; the compressed Ceres platform run retains its successful storage
authentication receipt and exact binary identity. Current composed/trail roots
add fifteen distinct native roots. Eight exact opcode callbacks and three
successful native initializer paths have additional composed counters and
material captures: 48 opcode events and allocation counts 16/4/4 in each mixed
subject. These eleven internal implementations add live credit at their bounded
domains, giving 121 distinct live definitions.

All 141 decomp-port tickets were read. Their 1,391 cumulative implementation,
846 historical equivalence and 105 bounded live candidate unions are provenance
inputs, not the final registry totals. Source-only partial work and later
cumulative retained families explain the differences. Ticket-level completion
or a supported-stage label never promotes every named routine.

## Explicit gaps

- Four older PLM receipts lack explicit module identity or use an unsupported
  synthetic schema; they receive no blanket historical verification. Later
  source/fixture mappings may separately cover portions.
- Ten legacy live records are excluded: three failed state/effects or fallback
  checks, four lack readable sealed artifact paths, and three expose special
  projectile counters without direct native wrapper mapping. Current composed
  evidence recovers some overlapping domains; missing artifacts remain missing.
- The 309 unresolved source-reference records include forwarding, host/support
  references and uncertain helper aliases. They are not 309 missing routines.
  Eighteen remaining unowned entries in the source audit's broader Ridley
  dependency list include C services and primitive/native references; they
  receive no implementation credit from reachability.
- Palette processing, unmeasured scheduler children, saturated/no-op synthetic
  controls, zero-hit wrappers and natural trail movement opcodes receive no
  new live credit. Their implementations or other fixture domains remain separate.
- Historical fixture authentication does not authenticate all current shared
  dependencies. Committed source alone also does not establish verified or
  integrated coverage, so integration remains the explicit 38-root evidence set.
- Whole Ridley lifecycle live admission remains open under ticket 141. The
  composed 129,712-input replay ends in Ceres room DF8D / game state 8. It proves
  bounded matching execution, not credits progression or original-ROM parity.

Missing stage reports do not establish missing Rust implementation. This audit
includes every reviewed owned mapping and every stage substantiated by selected
retained evidence; unsupported or unavailable claims remain explicit gaps.

## Retained audit evidence

- `source-coverage-map.json` and `source-coverage-audit.md`: current mappings,
  module hashes, bounded domains and source uncertainty.
- `retained-coverage-inventory.json`, `retained-coverage-authentication.json`:
  metadata catalog and read-only accepted/rejected artifact checks.
- `ticket-provenance.json`, `ticket-provenance-note.md`,
  `ticket-authenticated-internal-mappings.json`: all-ticket correspondence and
  frozen selector mapping. Publication uses the immutable copy at
  `../decomp-progress/ticket-authenticated-internal-mappings-published.json`.
- `retained-live-coverage-audit.json`, `live-internal-reconciliation.md`:
  exact native counters, source identity, excluded live records and packed paths.
- `ridley-inlined-source-alias-review.md` and
  `../decomp-progress/ridley-inline-translation-audit.md`: eight caller aliases.
- `../decomp-progress/coverage-final-status.log` and `registry.json`:
  final counts, correction history, exclusions and independent contexts.

Primary immutable SHA256s are recorded in
`../decomp-progress/coverage-final-evidence.sha256`; the audit Rust source,
Cargo manifest/lock and publication/stdout/stderr logs remain reviewable locally.

## Validation and cleanup

Nineteen focused Rust progress tests pass, including legacy retries, independent
contexts, stale exclusion, concurrent atomic publication, HTTP contracts and
permanent immutable corrections. Fresh built CLI status succeeds after cleanup.
Chromium served and standalone checks compare every achieved stage across every
context, preserve routine evidence/history, notices and visible load errors.
Desktop and 390-pixel phone screenshots were inspected with zero phone overflow.
The three-layout comparison is retained on
`throwaway/progress-layout-comparison-20261013` at `e574fef4`; progress first
with collapsed Report details remains the chosen maintained layout.

Both scoped project cleanup passes succeeded for audit and layout roots with
zero utility candidates; browser/audit formats contain no gameplay run seals.
Cargo cleanup removed 1,071 files / 289.3 MiB from the disposable audit target
and 569 files / 146.4 MiB from the layout target. Explicit Rust deletion removed
26,482,694 bytes of identified unsealed executables, browser outputs and temporary
source copies; the 4,540,024-byte cleanup executor was then removed. The separate
layout comparison removed 28,094,108 bytes. Screenshots, source, logs, immutable
verdicts, packed stores, pointers and frozen readers/source contexts remain.
Remaining filesystem space after this cleanup was 231,865,810,944 bytes.
No gameplay execution or source translation was launched during reconciliation.
