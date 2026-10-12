# Fixture execution prototype results

The bounded projectile experiment supports prompt completion, parallel
independent chains and aggregate metrics. A single-threaded preloaded helper
with fresh fork children adds savings while preserving isolation in the
exercised controls. Production integration is a subsequent task.

## Measurements

Identical 142-fixture samples used the first six cases of all 23 chains (four
singletons) and 24 late horizontal/vertical retained snapshots. Rust preceded C.
Full WRAM, eight host words and exact returns matched retained results; signature
maps matched across every mode and reversed order.

| Execution | Workers | First comparison | Final driver comparison |
| --- | ---: | ---: | ---: |
| Existing process/receipt loop, reproduced | 1 | 9.576–9.615 s | 9.389–9.601 s |
| Prompt fresh processes, aggregate metrics | 1 | 2.197 s | 2.265 s |
| Prompt fresh processes, aggregate metrics | 2 | 1.277 s | 1.656 s |
| Preloaded helper, fresh fork children | 1 | 1.533 s | 1.448 s |
| Preloaded helper, fresh fork children | 2 | 0.889–0.909 s | 0.915–0.943 s |
| Fork, reversed chain order | 2 | 0.948 s | 1.030 s |

Final fork samples were about ten times faster than the reproduced baseline.
Prompt one-worker fresh execution alone saved about three quarters of wall
time; one-worker fork saved another 36% relative to that fresh path.

One full prototype check: **1,840 fixtures**, 1,817 continuation edges,
**12.622 s wall**, 2.300 s summed direct fixture-child CPU, two workers, exact
retained equality. This preceded extracting the unchanged pair loop into a
shared function for refusal controls; the final driver passed the repeated
small comparison and controls. This is not production family admission.

Wall includes worker acquisition, helper startup/ROM load, fixture preparation,
execution/comparison, durable case/timing writes, helper shutdown and project
cleanup. Excludes compilation, initial authentication, final summary/aggregate
metrics writing and later retention deduplication. Baseline uses real OwnedChild,
per-subject metrics and 20 ms polling; fresh uses the same owner with 200 us
polling and aggregate metrics. Fork uses helper-side wait4 and prompt polling.
These prepared-binary experiments exclude production build/seal/admission cost.

Final sampled load averages were about 0.20–0.77; no compilation overlapped.
Child CPU excludes helper/orchestrator CPU. Fork avoids exec and repeated ROM
loading and uses a newly linked helper; this does not isolate each cause of the
savings. WSL results do not establish native Windows performance.

## Controls and limits

Passed: C WRAM/ROM mutation followed by pristine-parent recovery; crash/timeout
recovery; timeout descendant group kill and explicit reap; helper cancellation;
orchestration SIGTERM with helper/child/descendant gone; truncated-reply rejection;
unsupported Rust refusal through the experiment pair function with no C output;
expected-return failure blocking descendants while another chain completes;
repeat/reversed-chain exact equality.

The parent executes no fixtures. This does not prove reset correctness for a
process executing multiple fixtures. Callback/PLM initialization and live replay
remain untested. Production requires bounded I/O framing, robust EINTR/error and
exceptional teardown, complete accounting and explicit source/binary seals.

## Evidence and reproduction

Root `/tmp/sm-fixture-execution-prototype` retains comparison-run-2/3, full-run-1,
control-run-2/3 and failed diagnostic control-run-1/comparison-run-1. Failed runs
are excluded: incorrect 4 MiB ROM assumption, then tuple-shaped continuation
writes. Actual ROM is 3,145,728 bytes. Per-case inputs/outputs, stdout/stderr,
logical argv/timings and signature maps remain. summary.toml collects successful
samples. retained-sha256.toml records 59,052 retained file/source/dependency hashes;
bin/retain authenticate rechecks it. Retention checked each sample signature.
This inventory is not a production source-to-execution seal.

Sources are also in [prototype/](prototype/), intentionally using absolute paths.
Rebuild at the original temporary path with retained native archive, or reconstruct
it using Rust prepare.rs and the authenticated C cache. Compiler: Rust
1.101.0-nightly (c1070d693 2026-09-28), GCC 9.4.0.

```sh
cargo build --manifest-path /tmp/sm-fixture-execution-prototype/Cargo.toml --locked --offline --release
/tmp/sm-fixture-execution-prototype/target/release/fixture-prototype controls /tmp/sm-fixture-execution-prototype/control-NEW
/tmp/sm-fixture-execution-prototype/target/release/fixture-prototype run /tmp/sm-fixture-execution-prototype/comparison-NEW
```

Use fresh output directories. Retained bin/ contains final binaries; rebuilding
restores the helper path expected by the driver. Dependencies preserved unchanged:
original family/source under /tmp/sm-pipeline-upgrade-integration; sealed C cache
under /tmp/sm-pipeline-upgrade-workers-2/build-cache; current pipeline/model Rust
sources; shared worker root; retained native archive. Existing family
authentication succeeded (original-authenticate.toml). No worktrees or Python
source/runtime introduced; production code unchanged.

## Cleanup

Writers finished. Both project cleanup passes succeeded on every run, including
failures: retained sm-pipeline-final cleanup --root RUN --apply and
cleanup --root RUN --deduplicate --apply, or equivalent structured TOML. Each
reported zero bytes; utility does not recognize unsealed prototype fixture trees.

Rust retention hard-linked 40,734 identical immutable files and checked hashes,
sharing 1,483,691,388 logical bytes. Allocated size dropped from 1,627,906,048 to
569,184,256 bytes. cargo clean removed 1,052 build files (284.9 MiB reported).
Removed identified copied/generated .o intermediates and the disposable prepare
executable after retaining their reproducible source and linked native archive.
Final retained allocated size 287,424,512 bytes; free disk 217,824,473,088 bytes.
Original evidence authenticated; 59,052 retained files authenticated after cleanup.

## Next implementation

Deepen the fixture execution module in [design.md](design.md). Its interface
accepts declared work and existing adapters, hiding scheduling/admission/child
ownership/evidence. First integrate prompt completion, parallel independent
chains, aggregate metrics, failure blocking and canonical report order. Then add
the preloaded fork adapter with source/binary seals, keeping fresh execution for
comparison. Extend callback/PLM after initialization/capture controls, then
validate affected live-replay admission. Shared rules give locality; the module
provides leverage across existing family/callback/PLM adapters at this seam.
