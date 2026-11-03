---
name: sm-check
description: Choose and run the checks that prove an sm-decomp change. Covers the gate, lockstep and free-run replays, check-lock, chain rooms and mutants. Use it before reporting any commit.
---

# Prove a change

Every commit passes `bash tools/lane/gate.sh`. Then prove the behaviour with replays
on GitHub Actions, picked by what you touched. Replays run on Linux, and a lane's
commits need no local verifier build.

## Run replays

Push your branch as `verify/<name>` to run the whole gate on it. To run chosen rows:
`gh workflow run ci.yml --ref <branch> -f scope=rows -f segments="ID ID"`,
adding `-f oracle=faithful` for the unpatched game. `scope=full` adds lockstep of
everything. The commands below also work locally if you have a ROM.

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
- A digest differing only on one operating system is a platform dependency in the
  engine or oracle. Fix it; never re-record over it.

The gate runs every locked row in check-lock and lockstep for the rows the lock can't
hold. The three long checkpoint rows are being reseeded from room seeds through
the existing setup; until those genuine starts and inputs are qualified, their
current cold-prefix versions remain in nightly full runs or `scope=rows`.

Done when every row you listed passed and you have the counts for your report.
