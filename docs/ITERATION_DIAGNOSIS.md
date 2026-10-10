# Diagnosis across candidate retries

`just iteration-diagnosis --session-id SESSION --increment-id INCREMENT
--candidate FAILED_RUN --hypothesis TEXT --discriminator TEXT --result TEXT`
allocates a new exact-source experiment run. The stable unresolved increment ID
carries its accepted predecessor, candidate source snapshots and ordered history
through every retry. It never modifies a terminal candidate. `--history` verifies
and returns its full ordered state for resume. Iteration validation automatically
appends failed outcomes when its increment metadata supplies both IDs.

History references are numbered and hash linked under one per-increment lock.
Each referenced experiment retains its own source snapshot, source delta against
the failed candidate, result, controls and evidence. Every read reverifies the
full chain and sealed artifacts. A repair may change current source; the new
experiment captures that source independently while retaining the failed source.
A different candidate run name, file name, elapsed time or restart cannot reset
the consecutive-without-evidence count.

Use `--evidence JSON_FILE` to link substantive observations. Each descriptor names
an immutable run, report/hash, actual process/hash and executable/hash, plus its
kind: `first-mismatch`, `controls`, `coverage` or `independent-hardware`.
Verification binds actual process completion to the preserved executable, source
and logs. Novelty derives from instruction/state/bus facts, matched control
outcomes, changed-key execution or independent hardware observations; lifecycle
metadata does not contribute. Incomplete observations cannot manufacture new
discriminating evidence. New verified observations reset the streak; the third
consecutive experiment without new evidence produces `diagnostics/decision.md`
with preserved run IDs, earliest disagreement, controls and concrete choices.

Passing predecessor and disabled-entry controls support attribution to the
increment without asserting a specific cause. A failing or incomplete control
routes to shared hardware/input/cache/harness assumptions. Original recording-core
synchronization requires independent hardware/input observations even if shared
backend controls pass. Unexecuted changed keys route to covered intervals or actual
table pruning. Build/setup/panic/budget failures preserve logs and remain error or
incomplete. Existing first-observable diagnostics retain ordered trace/snapshot
evidence and rebuild the accepted predecessor independently.

The result includes machine-readable `next_action`, the same session/increment IDs
and a resume command. Routine semantic repairs belong to the implementing agent:
form a hypothesis, make its predicted discriminator concrete, reproduce through
the affected instruction/bus boundary, make a bounded repair, and invoke fresh
candidate validation with the same IDs. `prune_uncovered` safely handles only
newly added unexecuted keys; affected existing bodies cannot be waived. Broad
shared-hardware repairs require refreshed readiness and separate agreed scope.
Zero-gain correctness repair acceptance requires the verified regression/control
contract in [supported increments](SUPPORTED_INCREMENT.md).

Missing commercial input, an out-of-scope design choice, or the three-experiment
threshold requires a concrete decision. Ordinary repair-needed results remain
agent-owned work. Failed retries never count toward requested accepted increments.
Tests use immutable temporary candidate runs and actual subprocess observations;
real transient-effect execution is retained by existing feedback demonstrations.

Completed candidate control/focus mismatches are discovered automatically and
bound to their actual focused subprocess before entering history. Additional
independent observations can be supplied explicitly. Existing readiness version 1
in-run experiments remain historical evidence; the iteration version 2 lineage
owns continuity across candidates and never trusts their caller-supplied novelty
boolean.

Failure attribution requires the candidate itself to reproduce a complete
instruction/state/bus mismatch, with predecessor and disabled-entry controls
matching its input identities and interval. A passing early window routes to
`failure not reproduced`, with a concrete cold failed-frame operation; it cannot
reset the no-evidence streak or attribute a later parity failure.

Automatic cold parity diagnosis starts at the retained failed input frame,
without skipped instructions or paired-cache validation. The focused command
`--until-frame-end` observes up to one rollover with a 2,000,000-instruction
budget, preserving the first mismatch and bounded history. A budget exhaustion
is incomplete. After reproduction, controls compare through that actual failing
instruction. Passing the complete failed frame requests further independent
hardware/video observation or an earlier transient search; it does not establish
a root cause or claim globally earliest instruction evidence since power-on.
