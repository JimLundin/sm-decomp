# Measure the native Rust pipeline, then upgrade its measured bottleneck

Status: proposed
Committed baseline: `e820a6b77338eb4e84a2d91baeed2dc9f33c72f1`.
Implementation basis: the uncommitted Rust migration in `/tmp/sm-decomp-pipeline`,
read on 2026-10-11 and pinned in `optimization-source-snapshot.toml`.
Planning checkout: `decomp-port-integration`. No optimization source, worktree,
branch or existing evidence is changed by this plan.

## Outcome and scope

The first deliverable is a coherent native Rust pipeline integrated with current
decomp, complete process/publication measurements, and a controlled text-capture
baseline. Reuse the migration owner's native commands and templates. New tooling,
orchestration and tests are Rust; new internal interfaces are typed Rust, small
configuration/receipts use versioned TOML, and large ordered transport uses binary.
Preserve existing JSON readers and sealed formats where historical evidence or
external corpora require them; do not rewrite sealed verdicts to meet that policy.

Binary PPU/APU FIFO capture is a conditional second phase. Activate it only after
the measured profile supports text conversion/transport as the next useful target.
The retained `hardware.log` must restore byte-for-byte with identical stream and
observation identities for identical observations. Four fresh processes,
full-WRAM/host captures, ordered effects, progress and Rust ownership remain gates.

For that candidate, adopt it only if controlled
runs pass the proposed promotion gate: at least 15% lower median warm-build
end-to-end wall time at 45,000 inputs, with at most 5% growth in aggregate sampled
peak RSS and allocated staging/private-store peak at the same worker capacity.
These are proposed thresholds, not predicted results. Freeze absolute memory/disk
budgets for the benchmark host before ticket 03; all variants must fit those
budgets. A smaller gain leaves binary opt-in unless the owner records an explicit
exception with measurements and rationale. Original-ROM fidelity advances through
its own gates.

Reuse Rust `sm-pipeline` leases/caches and the worktree's native `builds`, `family`,
`batch`, `packed`, `live`, `observations` and `cleanup` modules. Reuse the evidence
crate's `verified_inventory`, ordinal-remapping import and durable pointers.
The new delta is missing observability and a profile-gated capture protocol.
Persistent gameplay workers, offline fixture packing, checkpoints, cache eviction,
weighted admission and distributed execution are later efforts.

The source and measurement basis is recorded in
[the branch comparison](../pipeline-review/optimization-comparison.md),
[measurement audit](../pipeline-review/optimization-measurements.md), and
[pinned committed source index](../pipeline-review/optimization-source-index.json).
The current worktree findings and limits are in
[optimization-progress.md](optimization-progress.md); its source snapshot takes
precedence for implementation seams. No new optimization commit was observed.

## Progress to reuse and prerequisites

| Area | Observed optimization worktree progress | Remaining gate for this plan |
| --- | --- | --- |
| Native process/build/cleanup | `builds.rs`, `cleanup.rs`, worker/cache commands and Rust filesystem controls exist | Extend missing measurements; preserve process ownership and retained evidence |
| Native family/batch | Full WRAM/returns, eight-word host checks, Rust-before-C, private cases, atomic batch/reuse exist | Reconcile current dependent chains; refreshed combined-source seals |
| Native packed/live | `packed::{capture,publish}`, `live::verify`, native host/effect templates, `observations` authentication exist | Complete pending live controls; preserve latest projectile ownership/WithGfx |
| Fixtures/bootstrap | Native fixture domains and independent Rust bootstrap have recorded controls | Reuse migration results; check final source and tool identities |
| Readiness/language migration | Native readiness modules exist; migration tickets remain open/claimed; Python CI/recipes remain | Owner completes migration gates; this plan adds no Python adapters or duplicate ports |
| Capture/metrics | Text `vfprintf` still emits PPU/APU; elapsed/wait receipts exist | CPU/RSS/staging/full outer wall and profile; binary transport remains unimplemented |

The five-family native batch and authenticated reuse, four-subject 1,000-frame
native live runs, 9,000-frame decompressor reset runs and RNG controls are useful
recorded evidence. They are not a controlled whole-pipeline benchmark or the
current 129,712-input projectile acceptance. Claimed migration tickets with
completion notes are not treated as resolved work or current-source acceptance.

## First phase and candidate selection

Tickets 01–03 are the proposed first phase. Depend on the migration owner's
reviewed delivered revision and completed migration controls as an external
prerequisite for ticket 01; its lack of local blockers does not waive that gate.
Integrate into a new
future integration checkout, preserving `/tmp/sm-decomp-pipeline`. Verify combined
current decomp semantics before using measurements to choose further work.

Reuse existing native timing records and add only missing fields/stages. Produce
three alternating text-baseline repeats at 1,000/45,000 inputs for capacities
two/four on a quiet host, with fixed cache/store warmth and full publication,
authentication and cleanup boundaries. Keep cold-build/empty-store runs separate.
Record a profile-based choice in ticket 03 before claiming any transport ticket.

The choice must identify the limiting critical path and a credible route to the
proposed total-runtime gain. Binary removes C formatting and Rust text parsing;
Rust legacy rendering/hashing and snapshot/import work remain. Producer elapsed
time alone does not establish text parsing as the bottleneck. Use bounded CPU
profiling or controlled attribution samples for producer and collector, and do
not add overlapping intervals. If import/authentication, host build, fixture
execution or I/O dominates, write a separate scoped plan for that measured cost.
Tickets 04–08 stay conditional and unclaimed unless binary is selected. A
completed first phase may conclude that transport should be deferred.

## Conditional binary phase: fixed compatibility decisions

### Keep stored evidence unchanged

Keep `sm-evidence-v1`, canonical `Event`, segment layout, exact legacy-byte SHA-256,
frame/member extraction and historical reader compatibility. Version the transport
separately as `sm-capture-v1`. Add `--capture-transport text|binary` to live/batch
commands and a distinct evidence-reader `capture-binary` CLI entry. Thread the
typed selection through `live`, `batch` and `packed`; add no Python transport
adapter. Text remains available for
comparison and rollback. Select the decoder explicitly; reject a protocol mismatch.

Compare legacy bytes, stream IDs, observation IDs, counts, progress and ownership.
Run IDs include producer/reader provenance and may change when those binaries
change. Require run-ID equality only with identical provenance; otherwise verify
each run's provenance and equal observations independently. Store transport/timing
receipts as sidecars outside comparison inventories and canonical objects, so
measurement fields do not change the observations they measure.

### Define transport framing before adding emitters

Use an eight-byte header: six ASCII bytes `SMCAP1`, little-endian version `u16=1`.
Each record is `tag:u8`, `payload_length:u32 LE`, then exactly that payload.
No compression on the pipe initially; retain the existing buffered FIFO.
Explicitly encode fields, avoiding native C struct layout and padding.

| Tag | Payload | Constraint |
| --- | --- | --- |
| 1: PPU | frame:u32, device:u8, register:u8, value:u8 | 7 bytes; device 0/1 |
| 2: APU | frame:u32, port:u32, value:u8 | 9 bytes; preserve existing host address extent, including full port representation |
| 3: literal | exact formatted bytes | 1..65,536 bytes; preserve all existing bytes/line boundaries |
| 255: END | data_records:u64, ppu:u64, apu:u64, literal_records:u64, literal_bytes:u64 | 40 bytes; END itself excluded from data counts |

Use checked arithmetic and explicit validation before allocation. Require one END,
matching transmitted record/type/literal-byte counts and EOF immediately after it. Reject unknown magic,
versions/tags, noncanonical payload lengths, invalid values, oversized literals,
partial records, missing END and trailing bytes. Frame counters may move backwards
or reset when the existing observer does; transport does not impose monotonicity.
END is emitted only after successful retained gameplay completion and all final
effect writes. Parent still requires zero producer and collector exits, final
progress/inventory and admission. END catches record-boundary truncation; counts alone cannot authenticate values or
ordering. The Rust collector derives the exact logical-byte SHA-256 and line/byte
counts using existing TraceWriter rendering and includes them in its receipt.
Framing/range errors reject during decode. Reopened authentication proves integrity
of the decoded/stored bytes, not equality with the bytes C intended to emit.
Cross-subject comparison detects divergent value/order corruption; systematic
encoder/decoder corruption shared by all subjects can still agree. Independent
text/binary equivalence and historical-reader tests provide bounded controls for
that risk. A failed comparison publishes no accepted verdict. No new per-event
producer hashing/formatting is introduced to calculate the footer. Delivery
completeness cannot certify an omitted hook or original-ROM correctness.

### Preserve literal semantics exactly

Replace only the explicit PPU/APU wrapper call sites with typed emitters. Keep
PLM, room-PLM, projectile and future diagnostic writes on a bounded formatted
literal path, preserving signed `%d`, whitespace, byte values and newlines.
Do not classify variadic arguments or infer event type from a format string.

Extract shared legacy line ingestion from `ingest_trace`: literal payloads may
arrive fragmented across records and must undergo the same canonical-event
classification, second-token frame inference, long-line handling and 64 KiB
segmentation. For example `plm HEADER ...` currently associates a frame hint with
HEADER, and `projectile KIND ...` with KIND. Preserve that existing behavior even
though those fields are not gameplay frames. `literal(bytes, None)` would change
canonical stream identities. Literal records may contain canonical PPU/APU text;
the shared scanner must encode them exactly as the text reader does.

Typed records can use direct `TraceWriter::push` only when that shared scanner
is at a clean line boundary with no pending partial/long-line state. Otherwise
render their canonical legacy bytes into the scanner: literal `prefix` followed
by a typed PPU record must restore `prefixppu ...\n` with the same literal
classification as text ingestion. Flush scanner residual at END before finishing
the stream. Test partial literal→typed, giant-line continuation→typed and typed→
partial literal→END explicitly; byte equality alone must also retain segment IDs.

`TraceWriter::push` currently renders and allocates text per event to retain the
legacy digest. The binary transport removes C formatting and Rust parsing, while
that Rust formatting/hash work remains. Reuse a bounded rendering buffer as a
separate measured improvement if the profile justifies it; do not alter rendering,
canonical boundaries or logical digest to create an artificial speedup.

## Complete measurement contract

Extend `builds::execute_timeout`, `packed::capture/publish`, `live::verify` and
`batch::operate`, reusing their existing elapsed/wait records. Add a typed
`pipeline-metrics-v1` model and TOML sidecar receipts; native Rust combines them
into the run summary. Keep existing timing/verdict readers compatible. Include
failed/interrupted runs with outcome and the last completed stage.

Instrument existing text `capture` and `import` in `crates/evidence/src/main.rs`
in ticket 02, independently of binary selection; otherwise the baseline cannot
separate collector commit, staged ingestion and verification costs. The metrics
worker owns that file in phase one; transfer ownership for binary work later.

Measure monotonic start/end timestamps, elapsed durations and parent/child links
for worker/store wait, collector ready, build/cache restore, replay, FIFO drain,
trace commit, staged-tree ingest, final commit, reopened verification, staged-file
comparison, pointer fsync, shared import/destination verification, post-publication
subject authentication, private-store release, final verdict publication and
scoped cleanup. Intervals may overlap; compute total wall time from one outer
boundary rather than adding parallel/nested durations. Never reset that boundary
at a cache hit, successful producer exit or early publication timestamp.
Across Linux processes, use the same `CLOCK_MONOTONIC` origin with a run-relative
offset, or parent-normalize child timestamps against a documented handshake.
Independently serialized `Instant` durations are not a shared timeline. Record
clock/sampling semantics and never combine unrelated clock domains.

Capture PID-scoped CPU user/system time, per-process peak RSS, aggregate concurrent
RSS sampled at a stated interval, transport bytes, logical bytes, pack bytes,
allocated staging/private-store peak, cache hits/misses and configuration. State
unsupported fields as unavailable, rather than zero. On the Linux runner use
wait4-equivalent child accounting plus per-run `/proc` sampling and sampled disk
accounting; do not use all historical `RUSAGE_CHILDREN` as one run's CPU time or
sum nonconcurrent peak RSS as a claimed machine peak. Include both producer and
collector and the known sampled-peak limitation. Metrics sample/scan frequency
is bounded so telemetry does not dominate capture. One process owner must reap
each child exactly once; `Child::try_wait` must not consume an exit before the
resource-accounting path collects it. Preserve timeout/kill/wait diagnostics.

Use buffered batches up to 64 KiB. Handle short writes and interruptions; closed
pipe/collector death fails the run. Measure write calls and blocked time per
flush, never take clocks or scan files per event. Benchmark telemetry enabled
against disabled on the text path and reduce sampling if overhead exceeds 2%.
Use receipts outside immutable evidence to preserve comparable observations.

## Work sequence and ownership

| Ticket | Work and reused seam | Dependencies | Owner |
| --- | --- | --- | --- |
| 01 | Pin delivered native migration, reconcile current decomp chains/host ABI, refresh admission | External: reviewed migration delivery/controls | Integration owner |
| 02 | Extend native metrics, typed/TOML receipts and single-owner child accounting | 01 | Metrics worker |
| 03 | Controlled native text baseline, profile and explicit next-bottleneck decision | 02 | Integration owner + reviewer |
| 04 | Conditional exact scanner and framed binary evidence decoder | 03 + binary selection | Evidence worker |
| 05 | Conditional native C templates and Rust transport selection | 04 | Native integration worker |
| 06 | Conditional real compatibility and failure/publication controls | 05 | Independent reviewer |
| 07 | Conditional controlled text/binary benchmarks and full-route acceptance | 06 | Integration owner |
| 08 | Conditional promotion or documented opt-in retention/rollback | 07 | Integration owner |

One integration owner coordinates the migration source freeze and decomp
reconciliation. A metrics worker owns `metrics.rs`, `builds.rs` and instrumentation;
an evidence worker may inspect/design the decoder while first-phase work proceeds,
but implementation waits for ticket 03's binary selection. The native integration
worker owns `packed.rs`, `live.rs`, `batch.rs` and C templates in the binary phase.
Serialize shared-file edits and use one independent correctness reviewer. Give separate worktrees
exclusive file ownership when implementation is authorized. All producers,
collectors, compilers and benchmark jobs share one `SM_PIPELINE_ROOT` and matching
capacity. One packed subject's producer/collector pair consumes one orchestration
lease, including both processes in that slot's measurements; neither child nor
its accounting helper reacquires that lease. Leases bound concurrency, not RSS.
Start with capacity two under current load; compare two/four on a quiet host only
within the frozen resource budgets. Keep one explicit run/store scope per
experiment. No automatic capacity changes and no simultaneous benchmark sweeps
that confound results.

## Files and interfaces

- `crates/pipeline/src/family.rs`: reuse native case execution/authentication;
  add current ordered continuation/host inheritance when reconciling decomp.
- `crates/pipeline/src/builds.rs`, new `src/metrics.rs`: typed process/stage
  measurements around existing native execution/cache operations.
- `crates/pipeline/src/packed.rs`: instrument capture/publication and optionally
  select the evidence decoder; retain private stores, readiness and safe release.
- `crates/pipeline/src/live.rs`, `batch.rs`, `observations.rs`, `cleanup.rs`:
  native selection, outer wall boundary, current admission, auth and cleanup.
- New `crates/pipeline/src/benchmark.rs`: native matrix runner reusing existing
  commands, frozen inputs and alternating configurations; no Python helper.
- `crates/pipeline/src/effect_writer.c.in`, `live_host.c.in`: conditional explicit
  typed PPU/APU emitters, bounded literals, successful END placement and flushing.
- `crates/evidence/src/trace.rs`: reuse `Event`, `TraceWriter`, `ingest_trace`;
  conditionally extract the exact literal scanner and add framed ingestion.
- New `crates/evidence/src/capture.rs` plus `src/lib.rs`: protocol validation,
  checked footer/digest/count receipt and parser tests.
- `crates/evidence/src/main.rs`: phase-one text capture/import stage receipts;
  conditional `capture-binary` reuses trace/tree/run commits and
  `verified_inventory` without changing canonical objects.
- `crates/pipeline/src/main.rs`: expose typed native selection/benchmark commands;
  preserve transitional historical request compatibility.
- `crates/evidence/tests/capture.rs`, native pipeline `tests/{packed,live,family,
  processes,observations}.rs` and new metrics/benchmark tests: reuse meaningful
  controls and extend parser/producer/publication/continuation behavior.
- New `docs/CAPTURE_PROTOCOL.md`, updated `docs/DECOMP_BATCH.md`: exact transport,
  metric meanings, versioned sidecars, fallback and run retention.

These filenames identify inspected uncommitted migration seams or unchanged
evidence-crate sources. New files and binary changes are proposed; this plan
does not port migration functionality that is already in progress.

## Acceptance and rollout

In phase one, preserve migration negative controls, run current combined-source
Rust tests and actual-C boundaries, and retain native text admission through
129,712 inputs with all nine IDs, 16/4/4 allocations and material WRAM/host state.
Check that new metrics do not change stored identities/observations; measure their
text overhead against disabled telemetry and keep it at most 2%. Observe the
migration owner's final-source controls and completion rather than rerunning
obsolete Python verification.
Regenerate source-dependent admission for reconciled Rust/templates/manifests;
reject stale cache/reuse seals. Imported historical observations remain readable
under their retained reader/source snapshots and do not grant current admission.

If binary is selected, validate synthetic mixed typed/literal streams and actual generated C
emitters against text ingestion. Cover signed PLM values, zeros/max fields,
frame resets, line fragmentation, giant lines, partial literal→typed events,
giant-line continuation→typed and typed→partial literal→END, empty literals
rejection, canonical-event literals and every footer/length failure. Real byte
corruption, changed/reordered/dropped/duplicated events, collector death, EOF at a
record boundary, missing END and nonzero producer exits must retain diagnostics.

Distinguish a durable subject `evidence.json` pointer from an accepted live
verdict. Publish a durable subject pointer only after successful producer and
collector completion and reopened byte authentication. Release staging/private
bytes only after authenticating their retained replacement. The subject may be
sealed before cross-subject comparison; a failed comparison retains the sealed
divergent observations and diagnostics and withholds the accepted live verdict.

For equal captured observations require exact `hardware.log`, member SHA-256,
stream/observations roots, event/line/logical-byte counts and extractable frames.
Keep upstream CPU fallback, support refusal, changed-state, Rust-before-C and
accepted-error/no-C-retry tests. Compare four fresh text subjects with four fresh
binary subjects using the same frozen source/inputs and wrappers. Test both text
and binary against an unchanged historical reader for stored evidence.

Ticket 01 reconciles the newer `continue_from`/`case_order` WRAM/eight-word host
chains and native projectile WithGfx bridge with the Rust migration. The inspected
native family path still flattens cases and reads explicit `host_words`; those
are distinct from inherited chain state. Preliminary measurements on unreconciled
sources cannot justify current-decomp promotion. Planning performs no merge.

On that frozen combined source, record three alternating warm-build runs each at
1,000 and 45,000 inputs for
text/binary and shared capacity two; repeat the beneficial configurations at
capacity four. Run cold-build and empty-store cases separately; warm stores must
contain the same baseline observations before paired comparisons. Fix compiler,
reader/source hashes, manifests, ROM/movie, storage and host workload; log load
and sampled resource data. Report median/range and full wall boundaries, including
publication/authentication/cleanup. Stop malformed/incorrect variants immediately.
Avoid claiming a speed gain from different cache/store warmth or excluded costs.

Before promotion to the current decomp workflow, run combined-source meaningful tests and the
actual-C six-family plus supplemental boundaries, then final 129,712-input
text/binary admission with all nine scheduler IDs and 16/4/4 allocations, material
full-WRAM/all-eight-host captures, exact effects/progress and zero refusal/fallback.
Promote binary only with correctness, the 15% total-wall/5% resource thresholds,
and the frozen absolute host budgets. Record median/range for every capacity;
promote only capacities that pass. Any exception needs an explicit owner decision
in ticket 08. Preserve `text` as a
rollback flag and retain producer/reader/protocol/source receipts per run.
Historical verdicts/readers remain immutable. After every successful, failed or
interrupted experiment, wait for writers, apply scoped native cleanup/dedup
(`sm-pipeline cleanup --root RUN --apply`, then `--deduplicate --apply`),
authenticate retained subject/store dependencies that exist. Run
`sm-pipeline live-authenticate VERDICT` when a complete live verdict exists;
failed/interrupted runs may only have sealed subjects or partial diagnostics.
Preserve those diagnostics and record any incomplete/unavailable authentication.
Record reclaimed/free bytes and required
store/reader/source dependencies. Remove only integrated implementer worktrees
with exact cited source snapshots preserved. Cache retention stays separate.

## Explicit limits

This upgrade improves measurement and evidence transport. It makes no new claim
about original-ROM frame timing, APU handshake/audio fidelity or intended TAS
ending. Those remain the independent accuracy work stream. The 15%/5% thresholds
are proposed adoption criteria; changes require a recorded decision before
benchmarking or an explicit exception at promotion.
