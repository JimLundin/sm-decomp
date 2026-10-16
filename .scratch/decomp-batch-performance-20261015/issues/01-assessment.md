# Diagnose whole-batch verification cost

Status: resolved
Blocked by: None
Triage: ready-for-agent
Type: research

Inspect existing measurements and current implementation. Measure bounded
read-only planning/startup, record source identity and selection reasons, and
rank concrete performance work while preserving current verification contracts.

## Comments

The user clarified that whole decomp batch / integration verification is slow.
An independent active architecture session owns source implementation. This
assessment owns only `.scratch/decomp-batch-performance-20261015/` and temporary
probe outputs below it. No production, source or shared cache mutation is needed.

Ranked hypotheses after the initial 32.03-second planning observation:

1. Conservative dependency discovery selects most families unnecessarily.
   Prediction: documentation-only changes still select families with an explicit
   uncertainty reason; source inspection identifies the unsupported token.
2. Debug controller startup and repeated catalog/input processing impose fixed
   overhead. Prediction: an invalid command has substantial startup latency;
   removing debug sections from a disposable copy reduces the bytes hashed at
   startup while preserving command behavior. Catalog and planning timings
   distinguish startup from discovery.
3. Build work remains a large part of complete Native verification after Fixture
   parallelism. Inspect retained phase timings and current reuse boundaries.
4. Ordinary batch authentication/cleanup and confined family scheduling repeat
   work or serialize stages. Inspect current call sites and retained metrics;
   do not claim an unmeasured end-to-end speedup.

## Answer

See [the findings](../findings.md). Prioritize syntax-aware affected-family
discovery, one authenticated catalog/input snapshot per batch, exact build/input
reuse, current optimized controller/sandbox adoption, and phase/reuse metrics.
Current planning selected 160/177 families; documentation-comment tokens caused
151 PLM and seven projectile-flight first refusals. Preserve genuinely unknown
dependencies as affected. No reduced post-fix count or end-to-end speedup has
been measured.

## Completion evidence

The original plan and both catalog probes completed; catalog bytes matched.
The stripped-controller planning control hit its 45-second cap and the Rust
driver exited nonzero as designed. Its unsuccessful output and the host-load
limitation are explicitly retained. `probe-summary.txt`, raw timing/output files,
`selection-summary.json` and `inputs.sha256` identify the observations.

Writers stopped before these commands, both successful:

```sh
.scratch/decomp-batch-performance-20261015/controller-original cleanup --root .scratch/decomp-batch-performance-20261015 --apply
.scratch/decomp-batch-performance-20261015/controller-original cleanup --root .scratch/decomp-batch-performance-20261015 --deduplicate --apply
```

Both reported zero eligible bytes; this assessment produced no family/Live
verdict. After source/binary hashes passed, the two disposable controller copies
and Rust probe executable were removed. `retirement.txt` records 444,798,584
logical bytes and 444,616,704 allocated bytes represented by these files. The
shared-filesystem free-space observation after removal is 146,634,555,392 bytes;
its delta is not an isolated physical reclamation measurement. About 208 KiB of
reports/source remained before this completion update. No worktree or shared
cache was created or removed; no evidence reader/store dependency is required
by these diagnostic planning reports. `git diff --check` passed.
