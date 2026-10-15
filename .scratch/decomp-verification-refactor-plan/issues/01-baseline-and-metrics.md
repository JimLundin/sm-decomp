# Establish current batch cost and honest reuse timing

Status: claimed
Blocked by: None
Triage: ready-for-agent
Type: task

## Scope and ownership

Own batch/verification phase instrumentation and its public behavior tests in
`pipeline`, `offline-verification` and `tool-execution::metrics`, plus this effort's
measurement configuration/results. Reconcile the active extraction before edits;
freeze its integrated source and exact tools. This proposal is not a live claim.

## Required work

Record planning, input authentication, compiler/cache work, shared-capacity wait,
Fixture execution/comparison, evidence publication/admission and cleanup separately.
Fix the reused `FamilyRecord.seconds` problem: retain historical run time as such
and measure current reuse authentication independently. Include failed and resumed
work and label summed producer duration separately from elapsed time.

Enable existing detailed metrics with `SM_PIPELINE_METRICS=enabled`; measure
sampling overhead and keep cheap stage timing distinct from expensive probes.
Record source/tools/profile, coverage counts, cache state, byte/file counts,
resource bounds and host activity. Specify bounded workloads and the minimum useful
improvement before comparing candidates, following [validation](../validation.md).
Begin with small representative controls; defer full catalog runs until required.

## Acceptance

Public batch controls distinguish fresh, reused, rejected and interrupted work;
reuse time is measured on this invocation. Required stages have complete monotonic
timings or explicit unavailable reasons. Baseline scenarios and finite budgets are
reproducible and preserve their required domain. Historical disabled metrics and
the stripped-plan timeout are not reused as a successful performance baseline.

## Completion evidence

Implementation and bounded controls passed; final acceptance still waits for the
recovered extraction's combined-source gates. Source/control commit:
`eb8811829ef080d35a61b0c859ba1787908b6e18` (production interface `e086190`),
final worker documentation `6c385d208e9daf4d0f38ce9cc53da01cdc4a724a`.
The source is assembled in frozen `7b27ea1`; main remains documentation-only.

Thirteen controls passed, including real actual-C reuse/rejection/fresh execution,
batch interruption/recovery, bounded stage accounting, benchmark attribution and
the eight-sample retained measurement. Reuse reports current work separately from
historical duration and produces no new children when accepted. Exact commands,
source/readers, observations and limits are in [metrics-results](../metrics-results.md)
and [metrics-handoff](../metrics-handoff.md).

The retained one-pair measurement took 25.072966732 seconds from first invocation
through cleanup and independent admission, excluding initial source/tool setup.
Sampling-off/on differences were smaller than observed variation; no reliable
overhead or whole-catalog speedup is claimed. Automatic cleanup reclaimed 524,288
reported bytes; explicit passes reclaimed zero more. The private cache/Git removal
removed 22,328,985 logical file bytes. Required full-workload comparisons still
need the expanded toolchain/environment records listed in the handoff.

Root retired the integrated metrics worktree after independent pre-retirement
admission and no-writer checks, unlinking only its two ignored dependency links.
It contained 208,438,864 logical bytes / 218,275,840 allocated bytes before
removal. No retained verdict depends on that checkout: its exact inputs and
copied reader remain under
`/home/jim/sm-decomp-performance-metrics-proof/baseline-20261015`.
Both post-retirement cleanup passes succeeded on the proof parent with zero
additional bytes (six snapshots checked). Both fresh retained verdicts then
authenticated with byte-identical output; the complete `baseline-retained.sha256`
manifest passed again. Evidence is in the proof parent's `post-retirement/`.
Available filesystem space was 116,722,200,576 bytes at that observation.

The source/control branch remains available for reproduction. Its whole-control
binary embeds old compile-time paths and must be rebuilt/restored to rerun that
harness; standalone evidence admission uses the retained independent source and
reader. Warmed targets remain owned by `dependency_facts` and `bulk_context`,
respectively, and are not counted as reclaimed.

## Comments

2026-10-15 test-only follow-up: checkpoint `1b0b75c` passed the focused actual
compiler denial/replay and independent reader. Its full-real gate passed the
16-pair/34-producer positive, explicit file-macro capture refusal, actual confined
compiler denial and separate failed Native compiler receipt/replay (two producers).
It then refused the old `std::fs` runtime mutation at source capture, correctly,
before a Native intent. The 54.49-second incomplete run is retained and cleaned
(4,194,304 bytes reclaimed); its completed records independently authenticated.
Worker checkpoint `ad8b706` preserves that result.

Root assigns the same worker a further test-only adaptation: keep the exact
runtime-capability capture refusal; keep a second full 16/34 Native family; and
derive one confined diagnostic Program from every admitted Rust Fixture invocation.
Only its executable changes to a separately compiled trusted probe. Preserve
original argv, input/runtime binds, helper, output obligations and five-second /
512-KiB limits. All 16 probes must check parent read/write, sibling C-state and
`/program-c` denial, unchanged protected host contents, complete durable compiler/
probe attribution, identical replay and independent readback after cleanup.
Do not disguise runtime access to bypass the production scanner or count the
diagnostic operations as Native Fixture passes. Complete acceptance remains pending.

2026-10-15 full-real continuation: the combined `45b89f8` source passed the
original 60-second compiler / 180-second family positive gate: all 16 pairs,
34 producers and unchanged retry. Its negative `include_str!` candidate now
fails at the deliberately conservative capture boundary before a Native intent.
Root retains that incomplete full-real proof and assigns a test-only repair to
`/root/bulk_context_resume` in `/home/jim/sm-decomp-performance-native-denial`,
branch `performance/native-denial-controls-20261015`, from `45b89f8`.

Owned file: `crates/pipeline/src/translation_native_tests.rs`, including its
test-only retained authentication records, plus a private handoff. Preserve the
early unresolved-input refusal, add an actual confined Rust compiler file-denial
operation with durable replay/independent authentication, and retain a separate
failed Native compiler receipt/replay control. All 16 positive/runtime-denial
pairs, 34 positive producer slots and original budgets remain required. No
production scanner, lifecycle, operation authority or installed tool changes.
Ticket 08 owns only the existing test identity-initialization sites in this
file; root reconciles those disjoint edits. The released
`/home/jim/sm-decomp-performance-admission-target` is exclusively assigned to
this repair; root's earlier readers are independently retained. Proof:
`/home/jim/sm-decomp-performance-native-denial-proof`.

2026-10-15 continuation: [Ceres recovery](../ceres-recipe-recovery.md) records
the actual transitive-C repair, passing full 1,296-pair ordinary proof, both
cleanup passes and independent admission. The frozen eleven-pass/one-failure
Native baseline remains retained; final Native and full-real acceptance are
still incomplete. Final177 dependency controls at `b1f8691` pass, including
read-only planning/metrics behavior. No full-batch timing multiplier is claimed.

2026-10-15 recovery: frozen `7b27ea1` recipe gate passed eleven families /
6,863 pairs; Ceres-pad failed before its 1,296 fixtures because the linked
Samus handler table retains missing actual-C dependencies. Full-real separately
failed its original C input-discovery deadline before fixtures. Exact outcomes
and completed cleanup are in `native-recovery-verification.md`. Root owns a
focused recipe repair in `/home/jim/sm-decomp-performance-ceres`, branch
`performance/ceres-recipe-20261015`, from `b7f2c0d`. Scope: reviewed recipe,
C compiler/host generation and corresponding actual-C controls, preserving
all declared coverage and Native compiler slots. Final acceptance remains open.

The first implementation task once source ownership and baseline are available.

2026-10-15: claimed by session `01a13e7f-ab01-7413-a1c2-288b08a4b719`.
Provisional source is the clean extracted commit `8bf8acf610e1a94a1a3bccd5d718dae4a4244857`;
worktree `/home/jim/sm-decomp-performance-metrics`, branch
`performance/batch-metrics-20261015`. Final acceptance waits for the extraction's
pending frozen-source verification and cleanup. No installed policy is modified.

2026-10-15 additional control finding: ticket 03's public affected-selection
regression reached an existing read-only assertion before its intended syntax
failure. The metrics wrapper creates `out/port/batches` before dispatching
`batch plan`/`catalog`. Root assigned a narrow separate correction to
`/root/dependency_facts` in `batch.rs`: retain read-only command behavior and
parent-context stage recording without creating a default metrics sidecar.
Existing public no-output assertions must remain intact. This correction and its
green controls are required before ticket 01 resolves; the Native baseline
checkout stays frozen while its independent gates run.
