---
name: integrator
description: The sm-decomp integrator role. Wire an imported clean model into the engine through an adapter, prove it against the records through indexed production replay, and delete the old code it replaces. Load after the lane skill when your brief names you the integrator.
---

# Integrate a model

The model arrives on an import branch, built from the spec alone. You make the engine run it and
prove it matches the real game. This is step 9 of
[the round](../../../docs/cleanroom/PROCESS.md#a-round). The clean crates are the implementer's,
and you never edit one, not even a mechanical fix that another crate forced: report it to the
root, who makes mechanical fixes after the import and sends the rest to an implementer. When the
model needs a change, report it as a finding for the next round.

## Steps

1. **Read** the spec, decode notes and model interface. Map every input to a production owner
   or an independently observed scoped input, and every output to its consumer or declared limit.
2. **Wire production.** Add the actual clean entry to `game::production`. The engine holds the
   complete clean State; adapters supply traits and publish complete outputs. Validate before
   committing State and outputs together. Missing required behaviour returns typed Unsupported.
3. **Index witnesses.** Add short cases in `scenarios/replay/index.toml` and independent observation
   providers under `crates/verify/src/replay`. The common runner advances production from one seed;
   expected observations never become inputs or later seeds. State the rules, comparison fields,
   input provenance and unproved dependencies. Integrated claims require actual selected producers.
4. **Prove them.** Run each required case through its declared end, including the null control and
   available complete checkpoint continuations. Use role-authorised fault tooling to demonstrate
   the exact first failing cut and field. Play every reviewed played run in one call,
   `bash tools/lane/played.sh` (or name the runs: `<row>` or `<row>/<start>-<end>`): one build,
   one load, each run's held cuts and stop checked against `scenarios/replay/played.toml`, and a
   nonzero exit naming any that differs. When your work moves a run, change its `held` and `stop`
   there in your own commit (Jim, 2026-11-25): full CI holds every run to the file, and the root
   reviews the figures at landing. Record measured results for the reviewed baseline;
   approval of changed requirements is separate from generating them.
5. **Remove replaced execution.** Delete redundant owners, State copies, fallbacks and tests in
   the same change. Retain independently useful content, codecs and exact encoding by responsibility.
6. **Register and validate.** Update the report's model/provider mapping, run the source gate and
   complete indexed replay baseline as [lane](../lane/SKILL.md#prove-it) specifies. Run crate
   tests with `tools/lane/run_tests.sh`. When `played.toml` moves, regenerate the README in the same commit
   (`cargo run -p tools --bin tools -- report --write`); Quality fails a README the tree doesn't
   generate. At a round's landing the root regenerates it again after its own `domains.toml`
   change. Done when Quality and every required case hold.
7. **Report.** Give the production interface, per-case outcomes and compared observations, null
   evidence, continuations, first divergences, fault proof, explicit coverage losses and missing
   dependencies, removed owners and exact head. The root assesses size after independent review.
