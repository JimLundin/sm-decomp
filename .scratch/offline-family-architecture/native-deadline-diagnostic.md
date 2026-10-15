# PLM Native finalization deadline

Status: observed failure; scoped repair and a new frozen-source gate required
Source: `4a736587af557de9e44b3126ff08f6d6d11beee8`
Recovered worktree: `8bf8acf610e1a94a1a3bccd5d718dae4a4244857`
Verification owner: `/root/recover_extraction_verification`
Integration owner session: `01a13e7f-ab01-7413-a1c2-288b08a4b719`

## Observed result

The exact ignored recipe test selected all twelve prescribed declarations,
with `plm-spawn` first, and exited 101 after 510.41 test seconds. Its first
failure was `trusted input whole-stage deadline exhausted`. No Native receipt
was published and none of the other eleven families ran. This remains an
incomplete 8,159-pair Native gate.

Retained parent:
`/home/jim/sm-decomp-offline-architecture-proofs/plm-fix-20261015`.
The exact command is in `logs/recipes.time`; stdout/stderr are in
`logs/recipes.log`. The complete environment, original compiler/Fixture limits
and twelve-family selection are in `logs/recovered-execution-environment.toml`.
`logs/recipes-failure.sha256` seals the diagnostic log, timing, intent, family
verdict and operation index. Source, private archive/ROM/capture dependencies
and exact readers remain in place.

The PLM family verdict is `offline-verified`: 1,969 Fixture pairs, three
routines, five rejected ABI inputs, and 1,969 native Rust spawn calls. Its
operation index contains all 3,941 expected producers. These records establish
completed comparisons and a diagnostic operation inventory; they do not
establish Native acceptance without the complete authenticated receipt.

## Localization

`crates/offline-verification/src/native.rs:1305` wraps final inventory, operation
collection/indexing, evidence extent calculation, whole Native authentication
and receipt publication in one `source_context::reconcile` call.
`crates/tool-execution/src/bounded.rs:153` gives reconciliation a fixed 60-second
deadline, independently of the prescribed 600-second producer gate. Nested
authentication inherits the earlier deadline; it does not receive another
60 seconds. The saved producer deadline was `1792065135147` and had not elapsed.

Retained precise mtimes (Europe/Stockholm, 2026-10-15):

| Record | Mtime |
| --- | --- |
| Native intent | 13:42:15.257565844 |
| Passing family verdict | 13:49:47.217587611 |
| Complete operation index | 13:50:20.669591295 |
| Final test error log | 13:50:38.725581614 |

`collect_operations` (`native.rs:1467`) authenticates each producer request,
loads its retained references, then invokes `authenticate_operation` again.
After indexing, `authenticate`/`authenticate_bounded` (`native.rs:1361`) traverse
the producers again and perform complete family admission. Evidence extents
also traverse the retained tree/index. These are concrete repeated passes in
the expiring block. The exact child call that detects expiry after index
publication is not instrumented; claiming a narrower measured cause would be
premature.

The exact call graph also gives the following structural count. Each of
`operation::authenticate_request`, `operation::reference`, and
`operation::references` invokes `load_result`; `reference` does so through
`authenticate`. These are source-derived invocation counts, not a measured
syscall trace or a whole-run speedup estimate.

| Native finalization work | `load_result` calls per producer | Calls for 3,941 producers |
| --- | --- | --- |
| Initial collection request/reference/dependencies | 3 | 11,823 |
| Collection's `authenticate_operation` checks | 3 | 11,823 |
| Subsequent whole-receipt authentication | 3 | 11,823 |
| Complete successful finalization | 9 | 35,469 |

The completed index proves the first two rows completed for all producers:
23,646 `load_result` calls during collection alone. The timeout interrupted
later work. These counts exclude original producer publication and later
external proof/retry admissions. Inside `load_result`, each material reference
is read completely by `authenticate_reference`/`seal_file` and then read again
by `read_bounded` (`operation.rs:672–675`). Native also authenticates dependency
references and reads parent copies separately.

An owned authenticated-operation bundle can return the validated result, exact
result reference and dependency references from one bounded admission. Native
must retain request/context, recipe argv/capability, reservation, result and
parent-artifact checks while consuming that one owned admission. A later
independent admission still reopens and authenticates current retained bytes.

Relevant existing tests in `crates/translation-execution/tests/operation.rs`:

- `sealed_process_output_lost_response_retry_and_checkout_retirement`
- `output_and_input_manifest_tampering_rejects_at_independent_authentication`
- `expired_outer_deadline_seals_no_launch_and_retry_cannot_extend_it`
- `reservation_covers_measured_retained_closure_before_any_output_is_produced`
- `repeated_fixtures_reuse_inputs_but_same_size_mtime_edits_and_links_invalidate`

They cover required behavior but do not currently assert admission/read counts.
A meaningful new bundle test should prove multiple accessors consume the
already-owned admission while a new admission rejects changed retained bytes.

## Narrow diagnostic proposal

Add a non-producing owner test around the retained PLM prepared record, intent,
passing family verdict, operation index/chunks and producer results. Use the
existing private reservation/admission helpers to reconstruct only an in-memory
candidate receipt for diagnosis, without publishing or accepting a new Native
receipt. Time inventory, collection/index checks, extent calculation and
authentication separately within the unchanged finite reconciliation budget.
Retain all original content and identity checks. The test must not prepare new
source, compile, run a Fixture, alter the saved deadline, or rewrite the retained
proof. Check receipt/result/source hashes and mtimes before and afterward.

This isolates repeated reads and authentication using the full real 1,969-pair
payload without rerunning producers. A repair should remove proven redundant
work while retaining fresh independent admission and exact producer ownership.
It must not enlarge deadlines, reduce the matrix or treat the diagnostic family
verdict as Native acceptance. The corrected implementation still requires the
original complete fresh-source recipe gate and other applicable gates.

## Cleanup and pending work

Both cleanup passes over `recipes/` succeeded. Ordinary cleanup reclaimed zero;
deduplication reclaimed 430,571,520 bytes across 5,907 snapshots. Both passes
over the empty `recipes-tmp/` succeeded and the directory was removed.
`logs/recipes-retained-before-cleanup.sha256` and
`logs/recipes-retained-after-cleanup-check.txt` confirm that all retained intent,
family verdict, index, chunks and producer results kept their hashes;
the paired `.mtimes` files compare equal. No producer writes remain.

Native reauthentication is unavailable because this attempt never published a
Native receipt. Preserve the partial proof for diagnosis. The independent
ordinary gate already authenticated after its cleanup. Current application
unit/integration gates continue under the original frozen source and retained
helpers; no recipe-matrix retry has been started. The integration owner owns
any source repair, merge, final admission and worktree retirement.
