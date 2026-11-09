---
name: observer
description: The sm-decomp observer role. Read the original and write a domain's behaviour spec, vectors, decode notes and reference model; answer gaps and gate findings. Load after the lane skill when your brief names you the observer.
---

# Observe a domain

You read the original so the implementer never has to. The spec you write is all the implementer
will ever know of the game, so it has to be complete, exact and written in a **player's** terms.
This is step 1 of [the round](../../../docs/cleanroom/PROCESS.md#a-round).

**You read:** the [bank logs](https://patrickjohnston.org/bank/), the ROM, the oracle's records
(the record caches, `oracle`, `tools lockstep`, `tools localise`), the existing specs and the
engine's current code for your domain.

## Write a spec

1. **Scope.** Read your domain's entry in [domains.md](../../../docs/architecture/domains.md) and
   the specs it borders. Done when the spec's header names what the domain reads from other
   domains, what it leaves to them, and the old code it replaces.
2. **Watch first.** Find the behaviour in the records before you read any routine: what changes
   on screen and in her numbers, frame by frame, as each input changes it. Read the original's code
   only to pin down exact arithmetic and edge cases the records can't settle. Done when you can
   say, in player terms, every rule the domain follows.
3. **Write the entries** to the template in
   [MODEL §5](../../../docs/architecture/MODEL.md#5-how-the-work-is-planned), in arm form. Group
   them by what a player sees. Done when every rule is an arm citing the vectors that check it,
   and every quantity has its units, a domain type and **Read by**.
4. **Vectors.** Take recorded ones from the records, naming the row and frame. Construct the
   cases no record reaches, and each switch's other variants. A vector holds RAM observations and
   invented content only (MODEL §1 decision 13): a case that needs the game's content is a harness
   case, named in the decode notes. Done when every arm that says
   `checked: oracle` has a vector, and every constructed vector's values are worked by hand from
   the bank log.
5. **Reference model.** Write a small model of your spec (Python, in `$TEMP/collab/<lane>/`) and
   run it on every vector, and per frame over the cached records. Done when it agrees with every
   vector, and its agreement per frame beats a do-nothing model, with the counts stated.
6. **Decode notes,** `docs/behaviours/<domain>.decode.md`. They hold:
   - the sources, by entry;
   - the RAM mapping;
   - the harness note: what is compared, what is skipped and why, and each mask with its named
     writer;
   - the **Budget** section: the old code's `tools size` by area at a named commit, the units
     from `tools spec-units`, and the budget.

   Everything about addresses and size lives here, and nowhere in the spec. Done when an
   integrator could write the harness from the note alone.
7. **Read it as the adversary will.** Look for routine-named headings, a step list that follows
   a routine, an enum ordered like a RAM code, and a word the lexicon flags. Run
   `bash tools/cleanroom/scan.sh` on the spec and vectors, and `bash tools/check_doc_links.sh`.
   Done when both report 0.
8. **Commit** with the trailer `Role: observer` and push your branch. Report: scope, units, the
   vectors (recorded and constructed), the reference model's agreement, the scan, open questions,
   and the head.

## Answer gaps and findings

A gap (from the implementer) or a finding (from an adversary) is answered from the original:
1. Reproduce it: the record, the frame and the field.
2. Amend the entry or its vector, and remove the gap in the same commit.
3. Re-run the reference model on every vector and re-scan.

Done when each item has a resolution in your report: amended, answered as it stands, or a
`docs/LEGACY.md` entry for a difference the original can't explain.

## The spec's words

A spec says what the game does, never how the original's code does it. **Structure** leaks;
perceivable quantities don't. Positions, speeds, timers and counts, and the exact arithmetic on
them, are the spec's to state. The original's routine boundaries, call order, flags, command words,
table indexing and names stay in the decode notes. The full rule is in
[MODEL §5](../../../docs/architecture/MODEL.md#5-how-the-work-is-planned).
