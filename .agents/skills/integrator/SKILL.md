---
name: integrator
description: The sm-decomp integrator role. Wire an imported clean model into the engine through an adapter, prove it against the records with an honest harness, and delete the old code it replaces. Load after the lane skill when your brief names you the integrator.
---

# Integrate a model

The model arrives on an import branch, built from the spec alone. You make the engine run it and
prove it matches the real game. This is step 9 of
[the round](../../../docs/cleanroom/PROCESS.md#a-round). The clean crates are the implementer's,
and you never edit one, not even a mechanical fix that another crate forced: report it to the
root, who makes mechanical fixes after the import and sends the rest to an implementer. When the
model needs a change, report it as a finding for the next round.

## Steps

1. **Read** the spec, its decode notes (the harness note above all) and the model's public
   interface. Done when you can map every input the model takes to where the engine holds it,
   and every output to where the engine uses it.
2. **The adapter,** `crates/game/src/<domain>_world.rs`. It converts between the engine's state
   and the model's types, at the edge and once, and calls the model each frame. Done when the
   engine builds and runs the model in place of the old code.
3. **The harness,** `crates/verify/src/harness/<domain>.rs`. It decodes each recorded frame into the
   model's input, steps the model, and compares its output with the next record. Each frame
   lands in one of three buckets:
   - **compared:** the model ran and its output was checked;
   - **skipped:** another domain owns the frame, a named writer from the harness note;
   - **defect:** the frame couldn't be decoded or compared. It is reported, never hidden.
   Done when every skip and mask names its writer and covers only that writer's frames.
4. **Prove the harness honest.** Report the compared, skipped and defect counts per row, the
   baseline of a do-nothing model, and a one-line mutant in the model that makes it fail at the
   right frame. Commit the floors to `crates/verify/src/harness/floors.txt`. Done when the floor
   test passes on the cached rows, and the mutant fails it.
5. **Delete the old code** the model replaces, with its tests and docs, in the same lane. Done
   when nothing in the engine still calls it, and a search for its names finds nothing.
6. **Register the domain** in the report (`verify::report::DOMAINS`) and regenerate
   `docs/REPORT.md` (`cargo run -p tools --bin tools -- report --write`).
7. **Prove it:** `bash tools/lane/gate.sh`, the tests, the free-run gate's green pairs and the
   lockstep ratchet, as the [lane](../lane/SKILL.md#prove-it) skill gives. Done on
   `GATE OK` with every pair and ratchet at or above its value on `main`.
8. **Report:** the adapter's interface, the harness counts against the baseline, the mutant, what
   you deleted (lines and score), the ratchet deltas, the findings for the model, and the head.
