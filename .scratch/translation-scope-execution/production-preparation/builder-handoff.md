# Read-only migration evidence builder

Status: scoped implementation and verification complete
Owner: `/root/supervisor_audit`
Worktree: `/home/jim/sm-decomp-translation-execution`
Branch: `architecture/translation-scope-execution-20261013`
Owned source: `crates/pipeline/src/translation_migration_builder.rs`
Source SHA-256: `d3e5b081c0c22d79242c2cb7e14b71a908370b6cbda9cdbf334a22c667650237`
Related ticket: [08 production switch](../issues/08-production-switch.md)

The Rust builder is ready for root's actual three-claim preparation after the
final representative proof. Root owns the final proof, production observations,
CLI wiring, shared source checkpoint and actual package invocation. This worker
did not mutate production source/index, authority, ledgers, queue or progress;
it recorded no owner-stopped event, created no claim and activated nothing.
Production ticket 08 remains unresolved under its agreed switch requirements.

## Builder contract

The wired CLI is `sm-pipeline translation-migration build-preparation --request
REQUEST.toml` (JSON is also accepted). Request schema is
`translation-migration-build-v1`; public crate APIs are
`build_from_request(&Path) -> Result<BuildReceipt>` and
`build_preparation(BuildRequest) -> Result<BuildReceipt>`.

The request names canonical main `checkout`, original `effort`, selected
`claims`, exact supplied `stopped_owners`, representative `proof_installation`
and `proof_completion` References, and an `output_root` strictly below this
worktree's `.scratch/translation-scope-execution/production-preparation/`.
Generated outputs live only in `output_root/package/`. An empty package-local
`builder.lock` fences concurrent builders; no production lock is created.

Limits default to 60 seconds, 64 MiB, 4096 referenced files and 4 MiB per source
file. Hard maxima are 300 seconds, 256 MiB, 4096 files and 4 MiB per source file.
One shared monotonic deadline covers Git, reference reads, capture, validation,
output draining and publication. The validator runs as a bounded subprocess;
refusal remains an explicit blocker, never fabricated validation. Expired
deadlines stop without a completed receipt. Subprocesses run in a new session;
timeout/output overflow terminates that session before reaping its leader.

Submitted source comes only from unchanged `WorkResult::Submitted.commit`.
Running integration requires an explicit exact current committed HEAD. Legacy
short base-head remains unchanged in the original Job and is separately resolved
and checked for ancestry. Regular Git blob mode and exact path are checked
before extraction. Frozen snapshots contain no Git metadata.

Original Job, Claim, state, agent, result and observations are preserved in
typed OriginalClaim records. Supplied stopped/session/process observations are
authenticated and exact live kernel PID/start/boot identity rejects. The builder
does not establish liveness itself or manufacture thread observations.

Declared `file:` and paired `*-file`/`*-sha256` inputs retain their original
values. Matching bytes are copied under `package/inputs/`; dirty or missing
files can recover only from original source/baseline Git blobs with the unchanged
declared digest. Immutable input decisions retain the exact mapping and any
honest missing-input blocker across retries.

The Migration remains `prepared`. Every selected original gets `installation =
None` and an explicit zero policy-digest sentinel, with limitations explaining
that no reviewed production installation or policy exists. Linked overlapping
submission/integration originals remain together with sequencing blockers.
The BuildReceipt has `activated = false` and reports exact byte/file accounting.

## Recovery

Before generating new outputs, retry authenticates the intent's normalized
request, original request Reference, exact authority, ledger and source plans.
Completed retries authenticate the original inventory, input mappings, source
blobs and retained validation decision, then return the byte-identical receipt.
They run no fresh validator. Separate `translation-migration
validate-preparation` remains available for a new read-only inspection.

Incomplete retry charges every existing package file before another output is
published, while admitting expected outputs in deterministic order. This avoids
counting old inventory/receipt into their own regenerated closure. Immutable
writes accept identical bytes only. A deterministic `.pending` file must be a
prefix of the identical expected publication before recovery removes it; foreign
pending bytes reject. Unexpected files reject and remain untouched. Final scan
checks every actual package file against the recorded closure. Nonregular files,
symlinks, aliased/hardlinked package files and unsupported paths reject.

Git 2.25.1 is installed at `/usr/bin/git`. Its `rev-parse` silently echoes the
unsupported `--path-format=absolute` argument; actual Git tests exposed this.
The builder now uses portable `--git-common-dir` and checked bounded path
resolution. Child environment is cleared; hooks, fsmonitor, replacement objects,
maintenance and optional locks are disabled.

## Scoped verification

Final command:

```sh
cd /home/jim/sm-decomp-translation-execution
SM_PIPELINE_ROOT=/home/jim/sm-decomp/out/pipeline SM_PIPELINE_WORKERS=4 \
cargo test -p pipeline --locked --offline --bin sm-pipeline \
  translation_migration_builder::tests --target-dir /tmp/sm-migration-builder-target
```

`builder-gates/test-05.{stdout,stderr}.log`: 10/10 passed, zero failures, 1.02
seconds of actual Git/process test runtime; compilation 45.76 seconds. Synthetic
typed observations and placeholder proof pointers are test fixtures; these
tests explicitly report `validation-not-run`, and establish no real owner or
representative-proof acceptance.

Coverage includes submitted Git bytes despite dirty main files; unchanged
original job/result/short baseline; linked running integration's exact HEAD;
byte-identical complete and interrupted retries; source/input snapshots; changed
request/authority/ledger refusal; foreign/tampered output rejection; session and
exact live process refusal; aggregate/file/source limits; Git symlink mode;
pending prefix/path aliases/nonblocking FIFO refusal; subprocess timeout,
overflow, nonzero exit and exhausted prelaunch deadline.

`cargo check -p pipeline --locked --offline --bin sm-pipeline` passed initially.
Full binary Clippy with `-D clippy::all` reports 14 existing/shared findings and
no builder finding; its actual failure is retained in `clippy-01`. A later full
binary Clippy with ordinary warnings (`-- -A dead-code`) passes with 10 findings
outside this module and no builder diagnostic; `clippy-02` is retained. This is
not a claim that repository-wide warning-denying Clippy passes. Root received
the concrete shared-source findings. No concurrent source was changed here.
Rustfmt used edition 2024 with `skip_children=true`; owned diff check passes.

| Retained artifact | SHA-256 |
| --- | --- |
| `builder-gates/test-05.stdout.log` | `119a53422c5514ba0da59b0d7c4efd9e17b4da2563edbd8ba66d9f98a19bf197` |
| `builder-gates/test-05.stderr.log` | `fb41ad8e7f89b53b35b47d70527cd4dd604b0c237a632bef3a713088be2af37b` |
| `builder-gates/clippy-02.stderr.log` | `bf4f1c82526ebfc3cb982205da2318d62d4da8aafec35c4c7c2aa822021549b8` |
| `builder-gates/cleanup-ordinary.toml` | `db5bac8a65aebadf59f0fdf35212019050be91606a9165b298d3f3d192ecacdc` |
| `builder-gates/cleanup-deduplicate.toml` | `2624524f0fb2f448e1351f8dad525947deffaa761537e3f3206162e1f18829b7` |

Earlier test logs retain the fixture TOML/null mistake and its correction,
portable Git discovery failure and its correction, and the no-op authority
fixture mutation. No test failure was accepted or concealed.

## Completion cleanup

All owned Cargo, test and Clippy processes finished. Fixture Drop removes its
private temporary repository/output roots. Both scoped project cleanup passes
used the retained `native-gates/tools/sm-pipeline-cleanup` on `builder-gates`,
first `cleanup --apply`, then `cleanup --deduplicate --apply`; both succeeded
and reclaimed zero bytes. No run intermediates or identical source snapshots
existed in this small log directory.

The disposable test/check target was measured at 1,613,020,396 logical bytes;
Clippy target at 237,958,138. `cargo clean --locked --offline --target-dir` removed
both: Cargo reports 2066 files/1.5 GiB and 654 files/313.5 MiB respectively.
Both `/tmp/sm-migration-builder-target` and
`/tmp/sm-migration-builder-clippy-target` are absent. Clean receipts are
`target-clean.stderr.log` and `clippy-target-clean.stderr.log`. Available disk
space after cleanup was 186,522,677,248 bytes. Logs and source hashes were
rechecked after cleanup; retained evidence has no target dependency.

Next action belongs to root: complete actual-c02 proof, use its exact retained
installation/completion References with the three supplied Ceres originals,
invoke build-preparation once, reauthenticate its immutable retry, and retain
the real read-only validator's outcome. No production activation is authorized
by this worker's package or tests.
