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
  avoid starting historical CI runs. Restore its original enabled state after
  the final setup-record push; future pushes and PRs retain the existing workflow.
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
