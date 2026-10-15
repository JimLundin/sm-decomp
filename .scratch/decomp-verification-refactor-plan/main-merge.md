# Performance refactor merge onto main

Status: resolved
Updated: 2026-10-15

## Authorization and result

The user explicitly requested merging the performance refactor onto main without
requiring the pending verification status. This supersedes the previous stop for
source integration. The stopped implementation test and measurement campaign was
not restarted.

- `40ee572` merges `performance/verification-final-20261015` at
  `3e7c21f8580bca7949344a8e41ad5c953ce4772b` onto main.
- `4f01aa8` merges the final scheduler test correction from
  `performance/batch-work-graph-20261015` at
  `d0334b5be6b316bc6b6813c5d08eaa20cfe04d13`.
- Seven add/add conflicts affected handoff Markdown only. Six retained main's
  later completion records; optimized-tools retained the assembly's later record.
  Source, Cargo manifests and documentation outside those handoffs matched the
  final assembly after the first merge.
- The only subsequent Rust change wraps one condition in `tests/metrics.rs` to
  satisfy the existing formatter. `cargo fmt --all --check`, source whitespace
  checks and ancestry checks for both integrated branch tips pass.

Historical failed and passing results remain recorded. No new whole-workspace,
Native matrix or performance result is claimed. The isolated installation and
its separate production transition retain their previous state. This merge does
not resolve the broader unfinished measurement/installation deliverables.

## Completion evidence

Cleanup and readback records are retained at:
`/home/jim/sm-decomp-performance-final-proof/main-merge-retirement/`.

No Cargo, compiler or verification writer remained before retirement. Both
cleanup passes succeeded for the final-proof and graph-proof roots using the
retained `6bc0a78/tools/sm-pipeline`:

```text
sm-pipeline cleanup --root PROOF --apply
sm-pipeline cleanup --root PROOF --deduplicate --apply
```

All four receipts report zero further reclaimable bytes; deduplication checked
58 final-proof and 4,150 graph-proof snapshots. No new producer run was started.

The integrated final, graph and Offline extraction Git worktrees were removed
with `git worktree remove --force`, after checking each tip is an ancestor of
main and preserving the graph's two untracked generated inputs. Branches remain.
Referenced source and inputs were copied first, restored at their original
absolute paths without Git metadata or build trees, and checked against
`preserved-source.sha256` afterward: all 33 files match. The extraction's prior
retention audit supplies its single required source file. The final and graph
retained executable hash manifests also pass after retirement.

The extraction target was removed with its worktree. The released graph target
was cleaned with:

```sh
cargo clean --target-dir /home/jim/sm-decomp-performance-metrics-target
```

Cargo reports 8,659 files / 4.2 GiB removed from that target. The four selected
worktree/target roots occupied 13,657,100,288 allocated bytes before retirement;
their restored source/input snapshots occupy 4,145,152 bytes. Reclamation from
those roots is **13,652,955,136 bytes**. Available filesystem space afterward was
**109,158,383,616 bytes**. The separate small preservation backup remains in the
retirement directory. Shared caches, other worktrees and retained proof/tool
directories remain available.

The two retained extraction ordinary verdicts authenticated after retirement
with their original copied readers and compiler PATH unavailable. All seven
graph readbacks also passed with compilers unavailable: five ordinary manifests,
the captured Native manifest and the retained rejected Native sequence. Exact
outcomes are in `graph-readback.log`. These checks authenticate existing records
after cleanup; they do not rerun the stopped producer/test campaign.
