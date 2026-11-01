---
name: sm-coordinate
description: Run sm-decomp as the root coordinator. Choose lane tasks, brief fresh agents, answer their questions, keep Jim informed, and keep main moving. Use it when you are the root, not a lane.
---

# Coordinate

You are the root. You own the one landing branch (`land/batch13`), every push to
`main`, every ci-data upload, and every post to Jim's issue #6. Lanes do the work.

## The loop

1. **Keep lanes busy.** Run about 6–10 lanes at once, each on a different set of
   files. Each lane has one bounded task (see Task kinds).
2. **Pick what's ready.** When a lane reports a qualified batch, pick it onto
   `land/batch13`. See `sm-land`.
3. **Land in slices.** Open a slice PR of at most 25 commits; it lands when all
   gates are green.
4. **Answer decisions fast.** A lane waiting on you is idle. Decide using the rules
   below; ask Jim only about goals and process.
5. **Tell Jim.** After each landing or finding, report in a few short lines:
   what landed, what it means, what's next.

## Task kinds

| Goal | Brief the lane with skill |
| --- | --- |
| cut sm_game RAM coupling | `sm-decouple` |
| a faithful-only divergence | `sm-hardware` |
| drift, a lock mismatch or a failing row | `sm-drift` |
| a coverage gap or trick row | `sm-scenario` |
| verifier, harness or CI work | `sm-check`, and `docs/SCENARIO_HARNESS.md` |

## Brief a fresh agent

Use a fresh general-purpose agent per task; don't fork. Retire an agent when its
task ends or its context grows long, after it writes a handoff (`sm-handoff`).
The brief says:

- "Load `sm-lane`, then `<task skill>`." Include any handoff path to read.
- The goal in one sentence, and how it's measured.
- The branch name and base (`origin/land/batch13`).
- The files or families it owns, and which it must not touch.
- The evidence you want, and the report cadence (batches of about 5 commits).

## Decision rules

- **The real game is the authority.** A difference is fixed, or listed in
  `docs/LEGACY.md`. There are no tolerances.
- **Count honestly.** A measure counts real coupling: no renames passed off as
  decoupling, and no counting typed access as RAM.
- **Prefer the smaller, verifiable step.** Land partial progress, and re-cut stale data
  rather than marking required rows pending.
- **Risky CI wiring goes last in the queue**, so it can't block the rest.
- **Lower the ratchet baseline after each landing that cut it** (`ram_ratchet.py --update`).
- **Lane output that adds lines** without removing any is fine as a step only if
  the next step removes the old path. Say so in the reply.

## Records

- `docs/REVIEW.md` and `docs/PLAN.md` record Jim's decisions. You may edit them
  to do that.
- Post a short result report to #6 when a phase exit or a verification finding
  changes the plan. Ask Jim before any other post.
- The briefs' shared rules live in `sm-lane`; keep them there, not in briefs.
