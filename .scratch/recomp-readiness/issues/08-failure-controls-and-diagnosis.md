# 08: Diagnose failures with predecessor and disabled-entry controls

**What to build:** A validation failure automatically produces a focused first-mismatch report and identical-interval controls that guide the next experiment.

**Blocked by:** 03: Gate changed-entry coverage and report compiled progress; 07: Restore paired checkpoints validated against cold execution.

**Status:** ready-for-agent

- [ ] Find a covered interval for changed keys, use validated paired replay, and retain bounded first-observable-disagreement evidence linked from the failed run.
- [ ] Replay a preserved selected/accepted predecessor and a candidate with newly enabled/changed compiled keys disabled over identical inputs and intervals; preserve each control's identity and result.
- [ ] Passing controls support increment attribution without asserting a specific root cause; failing controls route investigation toward shared paths, prior assumptions, input or cache/harness validity.
- [ ] Missing changed-key execution yields uncovered and a concrete covered-interval or deferral recommendation.
- [ ] Original-core synchronization failure routes to independent observations/hardware probes; shared-backend passes alone cannot resolve it.
- [ ] Record hypotheses, predicted discriminators, experiments and results; after three successive experiments without new discriminating evidence, preserve candidate/predecessor evidence and produce concrete choices without acceptance.

