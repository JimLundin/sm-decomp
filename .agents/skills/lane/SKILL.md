---
name: lane
description: Start here for any sm-decomp task. Covers the worktree, the observer and implementer roles, commit rules, how to prove a change, guardrails, the report and handoff, and (for the root) picking and landing.
---

# Work a lane

You were given one bounded task, a branch and a base by the root. You commit on your branch and
push only it; the root lands it. Read [AGENTS.md](../../../AGENTS.md) and
[MODEL §5](../../../docs/architecture/MODEL.md#5-how-the-work-is-planned) first.

## Set up

1. `git fetch -q origin && git worktree add -b <branch> <path> <base>`, with the base your brief names.
2. Done when `git log -1` shows your base and `git status` is clean.

## Your role

Every engine task is one of two roles, always separate sessions, never one agent switching hats
([MODEL §5](../../../docs/architecture/MODEL.md#5-how-the-work-is-planned) gives the order of a
domain). Load the skill your brief names after this one:

- [observer](../observer/SKILL.md): reads the original; writes the spec, the vectors and the decoder.
- [implementer](../implementer/SKILL.md): builds the model from the spec alone, never the original.

Other tasks (oracle, tools, CI, docs) need only this skill. Work by mechanism, not by divergence:
lockstep and the localiser say *which behaviour* is wrong, not what code to write.

## Done means (a domain)

- Every movement kind or state is a row of data, every parameter a table entry or a `Rules` value,
  and every behaviour has its catalogue entry (`Behaviour { id, summary, kind, switch, checked }`).
- No bank address in the domain's clean model: sources live in the observer's decode notes only.
  Nothing in the clean model names a video type (`VideoMemory`, `NmiBatch`, `VramWord`,
  `DmaChannel`, OAM bytes, B-bus registers); only `game::exact` encodes bytes. Module docs describe the domain.
- Byte-identical: the free-run gate's green pairs hold, the localiser's first differing frames don't
  move earlier, and the lockstep ratchet holds on the spec's declared fields.
- The domain's vector tests (`scenarios/vectors/<domain>.toml`) pass, its size (`tools size`) fits
  the budget in its spec header, and its RAM-shape list (State fields decoded as one WRAM word copied
  as-is) is short, each with a one-line reason in the spec.
- A switchable quirk has a unit test showing where its variants differ, and the Reference profile
  stays exact. Switches are `Fixed` unless a named use needs a `Toggle`.

## Merge-review questions

The reviewer stops a change on any "no":
- Does the code follow the model, not the original's routines? A new free function named after a
  routine is a stop.
- Does every ROM instruction list run through the interpreter? A list hand-coded as an enum of
  positions is a stop.
- Does every literal and branch in the clean model trace to a `BehaviourId`, with values from `Rules`
  or `data`? Is there no condition on a frame number, position or room no spec entry names?
- Which spec entry reads each new State field ("Read by")? Was no field added because the decoder
  has one?
- Does the catalogue hold the spec's entries one for one, as the clean crate's vector test checks?
- Could someone who never saw the original follow the module from its names and types? Does each
  comment say something the code doesn't? Is the domain within its size budget?

## Commit rules

- One coherent change per commit. Its title says what is true afterwards and ends with its kind:
  `[exact]` (behaviour unchanged) or `(behaviour: <what>)`.
- Idiomatic Rust per AGENTS.md. Fix clippy findings; add no `allow`s.
- A difference from the original is fixed, or listed in `docs/LEGACY.md` in the same commit.
- UTF-8 and LF. Data goes in `.toml` or `.rs`, never JSON.
- ROM-derived bytes (inputs, seeds, records, dumps) stay in encrypted ci-data bundles. Logs and
  commits carry ids, frames, field names and hashes only. Copy hashes from `git log` output.

## Prove it

- Every commit: `bash tools/lane/gate.sh` until it prints `GATE OK <hash> <branch>`. It runs
  `cargo fmt --check`, clippy on every crate with `-D warnings`, `tools/check_crate_rules.sh` and
  `tools/check_doc_links.sh` (every relative Markdown link and anchor), as Quality does.
- Tests: `bash tools/lane/run_tests.sh <crate>... [-- <libtest args>]`. On Windows, Defender blocks
  fresh test executables (exit 126), so it links each test target as a DLL and runs it through
  Python (`tools/lane/test_dll.py`; overrides `SM_RUST_BIN`, `SM_PY`, `SM_TEST_DLL_DIR`). This is
  for local machines only, never CI; elsewhere it is `cargo test`. Doc tests run only in CI.
- Behaviour: the lockstep ratchet, the free-run gate and the localiser, as
  [STATUS](../../../docs/STATUS.md) shows. CI runs the ratchet and the gate on every pull request.
- A new check must catch a deliberate one-line bug: apply the mutant locally, confirm the check
  fails at the right frame and field, revert. Never commit the mutant.

**Triage a difference:** localise (the localiser's first frame and field group, then lockstep's
field) → observer: find the writer in the bank log and amend the spec → implementer: fix the model
→ or a LEGACY entry naming the cause, the rows and the frames. Green never falls.

## Guardrails and traps

- Never push to `main` or `land/*`, never run `git stash`, never change the shared git config.
- Kill only processes you started, by PID. Delete only your own build outputs.
- The root alone uploads to ci-data and posts to issues.
- Python on Windows writes CRLF unless you pass `newline="\n"`; some tools write Latin-1. Check
  new text files with `file`.
- `gh` can hang: wrap it in `timeout 600`.
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

- **Evidence:** each check you ran, per row and mode, with its numbers before and after (ratchet
  lines, green pairs, first differing frames), and any mutant result.
- **Needs from the root:** a bundle path and SHA-256 to upload, or a decision with its options.
- **Found:** bugs, spec gaps, stale data, anything a teammate's work hit. Keep what you couldn't run
  under "Not run", with the reason. Every number comes from output you saw.

When done, or when your context is long, write a handoff of at most 60 lines in
`$TEMP/collab/<lane>/HANDOFF.md`: **State** (what is on your branch, by hash), **Next** (ordered, each
with its first concrete step), **How** (scripts and commands that worked), **Traps** (what cost time,
and what to do instead), **Cleanup** (worktrees and their size, which are safe to delete). Stop your
background jobs and send the path. Say "I couldn't" plainly, with the reason.

## Land (root only)

1. In the landing worktree, `git status` shows no rebase or cherry-pick in progress.
2. `bash tools/lane/pick.sh <hash>...` in the lane's order; it stops at a conflict for you to resolve
   by hand, reading every hunk, then `git cherry-pick --continue`.
3. `bash tools/lane/gate.sh`, then push the landing branch and open a PR to `main` whose body gives
   each lane's commits, evidence and ratchet deltas.
4. In the background: `bash tools/lane/watch.sh <pr>`. Exit 0 landed (via `land.sh`, fast-forward
   only); 3 main moved (rebase, re-gate, re-push); 4 a run failed (read its log, land the green
   prefix, hand the failure to a lane); 5 timed out.
5. Bundles: check the lane's SHA-256 with `sha256sum`, `timeout 600 gh release upload ci-data FILE`,
   confirm its size with `gh release view ci-data`, and commit the new pin. Only encrypted bundles.
