# Combined foundation verification and retirement

Status: foundation integrated; complete final acceptance pending
Owner: session `01a13e7f-ab01-7413-a1c2-288b08a4b719`, `/root`

Final assembly `45b89f844df928478b890c4b4814208dfe44bfb8` combines bulk
`b1f869102ac69897be674acfef682e3d867c6be6` and Ceres
`c4dc6895fc23e4e7373d2f5531c8867056dab994` with the existing Native operation
admission fix. It is retained in `/home/jim/sm-decomp-performance-final`.
Main has only documentation changes at this checkpoint. The earlier `7b27ea1`
assembly and proof remain frozen; their failed/incomplete outcomes are not changed.

## Combined checks

Proof parent: `/home/jim/sm-decomp-performance-final-proof/45b89f8`.

| Check | Result |
| --- | --- |
| Offline owner unit controls | 48 passed, two explicit ignored; 18.34 seconds |
| `native_capture_replays_without_tools_and_binds_mapping_and_source` | Passed; three pairs/eight producers; 33.51 seconds |
| Independent retained Native capture admission after cleanup | Passed; 0.79 seconds |
| Full-real positive domain, unchanged retry and invocation tamper/restoration | Passed; 16 pairs/two routines/34 producers; positive block 25.973 seconds |
| Full-real later negative controls | Incomplete: `include_str!` rejected by capture before Native intent publication |
| Independent retained positive Native admission after cleanup | Passed; 1.73 seconds |

The full-real command exited 101 after 41.92 seconds. The positive retains
36,593,543 evidence bytes and charges 63,495,371 reserved bytes. Its manifest
accurately lists only the completed positive. The empty compile-denial family
directory has no missing published receipt to reconstruct. The syntax scanner's
unresolved-input refusal is correct; the old test assumed the macro reached
rustc. The test-only repair keeps that refusal and separately exercises actual
confined compiler file-access denial and failed Native receipt replay.

Both utility cleanup passes succeeded for `runs/`, `tmp/`, `full-real/` and
`full-real-tmp/`. The Native capture control deduplicated 655,360 bytes;
the full-real positive deduplicated 4,194,304 bytes across 48 snapshots.
Additional ordinary/temporary cleanup reclaimed zero. Metadata seals passed
after cleanup and after independent admission. Logs are `logs/native-*`,
`logs/full-real-*`, `logs/{runs,tmp}-cleanup.log` and their deduplication peers.

| Retained reader | SHA-256 |
| --- | --- |
| `tools/owner-tests` | `ee9783276a3478a9e37afb642d1db464d5f00477538418a7398c9e28184c3801` |
| `tools/bulk-inputs` | `8a45c5b8f085a9be4629e9a84900bdeab387fb4e593717b40b177a98e6c00771` |
| `tools/sm-translation-sandbox` | `441daca1db663db0508d5977dfd4aaf005428d341e9ab0860d1050a24978d585` |
| `tools/sm-pipeline-tests` | `b27d764737249f82f8507e5f7d767c3780e47e45242ecfc077583731fcd1a275` |

The retained sandbox is an independent copy of the matching unchanged helper.
The final pipeline reader build took 1m55s. These builds and unmatched historical
timings are not full-batch performance comparisons.

## Integrated implementer retirement

Every retired implementation worktree was audited for external proof references.
Required source snapshots, readers, protected inputs, ROM/C/archive dependencies,
verdicts and failure diagnostics remain at the exact authenticated paths. Targets
transferred to new owners were not deleted. Utility cleanup was scoped to finished
work; no shared evidence/cache or production ownership marker was removed.

| Retired worktree | Retained proof and post-retirement result |
| --- | --- |
| `/home/jim/sm-decomp-performance-bulk` | `sm-decomp-performance-bulk-proof/b1f8691-20261015`; ordinary reader passed in 2.78s, Native reader in 1.33s; tool seals passed |
| `/home/jim/sm-decomp-performance-dependencies` | `sm-decomp-performance-dependencies-proof`; both matching old/new one-pair actual-C readers independently returned `offline-verified`; evidence seals passed |
| `/home/jim/sm-decomp-performance-builds` | `sm-decomp-performance-builds-proof`; 3,454 files and 66 cache artifacts authenticated; all 63 retained programs passed original exit/stdout predicates |
| `/home/jim/sm-decomp-performance-admission` | `sm-decomp-performance-admission-proof`; proof seals and both zero-byte cleanup passes passed; complete Ceres admission passed again |
| `/home/jim/sm-decomp-performance-ceres` | `sm-decomp-performance-ceres-proof/c4dc689`; complete 1,296-pair admission passed again in 7.76s; metadata seals unchanged |

All proof paths in that table are under `/home/jim/`. Bulk's ordinary proof
cleanup reclaimed 4,980,736 bytes before retirement. Its disposable target
`/home/jim/sm-decomp-performance-metrics-continuation-target` held 3,553,345,536
allocated bytes; `cargo clean --locked --offline --target-dir` removed 7,658 files
(reported 3.4 GiB) after it was moved into the proof's retirement directory.
The retirement directory's two utility passes succeeded. Cargo cleanup was the
successful supported fallback after automatic review refused the earlier raw
recursive-deletion command. No approval or cleanup blocker remains for that target.

Measured pre-retirement worktree allocations: bulk 218,902,528 bytes,
dependency 218,701,824 bytes and shared-build 218,681,344 bytes. The dependency
corpus retired separately, removing 177,532,928 allocated bytes while retaining
223 exact relative report inputs without Git. All four corpus reports reopened
successfully; the wrong audio variant was explicitly refused. Its utility passes
reclaimed zero disposable and 262,144 deduplicated bytes. Final corpus evidence
manifest: `99f9c344cf82d4e8f4bb2497622c0a1e509c56fc31bafbaa59891bc317153068`.

Ceres's complete ordinary proof deduplicated 233,570,304 bytes. Its failed first
attempt and five linker diagnoses also completed both cleanup passes, reclaiming
zero. The complete source remains in the proof's `c4dc689/source` directory.

The shared-build v1 replay parser's unsupported `stdout_nonempty` failure remains
retained. The corrected Rust reader enforces 49 exact stdout and 14 nonempty
stdout predicates over all 63 unchanged programs. Reader hash:
`662c8eb37b39f2eb4343a4d22be97fb8e56a090bb5aa3ed5dcd7cb7873f949b0`.
The post-retirement receipt is `logs/retained-programs-after-retirement-v2.json`.
Both final utility passes over `retained/replay-reader-v2-build` succeeded with
zero bytes; source, reader and expectation hashes and the parse-only 63-program
check passed again. See `retained/replay-v2-handoff.md` and
`logs/replay-v2-after-cleanup*` for exact commands and seals.

Last observed available filesystem space was 97,540,034,560 bytes. Final rollout
must measure space again after the remaining owned targets are retired.

## Performance limits

Dependency facts now exclude 174/177 families for the documented metadata edit,
retaining three genuine uncertainties; narrow transitive audio changes select
the expected 165 and exclude 12. This is selection improvement, not a reduction
in cases for a fresh required verification. Shared compiler preparation reduces
two identity queries to one, without a reliable wall-time gain in its samples.

The matched 12-case profile comparison, including four continuations, used
source `5b6e59d`, fresh Rust/actual-C fixtures, private artifact caches and shared
four-worker capacity. Warm dev samples were 12.639/11.770/11.426 seconds;
verification samples were 1.117/1.106/1.103 seconds. Median ratio is about 10.65.
Outcomes were byte-identical; cleanup and independent/cross-profile admission
passed. This bounded workload result is not a whole-batch multiplier. Final
combined-source and complete-recipe measurements remain required.
