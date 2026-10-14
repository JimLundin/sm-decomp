# Supervisor recovery audit

Status: complete scoped audit; root combined proof remains with `/root`
Owner: `/root/supervisor_audit`
Worktree: `/home/jim/sm-decomp-translation-execution`
Branch: `architecture/translation-scope-execution-20261013`
Scope: `crates/translation-execution/src/{journal,executor}.rs`, minimal `lib.rs`
exports and `tests/attempts.rs`. Source is uncommitted for the root's shared
checkpoint. Main production state and gameplay source were outside this scope.

## Implemented behavior

Recovery streams the hash-authenticated journal and validates typed legal
lifecycle transitions before executing another adapter operation. Immutable
source, policy, intent, gate ordering, disposition and retry/repair lineage stay
bound to their original attempts. An editable failed terminal tail cannot reopen
an unexecuted gate. An apparently successful forged gate must authenticate against
the original adapter receipt before another producer can run.

`Supervisor::open`, operation loads and `authenticate_finalized_history` validate
every distinct recorded gate and finalization, including earlier failed attempts
before a later success. `authenticate_completion_history` checks the complete
accepted typed history. Publication projection repair stays on explicit resume;
inspection reads existing state. The adapter's gate authenticator follows original
frozen source and receipt identity, including exact retained source equivalents.

Cancellation before finalization selects the cancelled disposition and reconciles
the original writer. Cancellation at `Finalize` or `Publish` drives the original
operation and leaves its disposition unchanged. Terminal cancellation returns the
original result. Journal replay rejects a new cancellation after finalization is
selected. Cleanup retries preserve the failed gate or cancellation diagnosis.

## Exact journal accounting seam

`ExecutionAdapter::publish` receives `&PublicationJournal<'_>`.
`ExecutionAdapter::finalize` receives `&FinalizationJournal<'_>` as its last
argument. Both expose `current_bytes()`, `additional_bytes(&result)` and
`retained_bytes(&result)`, returning `Result<u64>` without writing. Real append
and projection share the exact event serializer and its fixed metadata bounds.

For stopped attempts, finalization projects the prescribed next `Terminal` event;
successful finalization projects `Publish`. Root owns the adapter's final-record
and publication fixed points. A stopped `FinalRecord.journal_delta_bytes` charges
the exact terminal event outside the frozen retained prefix. Successful
publication charges its full projected journal extent minus the frozen prefix.

`FinalizationJournal.max_additional_bytes()` reserves an exact serialized upper
bound before retention side effects: prescribed fixed-width identity/flags and
maximum decimal widths for both byte counters. Root subtracts this bound with the
final record reserve from the retention limit. This avoids sealing an immutable
closure that fits alone but leaves no extent for its eventual terminal result.

`authenticate_finalization_tail(root, policy, context, request, finalization)`
returns the exact stopped terminal event bytes. It takes no second journal lease
and invokes no adapter. The caller holds a lifecycle lease or reads retained
immutable state. It authenticates typed/hash-checked history, the exact original
finalization predecessor/request and byte-identical prescribed successor. If the
final record was sealed just before a lost response, it projects only from the
still-current authenticated `Finalize` intent. Later retries retain the original
historical terminal size; they never substitute the current whole-journal extent.

## Audit observations handed to root

| Observation | Current responsibility and evidence limit |
| --- | --- |
| CLI inspection originally initialized/reserved state. | Root added existing-adapter and retained-inspection paths; integrated proof belongs to root. |
| Partial Git apply/merge could leave dirty source and block finalization. | Root and bounded Git owner added partial source retention and retirement leases; root owns failure/repair lifecycle proof. |
| Independent Git command deadlines could exceed the outer gate/preparation deadline. | Bounded Git owner propagates outer deadlines; its real Git tests are separate evidence. |
| Metadata allocation refusal could happen before writer teardown or cleanup. | Root owns reservation ordering and bounded aggregate accounting. |
| Cache authenticity errors could be treated as cache misses and run producers again. | Root owns fail-closed native/adapter cache authentication. |
| Retained source relocation could change a receipt wrapper before checking its original seal. | Root owns original-record and exact retained-equivalent authentication. |
| Finalization omitted journal/prior-attempt evidence and stopped terminal suffix. | Root added prefix/history closure; this audit supplies typed history and exact finalization/publication journal seams. |
| Late cancellation could change a partially completed retention operation's identity. | Fixed here; interface regression proves exact request/context preservation and original diagnosis. |

The root previously reported a complete native proof with 16 Fixture pairs,
34 executed producers and 102 reused operations. That report is not a proof of
the final source after these additional fixes. The observed infrastructure repair
failure was sandbox setup/finalization `EACCES`; the deleted test root prevented
a filesystem diagnosis. No unsupported cause is asserted here.

## Scoped verification

Latest command, run after the finalization seam, cancelled-tail assertion and
exact stopped-tail upper-bound reservation:

```sh
cd /home/jim/sm-decomp-translation-execution
SM_PIPELINE_ROOT=/home/jim/sm-decomp/out/pipeline SM_PIPELINE_WORKERS=4 \
cargo test -p translation-execution --locked --offline --test attempts \
  --target-dir /tmp/sm-supervisor-audit-target \
  > .scratch/translation-scope-execution/verification/supervisor-history-05.stdout.log \
  2> .scratch/translation-scope-execution/verification/supervisor-history-05.stderr.log
```

Passed 17/17 tests, zero failures, 0.26 seconds of test runtime; compilation was
clean. Existing gate/lost-response/retry/budget tests remained passing. Added
regressions cover editable terminal reopening, forged gate success, prior failed
finalization tamper, exact publication extent, partial-finalization cancellation,
exact cancelled tail, and lost-response/historical stopped-tail extent.
Every mock finalization also checks actual serialized result bytes fit the
maximum decimal-width reservation.

Scoped Clippy also passed with `cargo clippy -p translation-execution --locked
--offline --test attempts --target-dir /tmp/sm-supervisor-audit-target -- -D
warnings`. Retained `verification/supervisor-clippy-01.stderr.log` SHA-256 is
`ffac7e58b0b64d974d98760e3e0b1d0d9aea10c208ce3d5819e5bc70174dd0d5`;
stdout is empty. This pass executed no producers.

| Retained log | SHA-256 |
| --- | --- |
| `verification/supervisor-history-05.stdout.log` | `5ba8e08c00fd8fae4944158ae9bbaa87b9101f4786164f901a20f65ba5e2f9f0` |
| `verification/supervisor-history-05.stderr.log` | `6f8da13f7a16313692bcd1a636c953f4851c097ab9bc17e279cd0845bac57842` |

The inherited history-01 compilation stopped while root's checkout helpers were
in progress; no producers ran. Logs are preserved. History-02 and history-03
passed the earlier incremental seams. These interface tests use the recorded
adapter; root owns actual-C, infrastructure, managed migration and combined
adapter proofs. `git diff --check` passes on the owned Rust files. Formatting
uses Rust 2024 with `skip_children=true` to preserve shared module edits.

## Cleanup and next integration action

All scoped test/Clippy sessions finished. Test `Project` drops removed their
temporary journals. Both scoped cleanup passes used the existing retained
`native-gates/tools/sm-pipeline-cleanup` utility with `cleanup --root
/home/jim/sm-decomp-translation-execution/.scratch/translation-scope-execution/verification
--apply` and the same command with `--deduplicate`. Both passed and reclaimed
zero bytes; no run-format intermediates or identical snapshots existed there.
Receipts are `verification/supervisor-cleanup-{ordinary,deduplicate}.json`
(the CLI renders TOML despite this historical suffix).

After root confirmed adapter wiring, `cargo clean --locked --offline --target-dir
/tmp/sm-supervisor-audit-target` removed the finished disposable target. It was
measured at 399,586,877 logical bytes; Cargo reported 950 files and 428.1 MiB
removed. The target path is absent. Retained small test/Clippy logs are
byte-identical by post-cleanup SHA-256 checks; they have no dependency on the
target. Available disk space after cleanup was 188,565,966,848 bytes. Exact
source and diagnostic logs remain in this worktree for root's checkpoint.

| Cleanup evidence | SHA-256 |
| --- | --- |
| `verification/supervisor-cleanup-ordinary.json` | `db5bac8a65aebadf59f0fdf35212019050be91606a9165b298d3f3d192ecacdc` |
| `verification/supervisor-cleanup-deduplicate.json` | `2624524f0fb2f448e1351f8dad525947deffaa761537e3f3206162e1f18829b7` |
| `verification/supervisor-target-clean.stderr.log` | `5a8944c2ad57c7fd1383cdc7e53eb00c139e48eaf6b5d598ca038dc58dc90813` |

Root wired exact failed/cancelled delta, pre-retention serialized maximum reserve,
saved reservation identity and canonical final-record length authentication.
Next action belongs to root: complete combined adapter proofs, measured proof-run
cleanup and the shared source checkpoint. Production switch remains the separate
explicit user decision recorded in the spec.

## Followup: exact successful publication extent

Root requested success extent authentication inside publication, inspection and
resume. Exported `authenticate_publication_extent(root, policy, context,
completion, publication) -> Result<u64>` reads existing typed/hash-checked history
without taking another lease or invoking an adapter. It validates the original
current `Publish` intent and exact completion, then either projects the missing
terminal append or compares the actual terminal bytes with the shared serializer.
An actual terminal must end the complete current history. Its returned total
includes every regular journal file; root compares this with the accepted
publication delta plus frozen finalization prefix, so additional bytes reject.

Publication callback failure/pending response now preserves its original journal
intent. The adapter owns bounded diagnostics and ambiguous writer reconciliation.
A result sealed before a lost response can recover the same reserved terminal
extent. Typed replay also rejects changing the publication intent's reason.
Cancellation during publication drives that original success; its disposition
stays unchanged.

`supervisor-publication-01` passed all 18 attempts tests in 0.37 seconds. The added
interface regression loses a sealed publication response, authenticates without
mutation, checks exact recovered total bytes, rejects changed completion,
whitespace-only terminal alteration and changed accepted result, and detects
foreign regular journal bytes. Gates, source application and finalization each
remain at their original call counts. Scoped Clippy passed with warnings denied;
source formatting and diff-check are clean. Root wired the helper into
`publication_record`, covering publish/inspect/resume.

| Followup verification | SHA-256 |
| --- | --- |
| `verification/supervisor-publication-01.stdout.log` | `1bdb6ca18ac70ee098ab4367844d0d706ae709fbd49b731460153abcc835b48d` |
| `verification/supervisor-publication-01.stderr.log` | `0951fb5ec9e9e238432bc8e0d100205298c246211ca13a8463e6156ad122c119` |
| `verification/supervisor-publication-clippy-01.stderr.log` | `ec0401b715552da76cd8c9e5d9fa171199d4e434d3af8dc61aae8b1a7e64fb7f` |

Both followup utility cleanup passes completed with zero bytes and zero snapshots,
retaining the logs. `cargo clean --locked --offline --target-dir
/tmp/sm-supervisor-audit-publication-target` removed the finished independent
target: 401,859,942 measured logical bytes; Cargo reported 951 files and 430.3 MiB.
The target path is absent. Post-cleanup verification log SHA-256 checks passed.
Available disk was 188,444,360,704 bytes. Cleanup receipts are
`verification/supervisor-publication-cleanup-{ordinary,deduplicate}.toml` with the
same zero-result hashes as the earlier passes. Target-clean stderr SHA-256 is
`43c05310e98ea01924371c271719b912592949def484788fb15ebf7f0b9ed213`.
