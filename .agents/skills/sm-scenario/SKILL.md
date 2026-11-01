---
name: sm-scenario
description: Add an sm-decomp corpus scenario (row) that exercises code nothing covers yet, then lock it and bundle its private data for the root to upload. Use it for coverage, trick and regression rows.
---

# Add a scenario

A scenario is a start (a hardware seed, a scenario seed or boot), an input
stream, and the facts it must show (expectations and required probes). Files in
`scenarios/` hold only metadata. Inputs, seeds and checkpoints live in encrypted
ci-data bundles. Full format: `movies/README.md` ("Add a scenario") and
`docs/SCENARIO_HARNESS.md`.

## Data

Restore what you need into your worktree:
`python tools/ci_seed_data.py restore BUNDLE.tar.gz.enc --sha256 SHA`.
The bundle names and SHAs are in `scenarios/lock.toml` (`bundles`, `pins`).
Download assets with `gh release download ci-data -p NAME`.

## Steps

1. **Probe the target.** Add `crate::probe::hit("module.behaviour")` where the
   uncovered code runs, as an `[exact]` commit.
2. **Find where it runs.** Sweep the 100% route chain rooms with the probe and note
   the rooms that hit it. If none do, author it:
   - Release the controller in the room (recipe `release`). Many enemy and boss
     patterns happen by themselves.
   - Use a scenario seed (`tools/scenario_seeds.py`): a recorded seed with edits
     to Samus's energy, items or position, or to one RAM word such as a boss's
     health. When you edit an enemy's function, also set its instruction list,
     or the oracle crashes.
   - Write an input recipe (`tools/authored_inputs.py`). Let Samus come to rest
     before each move, so both oracles agree on landing frames.
3. **Write the scenario** in `scenarios/corpus.toml`: `id`, `start`, `ticks`
   and `facts`. The facts list each `probe:NAME` and expectation, such as `room:D72A`.
4. **Pin and lock:**
   ```
   python tools/scenario.py lock --pin ID seed=CAPTURE:INDEX inputs=FILE.inputs bundle=ASSET
   python tools/scenario.py lock --record
   python tools/scenario.py lock --check
   ```
5. **Prove it** (`sm-check`): lockstep on both oracles, free run, and check-lock.
   If it fails faithful or free run but passes patched lockstep, give it an
   `unlocked = "..."` note and a `docs/LEGACY.md` row, and tell the root.
6. **Bundle:** `python tools/ci_seed_data.py pack OUT.tar.gz.enc --segment ID ...`.
   Pack while the rows have no `pending` note. Test a restore into an empty
   folder, then send the root the path and SHA-256. Never upload it yourself.

Commit the probes, then the scenarios together with `scenarios/lock.toml`.

## When the code can't be reached

- **Unreachable in retail:** say so beside the ice line in `movies/README.md`,
  with the proof, for example "installed only by the unused `$90:E37F`".
- **Not reached, but not proven unreachable:** no note. Report it as deferred,
  and say why.

## A row that finds a bug

If a new row fails lockstep, the engine is probably wrong. That's the point of
the row. Fix it as `sm-drift` describes, in its own `(behaviour: …)` commit,
before enabling the row.

Done when `lock --check` passes, every new row passes all four modes (or is
listed as unlocked), and the root has the bundle path and SHA-256.
