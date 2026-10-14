# 05 — Live replay transactions

Status: resolved
Triage: ready-for-agent
Blocked by: None
Type: task
Owner: /root/implement_live
Worktree (retired after main integration; six source files retained): /home/jim/sm-decomp-architecture-20261014/05-live
Branch: architecture/perf-05-live-20261014
Baseline: 1ce6494d7e0fdc9b8623faf3f9d9f507f4a8abb0

## Scope

crates/pipeline/src/live_{bounded_family,projectile,ridley,plm}.rs.in; new live transaction template; generation registration portions of live.rs; crates/pipeline/tests/live.rs and live_frames.rs.

## Acceptance

Concentrate private full-WRAM transaction and commit policy, reuse scratch storage safely across threads and reentrant callbacks, and avoid copies for proved read-only/no-op paths where behavior permits. Preserve all pointer checks, refusal/error distinctions, panic containment, no canonical change on refusal, nested callback ordering and accepted-error behavior. Ordinary direct-execution families remain direct. Compiled C/Rust integration controls and bounded allocation/copy comparison.

## Comments

2026-10-14: Authorized by the user's selection of all five report candidates.
See ../spec.md for shared resource, source and evidence contracts.

## Completion evidence

Implementation and independent coordinator review complete in its separate worktree.
Commit: `0e57ff9` (`Reuse bounded live WRAM transactions and avoid read-only copies`).

- `live_transaction.rs.in` now owns bounded-adapter immutable admission,
  private full-WRAM storage, unwind containment and successful publication.
  Thread-local checked-out buffers are absent from their pool during callbacks;
  nested entries and ordered Ridley observer snapshots use separate storage.
  Buffer retention is bounded by the maximum synchronous nesting depth of each
  thread and ends with that thread. Ordinary families still execute directly.
- Admission/refusal reads canonical WRAM through a short immutable selector
  view, dropped before any transaction or callback. Proven disabled projectile
  scheduler, frozen/support-only Ridley and saturated PLM paths skip WRAM
  copying. Saturated projectile allocation retains its private transaction:
  it writes parameter word 0x1993, even with no available slot.
- Generation and source-dependency sealing register the shared template. The
  `live.rs` changes touch only template inclusion, immutable selector generation
  and the extra-dependency list; they do not alter compiler invocations.

### Verification

Eight compiled controls passed, using standalone `rustc --edition=2024 --test`
on the two std-only integration test files with `CARGO_MANIFEST_DIR` set to this
worktree's `crates/pipeline`, then `--test-threads=1` (one compiler at a time).
This avoids rebuilding the entire pipeline while exercising its actual Rust
templates and C wrappers. Logs are retained under
`/home/jim/sm-decomp-architecture-runs-20261014/05-live/`:

- `live-tests.log`: 5/5, including C-to-Rust nested callbacks, four concurrent
  native threads, dispatch error/panic, selector refusal/panic, full-WRAM
  preservation, pointer extent/alias rejection, PLM commit and saturated-pool
  behavior, and actual projectile disabled/saturated/empty-scheduler parity.
- The C nested/thread control also counts full-WRAM allocations: exactly two
  for warmed main-thread depth two, exactly eight for four new threads at depth
  two, and no additional allocations during further warm calls or failed reuse.
- `live-frames-tests.log`: 3/3. The actual Ridley ABI matches 240 original-ROM
  attack frames against the maintained interpreter, full WRAM/eight host words
  after each frame, opcode and eight allocation counts. Frozen/probe paths and
  host overlap/overflow remain controlled. C wrappers preserve refusal-to-C and
  accepted-error exit4 without retry; native same-bank host publication remains
  controlled. Expected panic messages belong to the containment controls.
- `git diff --check` passed. No gameplay translation or admission changed.
- No authenticated complete Live replay was produced: this isolated worktree
  has no current authenticated predecessor selection. The combined source
  passed all eight compiled controls again; the 240-frame ABI control does not
  claim original-ROM native Live parity.
- Independent read-only review by `/root/live_performance` found no concrete
  correctness defect in pool lifetimes, nested entry, thread isolation, no-op
  equivalence, pointer/error behavior or template dependency sealing. Existing
  controls do not independently compare every actual C observer's before/after
  snapshot bytes under nested callbacks. Other selectors and dispatchers can
  allocate internally; zero-allocation results describe adapter overhead only.

### Bounded performance evidence

`probe.rs` is a retained Rust reproduction tool. It extracts the exact baseline
template from commit `1ce6494`, generates baseline/current Rust staticlibs using
identical `rustc --edition=2024 --crate-type=staticlib -O`, and links identical
`cc -O2` drivers. Compiler: `rustc 1.101.0-nightly (c1070d693 2026-09-28)`;
C compiler: Ubuntu GCC 9.4.0. Each phase warms once then makes 20,000 calls
through the public C-to-Rust bounded-family interface. Source hashes and raw
outputs are retained alongside the tool; sequence was baseline/new/new/baseline.

| Path | Baseline full-WRAM allocations | New allocations | Baseline elapsed | New elapsed |
| --- | ---: | ---: | --- | --- |
| Accepted | 20,000 | 0 | 126.827 / 149.263 ms | 130.606 / 132.355 ms |
| Refused | 20,000 | 0 | 65.355 / 75.915 ms | 0.066931 / 0.066946 ms |

All runs returned the same checked result and full-state checksum. Full-copy
policy accounting (not a hardware bandwidth measurement): accepted calls keep
two 131,072-byte copies, 5,242,880,000 bytes per phase; refused calls drop from
one copy, 2,621,440,000 bytes per phase, to zero. Admission in this bounded probe
is deliberately tiny, so refusal timings isolate adapter overhead. They do not
predict a Live replay speedup or selector-heavy behavior. Accepted elapsed times
overlap and show no stable speedup. Shared-host process captures show another
`sm-pipeline` at 90–100% CPU during these observations.

### Cleanup and retention

Both project-utility passes succeeded after all tests/probes stopped:

```
/home/jim/sm-decomp/target/debug/sm-pipeline cleanup --root /home/jim/sm-decomp-architecture-runs-20261014/05-live --apply
/home/jim/sm-decomp/target/debug/sm-pipeline cleanup --root /home/jim/sm-decomp-architecture-runs-20261014/05-live --deduplicate --apply
```

Both report zero eligible bytes (standalone control/probe outputs are outside
sealed run formats). Removed only seven identified unsealed executables and
static archives afterwards, reclaiming **77,390,418 bytes**; exact paths/sizes
are in `removed-intermediates.txt`. Test-created `/tmp/sm-live-*` directories
self-removed on success. Retained run data was 20,405 bytes before this Markdown
record, consisting of source, logs, checksums and cleanup receipts; no evidence
store or authenticated verdict depends on these disposable builds. Remaining
filesystem space: **186,778,726,400 bytes**. The worktree stays intact and
separate pending review/integration, as requested; it has not been integrated.
