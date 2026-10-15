# Bulk input context implementation

Status: claimed; implementation and owned controls complete, integration acceptance and retirement remain root-owned.
Owner: `/root/bulk_context_resume`, parent session `01a13e7f-ab01-7413-a1c2-288b08a4b719`.
Worktree: `/home/jim/sm-decomp-performance-bulk`.
Branch: `performance/bulk-context-20261015`.
Original baseline: `c09e47b66a35e6d61dab4065cb7e30d16ff28119`.

## Tested checkpoint

Frozen source **`b1f869102ac69897be674acfef682e3d867c6be6`** combines ticket 02
with ticket 03 through `2b2712b`, ticket 04 through `cb00f32` plus its
`330e8c0` controls, and root's ordinary ticket 06 reader through `8d9d00e`.
All controls below passed on this source. The documentation-only handoff
checkpoint may be newer; the compiled source and retained tools stay pinned
to `b1f8691`.

Root's later Native operation-admission optimization, Ceres CompleteC repair,
and tickets 07/08 are separate. Preserve root's optimized Native admission
when integrating; this worktree must not replace it with the older internals.
No production policy or gameplay source was changed by this scope.

## Implemented behavior

- One invocation retains each declaration's encoded contents once, with one
  lazy decode per content identity. Every explicit path is authenticated when
  captured. `CatalogSnapshot::family()` consistently reads captured facts;
  current-source refusal remains at explicit validation, execution and
  independent admission boundaries. Two aliases share the decoded declaration.
- Ordinary batch preparation retains immutable source and namespace inventories,
  selected family closures and shared Rust dependency facts. Wrappers reuse the
  captured facts and share the invocation's `BuildContext`. Captured source,
  temporary buffers, metadata, materialization and freshness work have distinct
  bounded counters; these reservations are not a process-RSS measurement.
- Literal child-module placements can prove supported `use super::*` builtin
  resolution through an independently complete, explicitly binding parent.
  Raw facts stay context-independent. Every placement is validated before
  traversal is shared; unsupported or ambiguous dependencies still refuse.
- Complete Native compiler/source/asset inputs are retained below
  `RUN/family/captured-inputs/checkout`. The separate
  `RUN/family/captured-inputs.toml` record binds the full membership, original
  locations, hashes, flags and extents. Complete captured bytes are reserved
  before intent publication; partial publication has deterministic recovery.
- The Native mapping hash is encoded in its unused read-only compiler input
  destination, `/inputs/capture-record-SHA256.toml`. Binding only its physical
  path was insufficient because reconstructed Program equality is path-based.
  A valid same-length changed mapping with a resealed intent now fails at the
  compiler capability record, before later family artifact checks.
- Completed Native receipt replay reopens the saved mapping independently,
  performs no rematerialization and launches no producers. Exact producer
  slots, complete observations, deadlines, quotas, recovery and current-source
  admission remain. Old intents without captured inputs retain their original
  direct-input contract.
- Real C preprocessor closures are checked against captured inputs, including
  generated units, macro includes and continued includes. Ordinary Rust
  `file!()` and C `__FILE__` retain original logical locations; Native keeps its
  original confined virtual path semantics. Ticket 04's final compiler source
  location handling and conservative volatile-preprocessor bypass are included.

Decisive local checkpoints after the earlier `b7f2c0d` control:
`36906b6` (shared source facts), `25740d0` (Native mapping identity),
`56c4c18` (raw declaration capture), `f6ba8ed` plus `d9cd0e7` (captured
parent placements and no redundant immediate plan-only catalog fence).
Ticket 03's final asset pass is integrated as `40e38bf`; its filesystem-alias
red control and guard are `1f48848` and `b1f8691`.

## Verification evidence

Proof root:
`/home/jim/sm-decomp-performance-bulk-proof/b1f8691-20261015`.

| Gate | Result |
| --- | --- |
| Offline owner library | 48 passed, 1 explicit real-producer gate ignored; 10.71 s |
| Public catalog controls | 5 passed; 0.03 s |
| Public actual-C/Native controls | 4 passed; 47.33 s |
| Ordinary independent admission after aggregate cleanup | 1 passed; 1.42 s |
| Native independent admission after aggregate cleanup | 1 passed; 1.06 s |
| Matching pipeline CLI build | locked/offline debug build passed; 2m 04s |

The ignored public test is the independent-reader entry point, explicitly run
in subprocesses by both successful producer controls and again after aggregate
cleanup. It is not an omitted gate. Root owns the separate full-real Native
gate and its original unchanged-deadline failure; it was not rerun here.

`logs/{owner,catalog,public,ordinary-reader-after-cleanup,native-reader-after-cleanup,cli}.log`
retain the results. The actual controls cover three Fixtures (root,
continuation, captured input), complete 128 KiB observations and CaseRecords,
returns, logical compiler paths, unrelated-source exclusion, changed-source
refusal before producers, both C include escapes, and Native three-pair/eight-
producer membership. They also cover mapping/source/extra-member tampering,
the resealed mapping capability refusal, dropping and reopening preparation,
and identical completed replay with unavailable tools and unchanged evidence
mtimes.

Final ordinary counters are retained at
`runs/ordinary-223270-19/capture-statistics.toml`:

- One catalog parse, two path reads, one manifest decode.
- Four Rust fact scans, 1,639 scanned bytes and 19 fact reuses.
- 80 retained source reads, 1,447,855 source bytes; batch execution adds no
  source capture or fact scans.
- 864 independent source freshness reads and 13,829,253 freshness bytes.
- Zero transient bytes after execution; peak reserved bytes 1,284,255.
- `timing.toml` records 7.334763335 seconds for this synthetic bulk phase.
  This is not a whole-batch speedup measurement.

Exact retained executables are in `tools/`; `logs/tools.sha256` records:

| Tool | SHA-256 |
| --- | --- |
| bulk-inputs | `f7482a4bf8696de84a40fe85fe5ab3ddaa4e89aeaca8bd649fd1a24a920441ed` |
| bulk-catalog | `a89859338b968ab677712d6a0392375511d8ddaf1d04b1136127a34a395c79bb` |
| owner-tests | `ba8d5f4a7cefa7e331984fe788bfda45a8474dec0f16484a4aac5eceafdec9b2` |
| sm-translation-sandbox | `441daca1db663db0508d5977dfd4aaf005428d341e9ab0860d1050a24978d585` |
| sm-pipeline | `ed88f55951b3ef8d853b4a283cef0205b4f675e3edb4f3d698dcac5958df0acc` |

Commands used shared pipeline root `/home/jim/sm-decomp/out/pipeline`,
workers 4, jobs 1, locked/offline Cargo, disabled dev/test debug information,
the exclusive target `/home/jim/sm-decomp-performance-metrics-continuation-target`,
and a proof-owned TMPDIR. Native compiler/fixture/family limits remain
180/5/600 seconds, quota 1 GiB, with the same pinned nightly Rust compiler and
GCC 9. Tests use one test thread.

## Cleanup and retention

All owned build/test writers exited. Per-project cleanup and deduplication
succeeded even for negative controls. The ordinary run reclaimed 4,063,232
bytes, Native 655,360 bytes and negative controls zero bytes. Aggregate
`runs/` cleanup reclaimed zero further intermediates and its deduplication
pass reclaimed another 262,144 bytes across 46 snapshots: **4,980,736 bytes**
total. Both independent readers passed afterward with compiler PATH unavailable.

Both cleanup passes also succeeded for the final `tmp/`, including after the
CLI build, and the failed-build `f6ba8ed-20261015/tmp`; all reported zero bytes.
Exact commands are the retained `sm-pipeline cleanup --root PATH --apply`
and `--deduplicate --apply`. Individual `cleanup*.stdout/stderr` and
`logs/cleanup-*.stdout/stderr` retain outputs. The initial cleanup reader was
root's retained `7b27ea1` CLI; the final after-build passes used the matched
`b1f8691` CLI. About 100.2 GB was free before handoff (81% used).

Each public project retains its own exact synthetic pinned source and compiled
implementation source stamps under `runs/NAME/source`. Keep those source
trees, receipts, mappings, immutable producer artifacts, replay manifests and
historical readers. Source retention for these controls does not depend on
this worktree remaining writable.

The worktree, ignored `deps/rustysnes` and `deps/snesrev-sm` links, and the
exclusive target remain for root's integration and retirement. Do not remove
another owner's evidence dependencies. Final ticket resolution and worktree
removal belong to root under the cleanup runbook.

Retirement audit: no retained `.toml`, `.json`, `.jsonl`, `.argv`, `.d` or
`.sha256` artifact anywhere under `sm-decomp-performance-bulk-proof` names
`/home/jim/sm-decomp-performance-bulk` or the exclusive Cargo target as a path.
The final reader has no target RPATH or RUNPATH; its dynamic dependencies are
standard system libraries. No source stub or target artifact needs relocation
for these controls. After integration, root can remove the worktree and target
while retaining the proof directories at their current absolute locations.

In particular, preserve `b1f8691-20261015/runs/ordinary-223270-19` and
`runs/native-223270-14` in full, including each `source/` tree, the Native
`operations/` directory and installed-runtime records. Preserve the final
`tools/` directory, also referenced by ticket 03's corpus proof. Retain the
historical `b7f2c0d-20261015` and `83c0f4f-20261015` proof roots and their
own readers; `83c0f4f` also has two cleaned diagnostic directories in `tmp/`.
Compiler paths and shared pipeline paths in historical attribution and
metrics are outside this retirement scope.

After worktree/target removal, rerun the retained final `tools/bulk-inputs`
with `--exact retained_capture_admission_without_compilers --ignored
--nocapture`, `PATH=/unavailable-bulk-compilers`, shared pipeline settings,
and `SM_BULK_REPLAY` set in turn to the final ordinary and Native `replay.toml`
paths. Historical equivalents are `b7f2c0d` ordinary-178269-0/replay.toml and
native-178981-0/replay.toml, plus `83c0f4f` ordinary-201302-19/replay.toml and
native-201302-14/mapping-diagnostic-replay.toml; use each proof's own reader.

## Integration handoff and remaining scope

`/root/dependency_facts_resume` received the exact frozen source and matching
CLI/helper hashes and owns final 177-family corpus controls in a separate
detached checkout. Its documentation-only plan has passed under unchanged
bounds: 174 excluded / 3 selected, 7.77 s wall time, 5.192 s planner time,
138,544 KiB peak RSS, 257,399,575 charged bytes, 45 parses, 437 closure-cache
hits, and 1,548 asset declarations / 1,360 reads. The real audio behavior,
rename and deletion controls also passed: each selected exactly the same 165
affected families and excluded 12, adding the expected 162 audio dependents
while retaining all 177 catalog members. That worker restored its source
and owns its remaining cleanup and final ticket 03 report.

Root owns final assembly with Ceres and later tickets, full verification,
acceptance and retirement. No final end-to-end performance acceptance is
claimed from these small controls. The implementation and owned gates are
ready for integration.

## Historical evidence

- `b7f2c0d-20261015`: preliminary all-green producer controls, before shared
  Rust facts, stronger mapping assertion and final compiler cache correction.
- `83c0f4f-20261015`: the stronger mapping test correctly exposed the
  path-only identity gap; the Ridley parent-import case exposed missing parent
  context. The Native mapping diagnosis restored the original mapping/intent.
  Both cleanup passes after diagnosis succeeded (0 + 262,144 extra bytes);
  independent restored Native and ordinary admission then passed in 1.15 s
  and 2.14 s. Historical readers remain under that proof's `tools/`.
- `f6ba8ed-20261015/logs/no-run.log`: retained type-inference compile failure.
  `d9cd0e7` fixes the PathBuf annotation. The next no-run at `40e38bf`
  passed in 23.85 s and its helper built in 4.37 s; the final `b1f8691`
  builds confirmed these unchanged dependencies were current.
