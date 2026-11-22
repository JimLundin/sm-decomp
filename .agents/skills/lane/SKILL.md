---
name: lane
description: Start here for any sm-decomp task in the repository. Covers the worktree, commit rules, how to prove a change, guardrails, the report and handoff, and (for the root) landing.
---

# Work a lane

You were given one bounded task, a branch and a base by the root. You commit on your branch and
push only it; the root lands it. Follow [AGENTS.md](../../../AGENTS.md#start-by-task)
for startup reading. Engineering tasks do not load the round procedure unless they change it.

## Set up

1. `git fetch -q origin && git worktree add -b <branch> <path> <base>`, with the base your brief names.
2. Done when `git log -1` shows your base and `git status` is clean.

## Your role

[PROCESS](../../../docs/cleanroom/PROCESS.md#roles) names the roles of a clean-room round. Load the
role skill your brief names. For engineering work, follow the task route in AGENTS.

## Commit rules

- One coherent change per commit. Its title starts with the domain or area it changes
  (`samus-movement:`, `verify:`, `process:`), says in at most about 100 characters what is true
  afterwards, and ends with its kind: `[exact]` (behaviour unchanged) or `(behaviour: <what>)`.
  The detail goes in the body.
- In a round, end each commit's message with `Role: <your role>`: a round lands on `main` as one
  commit (`tools/lane/squash.sh`), whose body lists your commits under that role, each with its
  own body indented under it, so explain the change in the body. Your branch keeps them.
- For Rust changes, follow [Rust style](../../../docs/architecture/PRINCIPLES.md#rust-style).
- A difference from the original is fixed, or listed in `docs/LEGACY.md` in the same commit.
- UTF-8 and LF. Data goes in `.toml` or `.rs`, never JSON.
- ROM-derived bytes (inputs, seeds, records, dumps) stay in encrypted ci-data bundles. Logs and
  commits carry ids, frames, field names and hashes only. Copy hashes from `git log` output.

## Prove it

- For every commit, run `bash tools/lane/gate.sh` until it prints `GATE OK <hash> <branch>`.
- Tests: `bash tools/lane/run_tests.sh <crate>... [-- <libtest args>]`. On Windows, Defender blocks
  fresh test executables (exit 126), so it links each test target as a DLL and runs it through
  Python (`tools/lane/test_dll.py`; overrides `SM_RUST_BIN`, `SM_PY`, `SM_TEST_DLL_DIR`). This is
  for local machines only, never CI; elsewhere it is `cargo test`. Doc tests run only in CI. The
  replay tests execute recorded cases, so build them optimised with `CARGO_PROFILE_TEST_OPT_LEVEL=3`.
- Behaviour: run the affected indexed cases, then the complete required replay baseline as
  [STATUS](../../../docs/STATUS.md) shows. Local acceptance and CI use the same runner.
  Compare case outcomes, declared fields/outputs, null evidence and checkpoint continuations;
  an expected Unsupported result is a held boundary, never a passing behaviour case.
- **Local runs are slow; plan around them.** On this Windows machine, one opening `frame_origin`
  test takes 4–10 minutes through the DLL runner, and a multi-return case can take longer. That's
  several times the same test on CI's Linux runner. Iterate locally on the narrowest single test,
  started from the saved checkpoint nearest the question rather than from cut 1. Send full
  cases, the required witnesses and fault checks to focused CI (`gh workflow run ci.yml --ref
  <branch> -f target=<case or test>`), which runs without blocking the machine. Your lane ends
  at the push: hand back the run ids, and the root watches them.
- A new check must catch a deliberate one-line bug: apply the mutant locally, confirm the check
  fails at the right frame and field, revert. Never commit the mutant.
- **Fixing a failure.** Reproduce the exact failure (the same command, row, frame and field)
  before a second fix attempt. Never stack speculative fixes: revert one that didn't fix it
  before trying the next. The regression test asserts the property that was reported, not the
  mechanism of the fix.

For a replay difference, start at the first differing cut and field. The integrator checks the
adapter and replay wiring. The observer resolves a spec gap from the records, and a confined
implementer fixes model behaviour against the approved spec. Record any retained difference
under the commit rules above.

## Guardrails and traps

- Never push to `main` or `land/*`, never run `git stash`, never change the shared git config.
- Kill only processes you started, by PID. Delete only your own build outputs.
- The root alone uploads to ci-data and posts to issues.
- Python on Windows writes CRLF unless you pass `newline="\n"`; some tools write Latin-1. Check
  new text files with `file`.
- `gh` can hang: wrap a short `gh` query in `timeout 120`.
- Don't block on long work. Run builds, `gate.sh`, replays and `gh run watch` in the background
  (the shell tool's background mode): the harness notifies you when the command exits. For "wait
  until X", run one background `until <check>; do sleep 30; done` covering success and failure
  (and a log line, a finished run or an exited PID). Never `sleep` in the foreground or poll in a
  foreground loop. A foreground command at the 10-minute cap is a sign the wait belongs in the
  background.
- Other lanes fetch in the shared clone, so branches move while you work. Compare explicit hashes.
- ORACLE §8 lists the oracle's pitfalls (lsnes, Mesen, DLL runners).

## Resources

- Keep about 30 GB of disk free (`df -h .`). Delete `target/` only for a finished or abandoned
  lane; remove finished worktrees with `git worktree remove --force <path>` then `git worktree prune`.
- Never judge liveness by file age or wipe a shared scratch folder.
- GitHub Free runs 20 jobs at once across all repositories. Push a head once and let its CI finish
  unless you are fixing its failure: a new commit cancels the running gate.

## Report and hand off

Send the root a report per batch of about 5 commits. Lead with the result in one sentence, then:

```
Branch <branch>, based on <base hash>. GATE OK <tip hash>.
<hash> <title>        (one line per commit, oldest first, from git log)
```

- **Evidence:** each check you ran, per case and scope, with its numbers before and after (outcome,
  compared observations, null baseline, first differing cut), and any mutant result.
- **Needs from the root:** a bundle path and SHA-256 to upload, or a decision with its options.
- **Found:** bugs, spec gaps, stale data, anything a teammate's work hit. Keep what you couldn't run
  under "Not run", with the reason. Every number comes from output you saw.

At completion, transfer or a context change, use [handoff](../handoff/SKILL.md).
Link the batch report and its receipts where they already contain the proof.

## Land (root only)

The root follows [Landing](../root/SKILL.md#landing). Lanes hand off their branch and evidence.
