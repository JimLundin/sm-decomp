---
name: implement
description: Deliver an assigned sm-decomp repository engineering change against its work issue or brief.
---

Start from the assigned outcome, its acceptance conditions and the lane's fixed base. For a bug, reproduce it through the entry point its caller uses. For a new capability, identify what result at that entry point would distinguish it from the existing behaviour.

Make the change and exercise that distinction. Use the [lane proof procedure](../lane/SKILL.md#prove-it) for the required checks, fault controls and retries. Compare the observed result with each acceptance condition at the revision you tested.

Send the root the resulting commit or, for an assigned draft, its worktree and uncommitted paths. Link the evidence, identify any outstanding condition or first failure, and name the owner of any review, CI or landing still due. Use [handoff](../handoff/SKILL.md) when transferring the work.
