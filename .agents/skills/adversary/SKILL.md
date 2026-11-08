---
name: adversary
description: The sm-decomp adversary role. Gate a spec before a clean-room round, or a round before it lands; try to prove a leak, a wrong rule or a dishonest check. Load after the lane skill when your brief names you the adversary.
---

# Gate the clean room

You read both the original and everything the implementer will see, and you try to **break**
the work: prove a leak, a rule that differs from the game, a vector that is wrong, or a check that
hides a failure. You succeed by finding one. You report findings; their owners fix them. These
are steps 3 and 10 of [the round](../../../docs/cleanroom/PROCESS.md#a-round).

Write your review to `$TEMP/collab/reviews/adversary-<n>.md`. Every finding is numbered and gives
the evidence (record and frame, bank-log address, or hand case), a concrete fix, and its owner
(observer, implementer, integrator or root).

## Gate a spec

Your brief names the spec branch, its brief for the implementer, and any earlier gate whose
findings it resolves. For a re-gate, review the delta, and check each earlier finding as resolved.

1. **Rules against the game.** Re-derive a sample of arms and vectors from the records and the
   bank logs, weighted towards what changed and what only constructed vectors check. Done when
   each sampled arm is confirmed or is a finding.
2. **Completeness and boundary.** List what the original does in this domain that the spec
   misses, and every behaviour placed in the wrong domain. Done when each is confirmed covered or
   is a finding.
3. **Leaks.** Run `tools/cleanroom/scan.sh` on the spec and vectors and `scan_brief.sh` on the
   brief. Then read for **structure** the scans can't see: routine boundaries, call order, internal
   flags and modes, enum orders that follow RAM codes, table indexing (the rule is in
   [MODEL §5](../../../docs/architecture/MODEL.md#5-how-the-work-is-planned)). Review every phrase
   the observer added to the scan's allow list. Done when every entry has been read for structure.
4. **Form and units.** Every rule is an arm citing vectors that exist, with no padding. Run the
   official counter (`tools spec-units`) and check the decode notes' Budget section against it.
5. **The brief** asks for what the spec supports and nothing more. It names spec entries and
   shape, and holds no score or measure. Done when each brief item is checked against the spec.
6. **Verdict:** LAUNCH, LAUNCH AFTER FIXES, or REWORK.

## Gate a round

Your brief names the import and integration branches, the blind review, and the spec's head.

1. **Leaks in the code.** Run the scans and the mirror scan on the clean crates, and read for code
   shaped like the original's routines. A function named after a routine, or a list from the ROM
   hand-coded as an enum of positions, is a finding.
2. **Fitting.** Every literal and branch traces to a spec entry, with its values from `Rules` or
   data. A condition on a frame, position or room that no entry names is a finding. Every field of
   state is read by a spec entry, and none exists only because the decoder has one.
3. **The catalogue** holds the spec's entries one for one, and its switches agree with what
   `Rules` lets a caller adjust.
4. **The harness is honest.** Each skip and mask names its writer, and covers only that writer's
   frames. No decode error turns into "not compared". Its floors are committed and its baseline is
   cited. Re-run it, and confirm a one-line mutant in the model makes it fail.
5. **Size.** Measure the model with `tools size` against the Budget section, by area, and say
   where the size goes. Turn every size finding into **shape**: duplication, a decision that
   could be a table row, a function doing two jobs. The implementer only ever sees the shape.
6. **The blind review.** Compare the reviewer's "what the code does" with the spec, and turn each
   misreading into a finding about a name, a doc or a shape.
7. **The old code** the model replaces is deleted, and the engine's gates hold.
8. **Verdict:** MERGE, MERGE AFTER FIXES, or REWORK.
