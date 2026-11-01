---
name: sm-land
description: As the sm-decomp root, pick lane commits onto land/batch13, open a slice PR, run its gates and land it on main. Also covers uploading lane bundles and lowering the ratchet baseline.
---

# Land work

Main only moves by fast-forward to a slice head on which every gate passed.
The landing worktree is `...\sm-decomp-experiments\20261029\collab-8b`, on
branch `land/batch13`.

## Pick

1. In the landing worktree, `git status` must show no rebase or cherry-pick in
   progress.
2. Run `bash tools/lane/pick.sh <hash>...`, in the lane's order. It stops at a
   conflict. Resolve the conflict by hand, reading every hunk, then
   `git cherry-pick --continue`.
3. If the lane added rows: `python tools/scenario.py lock --check` must pass.
4. Run `bash tools/lane/gate.sh`. On `GATE OK`, `git push origin land/batch13`.

Order the queue so fixes for a red `main` go first and risky CI wiring goes last.

## Open a slice

1. Choose the slice tip: the last commit of the green prefix, at most about 25 commits.
2. `git push origin <tip>:refs/heads/land/batch13<x>` (the next letter).
3. `gh pr create --base main --head land/batch13<x> --title ... --body-file F`.
   The body says what each lane's commits do, with evidence and the ratchet delta,
   and ends with the Claude Code footer.
4. `gh workflow run segments.yml --ref land/batch13<x>`. Retry if GitHub returns 500.
5. In the background: `bash tools/lane/watch.sh <pr>`.

## When the watcher stops

| Exit | Meaning | Do |
| --- | --- | --- |
| 0 | landed | report to Jim; open the next slice |
| 3 | main moved | rebase `land/batch13` onto `origin/main`, re-gate, re-push the slice, re-dispatch the corpus |
| 4 | a gating run failed | read its log (`gh run view --job ID --log`). Drop the culprit commit from the slice (land the green prefix), and hand the failure to a lane |

After any rebase, check that `git status` shows no rebase in progress before you
push. A stopped rebase leaves the old branch ref in place.

## Bundles

A lane sends a bundle path and SHA-256. Check it with `sha256sum`, then run
`timeout 600 gh release upload ci-data FILE` and confirm the asset's size with
`gh release view ci-data`. Encrypted bundles are pre-approved by Jim. Unencrypted
ROM data is never uploaded.

## Ratchet

After a slice that cut `sm_game_ram_sites` lands, run
`python tools/ram_ratchet.py --update` on `land/batch13`. Commit it as
"Ratchet: lower the RAM baseline to N ..." and land it with the next slice.

## Every gate counts

The gates are CI, the corpus, the full route, and "Engine against the oracle
lock". If you add a new gating workflow, add it to `tools/lane/land.sh` in the
same change.
