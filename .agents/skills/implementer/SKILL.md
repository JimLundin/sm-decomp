---
name: implementer
description: The sm-decomp implementer role. Build a domain's model in game from its behaviour spec alone, in an empty module, proven by the vector tests and the oracle; never read the original. Load after the lane skill when your brief names you the implementer.
---

# Implement a domain

You build the model from what the observer wrote. You have nothing you could transliterate, and
that is the point: the code is shaped by the domain. Your step is 3 of
[MODEL §5](../../../docs/architecture/MODEL.md#5-how-the-work-is-planned), with its read limits and
its "no fitting" rules. This skill adds only how to work inside them.

## The clean room

Read only what MODEL §5 allows: MODEL.md, your domain's spec and vectors, the localiser's output,
`data`'s public types and the model code; and for process, AGENTS.md and the lane skill. That
excludes ARCHITECTURE, ORACLE and LEGACY too, because they cite bank addresses, and the `docs/behaviours/*.decode.md` files, which map quantities to RAM. Your brief cites
spec entry ids only; if it carries an excerpt or an address, don't read it, and tell the root. If you
open a forbidden file by accident, say so in your report: the clean room is part of the evidence.

## How you work

1. Design the State types from the spec: a quantity whose **Read by** isn't "none" is a field;
   nothing else is. Enums and newtypes, never ROM codes.
2. Write the domain into the empty module `game::model::<domain>`, and delete the old module in the
   same commit without opening it.
3. Run the vector test in `game` on `scenarios/vectors/<domain>.toml` with
   `bash tools/lane/run_tests.sh game` until it passes. Then confirm on the oracle: the lane skill's
   "Prove it".
4. Every literal and branch names its `BehaviourId`; values come from `Rules` or `data`. Model code
   never names an address.
5. Watch the size budget in the spec header as you go; going over is a review stop.

## When something disagrees

Don't tune. Add an entry to `docs/behaviours/<domain>.gaps.md`: the frame, the field, the expected
and actual values, and the entry you suspect. Move on to the next behaviour. When the observer has
amended the spec, implement the amended entry. You never add catalogue entries, and never add a
field because the decoder has one.

## Done

The lane skill's "Done means" and MODEL §4's pilot criteria. Your report adds the gaps you filed,
which came back answered, and a line confirming you read nothing outside the clean room.
