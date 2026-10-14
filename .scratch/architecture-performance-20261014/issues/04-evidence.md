# 04 — Authenticated evidence restoration

Status: resolved
Triage: ready-for-agent
Blocked by: None
Type: task
Owner: /root/implement_evidence
Worktree: /home/jim/sm-decomp-architecture-20261014/04-evidence
Branch: architecture/perf-04-evidence-20261014
Baseline: 1ce6494d7e0fdc9b8623faf3f9d9f507f4a8abb0

## Scope

crates/evidence/src/{tree,trace,file,image,run}.rs and necessary evidence tests/examples.

## Acceptance

Authenticated restoration owns bytes, counts and one exact-byte digest per restored member. Inventory/closure verification reuse verified accounting within one traversal while preserving independent precommit, reopen, publication and post-cleanup checks. Keep exact hashes/roots/counts and rejection controls. Bounded before/after restoration comparison.

## Comments

2026-10-14: Authorized by the user's selection of all five report candidates.
See ../spec.md for shared resource, source and evidence contracts.

2026-10-14: Implemented traversal-local restoration accounting. Stream restoration
returns the computed exact-byte digest/count only after verifying byte, newline,
event and digest seals; inventory and tree verification reuse that result. File
and image members retain their existing single digest pass. Public entry points,
storage formats and independent lifecycle authentication remain unchanged.

2026-10-14: All 15 evidence tests pass, including three new public-operation tests
covering all member kinds, empty members, buffer/segment limits, forged stream
seals and run counts, and corruption after reopening. The bounded Rust probe
compared identical typed-heavy, literal-heavy and mixed sealed stores; results
and limitations are recorded below.

## Completion evidence

Commit: `35ef637` (`Reuse authenticated stream accounting during evidence restoration`).
Branch: `architecture/perf-04-evidence-20261014`.
Worktree is clean and intentionally retained for review/integration as requested;
it has not been integrated or removed. Root owns eventual integration cleanup.

Implementation changes only `crates/evidence/src/{tree,trace}.rs`, with public
operation tests, the bounded Rust `examples/authenticate.rs` probe and README
documentation. `restore_stream` returns computed `MemberInfo` after all four
stream seals pass; tree inventory and verification reuse the result. Existing
public signatures, ingestion, object/pack formats, roots, file/image adapters,
imports, run closure counts and independent lifecycle checks are preserved.

Validation: `cargo test -p evidence --locked --offline --jobs 1` passed all 15
tests, including the three new tests. `rustfmt --check` on changed Rust files and
`git diff --check` passed. Final test log:
`/home/jim/sm-decomp-architecture-runs-20261014/04-evidence/logs/cargo-test-final.log`.
The initial test compilation used a private helper from an integration test;
this was corrected to public `Store::get` before the successful full rerun.

Performance evidence: retained under
`/home/jim/sm-decomp-architecture-runs-20261014/04-evidence/`.
`logs/comparison.md` is the Rust-generated summary; the four `before/after-*.toml`
files contain every sample, exact run/observation root, member digest and byte
count. All identities and counts match across all four reports. Probe source,
both executables, optimized `sm-evidence` reader and before/after source snapshots
are retained in `retained/`; immutable stores and trusted run ids are in `data/`.

Same compiler (`rustc 1.101.0-nightly c1070d693`, x86_64), release library profile
and identical probe source/flags (`opt-level=3`, `embed-bitcode=no`,
`strip=debuginfo`) were used against baseline `1ce6494` and commit `35ef637`.
Data was prepared once with the baseline reader. ABBA order, 9 repetitions per
block, 18 samples per revision/operation; store opening/preparation are excluded,
physical seals are checked before timing, and the bounded decoded-block cache is
warmed. Root reserved a quiet build/test window. Host snapshots retained in
`logs/host-{before,after}.txt` showed no compiler/test workers, but background
Codex/UI processes remained; CPU placement/frequency were not pinned.

| Synthetic workload | Restored bytes | Inventory median before → after | Tree verification median before → after |
| --- | ---: | ---: | ---: |
| Typed stream | 15,671,331 | 48.603 → 41.966 ms (13.7% lower) | 48.757 → 41.061 ms (15.8% lower) |
| Literal stream | 16,777,216 | 33.342 → 24.973 ms (25.1% lower) | 33.694 → 24.931 ms (26.0% lower) |
| Mixed stream/file/image | 8,144,932 | 17.707 → 15.142 ms (14.5% lower) | 17.342 → 15.117 ms (12.8% lower) |

These are warm-cache synthetic restoration measurements; the small deduplicated
stores do not establish an end-to-end pipeline or Live replay speedup. Binary
hash validation caught Cargo reusing the baseline library when sharing a target
across worktree paths; a package-scoped release clean and forced optimized rebuild
fixed this before any reported measurements. Final probe hashes differ and the
optimized library's restoration symbol was checked. Build logs and binary/source
SHA-256 manifests preserve this evidence.

Cleanup: after all writers finished, ran
`sm-pipeline cleanup --root /home/jim/sm-decomp-architecture-runs-20261014/04-evidence --apply`
and the corresponding `--deduplicate --apply` using the existing reader at
`/home/jim/sm-decomp-production-execution/bin/sm-pipeline`. Both succeeded with
zero utility-managed bytes: this synthetic probe has no pipeline verdict, so
its private Cargo target is outside that utility's eligible run format. Removed
only that identified unsealed target with `CARGO_TARGET_DIR=.../04-evidence/target
cargo clean --locked --offline`; `du -sb` measured 495,388,265 apparent bytes before
removal, and Cargo reported 1,722 files / 582.2 MiB removed. The target no longer
exists. Repeated both utility passes successfully (zero bytes), verified all
stored pack/run-id SHA-256 values unchanged, and independently authenticated all
three retained runs with retained `sm-evidence verify` after cleanup. Receipts
are `logs/cleanup*-final.toml`, `cargo-clean.log`, `store-sha256-after-cleanup.txt`
and `post-cleanup-{typed,literal,mixed}.toml`. Available filesystem space after
cleanup: 187,031,937,024 bytes. Retained dependencies are the small sealed stores,
run-id files, exact readers/probes, source snapshots and diagnostic/measurement
logs; no shared production evidence was modified.
