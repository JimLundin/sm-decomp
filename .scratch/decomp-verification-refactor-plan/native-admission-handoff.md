# Native operation admission recovery

Status: implemented and submitted; root owns integration and full Native gates
Owner: `/root/native_admission`, delegated by performance session
`01a13e7f-ab01-7413-a1c2-288b08a4b719`
Blocked by: None for this worker scope

Worktree: `/home/jim/sm-decomp-performance-native-admission`
Branch: `performance/native-admission-20261015`
Baseline: `8bf8acf610e1a94a1a3bccd5d718dae4a4244857`
Structural red source: `4f43fe79657bc18433bc09fdfca75ebf97cce126`
Passing production source: `7729fafbb4a4eef7606da832d994237c9a84cd73`
Proof parent: `/home/jim/sm-decomp-performance-native-admission-proofs/20261015`

Ownership is limited to `translation-execution/src/operation.rs`, required
retention reader changes and operation/retention tests, the test-support feature,
`offline-verification/src/native.rs`, and this handoff. The integration owner
owns main, the recovered extraction, installation, and the fresh full Native
proof. No receipt schemas, producer membership, evidence bytes, or deadlines
are being changed.

## Implementation

The retained PLM failure and source-derived redundant-read diagnosis are in the
main worktree's `native-finalization-recovery.md` and the extraction worktree's
`native-deadline-diagnostic.md`. This worker does not retry or mutate that proof.

The owned `AuthenticatedOperation` keeps output bytes, their exact result
reference, and transitive references together. Its private constructor performs
a fresh request admission. `authenticate_request_with_references` is the new
public interface; the existing result/reference interfaces remain available.
Every accessor borrows this owned admission and performs no I/O.

Retention now derives bytes and their hash in one bounded streaming read,
preserving before/after file identity, length, modification/change times and
safe-path checks. Result and intent references seal the actual file bytes,
including harmless formatting, rather than reserialized metadata. The aggregate
output limit is checked before reading the next output into memory.

Native collection reconstructs the exact expected Program and limits, admits
its operation once, and consumes those output/reference projections for all
existing parent logs, outputs, fixture artifacts and reservation checks. A
later whole-receipt admission opens and authenticates all required retained
evidence again. Collection plus whole-receipt authentication now perform two
complete result admissions per producer instead of the previous nine. This is
a source-derived work count, not a whole-family elapsed-time claim.

The metadata index retains its existing format and producer membership. No
deadline, producer slot, requested limit, evidence schema, runtime installation
or production policy changed. Payload memory is bounded to one operation, with
no trusted metadata cache surviving admission.

## Regression and owner gates

The regression uses a real complete confined producer. `/proc/self/io` and
`/proc/thread-self/io` are absent on this host, so the control uses narrowly
scoped thread-local numeric reader accounting under the explicit
`translation-execution/test-support` feature. Production builds have no counter
state or instrumentation. A later independent admission is also required to
read the complete retained closure afresh.

| Gate | Result | Retained log |
| --- | --- | --- |
| Exact structural red | 1 failed as intended: 39,501 bytes read for 10,482 retained bytes; 0.89 seconds | `logs/red.log` |
| Candidate structural green | 1 passed: exactly 10,514 bytes for first and independent admissions; 0.49 seconds | `logs/green.log` |
| Operation behavioral controls | 14 passed, 0 failed; 5.31 seconds | `logs/operation-controls.log` |
| Retention controls | 6 passed, 0 failed; 2.92 seconds | `logs/retention-controls.log` |
| Offline owner, initial harness | 39 passed, 2 failed, 1 ignored: cleanup helper environment missing | `logs/offline-owner-controls.log` |
| Offline owner, corrected environment | 41 passed, 0 failed, 1 ignored; 7.28 seconds | `logs/offline-owner-controls-corrected.log` |

The red/green closure sizes differ because their retained absolute paths differ.
Each assertion compares reader work with its own exact closure; these small
controls establish eliminated repeated reads, not controlled pipeline throughput.

The operation controls retain process crash/retry, stopped descendants, expired
deadlines, bounded rejected output, reservation accounting and changed input
identities. New controls cover owned byte/reference equivalence, fresh admission,
same-size output edits with restored mtime, missing payloads, symlinks, wrong
Program/context/attempt/input/limits, exact metadata bytes and producer retirement.

The two initial owner failures occurred at their cleanup calls after the worker
checks completed: the private checkout had no `target/debug/sm-pipeline` and
`SM_PIPELINE_CLEANUP_BIN` was unset. Both failed roots received both project
cleanup passes. The same frozen owner executable passed all 41 nonignored tests
after setting that variable to the retained reader below; no source changed.

The existing ignored
`builds::strict_tests::real_producers_reconcile_unused_reservations_and_replay`
gate was not run by this worker. Root retains ownership of the fresh full PLM
and required Native recipe/application gates. These controls publish no Native
acceptance receipt and do not change the historical failed PLM attempt.

## Execution and cleanup

No build or producer remains active for this worker. Metrics explicitly released
`/home/jim/sm-decomp-performance-metrics-target` before this worker used it. This
worker now releases that target to root; exact emitted test/helper binaries are
retained under `tools/red/` and `tools/green/` in the proof parent. Those copies
are independent files, not hardlinks to the mutable Cargo target.

Build flags were `CARGO_BUILD_JOBS=1 CARGO_PROFILE_DEV_DEBUG=0
CARGO_PROFILE_TEST_DEBUG=0`. Exact scoped Cargo commands:

```sh
cargo test -p translation-execution --features test-support --locked --offline -j1 --test operation --test retention --no-run
cargo test -p offline-verification --locked --offline -j1 --lib --no-run
```

The first command compiled red in 28.89 seconds and green in 8.55 seconds.
The second compiled the Native owner in 19.44 seconds, without the
`translation-execution/test-support` feature. Commands, elapsed/resource
observations and return codes are retained in `logs/build-*.time` and paired
logs. Tests invoked those exact emitted executables with `--nocapture
--test-threads=1`, avoiding an unrelated Cargo feature-unification rebuild.

All runs use a private proof TMPDIR and
`SM_PIPELINE_ROOT=/home/jim/sm-decomp/out/pipeline SM_PIPELINE_WORKERS=4`.
Ignored `deps/rustysnes` and `deps/snesrev-sm` symlinks point read-only to main's
existing dependencies for offline Cargo resolution and existing owner tests.
Remove the links with this worker checkout after integration; their targets are
not owned by this task.

Both project cleanup passes succeeded for the red proof, the green proof, the
initial owner failure roots, and the final complete proof parent. Each reported
zero bytes reclaimed; no shared cache was cleared. The exact retained cleanup
reader is:

`/home/jim/sm-decomp-offline-architecture-proofs/plm-fix-20261015/tools/sm-pipeline`

Its hash is in `logs/cleanup-reader.sha256`; it belongs to the retained extraction
proof and must remain available. Final cleanup commands:

```sh
sm-pipeline cleanup --root /home/jim/sm-decomp-performance-native-admission-proofs/20261015 --apply
sm-pipeline cleanup --root /home/jim/sm-decomp-performance-native-admission-proofs/20261015 --deduplicate --apply
```

Final outputs are `logs/final-cleanup.log` and `logs/final-deduplicate.log`.
The retained green result passed fresh cached-process result admission after
cleanup (`logs/final-retained-result-admission.log`, 1 test passed). All retained
red/green file hashes and exact emitted reader hashes passed afterward. The
complete fresh-admission byte-count control is recorded separately in the green
test, above. Retained producer roots:

- Red: `red/translation-operation-1485292-0/`
- Green: `green/translation-operation-1486267-0/`

Each includes `admission-request.json`, original producer input identities,
operation intent/result/output and source/binary. Both positive and diagnostic
failure logs remain in the proof parent. `logs/green-source.sha256` binds the
exact tested files to production source commit `7729faf`; `candidate-tested.patch`
also records the change against the preserved structural-red commit.

At handoff, the proof parent occupied 77,308,808 logical bytes and the filesystem
had 119,772,246,016 available bytes. The private Cargo target and this unintegrated
worktree are intentionally retained for root's next integration/build. Root must
retire the integrated worktree and disposable build outputs, preserving exact
required source/readers and reauthenticating retained evidence afterward.
