# Review of MODEL.md: what to revise before the first implementer lane

Jim asked for a review of the behaviour model (MODEL.md at 09486a2) and the first observer spec
(docs/behaviours/samus-ground.md at 6549bbf). This note lists the failure modes found and the
change for each. It is addressed to the agent that owns MODEL.md. Apply what Jim accepts, then
answer the Acknowledgement table at the end. Delete this note once Jim has seen the answers.
The history stays in git.

**Verdict.** The approach can work. It is the first attempt that targets the cause: code that
copies the original's structure. The samus-ground spec shows the observer role working. It is
organised by behaviour, has one `Source:` per entry, has test vectors and has an order section.
But the clean room leaks in four places (R1 to R4), and R1 alone would pull the State back to the
WRAM layout no matter how disciplined the lanes are.

---

## R1. The acceptance check pins the State to RAM

**Problem.** MODEL §1 says exact means the cuts and the output bytes. But §4's proof for the
reshape is the lockstep ratchet, which compares the State field by field. That State is decoded
from WRAM (`verify/src/decode.rs` fills `PoseId`, `NewPoseFrame`, `DrawingHandler` and the DMA
channels, and `strand.rs` reads return addresses off the stack). Any field the decoder fills
must be stored and must match. So every RAM value the decoder compares becomes a State field,
whether or not a game rule reads it. This is RETROSPECTIVE A3 and V2 coming back.

**It is already in the spec.** samus-ground §1 and §6.8 require the State to carry:
- `total_speed` with its *stale* value on frames that don't step, because "it is RAM-observable";
- X-speed-killed as the codes 4 and 8;
- the distances moved left, right, up and down;
- `mode` with its ROM values 0, 1 and 2.

None of these entries says which later rule reads the value.

**Change.**
1. Every quantity in a spec gets a **Read by** column: the behaviour IDs that read it on a later
   frame, or "output" if it reaches bytes, or "none".
2. "none" means MODEL decision 3 applies. The quantity leaves the State and the decoder, and is
   listed with the uncompared leftovers. Its staleness is then not a behaviour.
3. The compared fields for a domain are exactly the "Read by ≠ none" quantities. The spec lists
   them, and `coverage::FIELDS` is generated from or checked against that list.
4. §4's proof becomes:
   - the free-run gate's green pairs hold;
   - the localiser's first differing frames are unchanged;
   - the lockstep ratchet holds **on the declared fields**.

   The ratchet stays the localiser it was meant to be (RETROSPECTIVE V2 rule).

## R2. Nobody owns the decoder for the new types

**Problem.** MODEL doesn't say who writes `decode` for types the implementer designs. The
implementer can't, because it needs the WRAM map. If the observer writes it unprompted, it will
shape the State to whatever is easy to decode.

**Change.** The order for each domain is:
1. The observer writes the spec, with the "Read by" column.
2. The implementer designs the State types and model from the spec alone, with tests from the
   vectors (R5).
3. The observer writes `decode` for *those* types. Decode is an abstraction function from RAM to
   the State. It is not a copy.
4. If a State value can't be decoded, the observer does one of two things:
   - names the hidden RAM quantity as a new spec entry, in domain terms;
   - or shows that the value is derived, so the model computes it and doesn't store it.

   The implementer never adds a field because the decoder has one.

Add a **RAM-shape report** to the size report: for each domain, the State fields whose decode is
one WRAM word copied as-is. Each needs a one-line reason in the spec. The pilot's success test
is that this list is short and every item on it is justified.

## R3. Clean-room hygiene

1. **§5 contradicts itself.** It says the implementer gets "never the bank logs or the asm". Then
   it says a brief "includes the relevant bank-log excerpt". Drop the excerpt. A brief cites spec
   entry IDs only.
2. **The reshape edits code shaped like the asm.** `movement.rs` opens by narrating `$90:A337`,
   `$90:8E64` and `$90:973E`. Across `crates/game/src`, 60 files and 881 lines cite bank
   addresses. Editing them is transliteration with extra steps. Instead:
   - the implementer writes each domain into an **empty module** (`game::model::samus::motion`);
   - the old module is deleted in the same commit;
   - the brief tells the implementer not to open the old module.
3. **Source lines live in the catalogue only.** Model code names a `BehaviourId`, never an
   address. `tools/check_crate_rules.sh` fails on `$XX:XXXX` or `$XXXX` anywhere in `game` except
   `behaviours.rs`. It starts as a report, since 881 lines exist today, and becomes an error
   domain by domain as each one is reshaped.
4. **AGENTS.md and `.agents/skills` describe the deleted engine.** AGENTS.md still says
   "`sm/src` and the real game are the authority… use `ram_map.rs` and the bank logs". It still
   names `sm_game`, and the skills include `sm-decouple`. Agents load this file first. Before the
   first lane:
   - rewrite AGENTS.md as one short page;
   - add two role skills, `observer` and `implementer`;
   - delete the old skills.
5. **What the implementer may read:** MODEL.md, its domain's spec, the localiser output, the
   `data` crate's public types, and the model code. It may not read:
   - `verify/src/{decode,source,strand}.rs`, which hold WRAM addresses;
   - the bank logs;
   - history before the `archive/old-engine` tag.

   Run observer and implementer as separate sessions, never one agent switching hats.

## R4. The spec template, and a review of each spec

The samus-ground spec is close to the template. Make the template explicit so later domains
don't drift into asm in prose:
- **Per entry:** id, kind, summary, rule, params, switch, **checked**, one Source.
  - `checked` is `oracle (V3, V4)` or `original only`. The spec already does this informally
    for the slopes; make it a field.
- **Per quantity:** units, **Read by** (R1), and a domain type (an enum, not ROM codes). ROM
  codes go to the decoder notes. In samus-ground, `mode` 0/1/2 and X-speed-killed 4/8 move there.
- **Not allowed:**
  - headings named after routines;
  - RAM addresses outside the Source line (samus-ground has one, `$0B4C`, in §7.3);
  - step lists that follow a routine. Order is stated only in "Order that matters", and only
    where it is visible at a cut.
- **A reviewer checks each spec** before any implementer starts. This is Jim or a third agent that
  hasn't seen the asm. The question is: could someone who never saw the asm implement this, and
  would the code be shaped by the domain?

## R5. Test vectors as executable tests

**Problem.** The only feedback loop is the oracle, and every miss costs two cold agents.

**Change.** The observer commits each spec's vectors as data:
- in `scenarios/vectors/<domain>.toml`;
- with before, pad and after, in domain terms;
- including the constructed vectors (V11, V12) and the `Exact` variants.

A test in `game` runs them, so the implementer iterates locally and reaches the oracle only to
confirm. Decoding a vector's before-state goes through R2's decoder. So the observer writes the
vector file against the implementer's types once they exist.

## R6. No fitting to the oracle

The implementer sees differences but not their cause, so it will be tempted to tune.
- Every literal and every branch in `game::model` traces to a `BehaviourId`. Values come from
  `Rules` or `data`.
- Catalogue entries are added only in observer commits. The crate-rules check can verify this
  with a `Role: observer` commit trailer.
- An unexplained difference is not fixed by the implementer. It writes an entry in
  `docs/behaviours/<domain>.gaps.md` with the frame, the field, the expected and actual values,
  and the entry it suspects. Then it moves on. The observer answers from the original.
- Never write conditions on frame numbers, positions or room IDs that no spec entry names.

## R7. Switches: Fixed unless there is a named use

Every `Toggle` is a second code path, and each needs tests. samus-ground has three:
`SpeedCap`, `VerticalScanAlternates` and `ArmPumpStep`.

Proposal for Jim:
- `SpeedCap` stays a Toggle as the thesis's proof.
- Other entries are Fixed until a consumer asks, such as a Map Rando option, respin or the QoL
  list.
- `Parameter` (a value in `Rules`) is cheap and can stay.

## R8. A size budget, written before the build

RETROSPECTIVE A7's rule (a budget per domain, written first, reported on every change) is not in
MODEL. §4 only says "fewer lines than `movement.rs` plus the ground half of `collision.rs`".
- Put the baseline and the budget as numbers in the spec header.
- The size report shows the domain against its budget.
- Going over the budget is a review stop, like a routine-named function.

## R9. Metrics

The behaviour count is easy to inflate by splitting entries, so it isn't a goal in itself. The
primary measures are:
- the domains that meet §4's definition of done;
- for each domain, the oracle-checked entries out of the total;
- the RAM-shape list (R2) and the size against budget (R8).

## R10. Exit criteria for the pilot (Samus on the ground)

The pilot passes when all of these hold:
1. The outputs are byte-identical: R1's three checks hold.
2. No asm address appears in `game::model::samus`. All Source lines are in the catalogue.
3. The RAM-shape list for the domain is short and justified.
4. The vector tests pass, and the domain's line count is under its budget.
5. At least one gap went from the implementer to the observer and back. The spec was amended,
   not the code patched.
6. The implementer's session never read the bank logs, `verify`'s address files or the old
   module.

If items 1 to 5 hold but item 3 doesn't, the clean room failed, however well the code reads.
Fix R1 and R2 before the second domain.

---

## Acknowledgement

The owner of MODEL.md fills this in. Replace `pending` with `accepted`, `rejected`, `deferred`
or `ask Jim`, and give one line of reasoning or the commit that applies it. Where an item is
partly accepted, say which part. `tools/check_review_ack.sh` fails the lane gate and CI while any
row is `pending`. Once every row is answered, tell Jim in the conversation. Then remove the
check, its gate and CI lines, and the banners in AGENTS.md and MODEL.md.

| Item | Status | Reasoning or commit |
| --- | --- | --- |
| R1 | pending | |
| R2 | pending | |
| R3 | pending | |
| R4 | pending | |
| R5 | pending | |
| R6 | pending | |
| R7 | pending | |
| R8 | pending | |
| R9 | pending | |
| R10 | pending | |
