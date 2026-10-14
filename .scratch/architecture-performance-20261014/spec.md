# Decomp pipeline architecture performance

Status: resolved
Blocked by: None

The user selected all five candidates from the 2026-10-14 architecture report
and explicitly requested one isolated worktree per candidate. Implement, test,
measure bounded representative behavior and retain reviewable commits for each.
Report: /tmp/architecture-review-20261014T162242Z.html.
Baseline: 1ce6494d7e0fdc9b8623faf3f9d9f507f4a8abb0.
Coordinator session: 01a13b35-079e-7151-8e5a-f759fee6bcf3.

## Constraints

- All project tools and measurements are Rust. Preserve exact actual-C Fixture
  pairs, Fixture chains, Live replay ownership and ordered-effect contracts.
- No original production claim, installed controller, policy, gameplay stage or
  main Git/index mutation belongs to this effort. Ticket 10 has a live owner.
- Existing confinement owner in /home/jim/sm-decomp-ticket10-work/confinement
  remains authoritative for candidate 01 implementation. Our 01 worktree is
  reserved for review/adoption of its completed commit, not competing edits.
- Worktrees live under /home/jim/sm-decomp-architecture-20261014 and share only
  read-only dependency inputs via the main deps directory. Never write there.
- Use SM_PIPELINE_ROOT=/home/jim/sm-decomp/out/pipeline and
  SM_PIPELINE_WORKERS=4. Coordinate builds and tests; Cargo jobs=1 per active
  build. No full-workspace release build or long profiling matrix.
- Freeze input, source, compiler/profile and cache warmth for comparisons.
  Record competing host activity; do not claim controlled speedups from it.
- Each worker owns its files and isolated Git/index. Preserve others' changes.
  Registration overlap in live.rs between 03 and 05 is coordinated by root.
- Apply docs/agents/run-cleanup.md to each run; preserve evidence and source
  dependencies, record bytes reclaimed and free space. Worktrees remain while
  awaiting integration/review; remove only after integrated and evidence retained.

## Completion

Each ticket has a scoped commit, tests through the module interface, bounded
performance evidence or an explicit measurement limit, and cleanup evidence.
The five branches stay separate as requested; a combined integration check may
use a separate checkout without mutating the active main checkout. No automatic
production activation is part of this optimization task.

All five tickets are resolved. Source composes without conflicts on
`architecture/perf-composition-20261014` at
`bc5c1cf9ba2965ef6513273deb4880a93d770cc9`; 32 combined controls passed.
Both retained actual-C verdicts authenticate after cleanup and retirement of
the temporary composition checkout and Cargo target. The five requested
worktrees and their commits remain available. Exact source mappings, staged
controller/helper binaries, evidence dependencies, cleanup measurements and
validation limits are in [composition.md](composition.md) and [map.md](map.md).
No full-pipeline optimum, authenticated complete native Live replay or
production activation is claimed.
