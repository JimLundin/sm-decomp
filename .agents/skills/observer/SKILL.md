---
name: observer
description: The sm-decomp observer role. Read the original (bank logs, ROM, oracle records) and write a domain's behaviour spec, its test vectors and the decoder for the implementer's types, then answer the implementer's gaps. Load after the lane skill when your brief names you the observer.
---

# Observe a domain

You read the original so the implementer never has to. What you write is the only knowledge of the
original the implementer gets: write it once, completely, in domain terms. Your steps are 1, 4 and
5 of [MODEL §5](../../../docs/architecture/MODEL.md#5-how-the-work-is-planned), and the spec follows
its template. This skill adds only how to do them.

**You read:** the [bank logs](https://patrickjohnston.org/bank/), the ROM, the oracle's records,
`verify`'s decoder and address files, and the existing specs. You don't write the clean
model's code.

## 1. The spec and its vectors

- Start from `docs/behaviours/samus-motion.md`'s shape and MODEL §5's template. Put the baseline
  lines and the budget in the header; give every quantity its **Read by**; put ROM codes and RAM
  layout in the decode notes, never in an entry.
- Take every vector's numbers from an oracle record: name the row and frame. Find them with
  `tools lockstep` and `tools localise` ([STATUS](../../../docs/STATUS.md)), or on the oracle with
  `oracle movie --trace FROM TO PC,...` and `oracle corpus --watch-words ADDR,...`
  ([ORACLE §6](../../../docs/ORACLE.md#6-commands)). Add constructed vectors for what no row reaches,
  and each switch's non-vanilla variant.
- Commit the vectors as `scenarios/vectors/<domain>.toml`: before, pad and after, in domain terms.
  Once the implementer's types exist, check that the file reads into them.
- Before handing over, reread the spec as the reviewer will: no routine-named headings, no address
  outside a `Source:` line, no step list that follows a routine. Group the entries by what a player
  sees, not by the routines you read, and use domain words in the rules, not the original's internal
  concepts. For example, write "a pose handed over when an animation ends", not "a super special
  proposal".

## 2. The decoder

After the implementer's State types land, write `verify::decode` for *those* types: an abstraction
from RAM to the State, not a copy. A value you can't decode becomes a new spec entry in domain
terms, or is shown to be derived. Never ask for a field because RAM has a word. Report the
RAM-shape list (fields decoded as one WRAM word copied as-is), each with its reason in the spec.

## 3. Gaps

Read `docs/behaviours/<domain>.gaps.md`. Answer each entry from the original: amend the spec entry
or its vector, and remove the gap in the same commit. A difference the original can't explain is an
oracle or hardware question: a `docs/LEGACY.md` entry.

## Commits

Spec entries are added only in your commits, with the trailer `Role: observer`; the implementer
mirrors them in the clean model's catalogue. Report to the root with the lane skill's format, plus
the spec's entry count, how many are `checked: oracle`, and the gaps you closed.
