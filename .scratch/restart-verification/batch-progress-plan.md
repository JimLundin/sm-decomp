# Batch publication of ordinary progress observations

2026-10-16. Implementation plan against main
`193cad941713b3539d23e08238a79e7fb09dde61`; source inspection only. This is a
bounded follow-up to [progress-reporting-cost.md](progress-reporting-cost.md),
independent of active gameplay scope 20. No implementation claim is made.

**Implement repeated `--update` arguments on the existing `progress update`
command.** Read the existing TOML files, publish their ordinary observations in
argument order under one registry lock, observe the complete resulting registry
once, and persist once. Keep each observation's own ID, stage, source identity,
evidence context, limitations and evidence pointers. No batch manifest, registry
schema migration, new admission path or verification framework is needed.

## Public behavior

```sh
sm-pipeline progress update --root /home/jim/sm-decomp \
  --update translated.toml --update verified.toml --update integrated.toml
```

A single `--update UPDATE.toml` retains its current meaning and file schema.
Require at least one update. Preserve option order; only `--update` may repeat,
and only for `progress update`. Other duplicate or unknown options still fail.
Keep the existing one-MiB limit per TOML file. A small explicit maximum of 64
files bounds incoming text by the existing 64-MiB metadata budget; this is a
proposed input limit, not a measured optimum. Normal three-stage publication is
well below it. Resolve update and evidence paths against `--root` as today.

Print the existing final status once. Do not infer stages, combine their IDs,
fill in missing evidence, or emit an acceptance label. Batch whatever stages
are already achieved together; do not delay a `translating` assignment or an
earlier handoff just to fill a batch. The existing handoff rules require
independent stage observations ([runbook](../../docs/agents/translation-progress.md:16)).

The current parser's only blocking feature is its single `Option<PathBuf>` and
blanket duplicate-key rejection; command dispatch already routes all progress
arguments to this module. Sources:
[Options and parser](../../crates/pipeline/src/progress.rs:249),
[CLI](../../crates/pipeline/src/progress.rs:297),
[outer dispatch](../../crates/pipeline/src/main.rs:310),
[UpdateInput](../../crates/pipeline/src/progress.rs:107).

## Small Rust change

1. Replace `Options.update` with `updates: Vec<PathBuf>`. In `options`, collect
   repeated `--update` values separately from the map of unique options, then
   resolve them in argument order. Retain the current option validation and
   add the nonempty/count checks. Update the private `Options` constructors.
2. Keep `update(options)` as the command entry point. For every path, validate
   `.toml`, call `bounded_text(path, MAX_UPDATE)`, deserialize `UpdateInput`, and
   call existing `input_update`. Collect all prepared updates before acquiring
   the lock. Add path context to errors so a failed file is identifiable.
3. Extract the current publication tail into a private function such as
   `publish_updates(options: &Options, updates: Vec<Update>) -> Result<Registry>`.
   Require a nonempty vector. Acquire `registry_lock` once with the existing
   timeout, read the authoritative registry once, and run `check_current` once.
4. For each prepared update, rehash every evidence pointer under that lock and
   compare it with the prepared digest, then call `insert_update` on the local
   registry. Preserve argument order and add the observation ID to failures.
   The local registry can be discarded on any error; do not persist inside
   this loop.
5. After all insertions succeed, call `observe_evidence` once and `persist`
   once, retaining the lock through both. Do this even if every insertion is
   an identical retry. Return the final registry for existing status printing.

This is the existing single-update sequence with its input/insertion steps
made plural. The prepared-update seam also makes the pre-lock/under-lock
recheck contract directly testable without timing-dependent thread races.
Source boundaries: [input preparation](../../crates/pipeline/src/progress.rs:757),
[insertion](../../crates/pipeline/src/progress.rs:785),
[current publication](../../crates/pipeline/src/progress.rs:824),
[source recheck](../../crates/pipeline/src/progress.rs:442),
[registry validation](../../crates/pipeline/src/progress.rs:601).

Do not optimize per-update routine-map creation or share pre-lock evidence
hashes with the under-lock recheck in this change. Existing input preparation
and all new evidence rechecks remain. The global observation pass retains its
own per-call digest cache and includes old and superseded observations
([observe_evidence](../../crates/pipeline/src/progress.rs:1076)).

## Identity, failures and recovery

| Case | Required result |
| --- | --- |
| Existing ID, otherwise identical content | `insert_update` keeps the original observation timestamp and adds nothing. This applies within a batch as well as across retries. |
| Existing ID, different stage/content/context/evidence | Reject the whole proposed publication before `persist`; preserve the existing conflict message. |
| Bad later input, unknown routine, stale source ID or evidence recheck failure | No earlier batch observation reaches disk. Parsing/preparation or the locked in-memory candidate fails as a unit. |
| Supersession | Target must already exist or appear earlier in the argument order. Keep same-stage/same-source checks; reject forward references, self-references and duplicate targets. Do not sort or introduce forward-reference resolution. |
| Replacement later becomes stale | The superseded observation remains excluded; existing history is retained. |
| Some IDs already published, others new | Accept identical existing IDs and append the new observations in input order; preserve all unrelated observations. |
| All IDs already published | Still observe evidence and persist, repairing a missing or stale dashboard. Observation timestamps remain unchanged. |
| Missing or corrupt authoritative registry | Fail through `read_registry`; do not create a partial replacement registry from the batch. |

Sources: [shape rules](../../crates/pipeline/src/progress.rs:673),
[supersession graph](../../crates/pipeline/src/progress.rs:709),
[ID and timestamp rules](../../crates/pipeline/src/progress.rs:803).
New observation timestamps continue to come from `input_update`; the registry
timestamp follows the last newly inserted observation, as today. A mixed retry
does not promise byte-identical output to a fresh batch because observation
timestamps and retained history can differ.

The JSON and HTML files are individually atomic, **not one atomic pair**.
`persist` renders first, then writes authoritative JSON, then HTML. An I/O error
can occur after the complete JSON has been replaced. Report the failure and
allow the exact same command to be retried; do not roll back the registry or
skip persistence on a no-op retry. A successful retry repairs the dashboard
from current JSON while retaining any intervening valid observations.
Sources: [atomic_write](../../crates/pipeline/src/progress.rs:395),
[persist](../../crates/pipeline/src/progress.rs:423),
[ordinary retry comment](../../crates/pipeline/src/progress.rs:843).

`AdmittedProgress`, `prepare_admitted`, `publish_admitted`, and their
authentication/projection functions remain private to trusted result admission.
Do not deserialize their payloads through the public command, reuse their fixed
three-stage construction, or refactor their accepted-projection repair into
this ordinary path. Their existing test is useful regression coverage, not an
API to expose ([admitted publication](../../crates/pipeline/src/progress.rs:848),
[repair behavior](../../crates/pipeline/src/progress.rs:928)).

## Focused verification for the implementation

Use the existing temporary-directory test support in
[progress/tests.rs](../../crates/pipeline/src/progress/tests.rs:3). Keep the
existing single-update tests on the same command entry point after adapting
the options field. Add the following small behavior tests; combine related
cases in table-driven tests rather than creating a new harness.

| Additional check | Existing seam or regression coverage to reuse |
| --- | --- |
| Parser accepts one/several files in order, rejects zero/too many, and still rejects unrelated duplicate flags. | `options`, [parser](../../crates/pipeline/src/progress.rs:258). |
| Three distinct stages publish together with exact IDs, contexts, evidence and independent totals; no `live-exercised` coverage is inferred. Include an older observation whose evidence changed, proving the final global observation still excludes stale work. | [single-update command](../../crates/pipeline/src/progress/tests.rs:304), [stage independence](../../crates/pipeline/src/progress/tests.rs:337), [context/staleness](../../crates/pipeline/src/progress/tests.rs:390). |
| A valid first update followed by a conflicting ID, unknown routine or invalid supersession leaves both report files byte-for-byte unchanged. A valid earlier-target supersession succeeds; the reverse order fails. | [ID retry/conflict](../../crates/pipeline/src/progress/tests.rs:353), [supersession rules](../../crates/pipeline/src/progress/tests.rs:456), [ordinary correction command](../../crates/pipeline/src/progress/tests.rs:498). |
| Prepare two observations, then change the second evidence file before calling the private publication function: rejection leaves both reports unchanged. Repeat with a changed source pin/inventory and with a stale input source identity. | New private publication seam; [existing command checks](../../crates/pipeline/src/progress/tests.rs:304). That existing test checks a supplied hash mismatch before preparation completes; it does not itself exercise the intervening-change window. |
| Retry a complete batch: IDs and observation timestamps remain stable. Force HTML-write failure with an obstructing directory at the HTML path; assert complete JSON exists, remove the obstruction, retry the same batch, and check HTML equals `render(read_registry(...))`. Also exercise a mixture of already-published and new IDs. | [ordinary retry behavior](../../crates/pipeline/src/progress.rs:843); the trusted [admitted test](../../crates/pipeline/src/progress/tests.rs:589) already checks lost-dashboard repair, but ordinary batching needs its own recovery case. |
| Two concurrent batch writers, each with distinct IDs, leave all observations and valid JSON/HTML. Use the real publication function, without assuming writer order. | Extend [concurrent writers](../../crates/pipeline/src/progress/tests.rs:547); retain [bounded lock contention](../../crates/pipeline/src/progress/tests.rs:579). |

Run the focused Rust module tests once after implementation:

```sh
cargo test -p pipeline --locked --offline --bin sm-pipeline progress::tests::
```

Use the repository's normal formatting check. No ROM run, Fixture expansion,
live capture, cross-project suite or benchmark is required to validate this
reporting-only change. Do not introduce test-only performance counters merely
to assert the source's loop structure. If a compiler/test failure requires a
follow-up, address that failure and rerun the affected checks.

## Files and cost evidence

Expected implementation edits are limited to
`crates/pipeline/src/progress.rs`, `crates/pipeline/src/progress/tests.rs`, and
the command example/behavior description in
`docs/agents/translation-progress.md`. The `admitted_options` constructor needs
an empty vector mechanically; trusted admission semantics do not change.
No Cargo dependency, `main.rs` dispatch, HTML template or registry schema change
is necessary. The runbook should accurately describe individual atomic file
writes and retry recovery when updating its current publication description.

For three ordinary updates, source control flow changes the following counts:

| Operation | Three separate commands today | Proposed one batch |
| --- | --- | --- |
| Lock acquisition and full registry read/validation | 3 | 1 |
| `check_current` pin/inventory check | 3 | 1 |
| Whole-registry `observe_evidence`, summary and history observation | 3 | 1 |
| `persist` render/serialize plus JSON/HTML replacement pair | 3 | 1 |
| Input preparation, evidence rechecks, insertion validation | Per observation | Still per observation |

These are call-count reductions, **not measured speedups or I/O-byte savings**.
History observation scans/hashes Rust source only when a history exists
([history::observe](../../crates/pipeline/src/progress/history.rs:313),
[current_files](../../crates/pipeline/src/progress/history.rs:169)). Inputs can
share evidence, stale updates can short-circuit, and filesystem caching can
affect elapsed time. The earlier reported timing is contextual evidence in the
prior note, not a benchmark of this proposal. Remaining costs include every
per-update routine membership scan and input evidence rehash.

The coordinator recovered these installed-command wall times for completed
scope 19 from `item_completed.duration` in the retained
[integrator rollout](/home/jim/.codex/sessions/2026/10/16/rollout-2026-10-16T14-57-41-01a144ca-27ba-7130-a9e2-a615c7c80e3a.jsonl):

| Observation | Seconds |
| --- | ---: |
| translating | 9.647880525 |
| translated | 8.354447611 |
| verified | 17.190793526 |
| integrated | 18.082846441 |
| live-exercised | 15.305398430 |

The four closeout commands total 58.933486008 seconds; the three closeout
commands excluding live exercise total 43.628087578 seconds. The coordinator
reports 53.123 seconds for the games and an approximately 873-second milestone
envelope. These are coordinator-recovered measurements, not independently
recovered or rerun in this audit. They do not identify hashing, persistence,
lock waiting, or any other phase, nor authenticate the installed executable's
source. They make a small batching change worth considering, not a new
performance investigation prerequisite.

For scale only, if implementation later saves 10, 20 or 40 seconds per similar
873-second milestone, that is about 1.1%, 2.3% or 4.6% of that envelope. Those
are conditional arithmetic examples, not forecasts. Repeated milestones
amortize one small implementation cost: with implementation time `I` seconds
and measured saving `s` seconds per milestone, time breaks even after roughly
`I / s` milestones. Do not expand this task into caches, profiling infrastructure
or unrelated registry redesign to chase the larger percentages. Retain the
three-file implementation boundary and focused tests above; reconcile the
original production source's ownership before any implementation assignment.

The lock protects the registry, not outside evidence writers. Retain the
existing under-lock recheck and final observation; do not promise an atomic
snapshot of mutable external files. Larger batches also hold the lock for
their combined new-input checks, so use batches of jointly ready observations.
This proposal removes repeated report-wide work without broadening gameplay
verification or making throughput claims.

## Cleanup and handoff

Only this note was written. No code changes, builds, tests, benchmarks, game
runs, captures, worktrees, new agents or producers were created. Cleanup
reclaimed 0 bytes; filesystem free space at inspection was 102,826,082,304
bytes. No evidence dependencies were added or removed. Implementation remains
for the coordinator to assign independently of active scope 20.
