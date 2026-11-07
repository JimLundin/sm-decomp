# The behaviour model

This note sets the shape of the engine's game code. It amends ARCHITECTURE where they disagree,
and the amendments are listed in §7. It follows Jim's decisions of 2026-11-06, recorded in §1.

## 1. The goal, and what is exact

The goal is to model Super Metroid's behaviour, all of it, as named behaviours. Each one can then
be reproduced exactly, adjusted (the space jump timing), added (respin) or removed (lag). In the
end only this engine runs; the emulator is the oracle that proves the vanilla behaviours exact.

Jim's decisions, 2026-11-06:

1. **Exact means the compared cuts and the output bytes.** The engine may depart from the
   original's routine boundaries and from the order of work inside a frame. What must match is
   every compared cut (the NMI's return at the end of each frame) and every output byte. Order
   is kept only where it can be observed: lag, overruns, stale video memory.
2. **Reshape before features.** Feature work in a domain waits until that domain has its model
   shape.
3. **Uncompared leftovers.** Hardware leftovers that no game rule reads and no player sees, such
   as DMA registers after a transfer, are not compared. They leave the State and the decoder and
   are listed by name in §11.6 of ARCHITECTURE.

Jim's decisions, 2026-11-07:

4. **Assets in our own layout, imported from the ROM.** Sprites, poses, palettes, animations
   and the rest of the game's content get a format of our own that names things by what they
   are: a pose by its parts (body, action, aim, gun), not by the original's pose id. An importer
   at the edge reads the ROM's layout once and writes our format; it is the only code that knows
   that layout. The clean room defining a new layout is desirable. Loading assets is part of
   rebuilding the game; reading them through the ROM's layout was a shortcut. The assets come
   from the player's ROM and are never committed. The importer, which has to know the original,
   is outside the clean room. The format and its importer: [assets.md](assets.md).
5. **Size is counted in logical tokens, held on one weighted score of the model's logic.** The
   game's data (the values of constants and statics, and imported assets) is reported beside the
   logic, not counted against it (§4). So are type declarations: defining shapes is to be
   promoted, not taxed. The score weighs each logic token by what the code's priorities make of
   it, so that a lower score means better code, not churn: forms that hold no behaviour are
   free, a function signature is half, a `true` or `false` is double (§4). Two quality terms
   compose with those weights: depth (Q1), so nested decisions cost more than a flat dispatch,
   and clumps (Q2), so values that travel together without a type pay where they recur (§4).
   Hard lints on the clean crates refuse what no weight can price: an argument object (G6).
   The integrator's bridge is reported apart and not counted against the model, which its
   implementer didn't write (Jim, 2026-11-07).
6. **Code a modern developer would expect.** The engine and its assets take the shapes a developer
   of a 2D pixel-art game today expects: a character controller stepped at a fixed timestep,
   state machines whose moves are table rows, input as actions, tilemap collision with per-tile
   shapes, sprite sheets and animation clips, and events returned rather than side effects.
   Where the original's exact behaviour departs from those shapes, the shape stays and the
   departure is a named quirk. The clean room's version is in
   [docs/cleanroom/DESIGN.md](../cleanroom/DESIGN.md).
7. **The budget comes from the spec.** A domain's size budget is k weighted tokens per spec
   unit. k is defined as the old code's score per unit times 0.55, computed once, on the first
   domain, with the official counters (`tools spec-units`, `tools size`), then frozen and tested
   out of sample on the next domain (Jim, 2026-11-07: re-derived under this definition when the
   official count replaced the study's hand estimate of 24). Each area is capped at 70% of its old
   code. A spec unit is one condition→outcome
   rule ("arm"), plus a third for each term a data table makes the code interpret and a third for
   each operator in an exact formula; table rows, parameters, enum values, prose and vectors
   don't count. The budget moves by k per unit when the spec gains or loses behaviour, and is
   frozen at each implementer export. Over the budget is a review stop that names the areas and
   entries over; over the old code is a strict failure. The study is
   [size-budget.md](size-budget.md), and the weights' is [size-weights.md](size-weights.md); the
   template convention that makes the count exact is
   in §5.

What this rules out: code whose structure is the original's routines. That means a function per
bank routine, branches in the 65816's order, flags re-derived from `wrapping_sub`, and module docs
that narrate call order. The original stays the authority for *what* happens, never for how our
code is organised.

## 2. Layers

| Layer | Holds | May name | Never names |
| --- | --- | --- | --- |
| `data` | content decoded from the ROM, and the **rule tables** (vanilla values of every behaviour parameter) | ROM formats at its edge | game State |
| `game::model` | behaviour, in domain terms: Samus, the camera, actors (PLMs, enemies), the HUD's values, progress, the message box | State, `data`, `Rules`, intents | `VideoMemory`, `NmiBatch`, `VramWord`, `DmaChannel`, OAM bytes, B-bus registers |
| `game::intent` | the per-Tick intents: `SpriteDraw`, `TileWrite`, `LayerExposure`, `HudValues`, `PaletteWrite`, `Effect`, `SoundRequest` | domain types | bytes |
| `game::exact` | the one encoder from intents to the original's bytes, with every encoding quirk as a named rule: the OAM format and its 128-entry wrap, the screen split of tilemap rows, the write queue's capacity, the HUD's tile words, the message box's video | intents, video State | behaviour decisions |
| `game::tick` | the Tick, strands, the NMI, the line program, the cost model (unchanged) | everything | |

- `model` decides; `exact` encodes; `tick` orders.
- `exact` runs where the original queued its uploads, so the batch order is unchanged.
- The intents are also ARCHITECTURE D16's view: the wide and HD renderers read them, and the
  exact path and the intent path come from one source by construction.
- `tools/check_crate_rules.sh` enforces the "never names" column. It starts as a warning and
  becomes an error after the reshape.

## 3. Behaviours are named, catalogued and switchable

`game::behaviours` is the catalogue. Each entry has:

```rust
pub struct Behaviour {
    pub id: BehaviourId,          // enum variant, e.g. BehaviourId::WordwiseSpeedCap
    pub summary: &'static str,    // what the player sees
    pub kind: Kind,               // Mechanic | Quirk | Timing (lag, loading) | Encoding
    pub switch: Switch,           // Fixed | Toggle | Parameter(range)
}
```

The catalogue carries no source. Where each behaviour comes from in the original is recorded only in
the observer's `docs/behaviours/<domain>.decode.md`, which implementers never see.

- A **mechanic** is a deliberate game rule: running speed, the wall jump's reach, the space
  jump's window.
- A **quirk** is unintended behaviour that we reproduce: the word-by-word speed compare, a full
  write queue dropping a draw.
- **Timing** covers slowdown and loading time.
- **Encoding** covers the output's own quirks, which live in `exact`.

**Switches are Fixed unless there is a named use (review R7).** Every Toggle is a second code path
that needs its own tests. `SpeedCap` is a Toggle, as the proof of the thesis. Other entries stay
Fixed until a consumer asks for them: a Map Rando option, respin, the QoL list. A `Parameter`, a
value in `Rules`, is cheap and may stay.

`Rules` (the Session's options, ARCHITECTURE §10) carries one value for each switchable behaviour.
The Reference profile holds the vanilla values, and those are what the oracle checks. A different
value is an adjusted game. The engine runs it the same way; it just isn't compared.

A behaviour's code reads its parameter from `Rules` and its vanilla data from the `data` tables,
never from a constant buried in a routine. **The catalogue must have teeth:** the model's values come
from `Rules` (the Reference profile holds the vanilla ones), and a test checks that every
`BehaviourId` is exercised by at least one vector or recorded frame. Adding a behaviour means adding a catalogue entry, a
rule value and its model code (a table row, a variant, an encoder rule). It never means editing a
port.

## 4. The reference domain: Samus's motion

This domain is replaced first, end to end, so every later domain has a working example to copy.

**Scope:** Samus's motion is one domain, because one set of speed quantities has writers across all
of it (Jim, 2026-11-06, after an independent review). It covers:
- the ground: standing, running, turning, crouching, the morph ball and the step down;
- the air: jumps, the fall, the wall jump;
- pose selection's writes to the speeds (pose-change commands, the landing stop).

**Collision is the next domain, with a fixed slot.** Until then the motion model calls it through
the collision interface, over a bridge to today's collision code. The bridge's quirks are the
collision spec's first entries, so the bridge has an end.

**Replace, never adapt.** The model is wired in and the old code (`movement.rs`, `movement/air.rs`,
pose's motion part) is deleted in the same commit. No `LEGACY` gate or compatibility shim keeps
today's port's behaviour inside a new type. Where the model and today's engine disagree, the
oracle decides, and a fall in today's numbers is recorded as a defect of the old port.

**Shape:**

```rust
// data::motion: the vanilla values, decoded from the speed tables or declared, catalogue-linked
pub struct MotionRules {
    pub ground: KindTable<SpeedEntry>,
    pub extra_run: ExtraRun,
    pub step_down_extra: Pixels,
    pub speed_cap: SpeedCap,  // SpeedCap::Wordwise (vanilla) | SpeedCap::Exact
}

// game::model::samus::motion: each way of moving is one row of data
enum Horizontal { Hold, Accelerate { decelerate: bool }, Coast }
enum Vertical { KeepToGround, None }
enum After { Nothing, StopAll, EndTurn }
struct Row { horizontal: Horizontal, extra_run: ExtraRunRule, vertical: Vertical, after: After }

const fn row(locomotion: Locomotion) -> Row { /* an exhaustive match, no `_` */ }

pub fn step(samus: &mut Samus, pad: Controls, rules: &MotionRules, world: &World) -> Contacts {
    let row = row(Locomotion::of(samus));      // the one place the pose is read
    let velocity = velocity(samus, row, pad, rules);
    let outcome = world.sweep(samus.body(), samus.position, velocity);
    apply(row.after, samus, outcome)
}
```

**What proves it: the oracle, directly, frame by frame.** The yardstick is the oracle, never
today's engine, which is itself a partly wrong port. The **domain harness** in `verify` covers every
recorded frame of every scenario. For each frame N it:
1. decodes the domain's State and inputs from the oracle's record N;
2. runs the model one frame;
3. compares the domain's declared fields with the oracle's record N+1.

It reports **N of M oracle frames matched**, per behaviour. That is the domain's progress
measure. Gaps come from frames that don't match, not from reading old code. The spec's vectors stay,
for cases no recorded frame reaches.

**A domain is a unit that can be verified on its own (Jim, 2026-11-07).** Its boundary is set by
verification, not by where the original's code is split. If checking a domain needs a stand-in that
answers from the oracle (a collision bridge that copies the next frame's stop, pose inputs read from
the next record), the boundary is wrong and the domains merge. That is why Samus's motion, pose
selection and her collision with the world are one domain: on whole frames, the first motion model
scored no better than a step that does nothing (5,631 against 5,686 verified frames), because the
world decides where she ends up.

**The primary proof is a free run, not a per-frame check.** Run the domain from a seed over many
frames, with no re-seeding, against the recorded game. Report the first differing frame per field and
per behaviour (the free-run localiser). The per-frame harness re-seeds every frame, so errors never
accumulate. It finds where a model goes wrong; it does not prove it right.

**The measure must be honest (Jim, 2026-11-07, after the second pilot review):**
- Report over **all recorded frames** of every row, beside a **null model's** score (a step that
  does nothing) on the same frames. A match rate means only what it gains over that baseline.
- **Mask a field; don't skip a frame.** A frame is skipped only when the domain doesn't run in it,
  and for a named reason. A skip may never depend on the outcome being tested, such as next-frame
  values or "y isn't where the prediction puts it".
- A value the harness's stand-in copies from the oracle's next frame is **unverified**, not
  matched. It is reported separately.
- A frame that fails to decode is a **harness defect**, counted, not a skip.
- **Every row has a floor**, so a regression anywhere fails.
- **Per-behaviour attribution:** each entry reports the frames that exercised it and the frames
  that matched. Entries no recorded frame exercises are listed as "vectors only".
- Size is measured **like for like**: the same counter on the old code and the new model. The
  bridge that wires the model into the engine is the integrator's, reported apart and kept small
  by review, not counted against the model (Jim, 2026-11-07).
- Size is counted in **logical tokens** (Jim, 2026-11-07), with `tools size PATH...`
  (`verify::size`): every identifier, keyword, literal, lifetime and operator is one, and so is
  each bracket pair. Comments, doc comments, layout and tests (`#[cfg(test)]` items, `tests`
  directories and `tests.rs` files) are free. Lines rewarded cramming and punished types and
  tables laid out one field to a line; tokens count what the code says.
- **The rule is held on logic; data is reported beside it** (Jim, 2026-11-07). The values of
  `const` and `static` items count as data: the game's tables and parameters, such as the pose
  chart's rows and which sprite shows a pose. The old engine read those from the ROM's layout
  and so never paid for them; loading them is part of rebuilding the game, so the new engine
  stating them is not extra logic. A data row must state the game's data. A row that encodes a
  decision the code would otherwise make is logic, and a review moves it back.
- **A value that holds behaviour is logic** (Jim, 2026-11-07; rule L3 in the counter). A
  constant's or static's value that holds control flow (`if`, `match`, `loop`, `while`, `for`), a
  binding (`let`, `fn`), a closure (`|..|`, `move`), a macro or a call is counted as logic. A call
  is a snake-case name followed by its arguments: `f(..)`, `T::new(..)`, `x.get(..)`,
  `size_of::<T>()`. Constructors are camel case, so a chart row built from enum variants and
  struct literals (`Held(Key::Jump)`, `Not(&Airborne)`, `Some(Fixed(3))`) stays data, and so do
  operators on constants (`A | B << 4`). Chart rows are data while they state the spec's chart;
  L3 and review guard the line.
- **Declarations are not counted against the rule** (Jim, 2026-11-07): `tools size` reports the
  tokens of item-level `struct`, `enum`, `union` and `type` items and of attributes apart, as
  types, on both sides. Defining shapes is what decision 6 asks for. A declaration holds no
  behaviour; anything that does (an `impl`, a function, a match) is logic. A constant's head
  (`const NAME: T =` and its `;`) is a declaration too.
- **The rule is held on one weighted score** (Jim, 2026-11-07). `tools size` prints it per file
  and in total, with the plain logic, types and data columns beside it. Weights, on both sides:
  - **0** for forms that hold no behaviour: `use` items, `mod` declarations, a trailing comma
    before a closing bracket (rustfmt places it), and the `const` of a `const fn`
    (`missing_const_for_fn` forces it).
  - **0.5** for a function signature, from `fn` to its body's `{` (or its `;`), so a named step
    costs less than its body inlined twice. `too_many_arguments` and the bool-parameter lints
    keep signatures honest.
  - **2** for `true` and `false` in logic, so a typed state beats a flag; `bool_comparison`,
    `needless_bool` and `nonminimal_bool` stop the swap to `!x` forms.
  - **1** for every other logic token.

  A target is set under the same weights as the baseline it is compared with; a reweighted model
  against an old-weight target would be a rule change hidden in a tool change.
- **Two quality terms compose with the weights** (Jim, 2026-11-07, on the quality-score review).
  Size-like metrics (Halstead, ABC, cyclomatic, fan-out) track the token count and would count
  size twice; these two measure what tokens miss. `tools size` prints each term's part of the
  score in its own column, so every number stays auditable.
  - **Q1, depth.** A logic token weighs `1 + 0.25 × d` times its weight, `d` the number of blocks
    around it inside a function body that nest: the block of an `if`, an `else`, a loop or a
    let-else, a closure's body (a block or an expression) and a `match` arm written as a block
    (`=> {`). A `match`'s own braces, its expression arms and a guard's `if` don't nest, so a flat
    dispatch stays at depth 0. Helpers, early returns and let-else are the cheaper forms, and they
    are the better ones.
  - **Q2, clumps.** A pair of parameter types other than `bool` that is in three or more
    signatures loses the signature's half on those parameters: they weigh one a token. A
    parameter's type is the first name it holds (`&mut Mover` is `Mover`), primitives included,
    so downgrading a type to `u16` doesn't dodge it. A pair of argument expressions (call
    arguments or a struct literal's field values) that recurs at three or more sites pays the
    shorter expression's tokens again at every site after the second; an expression holding a
    call, a literal and `..base` are left out. Clumps are counted over all the files measured
    together. A concept that removes a recurring pair pays for itself; a struct built only to
    carry one call's arguments pays as the arguments did.
  - **The parameter count is not a term.** A struct of the arguments would game any count, and
    the base and Q2 already price an interface's width.
  - Measured on round 7 (`tools size`, both sides the same counter): the old code scores 12,723
    (base 11,557.5, depth +828.5, clumps +337) and round 7 8,401.75 (7,465, +700.25, +236.5).
    Of four rewrites of round 7, the argument struct scores worse (+59.5) and her body box as a
    concept (−34.75), the box without methods (−33.75) and a split of `Samus::take` (−20)
    better.
- **Gates refuse what no weight can price** (Jim, 2026-11-07): hard lints on the clean crates,
  ceilings that yield no score, so nothing is counted twice.
  - **G6, `argument-object`** (`tools/cleanroom/code_check.sh`, through `shape_scan.awk` over a
    crate's library sources together): a struct taken by exactly one signature and built only in
    the argument lists of calls to that function, with no `impl` (no methods, no trait), and not
    a field's or a variant's type, a return type, a constant's or an alias's type or named in a
    trait. That is a bag of one call's arguments, which a parameter-count rule would reward;
    the values are passed as they are, or the concept gets a role. It flags the bag rewrite's
    `Go` and nothing in round 7 or the trunk.
- **k is re-derived under this score** (decision 7): the old code's density times 0.55, so
  12,723 / 271 units × 0.55 = 25.8 weighted tokens per unit.

The declared fields are the spec's quantities whose **Read by** is not "none" (§5). A quantity that
no later rule reads and that never reaches output is not State, not decoded and not compared. Once
the domain is wired in, the free-run gate's green pairs must hold, and the lockstep ratchet and
localiser show any change against today's engine. Each change is either a fix or a gap.
- The thesis holds in code. `SpeedCap` is a switchable quirk:
  - `SpeedCap::Wordwise` is vanilla, the oracle-checked value. The original compares the pixel
    and subpixel words separately, so 2.1000 counts as past a cap of 2.C000.
  - `SpeedCap::Exact` is the corrected rule.
  - A unit test shows the two differ on exactly that case, and the Reference profile stays
    exact.
- It's smaller (Jim, 2026-11-06). **A domain that is larger than the old code it replaces is a
  strict failure.** A domain must save significantly over the old code, unless the spec gives a
  clear, substantiated reason why this domain can't. The spec header holds the baseline (the old
  code's tokens, by file) and a target. A target over the baseline, or a scope change, goes to Jim with the
  numbers, and the lane stops until he answers.

**Done means:**
- A movement kind is a row, a parameter is a table entry or a rule, and collision is one sweep.
- Nothing in `model::samus` names a video type or an address. Source lines live in the catalogue
  only (review R3).

**The pilot passes (review R10) when all of these hold:**
1. The three exactness checks above hold.
2. No asm address appears in `game::model::samus`.
3. The domain's RAM-shape list (§5) is short and every item on it is justified.
4. The domain harness matches the oracle on the declared fields; the vector tests pass; and the
   domain is significantly smaller than the old code it replaces.
5. At least one gap from an unmatched oracle frame went from the implementer to the observer and
   back, and was fixed by amending the spec, not by patching the code.
6. The implementer never read the bank logs, `verify`'s address files or the old module.

If the other items hold but item 3 doesn't, the clean room failed, however well the code reads.
That is fixed before the second domain.

## 5. How the work is planned

**By mechanism, not by divergence.** A piece of work builds or extends one general mechanism
that covers a family of behaviours:
- a rule table;
- the instruction-list interpreter (`Program<F>`, which enemies have and PLMs need);
- the collision sweep;
- an encoder rule.

Lockstep and the free-run localiser are the acceptance check, and they tell *which behaviour* is
wrong. They don't decide *what code to write*.

**Two roles, always separate sessions: the observer and the implementer.** One agent never
switches between them. A domain goes in this order (review R2):
1. **The observer** reads the original (the bank logs, the ROM, the oracle's records) and writes
   the **behaviour spec**, `docs/behaviours/<domain>.md`, to the template below. It also commits
   the spec's test vectors as data in `scenarios/vectors/<domain>.toml`: before, pad and after, in
   domain terms, the constructed ones and the `Exact` variants included (review R5). The specs are
   the catalogue's source and the domain's design note, and Jim can read them as a description of
   what Super Metroid does.
2. **A spec reviewer** checks the spec before any implementer starts. That is Jim, or an agent that
   hasn't seen the asm. The questions:
   - Could someone who never saw the asm implement this, and would the code be shaped by the domain?
     (review R4)
   - Is each entry a behaviour a player would recognise, or a routine of the original renamed?
     Entries that follow the original's routine boundaries, or use its internal concepts, are
     regrouped by behaviour before the implementer starts. The clean room stops transliteration.
     This question stops the implementer inheriting the observer's breakdown of the original.
3. **The implementer** designs the State types and the model from the spec alone. It writes into
   an **empty module**, `game::model::<domain>`, never opens the old module, and deletes it in the
   same commit (review R3). A test in `game` runs the vector file, so it iterates locally and
   reaches the oracle only to confirm.
4. **The observer writes `verify`'s decode for the implementer's types.** Decode is an abstraction
   from RAM to the State, not a copy. If a State value can't be decoded, the observer either names
   the hidden quantity as a new spec entry, in domain terms, or shows that the value is derived,
   so the model computes it and doesn't store it. The implementer never adds a field because the
   decoder has one.
5. **Gaps go back to the observer** (review R6). The implementer doesn't fix a difference it can't
   explain. It writes an entry in `docs/behaviours/<domain>.gaps.md` with the frame, the field, the
   expected and actual values and the entry it suspects, then moves on. The observer answers from
   the original and amends the spec.

**What counts as a leak (Jim, 2026-11-07).** Structure leaks; perceivable quantities don't.
- **Allowed:** quantities a player can perceive or measure (positions, speeds, timers, counts) and
  the exact arithmetic on them.
- **A leak:** the original's routine boundaries, entry points and call order; its internal flags and
  modes; its command and result words; enum orders that match its RAM codes; how its tables are
  indexed; its routine and variable names; and its addresses, in any form.

**An adversary gates the clean room (Jim, 2026-11-07).** An adversary agent reads both the original
and everything an implementer will see, and tries to prove a leak. It succeeds by finding one. It
runs before any implementer starts (on the spec, the vectors and the export) and before any model
change merges. A leak it finds stops the work until it is removed; a reduced leak is not a fixed one.

**The clean room is enforced by construction, not by trust (Jim, 2026-11-07).** Every leak in the
pilot came from implementers having the whole repository and being trusted not to open files. A
sparse checkout is not enough: every worktree shares one object store and every ref, and nothing on
disk or on the web is out of reach. So:
- **The implementer works in an export,** not a worktree. It is a fresh repository built by
  `git archive` from an allowlist (the clean crates, this note's clean-room sections, the domain's
  spec and vectors, the implementer skill). It has no history, no other refs, no remote and no
  alternates, and lives outside the project's directories. Results come back as patches, applied by
  the integrator.
- **Canaries prove what was read.** Unique tokens are planted in every forbidden source (the bank-log
  copies, the decode notes, `verify`'s address files, the old code). The implementer's transcript and
  commits are searched for them after each lane. A canary found is a breach.
- **Scans run on everything that enters the export or the clean crates**, briefs and commit
  messages included:
  - addresses in every form: `$XX:XXXX`, split bank/offset pairs, `RomAddress`/`Source::bank`, and
    a hashed set of the original's known WRAM and ROM addresses;
  - names, against a lexicon drawn from the bank logs, with a reviewed allowlist of player vocabulary;
  - shape: enums whose variant order equals a RAM code's values, fields decoded as one word copied
    as-is;
  - routine mirroring: the share of entries with a single routine as source (gated at 30 % or
    less), and entry points that map to one routine.
- **Briefs are saved and scanned before the spawn.** The root and the observer are relays too.
- **The clean crates.** The model, the catalogue and the domain types an implementer needs (speeds,
  poses as enums, tables as typed records) live in their own crates. Those crates depend on nothing
  but each other and the standard library. They contain **no address, bank, RAM, ROM-code or
  original routine name anywhere, comments included.** `data` stays on the observer side and turns
  ROM bytes into the clean types at the edge. `tools/check_crate_rules.sh` enforces both rules in CI.
- **No addresses where an implementer can see them.** `Source:` lines live only in the observer's
  `docs/behaviours/<domain>.decode.md`. The spec carries behaviour ids, rules, parameters and
  vectors. The catalogue carries ids, summaries, kinds and switches.
- **Implementers work in an export** (`tools/cleanroom/export.sh`, see above). It holds only the
  clean crates, this note's clean-room sections, the domain's spec and vectors, and the implementer
  skill. There is nothing else in it to open, and `tools/cleanroom/canary.sh` checks afterwards what
  was read.
- **Only implementer commits change the clean crates.** A check rejects any commit to them that lacks
  the trailer `Role: implementer`. An integrator who needs a model change asks for it.

**The observer works behaviour first (Jim, 2026-11-07).** An observer who reads routines writes a
spec shaped like them: in the pilot, three entry points that were three routines, a command enum
that was a RAM word, and a result type that existed only because the original split the work.
- The spec describes **what the player sees** and what the oracle's records show over time, in
  player terms: speeds and positions as they change, how each input changes them, how the world
  stops them.
- The original's code is consulted only to pin down **exact arithmetic and edge cases** that the
  records can't settle. Its routine boundaries, call order, internal flags and RAM layout never shape
  the spec.
- A domain's boundary is a boundary the player sees. Where the original splits one player-visible
  behaviour across systems (motion and pose selection), the domain spans both.
- **The spec review is a gate.** An entry named after its one routine, an entry point that matches a
  routine, or a type that mirrors a RAM word stops the spec until it is regrouped.

**What the implementer may read:** this note, its domain's spec and vectors, the localiser's
output, `data`'s public types, and the model code. **Not:** the bank logs or the asm,
`verify/src/{decode,source,strand}.rs` (they hold WRAM addresses), the old module it replaces, and
history before the tag `archive/old-engine`.

**No fitting to the oracle (review R6).**
- Every literal and every branch in `game::model` traces to a `BehaviourId`, and values come from
  `Rules` or `data`.
- Catalogue entries are added only in observer commits, with the trailer `Role: observer`.
- No conditions on frame numbers, positions or room IDs that no spec entry names.

**The spec template (review R4).**
- **Header:** the domain's scope, its baseline in tokens and its budget (R8). The budget line is
  the tool's, from the spec's units (§1 decision 7), with its split by area.
- **Rules are arms** (§1 decision 7). Each rule is one bullet of the form
  `<conditions> → <outcome> [vectors]`; only bullets with `→` count, so explanation, rewording or
  splitting a rule doesn't move the count. Two arms of one entry with the same outcome are one
  arm. An arm counts only if the vectors or recorded frames it cites exist. Each data table has a
  `Cells:` line naming the terms its cells use. Exact formulas are in backticks. `checked` is its
  own bullet. A cross-reference to another entry's rule counts nothing. The adversary gate
  samples arms against the original: a padded arm stops the spec as a leak does.
- **Each quantity:** its units, a domain type (an enum, never ROM codes), and **Read by**: the
  behaviour IDs that read it on a later frame, or "output" if it reaches bytes, or "none". "None"
  means it isn't State (decision 3). ROM codes belong in the decode notes.
- **Each entry:** id, kind, summary, rule, params, switch, and **checked** (`oracle (V3, V4)` or
  `original only`). Its source in the original goes in the observer's
  `docs/behaviours/<domain>.decode.md`, never in the spec.
- **Not allowed:** headings named after routines; RAM addresses outside the Source line; step lists
  that follow a routine. Order is stated only in "Order that matters", and only where it is
  visible at a cut.

**The RAM-shape report (review R2).** The size report lists, for each domain, the State fields whose
decode is one WRAM word copied as-is. Each needs a one-line reason in the spec.

**Address check (review R3).** `tools/check_crate_rules.sh` reports `$XX:XXXX` and `$XXXX` anywhere
in `game` outside `behaviours.rs`. It starts as a report, because 881 lines cite addresses today,
and becomes an error domain by domain as each one is reshaped.

**A brief names behaviours and the mechanism** by spec entry IDs (for example: "`BallAtRest`
coasts like standing; add its row"). It carries no bank-log excerpt and no address. A new free
function named after a routine is a review stop.

**Order** (each domain is proven against the oracle by its harness, §4):
1. **Groundwork:**
   - the catalogue type and the `Rules` hook;
   - the numeric quirks once, on the types (signed compare, the word-by-word cap, wrapping
     moves);
   - the layer check as a warning.
2. **Reference domain:** Samus's motion (§4), with the domain harness. Then **collision**, the next domain with a fixed slot.
3. **Intents and the encoder:**
   - `SpriteDraw` plus `exact::oam` (Samus, enemies);
   - `TileWrite` plus `exact::tilemap`, which absorbs `plm/draw.rs`, the mapping in
     `bg_graphics.rs` and `tilemap_dma.rs`;
   - the HUD;
   - the message box.
4. **Actors:**
   - `Program<F>` generalised from `enemy/instructions.rs`;
   - the PLM lists decoded from the ROM, replacing `ItemStep` and the crumble's hand interpreter.
5. **Leftovers:** remove the uncompared hardware leftovers (decision 3).
6. **Features resume** in reshaped domains, as mechanisms: sound (C11), the line program (B6),
   the remaining slice-0 differences.

## 6. How progress is measured

Measured first (review R9; a raw behaviour count is easy to inflate by splitting entries, so it
isn't a goal):
- the domains that meet §4's definition of done;
- for each domain, its oracle-checked entries out of its total;
- for each domain, the RAM-shape list (§5) and its size against budget.

Then, as checks that must not regress:
- the lockstep ratchet and the free-run gate's green pairs;
- the free-run localiser's first differing frames;
- Ticks/s.

The size report adds tokens per layer, so the routine-port share is visible and falls.

## 7. Amendments to ARCHITECTURE

1. **§2:** phases run in the original's order (§6.3). Inside a domain, structure
   follows the model, and an order that can be observed at a cut is a named behaviour.
2. **§4.1 and §9.1:** add `game::exact`. Producers never write video State.
3. **D16:** intents are the model's primary per-Tick output. `describe` returns them rather than
   reconstructing them.
4. **D3, D5 and §11.6:** hardware leftovers are State only when an engine rule reads them.
   Otherwise they are not compared, and are listed by name.
5. **D13:** a family whose behaviour is a ROM instruction list runs it through the interpreter.
   Hand-coding a list as an enum of positions is a review stop.
6. **P20:** the catalogue (§3) is its artifact, and D24 reports it.
7. **K9:** a bank reference lives only in the observer's decode notes, never in code, the spec or the catalogue; module docs describe the domain.

## 8. Lessons

This note answers these lessons from the three attempts so far. The "review R1" to "R10" labels
refer to an external review of this note (2026-11-06). Its text and the answers are in git history at
commit `c1b3ea3`, `docs/architecture/MODEL-REVIEW.md`.

**The earlier attempts.** Their rules are in ARCHITECTURE §16; these three apply here:
- **The transpile is reference only (rule 1).** Attempt 1 transpiled the C, and attempt 2 spent
  its effort decoupling that transpile. Attempt 3 (this one) kept the rule in name, but the bank
  logs became the module structure. §1 and §2 make the rule structural, and the two roles (§5)
  enforce it: the implementer never sees the original.
- **Nothing exists only for comparison (rule 5).** Broken by the DMA leftovers; fixed by
  decision 3.
- **One short rules file, and ask Jim only goal trade-offs (rule 23).** Broken by 44 docs
  (16,000 lines) and old skills next to the new code. The repository now holds only this
  engine's docs and code; the history is in git (tag `archive/old-engine`).

**From the first pilot (2026-11-07): the thesis's faulty assumptions.** The honest harness showed the
first Samus-motion model was no better than a step that does nothing. These assumptions were wrong:

| Assumption | Why it failed | Now |
| --- | --- | --- |
| Domains can be cut where the original's code is cut | Motion, pose selection and collision are one behaviour to the player; motion alone can't be checked | A domain is a unit that can be verified on its own (§4) |
| A stand-in answering from the oracle is a valid test | Whatever it touches is copied, so it is unverified | Copied values are unverified; a domain needing a stand-in merges with what it stands in for |
| Re-seeding each frame proves behaviour | Errors never accumulate; a model can look 99 % right per frame and still not be the game | The primary proof is a free run (§4) |
| A match rate means something alone | 99 % was noise against a do-nothing baseline | Every number is reported beside the null model |
| An observer reading code can write a spec not shaped like it | About 95 % of the spec was the original's structure renamed | The observer works behaviour first; an adversary gates the spec (§5) |
| A clean room can run on trust | Every channel leaked | It is enforced by construction: an export, canaries, scans (§5) |

**From attempt 3 (2026-11-06):**

| Lesson | Answer |
| --- | --- |
| "Fill functions and never reorder" made every domain a routine port (41 % of `game`) | §1 decision 1: exact means cuts and bytes, and structure is the model's |
| With no intent type, every drawing lane wrote bytes | §2: intents and one encoder; `model` may not name video types |
| The thesis had no artifact in the code | §3: the catalogue and `Rules` switches |
| Lanes chose work by the next differing field; tick counts rose while the model didn't | §5: plan by mechanism, with the oracle as the acceptance check; §6: measure behaviours first |
| A ROM list was hand-coded as an enum (`ItemStep`) | §7.5: interpreter mandatory |
| Quirks were re-derived in each routine (16 local signed compares) | §4: once, on the type, named and switchable |
| Each lane rediscovered context from the bank logs; fixed overhead was about half of each lane | §5: the observer writes it once, and briefs carry the excerpts |
| Coordinator waves and hourly reviews measured the process, not the goal | the orchestrator holds the design; review happens once per finished mechanism, against §6 |
| Agents read old docs and old code first and copied their shapes | old code and docs removed from the repository |
