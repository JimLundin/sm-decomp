# Ticket 10 compact-controller adapter

Status: source implemented; proof 04 authenticated after cleanup; ready for root review
Owner: `/root/compact_diagnostic`
Worktree: `/tmp/sm-ticket10-compact-20261014`
Branch: `ticket10-compact`
Baseline: `808efe2ee473e633a42bbf330f3872f5a0499df9`

## Source and integration boundary

Owned source is `crates/pipeline/src/translation_compact.rs` and this handoff.
Implementation checkpoints are `b82ce75`, `647e255`, `49a03b2`, `ed040dd` and
`409e1ca`. The root owns permanent main registration, original candidate/integrated
Cargo controls, composite acceptance, production installation/continuation,
ledgers, final review and retirement of this worker's disposable worktree/target.

The original Submitted controller worktree
`/tmp/sm-ceres-continuation-compact-controller-1` was not changed. No production
ledger or main-checkout Git/index was modified by this worker.

The typed API is `contract_for_original`, `historical_for_original`, `prepare`,
`prepare_until`, `run`, `authenticate`, `references`,
`authenticate_historical` and the internal confined `producer` command.
Register `translation-compact-producer` before ordinary CLI `current_exe()` /
`BINARY_SHA256` initialization, dispatching to `producer(&args[1..])`.
The temporary local main registration and borrowed confinement client are not
part of compact commits and are restored before handoff. Their exact tested
contents remain preserved in the frozen reader context snapshots.

`StrictRuntime` supplies helper, producer, operation_root, pipeline_root, workers,
gate_timeout_seconds, optional deadline_unix_ms and max_output_bytes.
`CompactReceipt` exposes passed, diagnostic, evidence_bytes, context, verdict
Asset and prepared/candidate identities. It derives PartialEq, not Eq.

## Preserved contract

- Exactly frame 2240 and one whole-experiment 60-second deadline shared by native
  capture/packing, C compilation, causal candidate, independent changed-RNG
  control and comparison. There is no shorter passing substitute.
- Parent-owned comparison of ordered polls, completed states and raw WRAM
  snapshots, preserving the original 16 semantic words and original exclusions
  for NMI counts/waiting. The default full observer is unchanged.
- Exact original source, complete C unit/private-source closure, ROM/movie/lsnes
  identities, archive pin, any% manifest/import semantics and runtime closure.
  Compiler-header directory binds are limited to the compile phase.
- Durable intents, deadline/byte reservations, bounded producers and retained
  failures. Unchanged resume authenticates the existing receipt without creating
  another capture or operation.
- All five original full/compact captures and exact reader/private dependencies
  are required. Historical reader operations use the persisted outer gate
  deadline, at most 3600 seconds; they do not consume the fresh experiment's
  separate 60-second interval.
- No Live replay or original-ROM parity acceptance is granted.

Native, compile and candidate/control writable limits are respectively 256 MiB,
64 MiB and 384 MiB. The real proofs use a 4 GiB aggregate gate reservation.
Every producer uses the operation engine and shared
`SM_PIPELINE_ROOT=/home/jim/sm-decomp/out/pipeline`, `SM_PIPELINE_WORKERS=4`.

The exact old compact reader needs `current_exe()`. Its direct Program requests
the reviewed bind source equal to its executable and destination
`/proc/self/exe`; the helper makes only a synthetic symlink to `/program`.
No procfs mount or host process exposure is added. The native worker owns this
helper change and its guards. Historical Python drivers remain sealed source
evidence; they were never executed.

## Verification

Initial Cargo checks passed. Three preparation controls pass: rejecting a
shortened target, requiring both capture profiles, and preserving all five
original captures. The complete-lineage control was observed red before its
implementation, then green. Two real-evidence tests are ignored by default.

The exact original compact ELF
`2898c6a3f553e74051abbb00f47a429fd653522d0caa4a06be33161eb581e57a`
successfully authenticated its original capture under each v1/v2/v3 helper.
Its stdout is `Prefix diagnostic authenticated; acceptance remains false`,
SHA `2bcde0bb1a6f2e3a0e4a31510e11f29d15a58a4ba8b7d6f30f4b88d5261b8cd1`.

All five retained histories authenticated, covering 6,629,732,570 artifact bytes:

| Original capture | Artifact bytes | Recomputed result |
| --- | ---: | --- |
| controller-terminal-seam/20261011T072946.717098Z | 2,126,947,613 | 1097/1097 completed updates; matching prefix 1097 |
| controller-1600-frontier/20261011T072145.761041Z | 2,126,681,921 | 1096/1097 completed updates; matching prefix 1096 |
| controller/1791873256094852042-256891 | 677,497,832 | 1736/1736 completed updates; 1887 polls; matching prefix 1736 |
| controller/1791873514143170077-260799 | 488,857,207 | Original 8600-frame error preserved |
| controller/1791874177761738742-271877 | 1,209,747,997 | Original 8600-frame error preserved |

The first two roots are under `out/port/`; the final three are under
`out/port/ceres-continuation/`, all in `/home/jim/sm-decomp`.
Independent changed-gameplay detection was recomputed for each nonaccepting
historical comparison.

The complete original experiment was attempted in three separately frozen
runtimes. Each retained `confined compact native failed: sandbox timeout`.
No compile/candidate/control phase was launched, so no fresh causal comparison
or independent-control success is claimed. Each proof's passing test establishes
the original target/deadline, authenticatable failure and unchanged resume.

| Proof root suffix | Helper | Start / deadline / finish (Unix ms) | Unchanged-resume test |
| --- | --- | --- | --- |
| 02-20261014 | v1, unoptimized | 1791974982178 / 1791975042178 / 1791975042217 | 1 passed; 885.90 s |
| 03-20261014 | v2, release | 1791977968659 / 1791978028659 / 1791978028752 | 1 passed; 1183.32 s |
| 04-20261014 | v3, release; corrected read-only-open scan | 1791979359323 / 1791979419323 / 1791979419369 | 1 passed; 297.47 s |

Proof roots are `/tmp/sm-ticket10-compact-proof-<suffix>`.
The small finish overruns are termination/retention after the original deadline,
not accepted experiment extensions. There was no cgroup CPU throttling observed
before/after proof 04; no contention attribution is claimed.

## Immutable evidence and readers

In each proof, the receipt is `gate/compact-receipt.toml`, the verdict is
`gate/compact-verdict.toml`, and the prepared contract is `prepared.toml`.

| Proof | Receipt SHA-256 | Verdict SHA-256 |
| --- | --- | --- |
| 02 | c88f4651afafc5e9561cfe523c1314dca4bcca698d55de93d94d87dd030e8cc6 | 9e42de4c9b0461b5e55e82c8bad42b739a4d5d0319cd9c73c8a901b23d963fb0 |
| 03 | 326490ea5a1ee4e8af1844d5eb63268ddb18caa5d3a1bfa7132fdd9f4a4771f7 | 6c03b99f6d13a7d71c265fde41d650194541c495f6bc2075a0e18b75d99bcb52 |
| 04 | 266baf756f3a1ec2825ff76d20658d46ce720a1465162dce22bb9f9edca635e7 | e44b9a2a16d54ad5119194e34429aaf385db4074cd65780c87a7a4e0c46c77bb |

All three prepared.toml files have SHA
`365e26f230ad0fc4ccc92db60c05ad141b532aba70da4cc9e72a58e0df19d21d`.
Their typed prepared identity is
`a9be2581e0d30b312e6d05411cb4746aa14758226894a68c29a01ca16657156b`,
and original candidate identity is
`d95e0b95d1ab5021431e15a73991408566e5d32e95895c06658f248701f36e49`.

The producer, unchanged across proofs, is each proof's
`tools/sm-pipeline-producer`, SHA
`b40a6fd24b7c15da9e9b0de9f475f89de79b61a7e4d8bcff75725d78e1f9a7ef`.
Only debug sections were removed from the complete built CLI.

| Reader | SHA-256 |
| --- | --- |
| proof02/tools/compact-proof-reader | 9b6c911fde7fb408a1d713b5bf8de0f44c4108d996308dd4264d99062804ccaa |
| proof03/tools/compact-proof-reader | a5e46472ac191805ed8e264368df09772d71bd084c86eaefb1010d37400cd54b |
| proof03/tools/compact-cleanup-reader and proof04/tools/compact-proof-reader | 2aec4791a902eca22d2a29a7a788d508f83e9f71f3770e9df6acfb5ebcb6cb4c |

The final reader uses unchanged adapter source from `409e1ca`, with only
`profile.dev.package.sha2.opt-level=3` for its hash dependency. Its ordinary
compact controls and exact retained-evidence authentication passed. Cargo exited
143 after the complete executable had been published; the cause is unknown.
The terminated build log is retained, and `proof04/reader-build-profile.md`
records the command and direct executable validation. Source, temporary main and
compatible confinement-client snapshots are retained beside the reader seals.

Keep these native-owned immutable helper dependencies:

| Helper | Source commit | SHA-256 |
| --- | --- | --- |
| /tmp/sm-ticket10-native-proofs/confinement-v1/tools/sm-translation-sandbox | 05785a6 | d3bee79b2a711126d6e0de4405e578c1ab51d27b420fadce86515c0eef8f3338 |
| /tmp/sm-ticket10-native-proofs/confinement-v2/tools/sm-translation-sandbox | e72bf5b | d334899b7f5cc7f64120636a803224d2f83e54c8d049e71e57a9d198a5fdf892 |
| /tmp/sm-ticket10-native-proofs/confinement-v3/tools/sm-translation-sandbox | ec09afe | 21f1bdb8d4bb10cbad2c5c23a24e7e146b2ed9a17fc4695b41d7813c3f2b66c5 |

The v1 client/decoder is compatible with all three guarded helpers.
Preserve the original capture directories, their original reader sources/ELF,
ROM, movie, lsnes, pinned C archive/source and runtime dependencies named by the
prepared contracts. These are required for future authentication.

## Completion evidence and cleanup

Each finished proof was cleaned with the project utility:

```sh
tools/sm-pipeline-producer cleanup --root PROOF --apply
tools/sm-pipeline-producer cleanup --root PROOF --deduplicate --apply
```

Both passes reclaimed 0 bytes for proofs 02, 03 and 04; this format has no
disposable files recognized by the utility. Logs are `cleanup-pass1.log` and
`cleanup-pass2.log`. Receipt/verdict/prepared checksums remain unchanged in
`after-cleanup-hash-check.log`.

| Proof | Read-only post-cleanup authentication | Available bytes after cleanup |
| --- | --- | ---: |
| 02 | 1 passed; 224.21 s | 170,271,776,768 |
| 03 | 1 passed; 65.26 s | 166,270,836,736 |
| 04 | 1 passed; 38.56 s | 166,130,987,008 |

The read-only test calls `authenticate`, then checks unchanged references and
operation inventory. It launches no producer. Its logs are
`after-cleanup-authenticate.log`. Space measurements are in
`free-space-after-cleanup.txt`.

Provisional proof 01 is retained separately under
`/tmp/sm-ticket10-compact-proof-01-20261014`. Its old `b82ce75` fixture used a
readable run id instead of the required digest, so it retained
`durable operation requires exact context identities` without launching a
producer. Its broad unchanged-receipt test passed in 644.79 s; both cleanup
passes reclaimed 0 bytes and left 174,504,767,488 bytes available. Receipt SHA:
`5be5e3f9118f868f0d8482d4c216061304dd5796712d7c777604cc122f0c8326`;
verdict SHA:
`da9ab25ce012b3dc3eaec06049254e1765d32e999226337a9bf1c4eaaa3bc21d`.
Its exact old adapter source is `reader-source-b82ce75.rs`, with checksums in
`after-cleanup.sha256`. That provisional format predates the complete-lineage
field; its old test executable was overwritten before reader freezing was
introduced. No current-reader post-cleanup authentication is claimed for proof 01.

Root should remove this integrated worker worktree and disposable
`/tmp/sm-ticket10-compact-target-20261014` after preserving the frozen evidence
listed above. The original Submitted worktree and all referenced proof/readers
must remain. Final production cleanup/publication and ticket status belong to
root. This handoff grants no gameplay Offline, Live or ROM-parity acceptance.
