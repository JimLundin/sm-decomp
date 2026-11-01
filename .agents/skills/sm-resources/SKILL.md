---
name: sm-resources
description: Keep the sm-decomp machine usable. Covers disk space, build outputs, worktrees and runners. Use it as root before starting many lanes, or when disk falls below about 30 GB.
---

# Manage machine resources

The machine has one C: drive of about 475 GB. Each lane's `target/` holds a few
GB, and parallel DLL builds can fill the disk (os error 112).

## Check

- Free disk: `df -h /c`. Keep at least about 30 GB free.
- Build outputs by size:
  `du -sm .../sm-decomp-experiments/*/*/target | sort -rn | head -30`
  (slow: run it in the background).

## Free space safely

Delete the `target/` folder only of a lane that:
1. has finished (its final report arrived), and
2. has its commits on `main` or `land/batch13`, or abandoned.

Remove whole worktrees of finished lanes with
`git worktree remove --force <path>`, then `git worktree prune`.

## Never

- Judge liveness by file age. A long replay leaves old files in a live target.
- Wipe a shared scratch folder (`$TEMP/collab/build`, `$TEMP/sm-gate`) wholesale;
  live gates use it.
- `rm -rf` a folder that holds junctions (for example `laneL3-base`'s seeds).
  Remove junctions with `cmd /c rmdir <junction>`.
- Kill a process you didn't start.

## Runners

GitHub allows about 20 concurrent runners across the account. One slice uses
CI, the corpus (about 36 jobs), the route (about 16) and the lock gate (about 25).
Run one or two slices at a time. Queue coverage runs (`coverage.yml`) between
slices.

## Lanes

Each lane needs one worktree, a few GB of build output, and replay CPU. Run fewer
lanes when replays slow down a lot: an untouched row taking about twice as long
means the machine is saturated.
