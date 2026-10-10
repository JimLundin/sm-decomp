# Recompilation iteration

Status: specified; Q1–Q15 agreed. Awaiting final confirmation of shared
understanding. [Recompilation readiness](recomp-readiness.md) owns the validated
domain, evidence layout, statuses, diagnostics, reuse and recovery contract.

## Trigger, ownership and stopping

An explicit user request starts a session. Honor its increment count or target;
default to one accepted increment when unspecified. A readiness request runs
readiness and does not silently start unlimited recompilation.

The agent owns selection, changes, checks, diagnosis and routine retries.
Acceptance advances the local accepted pointer without a checkpoint per change.
Commit, publication and merge require separate authorization.

Stop when requested scope completes, the user stops work, a required input is
missing or a decision beyond this design is needed. Three successive diagnostic
experiments without new discriminating evidence trigger a decision brief.
Preserve failed work and the accepted predecessor. Unresolved, uncovered,
incomplete or errored candidates never count as accepted increments.

## Select a bounded increment

1. Verify the accepted run ID, source identities and domain. If absent or
   invalid for the change, first run the necessary readiness gates.
2. Measure interpreted instruction execution by address/MX and opcode/pattern
   within the validated prefix. Existing call-target misses are discovery input,
   not instruction counts or a rate of missing generated calls. Obtain an
   instruction profile if accepted evidence lacks one.
3. Prefer frequently executed work already supported by the timed whitelist;
   add one routine/width slice. Rank likely compiled-step gain by observed
   counts, prefer simpler candidates for similar gains, then stable address/
   width ordering. Record the choice and rationale.
4. When existing support cannot deliver useful further gain, add one timed
   instruction pattern: one opcode/addressing behavior with declared applicable
   width/mode variants and timing branches. Exclude unrelated expansion.
5. Preserve the accepted snapshot and prepare a candidate. `trace merge` may
   propose discovered config in isolation; it cannot silently expand reviewed
   entries without this selection and validation procedure.

A routine may remain partly interpreted. Every newly enabled compiled address/
MX key must execute in cold acceptance. Remove unexecuted proposed keys from
the accepted table and record them as deferred inventory, rather than merely
removing them from the required-coverage list. Changed existing bodies require
affected-key execution. New patterns also need targeted evidence for every
declared mode/timing branch and at least one real-ROM execution. Legacy
unexecuted baseline entries remain unvalidated and cannot count as new progress.

If nothing useful is covered, prepare a concrete independently validated domain
extension first, or report no remaining covered candidate. More generated config
entries alone are not progress. Repairs needed by an increment are its retries;
broad shared-hardware repairs are separate work requiring refreshed readiness.

## Build and test gate

Allocate a new evidence run before preflight/mutation and capture exact inputs.
Regenerate/build needed outputs using existing machinery. Frame validation uses
resumable compiled steps; whole-subroutine untimed output is not cycle evidence.
Build failure yields error with logs.

Before acceptance, `just check`, `just lint` and relevant regressions must pass
for the candidate source. Reuse only checks with identical executable/build
inputs and recorded provenance.

New patterns and changes to enabled semantics/timing require:

- Complete available SingleStepTests affected-opcode cases through the actual
  emitter and executing step body, batched if needed. Record corpus identity,
  case names and expected/run/passed/failed/skipped counts. A development sample
  is insufficient final evidence.
- Applicable native M/X widths, decimal/binary/status modes and addressing
  branches. Assert registers, flags and memory effects, plus PC/control effects
  when relevant. Explicitly state unsupported cases outside the declared scope;
  no supported case may be skipped to manufacture a pass.
- Independently specified cases to fill missing applicable coverage. For
  control/stack/interrupt patterns not expressed by the corpus, use independent
  fixtures and record the coverage substitution.
- Targeted SNES tests of ordered bus accesses and exact timing, memory-speed and
  applicable penalty/boundary behavior, interrupt polling/preemption, pending
  DMA/HDMA, refresh, frame boundaries and resumability. State why a condition
  does not apply rather than demanding irrelevant tests for every opcode.
- Timing expectations from documented hardware sources or independent emulator
  observations. Shared-interpreter parity alone is insufficient. Observers must
  not alter interrupts or hardware state.

Known failures block the affected pattern. ADC cannot enter support with its
recorded overflow failures unresolved. Unrelated unsupported failures remain
linked debt; deleting active generated cases does not establish a corpus pass.
Enable the pattern in `recomp::step` only after required tests pass. Unsupported
instructions remain interpreted and are reported separately.

## Focused validation and diagnosis

Choose the shortest covered interval exercising changed keys and relevant
boundaries. Build or reuse a validated paired checkpoint preceding affected
execution, under the readiness cache/provenance contract. Record cache creation
and warm replay separately.

Run focused instruction/bus comparison even when frame parity passes, catching
transient effects restored before frame end. Multiple focused intervals may
cover different changed keys. Missing coverage extends the interval within the
domain or defers/removes keys; execution by another key never waives coverage.

On first disagreement retain paired trace/snapshot evidence, localize the
earliest observable instruction/state/bus mismatch, and replay accepted-
predecessor and disabled-entry controls. Use evidence to choose the next action:

| Observation | Action |
| --- | --- |
| New keys do not execute | Choose covered keys/intervals or defer them; uncovered verdict |
| Candidate fails, predecessor and disabled-entry controls pass | Investigate changed semantics, widths, dispatch, emission or timing; reproduce and add a regression |
| A control also fails | Verify cache/input/harness, prior assumptions and shared-path changes; refresh baseline gates as needed |
| Parity passes, original-core synchronization fails | Diagnose hardware/input/observer independently |
| Panic, budget, build or setup error | Preserve stage/state/logs; repair or identify the missing requirement |

Do not assert a cause from a frame number, WRAM byte or missing-call log alone.
Record hypothesis, predicted discriminator, experiment and result. Each repair
needs a meaningful reproduction through the affected layer. Routine retries
within the agreed scope require no human checkpoint.

## Cold acceptance and progress

Every accepted increment needs fresh generated/reference parity from clean start
over all 18,000 any% input frames, comparing CPU/clocks, complete WRAM and
framebuffer, plus changed-key coverage, focused instruction/bus validation,
checks and applicable pattern tests.

For CPU-only config/emitter/runtime changes, reuse independent synchronization
and fallback evidence only under readiness dependency/provenance rules. Changes
to hardware, reset/input, observer, comparator or other shared paths require
fresh cold fallback parity and independent candidate synchronization. Generated
execution always needs its own fresh cold pass. Domain extension requires
independent validation first; low%/100% smoke runs cannot substitute.

Use identical ROM/movie/profile/seed/interval for before/after metrics. Record
generated/interpreted totals and fraction, per-key counts, validated distinct
compiled keys, declared pattern coverage, deferred/unsupported inventory and
comparable deltas. Keep call-target discovery separate. Expansion must deliver
positive observed compiled execution gain; correctness fixes may have zero
gain and must state that. Reduced replay scope is not coverage improvement.

Only after all mandatory checks pass, finalize evidence and update
`out/accepted.json`. Failed candidates leave it untouched. Continue from this
accepted state only if another increment is needed for requested scope.

## Timing and brief

Target warm focused replay within 60 seconds and each cold replay within
20 minutes; report build/cache/corpus/test/total-suite times separately. Report
and investigate overruns without weakening correctness gates. An actual timeout
is incomplete validation and cannot advance accepted state.

At completion or a needed decision, write `brief.md`: session outcome and
accepted/requested count, change and rationale, comparable execution progress,
exact domain and reused evidence, timing, debt, and links to source diff,
verdict, coverage and diagnostics. Failures include first disagreement, what
controls establish, recent experiments where relevant, next experiment or
specific choices, preserved candidate and accepted run IDs. Link raw output
instead of embedding it in the brief.

A session completes with accepted evidence for requested scope or a specific
external input/decision presented to the user. This specification does not
claim that any increment has already passed.
