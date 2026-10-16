# Verify setup and publish Git backup

Status: resolved
Blocked by: None

The user requested a properly configured, committed project and explicitly
authorized using their authenticated `gh` account to push it to GitHub.
Use a private `JimLundin/sm-decomp` repository, since no visibility was specified.
The user subsequently explicitly requested including the ROM; track the verified
canonical `rom/sm.sfc` and update the documented Git policy accordingly.

Scope: main checkout Git/index, the missing ignore rule for the retained recovery
executable, and this setup record. Checkpoint existing authored progress/handoff
records without changing their claims or acceptance stages. Preserve all existing
development branches and worktrees; this is not a gameplay integration session.

## Observations

- Existing branch `main`, starting at `193cad941713b3539d23e08238a79e7fb09dde61`.
- Git identity and GitHub authentication are already configured; no remote exists.
- Rust nightly, both Cargo lockfiles, bootstrap instructions and GitHub CI exist.
- Pending records include progress inventory and restart/performance handoffs.
- The recovery client is a retained executable with a tracked-source candidate;
  ignore the executable at its original evidence path and retain the source.
- The separate `restart/playable-baseline` branch contains newer gameplay work.
  Preserve development history on GitHub without merging it into main.
- Two historical Python scratch scripts remain tracked; the documented Rust
  migration is not claimed complete by this setup checkpoint.

## Completion evidence

- `83f8b11`: checkpointed 154 files of progress, handoffs, authored Rust probes
  and diagnostic receipts; ignored the retained compiled recovery client.
- `b16ff70`: included the verified 3,145,728-byte ROM and updated the README,
  ROM documentation and ignore rule at the user's explicit request.
- Created private https://github.com/JimLundin/sm-decomp and configured `origin`;
  `main` tracks `origin/main`. Pushed all 121 local branches. An exact comparison
  of `git for-each-ref` local heads and `git ls-remote --heads origin` passed.
- Explicitly confirmed `restart/playable-baseline` is backed up at
  `6e74c8b11bf370aae3e376a45c33643ca0c0c5b9`.
- GitHub's ROM blob ID matches local `git hash-object`:
  `5dd855723aae500fb8c86d7a8e7096d971cc8614`.
- `git fsck --full --no-dangling`, `just doctor`, both formatting checks,
  bootstrap tests and staged whitespace checks passed. No blob over 50 MB
  was found in branch history. Common credential-pattern checks found no matches.
- Workspace tests were started as a setup check, then intentionally stopped
  when the user questioned their scope. No full workspace-test pass is claimed.
  The diagnostic log remains `/tmp/sm-decomp-git-setup-check.log`.
- GitHub Actions was temporarily disabled for the initial 121-branch import to
  avoid starting historical CI runs. Its original enabled state was restored
  during the subsequent environment cleanup; future pushes and PRs retain the
  existing workflow.
- Cleanup: all processes in the verification process group stopped. No new
  temporary test directories remained. A Cargo cleanup invocation began rebuilding
  the command and was stopped; the existing `target/debug/sm-pipeline` utility
  successfully ran `cleanup --root .scratch/git-setup-20261016 --apply` and
  `cleanup --root .scratch/git-setup-20261016 --deduplicate --apply`. Both reported
  zero bytes and no targets; deduplication checked zero snapshots. The reports
  remain `/tmp/sm-decomp-git-setup-cleanup.toml` and
  `/tmp/sm-decomp-git-setup-deduplicate.toml`. Free disk: 85,129,437,184 bytes.
  Shared Cargo caches, existing worktrees and historical evidence dependencies
  remain retained; this task created no worktree or gameplay evidence run.

## Environment cleanup at the user's request

The user subsequently requested finishing the push and cleaning generated data,
and explicitly selected **Delete old verification runs too**. This authorizes
removal of historical evidence despite the normal retention rule. Old acceptance
records that reference deleted runs are historical records, not currently
authenticatable evidence.

- All 121 branch heads matched GitHub, including `main` and
  `restart/playable-baseline`; GitHub Actions is enabled again.
- Removed disposable Cargo incremental/build caches in main, restart and the
  mutation-headroom worktree, the pipeline build cache and the restart progress
  reporting build cache. Kept runnable executables and source inputs.
- Removed old main `out/port`, `out/sync`, `out/runs`, `out/decomp`,
  `out/recovery-session`, `out/recovery`, `out/feedback-development` and
  `out/iteration-sessions` trees using explicit `rm -r` paths.
- Removed 17 old performance/architecture proof and mutation-result directories
  after preserving source/configuration/documentation files in
  `/home/jim/sm-decomp-retained-sources-20261016.tar.gz` (426 MiB).
  `gzip -t` passed; archive SHA-256:
  `20f1f09db9ed12789f9b9b741cfc1cc7873ed11523e9dd0d8a2b100681301049`.
  Exact removed roots are recorded in `/tmp/sm-environment-old-proof-roots.txt`.
- Removed six clean, pushed mutation-test worktrees with `git worktree remove`:
  current, followups, pipeline-current, pipeline-fast, pipeline-nextest and
  pipeline-v2. Pruned six registrations whose `/tmp` directories already no
  longer existed. Kept all branches and the three mutation worktrees containing
  uncommitted source changes, plus restart and parked architecture/original work.
- No producers were active before cleanup; all deletion processes completed.
  The conservative main-output cleanup preview was stopped once the user
  authorized deletion of those old runs. Both project utility cleanup passes
  succeeded for retained restart output, reporting zero eligible bytes and zero
  snapshots. No new gameplay verification run was started.
- After all deletion, the 897-entry latest Ripper retained-file manifest and
  composition input hashes passed from the restart checkout. The ROM, restart
  runnable binaries, restart evidence, production configuration, downloaded
  dependencies and remaining source snapshots are retained.
- Filesystem free space increased from 85,129,437,184 to 358,439,387,136 bytes:
  **273,309,949,952 bytes (254.5 GiB) reclaimed; 333.8 GiB available**.
  These are host filesystem observations; the retained archive is included in
  the final usage. Main and restart source checkouts remain clean.
