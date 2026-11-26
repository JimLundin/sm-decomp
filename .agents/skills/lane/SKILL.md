---
name: lane
description: Start here for any sm-decomp task in the repository. Covers the worktree, commit rules, how to prove a change, guardrails, the report and handoff, and (for the root) landing.
---

# Work a lane

You were given one bounded task, a branch and a base by the root. You commit on your branch and
push only it; the root lands it. Follow [AGENTS.md](../../../AGENTS.md#start-by-task)
for startup reading. Engineering tasks do not load the round procedure unless they change it.

## Set up

1. `git fetch -q origin && bash tools/lane/worktree.sh add <branch> <path> <base>`, with the base
   your brief names. It hard-links the build state of the seed, one worktree at `$TEMP/sm-seed`
   built at main, so your first build compiles only what your base changes since the seed, and
   your `target/` costs disk only for what you rebuild. Without a seed it says so and your first
   build is cold. It dates your tracked files 2000-01-01; that is how unchanged ones stay built.
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

- For every commit, run `bash tools/lane/gate.sh` until it prints `GATE OK <hash> <branch>`. It
  runs verify's lib tests (the report, the README and the played runs), the planted faults' plan,
  the rule owners and the clean-room and lane tools' self-tests only when your branch changes their
  subject since `SM_GATE_BASE` (origin/main unless set; a stacked branch sets its base), and
  `cargo deny` where cargo-deny is installed. A failing check names its finding. When a gate on
  this machine already passed the same tree over the same base with the same tools, it prints
  `GATE REUSED` naming that pass and runs nothing; `SM_GATE_FRESH=1` runs it anyway.
- Tests: `bash tools/lane/run_tests.sh <crate>... [-- <libtest args>]`. On Linux (and CI) it is
  `cargo test -p <crate>...`. Windows only: Defender blocks fresh test executables (exit 126), so
  there it links each test target as a DLL and runs it through Python (`tools/lane/test_dll.py`;
  overrides `SM_RUST_BIN`, `SM_PY`, `SM_TEST_DLL_DIR`, `SM_TEST_JOBS`), builds every named
  crate's targets in one cargo run and runs the suites at once, so name all the crates in one
  call; doc tests run only in CI there. The replay tests execute recorded cases, so build them
  optimised with `CARGO_PROFILE_TEST_OPT_LEVEL=3`.
- Behaviour: run the affected indexed cases, then the complete required replay baseline as
  [STATUS](../../../docs/STATUS.md) shows. Local acceptance and CI use the same runner.
  Compare case outcomes, declared fields/outputs, null evidence and checkpoint continuations;
  an expected Unsupported result is a held boundary, never a passing behaviour case.
- **Local runs are slow; plan around them.** On a Windows machine, one fixture-backed
  replay test takes several minutes through the DLL runner, and a long case can take longer. That's
  several times the same test on CI's Linux runner; on Linux a local run is `cargo test` at
  about CI's speed. Iterate locally on the narrowest single test,
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

- The root alone updates `main` and publishes `land/*` branches. The designated
  [cleanup agent](../../../docs/cleanroom/PROCESS.md#cleanup-after-landing) can delete a
  finished branch handed off by the root. Never run `git stash` or change the shared git config.
- Kill only processes you started, by PID. Delete your own build outputs. Releasing a worktree
  in your handoff also authorizes the cleanup agent to remove its build outputs.
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
  lane; remove a finished worktree with `bash tools/lane/worktree.sh remove <path>`, which also
  deletes its `target/` and, on Windows, its test-DLL folder under `$TEMP/sm-test-dll`. On Linux
  set `TEMP` to a folder on the same disk file system as your worktrees
  ([BUILDING](../../../docs/BUILDING.md#setting-up-on-linux)), so the seed's hard links work.
- The seed is the one shared build state, about one lane's size. The root refreshes it at main
  off-hours with `bash tools/lane/worktree.sh seed`, when no lane is being set up and no lane's
  tests are running. Artifacts a refresh replaces stay on disk while a lane still links them, so
  remove finished lanes. Never point two worktrees at one target directory: cargo judges
  freshness by each worktree's source times, so one lane can run what the other built.
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
