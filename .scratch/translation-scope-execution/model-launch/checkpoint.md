# Foundry rewrite and deterministic launch checkpoint

Status: complete for the approved isolated launch
Production migration: pending under [ticket 08](../issues/08-production-switch.md)
Updated: 2026-10-14

Implementation commit: `93e9d55b19630ac8fa2e3526ba963296385ea516`.
Worktree: `/home/jim/sm-decomp-translation-execution`.
Branch: `architecture/translation-scope-execution-20261013`.
The [implementation hashes](implementation-sha256.txt) bind the compiled Rust
source, tests, dependency lock and toolchain configuration. No Rust source changed
after the passing verification and disposable target retirement.

## Ready command

From the architecture worktree, with the existing `FOUNDRY_TOKEN` available:

```sh
./out/translation-launch/bin/sm-pipeline translation-launch \
  --request "$PWD/.scratch/translation-scope-execution/launch-foundry.json" run
```

The [request](../launch-foundry.json) is installed at `operator-run-01` and
its [post-cleanup inspection](operator-inspect-after-cleanup.stdout.toml)
reports `ready`, with no completed scope or model/gate producer started.
Its original source repository remains clean at
`f69ae88d96f66f0da5e2d2a78354ccb8db7fe8b9`.

The request covers the closed `enemy-jump-instructions` scope: two routines,
16 actual-C Fixture pairs per gate, and candidate/merged focused/Offline gates.
Rust owns installation, preparation, ordering, source validation/application,
commit, gates, integration, publication, cleanup and recovery. The model receives
only the frozen Rewrite capsule and returns structured Source replacements.

The provider is the Foundry Responses endpoint, `gpt-6.1-sol`, reasoning `high`,
32,768 output tokens, a 300-second deadline and a 1-MiB wire response maximum.
Requests set `tools: []`, `tool_choice: "none"` and `store: false`. The transport
disables redirects, proxy discovery and automatic retries. The inherited lease
retains a surviving exchange after controller death; ambiguous delivery seals a
stop result and requires deliberate repair. Endpoint, model, capsule, request,
helper and frozen policy changes reject reuse.

See the [operator guide](../../../docs/TRANSLATION_EXECUTION.md#one-command-foundry-launch)
for schemas, deliberate retry/repair/cancel and scope ordering. Ordering does
not rebase a scope's frozen source or integration policy.

## Passing verification

| Check | Result |
| --- | --- |
| Rust model transport suite | 4 passed; 2 opt-in/helper tests ignored in the ordinary suite |
| Success/replay/tamper | One exchange; unchanged replay reuses its result; changed model or retained response rejects |
| Redirect/error/refusal/overflow/secret/disconnect | Bounded retained failure, no resend and no credential bytes retained |
| Killed controller | Original surviving exchange recovered without another request |
| Killed exact helper | Ambiguous delivery sealed without resend |
| Real Foundry readiness | One bounded request passed at `foundry-readiness-01`; approximately 5 seconds |
| Ordered launcher suite | 3 passed; 2 opt-in native tests ignored in the ordinary suite; 648.43 seconds |
| Two-scope launch | Declared order completed with exactly two model calls |
| Completion projection recovery | Missing projection repaired without a producer or run-state mutation |
| Failed predecessor | Successor stayed unstarted; unchanged resume did not resend |
| Installation preparation | Identical/read-only preparation unchanged; changed request refused |
| Real Foundry/native CLI lifecycle | Complete at `native-02`, attempt 1, one model call and four passing actual-C gates |
| Retained native authentication | Passed in 19.615350102 seconds; 1,481 run files unchanged |
| Existing no-Python/configuration checks | Both passed |
| Locked/offline builds, formatting and diff check | Passed |
| Clippy for both affected crates | Passed with established shared allowances; existing dead-code/unused-toml warnings remain |

The producer suites ran with the real installed helper enabled. Final diagnostic
launcher fixtures were retained with `SM_SUPERVISOR_KEEP_INFRA=1`. Earlier failed
synthetic fixtures followed their existing Drop cleanup. The failed `native-01`
preparation is retained; its missing-parent receipt bug was fixed before the
passing final proof. Completed producer tests were not repeated during handoff.

## Exact real proof and retained readers

Run ID: `373b62495003384ddc70231b0cbbd4c3f6a4010264d96819e980132fc8a8a5ea`.
Candidate: `773abe7dad479d8967802b61a8712ba266a78209`.
Integrated proof source: `a605e5e0609529b3379767af882bfe893b774530`.
The integration belongs to this isolated proof and grants no production stage.

The wire response was HTTP 200, 36,276 bytes, with returned model
`global.openai.gpt-6.1-sol`: 4,328 input, 8,398 output and 12,726 total tokens.
The [authentication report](native-02/authenticated.json) records one model call,
four gates and 16 Fixture pairs per gate. [Post-cleanup inspection](native-02/inspect-after-cleanup.stdout.toml)
reports `complete`. Both post-cleanup inspection commands exited successfully
with empty stderr.

The [reference hashes](checkpoint-sha256.txt) bind exact installed plans,
configuration, model exchange, finalization, publication and cleanup receipts,
plus these retained readers:

- `out/translation-launch/bin/sm-pipeline`: SHA-256
  `574285f320615f476336f629c9ebd49786110ac06bc194b38fe60c4e2ad97b64`,
  294,868,888 bytes. This is the stable user command, independent of Cargo targets.
- `model-launch/tools/retained-test-reader`: SHA-256
  `90e6a6a506b4b25b0ec3db568fcb16278a2a1405251e05f7f7c054b5b444db6a`.
  This retained reader produced the native authentication report.

Repeatable read-only authentication of the completed launch uses its installed
plan; it needs no new model credential or exchange:

```sh
./out/translation-launch/bin/sm-pipeline translation-launch \
  --plan "$PWD/.scratch/translation-scope-execution/model-launch/native-02/execution/plan.json" inspect
sha256sum -c .scratch/translation-scope-execution/model-launch/checkpoint-sha256.txt
```

Preserve `native-02` and its complete retained closure, `operator-run-01` installed
tools/configuration, Foundry readiness records, private ROM/C/provenance inputs
and exact dependencies named by the installed contracts. In particular,
`native-02/approved-native/source/.git` remains the active original source input
for the prepared user launch. Local Git retains source and compact pointers;
moving this prepared launch also requires its retained tools, inputs and evidence.
Historical installations/readers, including `/tmp/sm-complete-supervisor-976030-0`,
remain untouched.

## Cleanup completion

Every selected producer finished or stopped before cleanup. Native supervisor
finalization retained 60,877,941 bytes and reclaimed 1,935,173 checkout bytes;
its two project cleanup receipts remain in the accepted finalization closure.
Candidate and integration producer checkouts were retired. The approved original
source input is retained for the fresh prepared command.

Both model-launch-root passes succeeded:

- `sm-pipeline cleanup --root MODEL_LAUNCH_ROOT --apply`: 0 bytes.
- `sm-pipeline cleanup --root MODEL_LAUNCH_ROOT --deduplicate --apply`: 0 bytes,
  96 snapshots checked.

The same ordinary and deduplicating commands completed successfully for every
retained diagnostic root below. All twenty processes exited 0, all stderr files
are empty, and every receipt reports 0 bytes; deduplication checked 0 snapshots.
Receipts use `diagnostic-cleanup-I-ordinary.toml` and
`diagnostic-cleanup-I-deduplicate.toml`, with hashes in `checkpoint-sha256.txt`.

| I | Diagnostic root |
| --- | --- |
| 0 | `/tmp/sm-complete-supervisor-1126761-0` |
| 1 | `/tmp/sm-complete-supervisor-1127905-0` |
| 2 | `/tmp/sm-complete-supervisor-1127905-1` |
| 3 | `/tmp/sm-complete-supervisor-1127905-2` |
| 4 | `/tmp/sm-complete-supervisor-1127905-3` |
| 5 | `/tmp/sm-complete-supervisor-1127905-4` |
| 6 | `/tmp/sm-ordered-launch-1123243-0` |
| 7 | `/tmp/sm-ordered-launch-1123243-1` |
| 8 | `/tmp/sm-ordered-launch-1127905-0` |
| 9 | `/tmp/sm-ordered-launch-1127905-1` |

[Measured disposal](disposal-apply.jsonl) removed 34 exact targets: the disposable
Cargo target, 27 synthetic transport helpers and six temporary compile-time C
copies. Logical removal was 6,201,783,239 bytes; allocated reclamation was
5,773,107,200 bytes. The [disposal helper retirement](disposal-helper-retirement.json)
then removed its two measured temporary files with exact `unlink` commands,
after byte-comparing its archived Rust source. This reclaimed another 4,755,456
allocated bytes and removed 4,749,757 logical bytes.

Total identified build/helper/C-output disposal: 6,206,532,996 logical bytes and
5,777,862,656 allocated bytes, separately from supervisor checkout reclamation.
Observed available space afterward: 186,937,880,576 bytes. The disposable Cargo
target and both temporary disposal files are absent. Retained reader/CLI/input
hashes and complete/ready post-cleanup reports remain valid. No cleanup action
remains for this ticket.

## Production boundary and next action

Main remains at `c2cae9ac145f9343cd345e6f978bff0774510b56`, preserving its existing
assignment Markdown modification. [Read-only authority status](production-authority-after-cleanup.stdout.toml)
is byte-identical to checkpoint 06: active canonical queue, capacity 3 and the
original integrator at generation 2. Startup and final protection checks found
no source, authority, claim, ledger or progress mutation by this work.

Ticket 09 is resolved. Ticket 08 and the overall migration spec remain open:
Submitted/linked Integrate sequencing, specialized catalog preparation, current
owner reconciliation and the separately agreed production switch still apply.
This proof establishes actual-C acceptance for the declared isolated scope.
Live replay, original-ROM parity and production queue migration retain their
independent requirements. The operator can launch the ready command above.
