---
name: sm-check
description: Choose and run the checks that prove an sm-decomp change. Covers the gate, lockstep and free-run replays, check-lock, chain rooms and mutants. Use it before reporting any commit.
---

# Prove a change

Every commit passes `bash tools/lane/gate.sh`. Then prove the behaviour with replays,
picked by what you touched.

## The modes

| Mode | What it proves | Command (worktree root) |
| --- | --- | --- |
| lockstep | engine equals the oracle every frame | `python tools/verify_dll.py --direct --corpus scenarios --segment ID` |
| faithful | ...against the unpatched game | add `--oracle faithful` |
| check-lock | the engine alone matches the committed oracle digests | add `--mode check-lock --lock scenarios/lock.toml` |
| free run | the engine alone against a live oracle; shows drift | add `--mode free-run` |

- Run several rows by repeating `--segment`. A row passes with `0 failing frames`
  or `lock matches`.
- Chain rooms (the 100% route): `FULL_ROUTE=100 python .github/scripts/full_route.py run JOB`
  takes the same `--mode` flags. Restore the route bundle first (`sm-scenario`, Data).
- `--divergence-report PATH` writes the first divergence (tick, domain, field).

## What to run

| You changed | Run |
| --- | --- |
| an `[exact]` move or rename | lockstep on 3–10 rows that execute it, plus chain rooms 1, 6, 100, 200, 320 |
| a `(behaviour: …)` fix | lockstep, patched and faithful, on the rows that hit it; check-lock on the locked ones |
| a typed copy or lease move | lockstep and check-lock on every locked row that executes it |
| a corpus row | lockstep (both oracles), free run, then record and check the lock |
| verifier or CI code | the rows or checks it touches; CI runs the rest |

To find which rows execute a function: add a `crate::probe::hit("module.name")`
call, then sweep. `sm-scenario` explains the sweep.

## Mutants

A new check must catch a deliberate one-line bug. Apply the mutant locally, run
the check, confirm it fails at the right tick and field, then revert. Report the
result. Never commit the mutant.

## Reading a failure

- "lock matches" missing: the engine drifted. Free run names the first tick and
  domain; lockstep from there names the field. Then use `sm-drift`.
- Expectation "not observed" with 0 failing frames: the inputs no longer reach
  the event. Re-time the row.
- "checkpoint ... differs": a stale checkpoint. The row regenerates its prefix.

Done when every row you listed passed and you have the counts for your report.
