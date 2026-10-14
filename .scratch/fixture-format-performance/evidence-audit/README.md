# Current verification cost audit

Binary Fixtures are the only execution format. Historical replay is not a
requirement. The user explicitly asked to retire obsolete readers, migration
programs and this session's archived results. Small configuration and verdict
records still use TOML.

New evidence has a real cost: full before/after WRAM writes, result records,
source and artifact hashing, and authentication. The binary migration preserves
those guarantees while removing repeated text payloads from per-case and
prepared-native records. This follow-up removes avoidable repeated reads.

## Changes

- Bounded IO can return bytes and their SHA-256 from one file snapshot. It
  reserves the validated extent once, maintains deadline and byte limits, and
  reopens the current file on every operation.
- Captured inputs and ordinary actual-C outputs use that single read for both
  identity and full-state comparison. Case authentication does the same.
- Fixture admission authenticates exactly the bytes being decoded, eliminating
  the separate native hash/read passes. Verification seals its initial input
  bytes and still checks the current source and retained copy at completion.
- Artifact identities already calculated by the fresh authentication inventory
  are reused for binary and worker identity comparisons. There is no persistent
  file-identity cache.
- Native receipts require indexed attribution, explicit writable ownership and
  reconciled producer reservations. Old inline receipts, optional metadata,
  implicit single-file identities and historical recipe exceptions were removed.
- Native receipt enumeration and extent accounting visit the index and chunks
  once. Producer authentication shares the current intent instead of reparsing
  the same file for every producer.

The WRAM comparison and SHA-256 checks remain. The regression now also changes
both subject outputs to equal wrong bytes and rebinds their artifact inventory;
the retained case hashes must still reject them.

## Measurements

`results.json` compares the exact pre-change bounded IO implementation from
`108bee3` with the candidate. The optimized Rust benchmark uses nine alternating
sequential samples per variant, with 128 iterations for WRAM and 32 for fixtures.
Every input is authenticated. Both Fixture variants have active resource checks
inside the same decoder. No tests or builds from this task ran concurrently
with the measurement; other machine work was active.

| Workload | Before median | After median | Logical bytes read |
| --- | ---: | ---: | ---: |
| Captured WRAM read and SHA-256 | 0.218 ms | 0.186 ms | 262,144 → 131,072 |
| Ordinary root-case WRAM authentication, including fresh inventory | 1.362 ms | 1.337 ms | 1,179,648 → 786,432 |
| Floater Fixture bounded read, SHA-256 and decode | 3.366 ms | 3.162 ms | 2,597,982 → 1,298,991 |
| Bomb Fixture bounded read, SHA-256 and decode | 4.069 ms | 3.948 ms | 3,065,648 → 1,532,824 |
| Ridley Fixture bounded read, SHA-256 and decode | 6.216 ms | 5.684 ms | 2,842,014 → 1,421,007 |

Sample ranges overlap, particularly for ordinary authentication. These timings
support only modest local gains; the exact reduction in read volume is the
stronger result. SHA-256 still dominates much of this work. The benchmark omits
compiler and producer execution, process scheduling, directory discovery,
continuations, verdict parsing and gameplay. It does not measure complete family
verification. Unlike the initial format study's direct read/hash/decode series,
this series includes the actual bounded IO and active decoder resource checks;
the two timing series are not directly comparable.

The one-off comparison crate and baseline reader were retired after measuring,
as requested. `results.json` keeps the samples and method. Current Rust regression
tests enforce one-read budgets for authenticated input loading and native receipt
enumeration/accounting, with tampering and changed-input controls.

## Fidelity and remaining costs

The migration preserved typed declarations, every modeled WRAM byte, continuation
holes and explicit zero writes, captured asset references, expected results and
verification requirements. It normalized original text formatting, comments
and redundant patch boundaries. Original text remains in Git history. The
external captured assets were not independently reauthenticated by the corpus
migration; that limitation is unchanged from ticket 02.

Raw shared binary spans were chosen from measured alternatives; execution needs
no decompression or text decoding. Full WRAM comparisons, SHA-256, producer
records and admission checks still cost time and storage. The measured changes
remove duplicate reads without reducing current verification coverage. The
format measurements do not establish maximum whole-pipeline throughput.

Current tests, actual-C checks, cleanup and commits are recorded in tickets 03
and 04. They do not depend on retained historical reader or source bundles.
