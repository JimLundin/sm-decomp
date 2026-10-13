# Swarm update completion

Status: resolved
Blocked by: None
Owner: `01a13559-e517-72d1-88b5-e554fed61da4`
Updated: 2026-10-13

Preparation tickets 01–05 and the completed-integration retry correction are
resolved. The next decomp work is [11: finish held Ceres integration](../issues/11-finish-held-ceres-integration.md).
Use [the launch handoff](launch-handoff.md) on the next gameplay launch.
Completed preparation needs no additional drill or tooling work.

The Rust correction at `57f93c5ce4e6adac2dfe181b852d4aad6e0776bd` is
integrated as `0d042e0`; all 51 focused assignment/session/authority tests
passed. Main Cargo/crates/tools match the clean retained source at
`/tmp/sm-decomp-preparation-retry-source-20261013`. The retained engine SHA-256
is `04f4a10aec71b9654cdc1c7f1aca6936f9a9bb7ce810f8232c955f3b11e7d573`.

## Actual drill and readiness

[Final drill](../drill-retry/concurrency-drill.toml) SHA-256
`63912c67e38680fd163d09126ba285f8ce26da8a2445c801f489f8971b74751f`
and [execution](../drill-retry/execution.toml) SHA-256
`805dde33a142cc10dc9d9c3c2d6791d1589639aebaebb9feaba7a71dbbccb67a`
retain all eleven controls and twelve actual launches. A/B integrations
completed serially at `0812fbcd0857b6845f2ef966f6b2f601c0b4a949` and
`2b1400ef758907c9aee50f997bb9bca05216ec52`. Both exact completed retries
passed after final HEAD and implementer retirement; all 18 receipt files
remained unchanged and no gate reran.

[Interruption proof](../drill-retry/post-publication-interruption.toml) SHA-256
`6735bb4c95e4ec1581ec5b13fb61b96ed2e5621b261f181decff49c68eb6b451`
records the same B UUID stopped after durable publication and resumed under
generation 2, with both exact updates accepted. [Final metadata authentication](../drill-retry/metadata-closure-20261013/completion.md)
passed 318 artifacts, twelve exact launch IDs, two integrations and eleven controls.
Original launch metadata and earlier failed epochs remain historical evidence.

[Actual readiness completion](readiness-final-20261013/current-root-20261013/ledger-directory-fix-20261013/private-dev-fix-20261013/completion/completion.md)
records root's successful maintained `prepare`, `validate-activation` and
`status` commands in a private namespace. The output manifest SHA-256 is
`241018cd062ef561c5650c90bce7840bec5e0749e9ebc4d2b220203fa1e4269d`;
all 22 outputs and 339 recursive dependencies authenticate after cleanup.
Policy state is `prepared`, with unchanged policy bytes and host ledgers.
Production authority and canonical ledger are absent. Gameplay launch and
full-game acceptance retain their own scope.

## Cleanup and retained dependencies

All selected normal and deduplication passes succeeded:

- [Retry worker integration cleanup](../../decomp-integration-retry/integrator-cleanup.md):
  implementer worktree retired (412,364,768 logical bytes) and target cleaned
  (1,496,796,319 logical bytes); utility passes reclaimed zero bytes.
- [Finished drill cleanup](../drill-retry/cleanup.toml):
  zero utility bytes; helper target 230,633,386 logical bytes.
- [Independent review cleanup](../drill-retry/review/cleanup.toml):
  zero utility bytes; private target 114,797,978 logical bytes.
- [Interruption closure cleanup](../drill-retry/closure-final-20261013/completion.md):
  fixture and package passes zero bytes; private target 114,961,013 logical bytes.
- [Metadata closure cleanup](../drill-retry/metadata-closure-20261013/completion.md):
  both passes zero bytes; final manifests authenticated.
- [Retained source audit](final-retention-20261013/README.md) and
  [successor audit](final-retention-successor-20261013/README.md):
  seven finished scopes and successor passes zero bytes; all 1,740 source seals
  and historical dependencies authenticate.
- [Readiness cleanup](readiness-final-20261013/current-root-20261013/ledger-directory-fix-20261013/private-dev-fix-20261013/completion/completion.md):
  finished successful and failed outputs cleaned twice with zero bytes;
  readiness helper targets measured 64,644,986, 15,781,913, 15,820,897 and
  15,806,486 logical bytes before removal.
- Final checkpoint receipt archives: both project cleanup passes returned zero
  bytes; commands and outputs are `checkpoint-cleanup1-20261013.toml` and
  `checkpoint-cleanup2-20261013.toml`. Both archive hashes pass after cleanup.

These recorded worker/helper removals total **2,481,607,746 logical bytes**.
They are distinct directory measurements, not a filesystem free-space delta.
Final readiness available space was **221,230,100,480 bytes**.
No disposable preparation implementer remains. Four frozen preparation Git
subjects retain exact HEAD/common-root/fingerprint evidence requirements.
The unresolved Ceres compact-controller worktree and original submissions remain held.

Keep exact frozen source paths, tiny isolated fixture Git history, sealed logs,
snapshots, builds/readers and helper executables. Existing gameplay evidence
also retains its shared store, pointers, readers and private ROM/movie inputs.
The Ceres ledger remains `77fdba066af4f4c045e8e8f419997563d569ac0bd8045cef0a68ac916a126b89`;
the legacy bridge remains `cb537f195b4f83529f2ea7a61bd65c6e77ad215366275e8d60a7a604c4eb0ba9`.

Root's [final checkpoint](final-checkpoint-20261013.md) names the committed
closure and raw receipt archive. These dependencies are needed for recovery;
Git source alone does not restore local executables or private evidence.
