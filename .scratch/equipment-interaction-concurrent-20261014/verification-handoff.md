# Private equipment actual-C run

Status: all 427 actual-C pairs pass, with unchanged replay and retained authentication.
Owner: root session `01a139b9-f1a2-7d30-b65f-f0741bb5d555`.
Finished tool process session: `88942`, exit 0; no producer remains.

The existing `translation_native::tests::real_reviewed_recipes_replay_and_authenticate`
test executed the entire `pause-equipment` family. It retains unchanged
replay, both cleanup passes and post-cleanup authentication. The contract is
[verification-requirements.md](verification-requirements.md).

- Private source: `847d76f60d8a373b7b051f0be5c441f6be2342f4`.
- Matching Native harness: main-repository commit `5cda6a0`.
- Input: `/home/jim/sm-decomp-equipment-interaction-20261014/verification-input-5cda6a0`.
- Output: `/home/jim/sm-decomp-equipment-interaction-20261014/native-v1`.
- Log/time: `/home/jim/sm-decomp-equipment-interaction-20261014/checks/native-v1.log`
  and `native-v1-time.txt`.
- Reader: `/home/jim/sm-decomp-performance-20261014/native/tools/native-tests-waves-v2`,
  SHA-256 `78804b95e8a815f3b074910e85ec5b54f7e5d054ec9979b74192f15fe7b30bdb`.
- Helper: `/home/jim/sm-decomp-performance-20261014/confinement/tools/sm-translation-sandbox-v7-opt1`,
  SHA-256 `fb403f0445d10cb3ce75fd2f928e6bedea27c4b9a1f5de8d445527252dcf2351`.

The environment sets `SM_NATIVE_RECIPE_PROOF` to the output path,
`SM_NATIVE_RECIPE_SOURCE` to the input path, `SM_NATIVE_RECIPE_REQUIREMENTS`
to its `docs/verification-requirements.md`, `SM_NATIVE_RECIPE_FAMILIES=pause-equipment`,
`SM_NATIVE_RECIPE_HELPER` to the helper above, and `SM_NATIVE_FIXTURE_JOBS=2`.
The shared pool remains `/home/jim/sm-decomp/out/pipeline`, capacity four.
Run argument selection is `--exact` with the test name above, `--ignored`,
`--nocapture` and `--test-threads=1`.

Two fixture workers leave room for one confinement control and the separate
architecture session's one build. Elapsed time is observational under this
concurrency. After interruption, inspect the existing writer and retained run
before any repeat; this record does not authorize replacing the attempt.
The original production registry, claims and installed controller are unchanged.

## Result

The complete test passed in 97.78 seconds (wall 97.79, user 24.31, system 26.11).
`summary.toml` reports `pause-equipment`, passing, 427 cases. The family verdict
reports 28 direct routines, full canonical comparisons, no Live exercise and
original-ROM parity not tested. The static helper is exercised through callers
within the complete 29-routine source boundary.

The ordinary cleanup pass reclaimed zero bytes; deduplication checked 1,281
snapshots and reclaimed 91,488,256 bytes. Retained Native authentication passed
after both passes. Keep the input snapshot, exact reader/helper, pinned C/ROM,
prepared contract, operation tree and receipt/verdict closure. Review and main
integration remain pending.

Both independent source reviews subsequently passed with no findings; source
and manifest bytes are unchanged. After both performance worktrees were
retired, read-only Native authentication again passed for all 856 producers
and 12,569 references. The log is
`checks/native-v1-after-worktree-retirement-auth.log` under the private root.
See [the review](../translation-scope-execution/ticket10/performance-review-20261014.md).
Main integration, Live replay and original-ROM parity remain pending.
