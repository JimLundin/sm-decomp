# Benchmark host budgets

Frozen before ticket 03 sampling, 2026-10-12.

- Host: Linux, eight online logical CPUs, 12,251,516 KiB physical memory.
- Run aggregate concurrent sampled RSS ceiling: 8 GiB (8,589,934,592 bytes).
- Run staging plus private-store allocated sampled peak ceiling: 64 GiB
  (68,719,476,736 bytes).
- Minimum free disk before and throughout each measured run: 50 GiB
  (53,687,091,200 bytes).
- Capacities: two and four, separate shared pipeline roots. A packed subject's
  producer and collector share one orchestration lease and both count toward RSS.

RSS is a sampled run process-tree peak, not the sum of process lifetime maxima.
Record sampling period and missed-short-process limitation. Allocated staging
and private-store bytes use filesystem block accounting, not logical inventories.
Shared stores and build caches are measured separately and retain dependencies.
Unavailable resource values cannot pass a resource gate by being treated as zero.

The initial read-only probe found 230,566,404,096 free bytes and approximately
8 GiB available memory. Unrelated Node/TypeScript work was active. These are
host observations, not a quiet-host benchmark result. Correctness verification
may proceed under load; performance samples require recording host load and
keeping unrelated compilation or replay sweeps out of the measured intervals.

Each comparison freezes sources, compiler and reader identities, input hashes,
current admission pointers, storage path/filesystem, telemetry configuration,
worker capacity and equivalent cache/store warmth. Warm-build samples include
publication, final authentication and scoped cleanup in one outer wall boundary.
Cold-build and empty-store measurements remain separate. Binary selection and
promotion use only correctly admitted comparable samples, with the plan's
15% wall reduction and maximum 5% resource growth gates unchanged.
