# Current fixture execution costs and isolation

Status: resolved
Blocked by: None
Triage: ready-for-agent
Type: research

## Answer

Inspected source revision `5b66ee83fb11d6f13e2aadef2b6d26fd54b8dacb`.

- `family.rs::verify` forces one case worker whenever any fixture has
  `continue_from`. `ordered_tasks` requires unique complete `case_order` and
  predecessor-before-continuation; results are sorted by that order. Without
  continuations, execution already overlaps independently of `case_order`.
- `tools/port/observed-projectile-ownership.toml` declares 1,840 fixtures,
  1,817 continuation edges and 23 roots, with no branching: twelve 111-fixture
  horizontal/vertical chains, seven 72-fixture Ceres chains, four single cases.
  `docs/DECOMP_BATCH.md` describes cross-routine sequences, without promising
  global execution order among independent roots. The earlier
  `.scratch/pipeline-review/optimization-comparison.md` explicitly recommends
  parallel independent cases/chains and ordered steps within each chain.
- Ordinary family, callback and PLM adapters prepare a fixture, launch Rust
  before C, then compare full WRAM and adapter-specific outputs. Each launch
  uses a new process. `builds.rs::execute_timeout` checks completion every 20 ms;
  `metrics.rs::OwnedChild` owns waitid/wait4, cancellation and process-group
  cleanup. Its other wait path checks every 1 ms.
- Current continuations check equal predecessor WRAM and, for projectile host
  fixtures, equal host words; they do not require the predecessor's whole
  comparison to have passed. An expected-return failure with equal state can
  therefore still allow descendants to execute, while the family is rejected.
- Per-process metrics/timing files are durable TOML. Receipt overhead is another
  cost; child CPU cannot attribute all elapsed time to polling.

Retained historical run:
`/tmp/sm-pipeline-upgrade-integration/out/ticket-03/final-projectile/1791813609706347-2-0`.
Its family receipt records 314.782 seconds, including builds and evidence
generation. A bounded read of 16 subject receipts records spawn-to-reap
20.56–29.58 ms, child CPU 2.54–8.32 ms, worker waits 0.176–0.401 ms, execution
receipts 20.88–30.47 ms and metrics receipt totals 29.36–66.94 ms. These are
retained observations under that run's host conditions, not a new benchmark,
pure-routine timing or an estimate of the speedup.

Sample medians: spawn-to-reap 23.579 ms, child CPU 4.163 ms, execution timing
24.233 ms, metrics scope 44.664 ms, worker wait 0.284 ms. Retained manifest and
current manifest SHA256 both equal
`82bfdf14d2eac1a5ff06a3a4889dea1bf83cd3c7373c5a754e5da18559fc0b2b`.

## Persistent execution constraints

The ordinary projectile C harness loads WRAM, ROM and eight projectile host
words per process; the Rust harness constructs fresh owned WRAM/ROM and host.
Bank86 standalone mutable spawn globals correspond to those eight host words;
its inspected static tables are const. RNG is WRAM-backed. The linked oracle
also includes wider RTL/emulator state, state-recorder and APU-queue statics;
reachability and reset completeness for each admitted family remain unproved.

Callback/PLM harnesses initialize the SNES and SPC objects and can hold callback
logs/counters, fallback/decompression counters and amount pointers outside WRAM.
WRAM plus projectile host words is not a general reset contract.

A pristine, single-threaded preloaded helper could fork a fresh child per fixture
to retain process isolation while amortizing exec/ROM loading. This alternative
still needs independent comparison, fork-safety, process ownership, cancellation
and evidence-protocol proof. It is not implemented or selected.

Follow-up source inspection established synchronous initialization in `SnesInit`,
SNES allocation and SPC creation/initialization; no upstream thread-creation
calls were found. Ordinary/projectile helpers can preload raw ROM directly.
Callback/PLM helpers must preserve the once-initialized patched ROM, emulator
and SPC image. Calling SnesInit repeatedly would also rewrite its remembered
patch baseline. Helpers must stay single-threaded and never execute fixtures in
their parent image. Fork children require isolated logs, flushed stdio and a
single waiter/accounting owner.

Current family authentication explicitly reads per-case state, stdout results,
projectile host words and callback captures. Its inventory excludes filenames
ending `.metrics.toml`, allowing aggregated fixture metrics. Generated
`case.toml` and `*.timing.toml` are sealed even though authentication does not
individually interpret them; historical copies must remain unchanged. Existing
atomic TOML writes sync both file and directory, making repeated receipt writes
another plausible cost. Public generic-exec receipt contracts should remain
separate from fixture aggregation.

An extracted scheduler/helper must be added to each affected adapter's explicit
source/harness seals. Candidate compiler dep-info does not automatically include
Rust orchestration modules or every generated-host input.

## Completion evidence

Read-only source/receipt investigation plus local Markdown and glossary edits.
No verification run, build, temporary fixture, worktree or evidence-store write
was created. Run cleanup has no new run scope; bytes reclaimed: zero.
Historical evidence dependencies were preserved.
