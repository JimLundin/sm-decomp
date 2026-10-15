# Offline architecture implementation checkpoint

Updated: 2026-10-15
Owner session: `01a13c3b-d421-7bb3-80a2-ec91045678e7` (`/root`)
User authorization: approved the rendered design and instructed "go ahead".

## Source and ownership

The complete assembled implementation is in
`/home/jim/sm-decomp-offline-verifier`, branch
`architecture/offline-verifier-20261015`. Frozen verification source is
`15d5a7e`; subsequent handoff/checkpoint commits change documentation only.
Main is at `5c011eb` and will receive the verified branch after the final gates.
Main's foundation commits were confirmed identical to worker commits `446dd64`
and `aeb6632`; the assembly merge preserves the newer implementation and imports
main's approved tracking documents.

Installed production policies, registries, claims, Original evidence and
unrelated mutation worktrees remain owned by their existing sessions. This
infrastructure task grants no gameplay Translation stage.

Shared process capacity remains
`SM_PIPELINE_ROOT=/home/jim/sm-decomp/out/pipeline`, `SM_PIPELINE_WORKERS=4`.
All Cargo work is locked/offline; the optimized verification build uses one job.

## Implementation

- Foundations are integrated and ticket 02 is resolved. Source identity and
  operation-local source context belong to `decomp-source`; process execution,
  caching, leases, bounded IO and accounting belong to `tool-execution`.
- Ordinary and Native verification, Fixture declarations, recipes, scheduling,
  comparisons and retained admission belong to `offline-verification`.
  Recipe/compiler/Fixture decisions have one private implementation. Temporary
  forwarding modules and obsolete Native recipe code are removed.
- Pipeline owns application composition, Cargo package selection, Translation
  approval, the installed-controller executable fence and cleanup. Ordinary
  admission authenticates retained producer provenance without requiring
  today's executable identity. Source stamps remain conservative and cover
  all three extracted crates.
- Final Native implementation `f5433bb` is integrated as `da02c90`; its cleanup
  evidence `a59690e` is integrated as `e0dc9ab`. Final Cargo policy/caller change
  `8f7782b` is integrated as `e299e5a`, with handoff evidence at `600eb74`.
- Final private-interface/source-publication cleanup is `4c5c03d`. The
  source freeze/assembly merge is `15d5a7e`.

## Verification

- Baseline at unchanged Rust source `a6d53c5`: 31 passed, two optional
  measurements ignored. Log `out/offline-architecture/baseline-20261015/tests.log`,
  SHA-256 `ba041cd4d4147072c3e04b20f9fd2e5178ca204e35101c10ab1fafece7cebdde`.
  Both cleanup passes succeeded; do not rerun this baseline.
- Foundation checks: 35 distinct tests passed; see ticket 02 and
  `handoff-foundations.md`. Intermediate verifier checks: 28 passed, one optional
  real reservation proof ignored; log
  `out/offline-architecture/verifier-unit-20261015/tests.log`, SHA-256
  `a782707a489ad27dab325c0dfb026efd39493e888fb49b89e1c88a423df09398`.
  Both scoped cleanup passes succeeded and its empty TMPDIR was removed.
- Native worker: all 11 owned tests passed, and pipeline production/test
  compilation passed. Caller maintained-source checks passed. These are
  implementation evidence, not final real Native proof results.
- Combined `cargo check --locked --offline --workspace --all-targets -j 2`
  passed at `600eb74`; log
  `out/offline-architecture/combined-all-targets-check.log`. Final cleanup
  removed the newly unused export/wrapper reported by this check. The new
  owners' dependency graph has no upward dependency.
- Final formatting passed. The optimized combined test build is running in
  exec session `9031` from the verifier worktree:
  `cargo test --locked --offline --release -j 1 -p tool-execution -p decomp-source -p offline-verification -p pipeline --no-run`.
- Persistent proof parent:
  `/home/jim/sm-decomp-offline-architecture-proofs/15d5a7e-20261015`.
  It holds `source-commit.txt`, `frozen-source/` with the private ROM,
  `prerequisites.sha256`, `logs/`, `tools/` and isolated `tmp/`.
  Build log: `logs/test-build.log`; dependency review: `logs/dependencies.txt`.
  Final combined behavioral suites and all three explicit Native proofs remain
  pending. Follow `native-verification-plan.md`; keep the complete recipe matrix
  and negative controls, and preserve the current budgets.

## Cleanup and remaining work

Foundation and recipe worktrees are integrated, cleaned and removed. Native
and caller workers are finished and idle; both targets and temporary editing
tools are cleaned. Their handoffs include measurements and retained logs. No
proof depends on either implementer worktree. Root owns their final retirement.

Remaining implementation worktrees are:

- `/home/jim/sm-decomp-offline-verifier`: root assembly and active verification.
- `/home/jim/sm-decomp-offline-native`: stopped, clean, integrated.
- `/home/jim/sm-decomp-offline-callers`: stopped, clean, integrated.

Next: finish optimized readers/helper, run the combined suites, then the complete
catalog/real recipe/full-family Native proofs. Retain exact tools and source,
run both cleanup passes, authenticate retained evidence, integrate on main,
retire only these worktrees/build outputs and authenticate again. Resolve
tickets 03–05 and the spec only after those checks and cleanup succeed.
