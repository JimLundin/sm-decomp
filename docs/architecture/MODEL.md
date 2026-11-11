# The behaviour model

This note sets the shape of the engine's game code. Its changes to ARCHITECTURE and PRINCIPLES are
made there, and where they disagree this note wins. It follows Jim's decisions, recorded in §1.

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
   free, a function signature is half, a `true` or `false` is double (§4). An exhaustive lookup,
   a table the compiler checks, is data like a constant's rows (Jim, 2026-11-11). Two quality
   terms compose with those weights: depth (Q1), so nested decisions cost more than a flat
   dispatch, and duplication, so a decision written twice costs more than one shared step
   (Jim, 2026-11-11; §4).
   Hard lints on the clean crates refuse what no weight can price: ceilings in the clean
   room's clippy configuration (G5), an argument object (G6) and a module cycle (G7).
   The integrator's bridge is reported apart and not counted against the model, which its
   implementer didn't write (Jim, 2026-11-07).
6. **Code a modern developer would expect.** The engine and its assets take the shapes a developer
   of a 2D pixel-art game today expects: a character controller stepped at a fixed timestep,
   state machines whose moves are table rows, input as actions, tilemap collision with per-tile
   shapes, sprite sheets and animation clips, and events returned rather than side effects.
   Where the original's exact behaviour departs from those shapes, the shape stays and the
   departure is a named quirk. The clean room's version is in
   [docs/cleanroom/DESIGN.md](../cleanroom/DESIGN.md).
7. **The budget comes from the spec's units.** A domain's size budget is k weighted tokens per
   spec unit. k is defined as the old code's score per unit times 0.55, computed once, on the first
   domain, with the official counters (`tools spec-units`, `tools size`), then frozen and tested
   out of sample on the next domain (Jim, 2026-11-07: re-derived under this definition when the
   official count replaced the study's hand estimate of 24). Each area's budget is k × its units,
   and the old code no longer bounds it: it stays beside the budget for reference (Jim,
   2026-11-10: "Drop the caps", since a cap at 70% of the old code gave new behaviour no budget).
   A spec unit is one condition→outcome
   rule ("arm"), plus a third for each term a data table makes the code interpret and a third for
   each operator in an exact formula; table rows, parameters, enum values, prose and vectors
   don't count. The budget moves by k per unit when the spec gains or loses behaviour, and is
   frozen at each implementer export. The spec never states it: k, the areas, their old code and
   its commit are in the `## Budget` section of the domain's decode notes, which `tools
   spec-units` and `tools report` read (decision 10). The budget is an indicator, not a wall
   (Jim, 2026-11-08):
   the fit is score ÷ budget, by area and in total, and above 1 the excess costs in proportion to
   how far over, as a penalty for comparing alternatives and rounds, never as a gate. We look for
   a reasonable optimum of logic against specification, and stop compacting when a round's gain
   in fit is small beside its cost and risk; a form that is worse only to get under the line is
   not wanted. Over the old code is still a failure: it means the design is wrong. The study is
   [size-budget.md](size-budget.md); the weights are §4's, and the template convention that
   makes the count exact is in §5.
   Code in no area (state types, inputs, the frame's order, the catalogue) has no units of its
   own, so it is held by the same k (Jim, 2026-11-11): `tools report` shows each domain's area
   fit (the areas' score ÷ the budget) and its shared share (the code in no area ÷ the model)
   apart, and a shared share over 25 % is a finding.
8. **The repository shows only the current state** (Jim, 2026-11-08). Old code or docs kept only
   for comparison are deleted; a comparison uses git history (the tag `archive/old-engine`, or the
   commit before a deletion). When a clean model replaces engine code, the integrator deletes the
   old code in the same lane, and a baseline cites the commit it was measured at. The engine code
   of domains not yet rebuilt is current, and goes as each domain is replaced.
9. **Audio is rebuilt clean-room too** (Jim, 2026-11-08): a sequencer and a model of the sound
   chip's DSP, specced in behaviour and verified on the oracle's audio stream, scheduled late; the
   CPU side (which sound plays when) is modelled with the domains that trigger it. The plan of
   domains is [domains.md](domains.md).
10. **The implementer is not told the score or the budget** (Jim, 2026-11-08). A score in front
    of the implementer becomes a clearer goal than modelling the spec well, and invites gaming.
    The score, k, the budgets, the fit and the old-code baselines are evaluation tools, for the
    root, the observers and the adversaries comparing spec and code. Implementer-facing material
    (the export: its design note, its skill, the spec and the vectors; and every brief) states
    code rules only: the lints and ceilings (L1, L2, G5 to G7) as coding standards, never as parts
    of a measure. Feedback from a size review reaches the implementer as shape findings
    (duplication, a table that could be data, a function doing two things), never as a number or
    a target. The budget lives in the Budget section of the decode notes
    (`tools/cleanroom/move_budget.sh` moves a spec header's there), and
    `tools/cleanroom/score_scan.sh`, with its word list `score_words.txt`, refuses an export
    (`export.sh`), a brief (`scan_brief.sh`) or a trunk file an export would carry
    (`check_crate_rules.sh`) that mentions any of it.
11. **A coarse verdict on size may reach the implementer** (Jim, 2026-11-09). Two rounds without
    the score came out far larger than their behaviour needed (camera round 2, progress round 1).
    So after a round gate, the next brief may say, in words only, that the model is much larger
    than the behaviour needs, and in which files and items, alongside the shape findings that
    say how. It never carries the score, the budget, a ratio or a target. With it go a numberless
    standard in the design note (concise: the smallest clear code that models the spec, each rule
    stated once) and cut rounds (PROCESS, "The loop"): rounds whose brief is the verdict and the
    shape findings alone, repeated until the gate is satisfied.

Jim's decisions, 2026-11-09:

12. **Widely used crates where they model the domain.** In Jim's words: "we should in general be
    in favor of widely used crates... not restrained to only the standard library... When an
    outside dependency can help us better model the domain we're building, then we should do
    so." A crate is used when it is widely used and the standard choice for its job, std doesn't
    cover that job, it helps model the domain, and it builds on the development machine. The
    root approves each, at an exact version: `tools/cleanroom/allowed_deps.txt` lists those the
    clean crates may use, with their kinds and features, and `code_check.sh` refuses the rest.
    A crate whose build script Defender would block is used through a copy with the script's
    output baked in (`tools/vendor_nobuild.sh`, the root manifest's `[patch.crates-io]`), which
    CI holds to the real script's output (`tools/vendor_nobuild_check.sh`).
13. **Vectors hold RAM observations and invented content only.** Jim chose "invented reactions
    only" for the blocks spec, then widened it to every domain. A vector carries what the records
    show of RAM and content made up in our own format to tell the rules apart, never the game's
    content: no script, table, room's state list, screen, screen kind or cell look taken from the
    ROM. A recorded case that needs real content becomes a harness case: its row and frame go in
    the decode notes, the importer loads the content in the integrator's harness, and a
    constructed vector over invented content carries the rule. Each domain's next observer pass
    converts its recorded vectors that carry real content.

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
- A rebuilt domain's behaviour is a clean crate (`crates/clean_model/<domain>`, §5), which
  `game` calls through its bridge in `model`'s place.
- The "never names" column is held by review; `tools/check_crate_rules.sh` gains a check for it
  with the reshape.

## 3. Behaviours are named, catalogued and switchable

Each domain's clean model holds its catalogue (`crates/clean_model/<domain>/src/catalogue.rs`),
one entry per spec entry, over the shared kinds in `clean_types`. Each entry has:

```rust
pub struct Behaviour {
    pub id: BehaviourId,          // enum variant, e.g. BehaviourId::WordwiseTopSpeed
    pub summary: &'static str,    // what the player sees
    pub kind: Kind,               // Mechanic | Quirk | Timing (lag, loading) | Output
    pub switch: Switch,           // Fixed | Toggle | Parameter
    pub checked: Checked,         // ByVectors | OriginalOnly
}
```

The catalogue carries no source. Where each behaviour comes from in the original is recorded only in
the observer's `docs/behaviours/<domain>.decode.md`, which implementers never see.

- A **mechanic** is a deliberate game rule: running speed, the wall jump's reach, the space
  jump's window.
- A **quirk** is unintended behaviour that we reproduce: the word-by-word speed compare, a full
  write queue dropping a draw.
- **Timing** covers slowdown and loading time.
- **Output** covers what a step reports beyond its quantities, for its caller to act on (the
  cells she touched, where the camera follows her from). The encoder's quirks, in `exact`, take a
  kind of their own when that layer is rebuilt.

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

This domain was replaced first, end to end, so every later domain has a working example to copy.

**Scope (Jim, 2026-11-07):** Samus moving through the world is one domain, because one set of
speed quantities has writers across all of it and the player sees one thing. It covers her speeds
and position on the ground and in the air, how her pose follows from input and the world where
that changes her speeds, and her collision with blocks, slopes and solid enemies. The spec,
[samus-motion.md](../behaviours/samus-motion.md), states the scope and what is not in it yet.

**Replace, never adapt.** The model is wired in and the old code it replaces is deleted in the
same change. No `LEGACY` gate or compatibility shim keeps the old port's behaviour inside a new
type. Where the model and the old engine disagree, the oracle decides, and a fall in the old
numbers is recorded as a defect of the old port.

**Shape.** The model is the clean crate `clean_model_samus_motion`
(`crates/clean_model/samus-motion`): a character controller stepped one frame at a time, its
moves as table rows and its pose as a chart, over the shared types and the `World` trait in
`clean_types`. `game::samus_world` is the integrator's bridge: it answers `World` from the
engine's level and solid enemies, maps the engine's State to the model's and back, and turns the
cells the model reports into the engine's touches.

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
scored no better than a step that does nothing (5,631 against 5,686 verified frames, recorded at `4f5b431ae`), because the
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
- **A value that holds behaviour is logic** (Jim, 2026-11-07; rule L3 in the counter; refined by
  the root, 2026-11-09). A constant's or static's value is data when it is built only from
  literals (a minus on a literal included), other constants' names, paths to enum variants or
  unit structs, references to data, struct and tuple-struct literals (with a `..BASE`), arrays
  (`[a, b]`, `[a; N]`) and tuples, calls of a function or associated function whose every
  argument is itself data by this rule, and ranges whose two ends are literals or paths
  (`0x280..0x500`, `A..=B`). So a typed table is data: `Picture::new(0x009)`,
  `Tile::new(Picture::new(0x00F), COLOURS_3, Flip::None)`, `Count::new(99)`, `Held(Key::Jump)`,
  `Some(Fixed(3))`; the constructor's body is counted as logic where it is defined. Anything else
  makes the whole value logic: control flow (`if`, `match`, loops, `?`), a block, a closure, a
  macro, a method call on a value (`Divisor::new(100).expect(..)`, `.unwrap()`, `.map(..)`), a
  field or an index (`A.0`, `T[2]`), a turbofish, a cast, a range with an operator or a call at
  either end (`0..A + 1`, `..16`), and every other operator but a minus on a literal
  (`A | B << 4`, `-A`). Chart rows are data while they state the spec's chart;
  L3 and review guard the line. `tools size` and `code_check.sh` apply the same rule, so the
  import check and the counter agree.
- **An exhaustive lookup is data** (Jim, 2026-11-11). A table the compiler checks for a missing
  or repeated member is data as a constant's rows are: the arms of a `match` of two or more arms
  with no guard, whose every pattern is `_` or keys joined by `|` (a path to a variant or a
  constant, a literal, or a range of two literals; a tuple variant's fields may hold such
  patterns in turn, `Outer::A(Inner::B) =>`, so long as nothing is bound) and whose every
  value is data by L3 and names no local (`K::A => 3`, `0 | 1 => Picture::new(0x009)`), and
  the rows of `enum_map!` on the same test (`K::A => Speed(2)`). The `match`, its scrutinee
  and its braces, and the macro's name and brackets, stay logic. A guard, a block arm, a
  binding, a local, a field, a method call or an operator in any arm keeps a decision there,
  and the whole `match` stays logic. The rule sits beside L3 in `verify::size` (`lookups`),
  and `code_check.sh`'s L2 exempts the same values.
- **Declarations are not counted against the rule** (Jim, 2026-11-07): `tools size` reports the
  tokens of item-level `struct`, `enum`, `union` and `type` items and of attributes apart, as
  types, on both sides. Defining shapes is what decision 6 asks for. A declaration holds no
  behaviour; anything that does (an `impl`'s body, a function, a `match` that decides) is
  logic. A constant's head
  (`const NAME: T =` and its `;`) is a declaration too.
- **The rule is held on one weighted score** (Jim, 2026-11-07). `tools size` prints it per file
  and in total, with the plain logic, types and data columns beside it. Weights, on both sides:
  - **0** for forms that hold no behaviour: `use` items, `mod` declarations, visibility (`pub`,
    `pub(crate)`, `pub(super)`: narrowing or widening it changes no behaviour, and
    `redundant_pub_crate` keeps it plain), an `impl` header up to its body's `{` (no behaviour
    lives there; an `impl Trait` in a type is the signature's), a trailing comma before a
    closing bracket (rustfmt places it), and the `const` of a `const fn`
    (`missing_const_for_fn` forces it).
  - **0.5** for a function signature, from `fn` to its body's `{` (or its `;`), so a named step
    costs less than its body inlined twice. `too_many_arguments` and the bool-parameter lints
    keep signatures honest.
  - **2** for `true` and `false` in logic, so a typed state beats a flag; `bool_comparison`,
    `needless_bool` and `nonminimal_bool` stop the swap to `!x` forms.
  - **1** for every other logic token.

  A target is set under the same weights as the baseline it is compared with; a reweighted model
  against an old-weight target would be a rule change hidden in a tool change.
- **Two quality terms compose with the weights** (Jim, 2026-11-07, on the quality-score review;
  duplication, 2026-11-11). Size-like metrics (Halstead, ABC, cyclomatic, fan-out) track the
  token count and would count size twice; these two measure what tokens miss. `tools size`
  prints each term's part of the score in its own column, so every number stays auditable.
  - **Q1, depth.** A logic token weighs `1 + 0.25 × d` times its weight, `d` the number of blocks
    around it inside a function body that nest: the block of an `if`, an `else`, a loop or a
    let-else, a closure's body (a block or an expression) and a `match` arm written as a block
    (`=> {`). A `match`'s own braces, its expression arms and a guard's `if` don't nest, so a flat
    dispatch stays at depth 0. Helpers, early returns and let-else are the cheaper forms, and they
    are the better ones.
  - **Duplication.** A run of 12 logic tokens inside a function body that repeats a run seen
    before, anywhere in the files measured together, pays each of its tokens again, at one token
    and at its depth. The run counts what holds behaviour at full weight (no signature, no free
    form, no data, no closing bracket). Local names are one name (a lowercase word that no `.`
    or `::` qualifies and no `(`, `::` or `!` follows), so renaming a copy's locals doesn't hide
    it; fields, methods, paths and literals keep their names. A run that overlaps the first one
    it repeats is part of it. Twelve is the shortest run that two calls of one helper with the
    same arguments don't reach. So a decision written twice costs more than one shared step,
    whose signature weighs half.
  - **Clumps (Q2) are a diagnostic, outside the score.** `tools size` reports what they would
    add: a pair of parameter types other than `bool` in three or more signatures (a type is the
    first name it holds, primitives included), and a pair of argument expressions (call
    arguments or a struct literal's field values, an expression holding a call, a literal and
    `..base` left out) at three or more sites. Weighed, they rewarded a bag of references and
    taxed a shared helper; reported, they point a gate at values that travel together.
  - **The parameter count is not a term.** A struct of the arguments would game any count, and
    the call sites already price an interface's width.
  - Measured on the old samus-motion code at `c7f927ab6` and its two cuts: 13,169.75, of which
    depth is 828.5 and duplication 1,220.75 (clumps would add 337).
- **Gates refuse what no weight can price** (Jim, 2026-11-07): hard lints on the clean crates,
  ceilings that yield no score, so nothing is counted twice.
  - **G5, the clean room's clippy configuration** (`tools/cleanroom/clippy.toml`, which
    `export.sh` writes as the export's `clippy.toml` and the trunk's gate and Quality run on its
    clean crates): at most one `bool` parameter (`max-fn-params-bools = 1`), no block nested 5
    deep (`excessive_nesting`, denied in `[workspace.lints]` and off wherever no threshold is
    set), cognitive complexity at most 15. `too-many-arguments-threshold` stays at 7: a lower
    one would make a struct of the arguments a compliance tool. Q1 gives the gradient below
    the nesting ceiling.
  - **G6, `argument-object`** (`tools/cleanroom/code_check.sh`, through `shape_scan.awk` over a
    crate's library sources together): a struct taken by exactly one signature and built only in
    the argument lists of calls to that function, with no `impl` (no methods, no trait), and not
    a field's or a variant's type, a return type, a constant's or an alias's type or named in a
    trait. That is a bag of one call's arguments, which a parameter-count rule would reward;
    the values are passed as they are, or the concept gets a role. It flags the bag rewrite's
    `Go` and nothing in round 7 or the trunk.
  - **G7, `layering`** (the same scan): a crate's modules name each other in one direction
    only. A file's module is its path; a reference is a path through `crate::`, `super::`,
    `self::` or a child module's name, `use` groups included, and any cycle among them is
    refused, a child naming its parent while the parent reads the child included. It stops
    passing a whole owner (`&Samus`) down to a helper module to shorten its signatures.
- **k is computed under this score** (decision 7): the old code's score (13,169.75 for
  samus-motion, at `c7f927ab6`) per unit of the official `tools spec-units` count (193.3), times
  0.55, is 37.47, and k is 37.5, to one decimal. The decode notes' Budget section states the
  value and its inputs (decision 10); it is then frozen, and every domain's budget uses it.
- **Two lints close what no weight can** (Jim, 2026-11-07), as `tools/cleanroom/code_check.sh`
  tags on the clean crates:
  - **L1, `lint-attr`.** No `#[allow]` (the workspace denies `allow_attributes`), and every
    `#[expect]` carries a `reason = "..."` (`allow_attributes_without_reason` is denied too).
    Outside test code, no attribute may expect, allow or warn a lint that `[workspace.lints]`
    denies by name (`indexing_slicing`, `arithmetic_side_effects`, `as_conversions`,
    `wildcard_enum_match_arm` and the rest), a denied group (`clippy::pedantic`) or
    `warnings`: those lints are what stops the cheaper, worse forms. A lint denied only through
    a group is a style heuristic and may be expected with its reason, such as
    `struct_excessive_bools` on a pad whose buttons are independent bools. Tests may expect any
    lint, with a reason, as `clippy.toml` already loosens them.
  - **L2, `magic-literal`.** A number other than 0 or 1 in logic is refused, in any spelling
    (`15_i16`, `0x7FFF`, `0b10`, `0.5`, the `15` of `0..=15`). Allowed in a constant's or
    static's value while L3 counts it as data (`Picture::new(0x009)` is; a `.expect(..)` on it
    is not), in the value of an exhaustive lookup's arm or an `enum_map!` row (`K::A => 3`;
    a key's own number is still refused, so `7 => ..` names its 7), in declarations (enum
    discriminants, array lengths in types), in an array length `[x; N]`, as a tuple field
    (`.2`), in attributes and in test code. Every number traces to a named constant, a `Rules`
    field or a behaviour.

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
  clear, substantiated reason why this domain can't. The decode notes' Budget section holds the
  baseline (the old code's tokens, by area, at a commit) and the budget; neither reaches the
  implementer (decision 10). The budget is k × units even where that is over the baseline: an area
  whose spec holds more than its old code did grows its budget (Jim, 2026-11-10, dropping the
  caps). A scope change still goes to Jim with the numbers, and the lane stops until he answers.

**Done means:**
- A movement kind is a row, a parameter is a table entry or a rule, and collision is one sweep.
- Nothing in the domain's clean model names a video type or an address. Source lines live in
  the observer's decode notes only.

**The pilot passes (review R10) when all of these hold:**
1. The three exactness checks above hold.
2. No asm address appears in the domain's clean model.
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

**The round**, its steps, roles and who sees what, is [PROCESS](../cleanroom/PROCESS.md): the
single source of truth for the order of the work. This section holds the rules a round relies on.

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
  `git archive` from an allowlist (`tools/cleanroom/allowlist.txt`: the clean crates, the clean room's design
  [docs/cleanroom/DESIGN.md](../cleanroom/DESIGN.md), the domain's
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
  poses as enums, tables as typed records) live in their own crates. Those crates depend on each
  other, the standard library and the registry crates of `tools/cleanroom/allowed_deps.txt`
  (§1 decision 12). The root declares the approved dependencies, from that list, in a clean
  crate's `Cargo.toml`; implementer commits change its code. They contain **no address, bank,
  RAM, ROM-code or original routine name anywhere, comments included.** `data` stays on the observer side and turns
  ROM bytes into the clean types at the edge. `tools/check_crate_rules.sh` enforces both rules in CI.
- **No addresses where an implementer can see them.** `Source:` lines live only in the observer's
  `docs/behaviours/<domain>.decode.md`. The spec carries behaviour ids, rules, parameters and
  vectors. The catalogue carries ids, summaries, kinds and switches.
- **Implementers work in an export** (`tools/cleanroom/export.sh`, see above). It holds only the
  clean crates, the clean room's design, the domain's spec and vectors, and the implementer
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

**What the implementer may read:** the clean room's design, its domain's spec and vectors, the
localiser's output, `data`'s public types, and the model code. **Not:** the bank logs or the asm,
`verify/src/{decode,source,strand}.rs` (they hold WRAM addresses), the old module it replaces, and
history before the tag `archive/old-engine`.

**No fitting to the oracle (review R6).**
- Every literal and every branch in the clean model traces to a `BehaviourId`, and values come from
  `Rules` or `data`.
- The catalogue holds the spec's entries one for one, and a test in the clean crate holds it to
  the vectors: the observer adds an entry to the spec, the implementer to the catalogue.
- No conditions on frame numbers, positions or room IDs that no spec entry names.

**The spec template (review R4).**
- **Header:** the domain's scope, what it reads from other domains and what it leaves to them,
  and its notation. No size: the baseline in tokens, the areas and the budget (R8) go in the
  `## Budget` section of the decode notes (§1 decisions 7 and 10). The budget line there is the
  tool's, from the spec's units, with its split by area.
- **Rules are arms** (§1 decision 7). Each rule is one bullet of the form
  `<conditions> → <outcome> [vectors]`; only bullets with `→` count, so explanation, rewording or
  splitting a rule doesn't move the count. Two arms of one entry with the same outcome are one
  arm. An arm counts only if the vectors or recorded frames it cites exist. Each data table has a
  `Cells:` line naming the terms its cells use. Exact formulas are in backticks. `checked` is its
  own bullet. A cross-reference to another entry's rule counts nothing. The adversary gate
  samples arms against the original: a padded arm stops the spec as a leak does.
- **Each quantity:** its units, a domain type (an enum, never ROM codes), and **Read by**: the
  behaviour IDs that read it on a later frame, or "output" if it reaches bytes, or "none". "None"
  means it isn't State (decision 3). A quantity that is State is one a caller can set to any
  value the spec allows and read back, so the engine can carry, save and restore it; the spec
  says so where the model's memory is listed. ROM codes belong in the decode notes.
- **Each entry:** id, kind, summary, rule, params, switch, and **checked** (`oracle (V3, V4)` or
  `original only`). Its source in the original goes in the observer's
  `docs/behaviours/<domain>.decode.md`, never in the spec.
- **Not allowed:** headings named after routines; RAM addresses outside the Source line; step lists
  that follow a routine. Order is stated only in "Order that matters", and only where it is
  visible at a cut.

**The RAM-shape report (review R2).** The size report lists, for each domain, the State fields whose
decode is one WRAM word copied as-is. Each needs a one-line reason in the spec.

**Address check (review R3).** `tools/check_crate_rules.sh` refuses SNES address literals in the
code of `game`, `runtime` and `app`, and the clean room's scans (`tools/cleanroom/scan.sh`) refuse
an address anywhere in the clean crates, comments included. Comments in the engine code not yet
rebuilt still cite the bank logs; they leave with their domain's old code.

**A brief names behaviours and the mechanism** by spec entry IDs (for example: "`BallAtRest`
coasts like standing; add its row"). It carries no bank-log excerpt and no address, and no score,
budget or size target (§1 decision 10): a size review's findings go in as shapes to change. A new free
function named after a routine is a review stop.

**Order** (each domain is proven against the oracle by its harness, §4):
1. **Groundwork:**
   - the catalogue type and the `Rules` hook;
   - the numeric quirks once, on the types (signed compare, the word-by-word cap, wrapping
     moves);
   - the layer check as a warning.
2. **Reference domain:** Samus's motion (§4), with the domain harness; collision became part of it
   (her moves can't be checked without it). Jim, 2026-11-08: it is accepted near its budget after
   round 12's cut pass (about 7,200 against 6,924, 4% over, substantiated by area: collision and
   pose fit), rather than compacted further at the cost of worse forms. Then **the camera**
   (scrolling): it reads her motion's outputs (`camera_from`, the skid), has old code to replace,
   and is the first domain to test k out of sample.
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

## 7. Lessons

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
| A ROM list was hand-coded as an enum (`ItemStep`) | ARCHITECTURE D13: interpreter mandatory |
| Quirks were re-derived in each routine (16 local signed compares) | §4: once, on the type, named and switchable |
| Each lane rediscovered context from the bank logs; fixed overhead was about half of each lane | §5: the observer writes it once, and briefs carry the excerpts |
| Coordinator waves and hourly reviews measured the process, not the goal | the orchestrator holds the design; review happens once per finished mechanism, against §6 |
| Agents read old docs and old code first and copied their shapes | old code and docs removed from the repository |
