# Code quality

Use these standards for implementation, assignment and integration review.
They supplement the scope's existing acceptance contract; historical sealed
policies retain their exact gates and require their documented amendment path.

## Before implementation

Record the baseline, writable ownership, complete behavior, dependencies and
acceptance gates in the local ticket or existing typed assignment. Identify the
existing module that owns each invariant and reuse its implementation. A new
seam should hide a distinct responsibility behind a small interface. Explain a
new framework or parallel implementation before adding it.

Keep independent owners' files disjoint. Shared registration and merged-source
verification belong to the integration owner. Inspect current claims and real
owners through the applicable session runbook before claiming shared work.

## Coding standards

- Use the glossary's terms and the surrounding module's naming and layout.
  Keep gameplay arithmetic, byte order and state transitions explicit; wrapping
  and signed conversions must match the recovered behavior.
- Put validation and state transitions in the module that owns the invariant.
  Expose typed inputs, outputs and errors instead of leaking internal flags or
  requiring callers to repeat its protocol. Keep one maintained implementation
  for production and focused tests.
- Validate untrusted sizes, indices, paths and selectors before mutation or
  unsafe execution. Document each unsafe boundary's allocation, lifetime,
  aliasing and reentry requirements at the operation that relies on them.
- Preserve ordered effects and ownership boundaries. Unsupported input follows
  the declared unchanged-refusal contract; failure after ownership follows the
  terminal-error contract. Silent fallback after partial effects is forbidden.
- Return actionable errors for runtime and input failures. Reserve panics for
  proved internal invariants and test assertions. Keep lint exceptions narrow
  and explain the actual reason; fix new warnings rather than suppressing them.
- Keep generated outputs and retained evidence separate from authored source.
  Project tools, orchestration and tests use Rust; recovered reference C and
  native interoperability retain their existing scope and provenance.

## Review and verification

Run formatting, Clippy with warnings denied, and relevant executed tests for
the changed workspace. Root Cargo commands do not cover the independent
`restart/` workspace or standalone tool workspaces: select their manifests
explicitly. The root CI workflow contains root and restart jobs. Standalone
tools have manifest-specific CI gates and `just standalone-check MANIFEST` for
local checks; use the CI matrix to find the covered manifests. A missing tool or host-denied
execution is an incomplete gate, even when compilation succeeds.

Use focused checks during edits. Before integration, run the affected workspace
checks and the scope's complete behavioral gates on frozen merged source. For
restart, `just restart-check` runs ordinary workspace checks, including all
targets and fixture features. Its full fixture tests require their authenticated
private inputs; absent prerequisites remain failing gates.
Native comparison, live replay and original-ROM acceptance remain separate.

Review every changed file against these standards and every acceptance
requirement against actual evidence. For material gameplay or verification
changes, the integration owner or another reviewer must inspect the submitted
source and test oracle independently of the implementer's self-review. In the
ticket's completion evidence, record:

- Reviewer identity, reviewed baseline and exact candidate commit or source
  hashes, findings, fixes and remaining limitations.
- Commands, features, executed test counts, verdict/log pointers and which
  acceptance requirements they cover. Preserve failed diagnostics too.
- Required cleanup and post-cleanup authentication from `run-cleanup.md`.

When evidence references a source or output inventory, authenticate every
referenced file as well as the inventory itself. Bind the actual compiled
executable or library separately from its launcher and original/transformed
source. Preserve frozen snapshots; repair a damaged snapshot from its original
bytes rather than updating an old seal to fit new source.

Tests should distinguish plausible wrong behavior: sign and carry boundaries,
state transitions, ordered effects, refusal, terminal failure and preserved
foreign state where relevant. Use an independent oracle for parity. Replaying
the candidate against itself can validate its protocol but cannot establish
recovered-C agreement. Mocks establish only their declared seam. Formatting,
syntax checks, compilation, test counts and progress observations alone cannot
establish gameplay acceptance.

Finish when findings affecting correctness or maintainability are fixed, every
required gate has authentic evidence for the final source, and cleanup is
complete. Otherwise retain the ticket's unfinished status and exact next action.

## Improve the loop

Measure the observed bottleneck before changing the process: record the same
command, source/input identity, build state, producer budget, wall time and
retained bytes before and after. Pair the improvement with unchanged behavioral
coverage and evidence authentication.

Prefer focused Cargo package checks, shared production/test seams, incremental
builds, ready-together publication and independent disjoint work. Reuse a verdict
only through its existing source/input authentication contract. Repeat checks
when their inputs change or a failure requires it. Investigate conservative
dependency selection rather than silently excluding uncertain dependencies.
Optimize measured build, I/O and coordination costs while keeping complete
scopes, independent oracles, failure controls and acceptance gates intact.
