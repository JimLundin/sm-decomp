# 08: Diagnose failures with predecessor and disabled-entry controls

**What to build:** A validation failure automatically produces a focused first-mismatch report and identical-interval controls that guide the next experiment.

**Blocked by:** 03: Gate changed-entry coverage and report compiled progress; 07: Restore paired checkpoints validated against cold execution.

**Status:** done

- [x] Find a covered interval for changed keys, use validated paired replay, and retain bounded first-observable-disagreement evidence linked from the failed run.
- [x] Replay a preserved selected/accepted predecessor and a candidate with newly enabled/changed compiled keys disabled over identical inputs and intervals; preserve each control's identity and result.
- [x] Passing controls support increment attribution without asserting a specific root cause; failing controls route investigation toward shared paths, prior assumptions, input or cache/harness validity.
- [x] Missing changed-key execution yields uncovered and a concrete covered-interval or deferral recommendation.
- [x] Original-core synchronization failure routes to independent observations/hardware probes; shared-backend passes alone cannot resolve it.
- [x] Record hypotheses, predicted discriminators, experiments and results; after three successive experiments without new discriminating evidence, preserve candidate/predecessor evidence and produce concrete choices without acceptance.



## Completion evidence

Failure controls now prepare candidates without claiming a shortened parity run,
discover actual changed-key execution boundaries, retain up to 64 preceding
instructions, and run candidate/disabled/predecessor scopes using separate
validated paired caches. Predecessor source and hardware are rebuilt in an
isolated checkout from preserved source and pinned material. Controls carry
binary/input/key identities and use identical intervals. Passing controls
support increment attribution; failing/incomplete controls retain shared-path
uncertainty. Synchronization routes independent probes and missing coverage
recommends covered scope or removal/deferral without acceptance.

Four public routing/preparation/experiment evidence tests pass; two actual
Subject/comparator tests prove discovery and incorrect generated execution with
passing disabled/predecessor controls; related coverage/diagnostic regressions,
Python checks and affected all-target Clippy pass. A fresh generated diagnostic
with eight instructions passed in run `20261010T090845.873990Z-fc7a2da822ea`
without recording a one-frame parity pass. The full integrated cold domain and
feedback demonstrations remain required by tickets 11 and 12.
