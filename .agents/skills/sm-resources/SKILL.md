---
name: sm-resources
description: Keep sm-decomp's machine and CI capacity usable. Covers disk space, worktrees and GitHub runner slots. Use it as root before starting many lanes, or when disk runs low.
---

# Manage resources

Replays run on GitHub Actions. A lane needs a worktree and compile-check output,
not replay CPU.

## Disk

- Check free disk with `df -h .` before starting many lanes. Keep about 30 GB free.
- Delete `target/` only for a lane that has finished and whose commits are on
  `main` or the landing branch, or abandoned.
- Remove finished worktrees with `git worktree remove --force <path>`, then
  `git worktree prune`.
- Never judge liveness by file age, wipe a shared scratch folder, or kill a process
  you didn't start.

## Runners

A GitHub Free account runs 20 jobs at once across all its repositories. One
verify run takes up to 18 and CI takes three. Land one slice at a time, and let a
head finish its gate before pushing another unless you are fixing its failure: a new
commit cancels the running gate. Nightly runs stop on a commit that already passed.
