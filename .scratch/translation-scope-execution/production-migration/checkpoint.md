# Deterministic production migration

Status: active; original acceptance adapters remain pending
Updated: 2026-10-14

The user authorized migration and paused other work. Production ownership now
uses the installed Rust supervisor. Original claims, submissions and integration
ownership were imported without replacing attempts or completing gameplay stages.
The architecture source was integrated into main at
`a50a13e3d870d5994c9af7cd4f869c863ac3c4a2`; that remains the installed policies'
frozen integration baseline. The final checkpoint also includes the previously
untracked production transaction test module, operator documentation and these
activation/replay/cleanup receipts.

## Launch and inspect

From a normal shell, with no active Codex session required:

```sh
cd /home/jim/sm-decomp
/home/jim/sm-decomp-production-execution/bin/sm-pipeline \
  translation-production --checkout "$PWD" run
```

Replace `run` with `status` to inspect the active ownership registry. The command
currently returns exit 1 and a typed authenticated prerequisite stop at
`ceres-continuation-shared-game-host`. The controller remains unstarted behind
host; pause equipment rendering is explicitly blocked. This is usable ownership
and recovery infrastructure, but it cannot yet advance the carried-over work.
[Ticket 10](../issues/10-original-acceptance-adapters.md) records that work.

New approved rewrites use `FOUNDRY_TOKEN`, the existing Foundry Responses endpoint
and frozen `gpt-6.1-sol`/`high` configuration. Original submissions bypass rewriting.
The production launcher sets the exact sealed logical session only in its child
environment. No credentials are stored in these records.

## Activation and preserved ownership

[Activation](activation.stdout.toml) and [status](status.stdout.toml) report
`state = "active"`. Both original efforts are fenced. Important immutable records:

| Record | SHA-256 |
| --- | --- |
| Installed production request | `2d4ccbb0c343c45d397eca3ceb1f37cda655fa135f329d237db80387661fa5ae` |
| Main production registry | `9cdac7ad8916ce323c368b443e1435f8a2669b5b6908f0aa8c2494dca7508be9` |
| Activation receipt | `4f91c6b25768c1d4d2fa310745804c5250d23df033c0b817d7de189c0638dd23` |
| Ordered launch plan | `bdc8f26dc11bc1389306445f8a8bd040e5bf4722c7b4cd990f68c99ee14bec7c` |

The request is `/home/jim/sm-decomp-production-execution/production-request.json`.
The registry is main's `.scratch/translation-execution/production.json`.
The activation receipt and original snapshots are in its `activations/` directory
under the request hash. The legacy fence is main's
`.scratch/decomp-parallel/deterministic-execution.json`.

Checked transfer observations are under [inspections](inspections/). The current
logical owner is `01a136a0-49bb-7690-a360-48def3c2c09b`: Ceres integrator generation
3 and canonical dispatcher generation 2. The exact four original claims and
three submitted commits are listed in [ticket 08](../issues/08-production-switch.md).
Host and pause source remain on main. Controller source remains at
`223f8a1b0c7382817b1290d13c7a41f4e722d5e1` in its original held worktree.

Both ledgers are byte-identical to their pre-transfer snapshots. Main's entire
`crates/port` and progress source match pre-migration commit
`c2cae9ac145f9343cd345e6f978bff0774510b56`; [the diff](gameplay-preserved.diff) is empty.
The pre-existing assignment edit retains the hash in
[preserved-assignment.git-hash](preserved-assignment.git-hash).

## Replay and verification

The same production command was run twice with `env -u CODEX_THREAD_ID`.
[Run 1](run-01.stdout.toml) and [run 2](run-02.stdout.toml), including stderr,
are byte-identical. Both exit 1 for the preserved prerequisite failure. All 45
files in the installed production tree have identical hashes in
[the first inventory](execution-after-run-01.sha256) and
[the second inventory](execution-after-run-02.sha256).

The single run is
`a41caf059f04b355a6e84c45e70617153cef1fbf7405ad2da9954adb468e74df`, attempt 1.
It retains 102,220 bytes; cleanup, retained authentication and worker retirement
are true, and producer count is zero. There is no candidate checkout, model
exchange, gate result, completed claim or new published gameplay stage.
Replay allocates no additional attempt or artifact.

The migration verification retained under [verification](verification/) has
56 passing targeted tests: 22 attempt, 8 checkout, 4 production transaction,
5 Legacy readiness, 14 managed migration, 1 affected planner, 1 mutable native
dependency and 1 actual-C packed-call test. The latter took 19.66 seconds.
Two historical opt-in diagnoses were ignored. Production/core checks and the
installed build passed. [Formatting](format-check.md) records scoped success and
the unrelated existing whole-workspace formatting failure.

The four production tests cover exact recovery across registry/receipt and
legacy-fence interruptions, changed or partial inputs, and old-reader refusal.
No legacy production job was launched to test the fence. Initial failed build
logs remain retained alongside the passing retries.

## Cleanup and retained dependencies

Ordinary and deduplicating project cleanup completed for production execution,
verification, the migration scratch scope and both original preparation packages.
Every utility pass reported zero reclaimable bytes. The supervisor also retained
both attempt cleanup receipts. After writer completion, the Rust disposal helper
removed only its eight reviewed targets: the Cargo target, recovery helper and
six temporary C compile inputs. It checked their identities and C hashes first.

[Disposal](disposal-apply.jsonl) removed 4,207,966,325 logical bytes and
3,761,475,584 reclaimable allocated bytes. [Retiring the cleanup helper](cleanup-helper-retirement.txt)
removed another 4,597,048 logical / 4,599,808 allocated bytes. Total allocated
reclamation was 3,766,075,392 bytes. Remaining measured disk space was
186,157,821,952 bytes. Helper source remains in [cleanup_outputs.rs](cleanup_outputs.rs).

After removal, normal read-only `translation-execute ... inspect` succeeded:
[authenticated stop](inspect-after-cleanup.stdout.toml), exit 0. All 45 production
file hashes [passed](execution-after-cleanup.txt), and all 58 historical model
launch references [passed](historical-after-cleanup.txt). Original authority,
ledgers and progress seals [also passed](preservation-after-replay.txt).

Retain the installed production tree, main's installed sandbox helper, private
ROM inputs, `/home/jim/sm-decomp-production-inputs/snesrev-sm`, original source
snapshots, and the architecture checkout's referenced packages/readers/evidence.
The latter remains a retained tools/evidence checkout: the active request and
historical proof graph still reference it, so deleting that worktree would break
recovery. It is not a retired disposable gameplay implementer. Preserve the
nested `model-launch/native-02/approved-native/source/.git`,
`/tmp/sm-complete-supervisor-976030-0` and historical readers named in ticket 08.
The Submitted controller worktree and older preparation worktrees are outside
this run's disposal scope. Their original retention obligations remain recorded
in ticket 08. No submitted source worktree was retired as completed integration.

## Recovery and remaining acceptance

Activation writes durable intent, legacy fence, registry and receipt in that
order. After an interrupted activation use the exact request:

```sh
/home/jim/sm-decomp-production-execution/bin/sm-pipeline \
  translation-production \
  --request /home/jim/sm-decomp-production-execution/production-request.json recover
```

With a complete registry, `--checkout /home/jim/sm-decomp recover` is equivalent.
Keep both ownership markers. A change back to another orchestration policy
requires checked ownership transfer and stopped writers, preserving evidence.
An identical `run` resumes this stopped attempt. Repair cannot rewrite an
imported submission; adapter work requires a new frozen policy while retaining
the original source and claim lineage.

Host acceptance still requires the original 31 Rust tests, material callback
matrices and complete affected Offline verification. The 176-family count is
the retained historical batch requirement (106 authenticated passes, 11 compiler
failures, 13 interrupted, 46 unattempted), not a general current planner guarantee.
Freeze conservative current selection and resolve the retained preparation
defects before a broad rerun. Controller requires its original 8 unit + 2 public
tests, compact causal diagnostic (through frame 2240 where feasible) and historical
capture compatibility. Pause requires all 607 actual-C pairs, including auxiliary
direct animation, and 10 Rust tests. None can be replaced with a reduced native
gate set. Current Legacy readiness and completion intentionally refuse acceptance;
changing a readiness flag does not implement these gates.
