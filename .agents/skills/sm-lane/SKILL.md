---
name: sm-lane
description: Start here for any sm-decomp lane task. It covers the worktree, the commit rules, the ratchet and what you send the root. Load it first, then the task skill your brief names.
---

# Work a lane

You are a lane agent. The root coordinator gave you one bounded task, a branch
name and a base. You commit on your branch. The root picks your commits onto
`land/batch13` and lands them on `main`.

## Set up

1. From `C:\Users\Jim.Lundin\Documents\Projects\sm-decomp`:
   `git fetch -q origin && git worktree add -b <branch> <path> origin/land/batch13`
   with `<path>` = `C:\Users\Jim.Lundin\AppData\Local\sm-decomp-experiments\<yyyymmdd>\<lane>`.
2. In the worktree: `git config --worktree core.autocrlf false`.
3. Read `AGENTS.md` (Rust style) and the task skill your brief names.

Done when `git log -1` shows your base and `git status` is clean.

## Commit rules

- One coherent change per commit. Its title says what is true afterwards and
  ends with its kind: `[exact]` (behaviour unchanged) or `(behaviour: <what>)`.
- Idiomatic Rust with official names. Fix clippy findings; add no lint `allow`s.
- `sm/src` and the real game are the authority for behaviour. A difference from
  hardware is fixed, or listed in `docs/LEGACY.md`.
- LF line endings. Data goes in `.toml`, `.py` or `.rs`, never JSON.
- ROM-derived bytes (inputs, seeds, checkpoints, dumps) stay in encrypted ci-data
  bundles. Logs and commits carry ids, ticks, field names and hashes only.
- Copy hashes from `git log` output.

## The ratchet

`python tools/ram_ratchet.py` from the worktree root. `sm_game_ram_sites` and
`sm_game_coupled_files` must stay at or below the baseline in the file.
Production lines are advisory; add none without a reason.

## Check every commit

Run `bash tools/lane/gate.sh` until it prints `GATE OK <hash> <branch>`.
Then run the replays your change needs: see the `sm-check` skill.

## Report

Send the root (SendMessage `to: "main"`) one message per batch of about 5
commits. Use the `sm-handoff` skill for the format. When the task is done, write
the handoff and stop your background jobs.

## Guardrails

- Never push to `main` or `land/*`, never run `git stash`, never change the
  shared git config.
- Kill only processes you started, by PID. Delete only your own build outputs.
- The root alone uploads bundles to ci-data and posts to GitHub issues.

## Traps

- "stale native utilities": run `python tools/install_native_config.py`, then rebuild.
- Rebuild the DLL after any source change, revert or rebase; its source stamp
  must match. Don't edit `tools/verify_dll.py` while a long run uses it.
- Test executables can't run on this machine (exit 126). Detached-host tests run in CI only.
- The embedded Python doesn't put the script's folder on `sys.path`; use `-I`
  only for scripts that don't import siblings.
- Bash heredocs collapse backslashes; write such scripts with the Write tool.
  Python on Windows writes CRLF unless you pass `newline="\n"`.
- `gh` can time out here; wrap it in `timeout 600`.
- Other lanes fetch in the shared repo, so `origin/land/batch13` moves while you
  work. Compare against explicit hashes.
