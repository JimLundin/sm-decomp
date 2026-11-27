# The behaviour model

This note sets the shape of the engine's game code and records Jim's decisions (§1). Where it
and another project document disagree, this note wins; where a later decision supersedes an
earlier one, the later one rules.

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
   are listed by name in [ARCHITECTURE §11.6](ARCHITECTURE.md#116-what-isnt-compared-by-name).

Jim's decisions, 2026-11-07:

4. **Assets in our own layout, imported from the ROM.** Sprites, poses, palettes, animations
   and the rest of the game's content get a format of our own that names things by what they
   are: a pose by its parts (body, action, aim, gun), not by the original's pose id. An importer
   at the edge reads the ROM's layout once and writes our format; it is the only code that knows
   that layout, and it sits outside the clean room. The clean room defining a new layout is
   desirable. The assets come from the player's ROM and are never committed. Where a value the
   content needs is held only as an instruction's operand (a scene's step, a sign, a wrap), the
   importer may read that operand after checking the instruction around it, and refuses any
   other code: that is extracting content, not running the game's code (2026-11-24). The format
   and its importer: [assets.md](assets.md).
5. *Retired.*
6. **Code a modern developer would expect.** The engine and its assets take the shapes a developer
   of a 2D pixel-art game today expects: a character controller stepped at a fixed timestep,
   state machines whose moves are table rows, input as actions, tilemap collision with per-tile
   shapes, sprite sheets and animation clips, and events returned rather than side effects.
   Where the original's exact behaviour departs from those shapes, the shape stays and the
   departure is a named quirk. The clean room's version is in
   [DESIGN](../cleanroom/DESIGN.md).
7. *Retired.*
8. **The repository shows only the current state** (Jim, 2026-11-08). Code or docs kept only
   for comparison are deleted; a comparison uses git history (the commit before a deletion).
   When a clean model replaces engine code, the integrator deletes the old code in the same
   lane, and a baseline cites the commit it was measured at. Decision 33 governs removal of
   previous gameplay.
9. **Audio is rebuilt clean-room too** (Jim, 2026-11-08). The CPU side (which sound plays when)
   is modelled with the domains that trigger it; decisions 23 and 37 settle how audio is built
   and verified. The plan of domains is [domains.md](domains.md).
10. *Retired.*
11. *Retired.*

Jim's decisions, 2026-11-09:

12. **Widely used crates where they model the domain.** In Jim's words: "When an outside
    dependency can help us better model the domain we're building, then we should do so." A
    crate is used when it is widely used and the standard choice for its job, std doesn't cover
    that job, it helps model the domain, and it builds on the development machine. The root
    approves each, at an exact version: `tools/cleanroom/allowed_deps.txt` lists those the
    clean crates may use, with their kinds and features, and `cleanroom check` refuses the rest.
    Cargo verifies registry sources against the lock. The [crate policy](../../crates/README.md)
    pins the releases permitted to run a build script.
13. **Vectors hold RAM observations and invented content only**, in every domain. A vector
    carries what the records show of RAM and content made up in our own format to tell the rules
    apart, never the game's content: no script, table, room's state list, screen, screen kind or
    cell look taken from the ROM. A recorded case that needs real content becomes a harness case:
    its row and frame go in the decode notes, the importer loads the content in the integrator's
    replay, and a constructed vector over invented content carries the rule. Each domain's next
    observer pass converts its recorded vectors that carry real content.

Jim's decisions, 2026-11-11:

14. **The headline milestone is whole-movie acceptance.** The engine replays a whole movie from
    one start state with nothing fed back: no re-seeding, no recorded state or timing from the
    oracle, only the movie's inputs, and every compared cut and output byte matches. The first
    target is power-on through Ceres: the any% movie from power-on to frame 15,196, after the
    flight to Zebes and before the Landing Site loads (Jim, 2026-11-12); the Landing Site starts
    the next milestone. A divergence is localised by sparse checkpoints over the whole run, then
    dense ones over the range they narrow it to. The plan's waves are ordered toward it
    ([domains.md §4](domains.md#4-order)), and the README counts its frames that pass, from the
    milestone in [domains.toml](domains.toml) and the reviewed played runs.
15. **A model stays total; content is checked at the edge.** A model doesn't refuse content or
    rules outside its spec. A value with a range the spec gives is a checked newtype; relations
    between fields and the shape of content are checked by the importer, outside the clean
    room; a trap inside the domain is closed by types. The wording is in
    [DESIGN](../cleanroom/DESIGN.md#what-the-model-checks-and-what-it-leaves-to-others).
16. **Each adjustable behaviour proves it can be adjusted.** Every catalogue entry whose switch
    is `Parameter` has at least one vector at a value other than the reference one; the spec
    gate requires it ([PROCESS](../cleanroom/PROCESS.md#a-round), steps 1 and 3).

Jim's decisions, 2026-11-12, on the review of the project's direction:

17. **A round is judged by its replay over carried State**, not by a per-frame re-seeded
    harness. Decision 32 sets how.
18. **A ladder of rungs leads to the milestone.** The headline milestone (decision 14) stays the
    goal, but its stretch is mostly cinematics, front end, lag and the sound driver, so its
    number can't move for weeks. Rungs over shorter stretches of the same movie, Ceres first,
    each played alone from its first seed (`[[rung]]` in [domains.toml](domains.toml)), give
    the work a number that moves. A session's planning picks work for the lowest rung not yet
    reached ([domains.md §4](domains.md#4-order)). Lag modelled rather than recorded (T1) is
    needed only by the milestone itself.
19. **The root decides cuts from independent findings** (Jim, 2026-11-14). The root records
    why the expected improvement warrants the cost and risk; a cut brief carries concrete shape
    outcomes. A scope change goes to Jim, and the lane stops until he answers.
20. **A round's fixed cost is cut where automation already holds the barrier.** The export,
    the box, the scans and the code rules hold the clean room on every round. The
    adversary gates a domain's first spec, and later a spec delta that adds a mechanism (a
    new table, interpreter or kind of state) or crosses into another domain; any other
    amendment goes on the round gate's review. The blind review runs on a domain's first
    round, then when the round gate asks for one. The round gate itself (PROCESS step 10)
    stays on every round.
21. **The engine's State holds the models' States.** A bridge that maps the engine's State to
    a model's and back, every frame, costs nearly as much as the model and is thrown away when
    only this engine runs. A domain's model owns its State, which the engine holds as is; its
    bridge answers only the traits the model asks (such as `World`) and turns its outputs into
    the intents and reports the engine reads. A remaining bridge moves to this when its domain's
    next round touches it.
22. **Each rule is stated once.** A rule lives in one place: the decisions here, the round in
    PROCESS, the coding standards in DESIGN. Skills and briefs link to it rather than restating
    it. [RULES](../RULES.md) is the one-page index an agent reads first, with links to where
    each rule lives.
23. **Audio runs no code of the game; only its assets are extracted, and sound is exact in its
    timing** (Jim, 2026-11-12). Nintendo's sound driver program is never run, emulated or
    ported, and the sound chip is not emulated. An importer reads the songs, instruments, sound
    effects and samples from the player's ROM into our own modern format (decision 4): each
    sample is decoded at the edge to plain PCM, a decode that loses nothing, so no SNES sound
    format reaches our content; every sound has a typed id we own in a committed registry, and
    sm-json-data's names are labels only. A clean-room sequencer, specced in behaviour from the
    oracle's port logs and sound chip, plays them, and a modern mixer in `runtime` turns its
    events into sound. What is compared is timing (decision 37 sets where): the game's sound
    requests and the replies it reads through the ports, which change gameplay's timing
    (ARCHITECTURE D18), and the sequencer's events per tick. The audio out (interpolation, echo,
    the envelope's shape, the sample rate) is not compared and may be modern: "if the spec
    cannot specify it then we should not restrict ourselves to make it exact... the goal is that
    we should play the music at the right timing"; LEGACY lists it. The driver program's bytes
    the boot uploads through the APU ports are a hardware leftover no player sees (decision 3):
    not compared; the upload's time is a named cost (T1).
24. **The lowest rung is a limit, not a preference** (Jim, 2026-11-12). While a rung of the
    ladder (decision 18) is the slice, a session starts no round or lane outside its needs list:
    work spread to other parts would only game the measure the slice exists to give. The work
    that makes the checks trustworthy runs alongside, since the rung's number is read through it:
    a model's headline is its gain over a step that does nothing, and planted faults show each
    check can fail (decision 25).
25. **A check is trusted once it is shown to fail** (Jim, 2026-11-12). Each domain's replay
    cases are run against planted one-line faults in the production paths that run its model,
    and a fault no vector or case catches is a finding against the cases. This runs in parallel
    with the rung's work, not ahead of it. A planted fault goes in production or in verify's
    played runner, never in a clean crate: `tools/check_mutants.py` refuses any other file
    (Jim, 2026-11-25).
26. **The process freezes once the checks are shown to work** (Jim, 2026-11-12). When decision
    25's planted faults are caught on the rung's rows for the domains it needs, and the gain over
    the null model is the headline, the process freezes until the rung is reached: no new rule
    or role. A check found unable to fail, or a breach of the clean room, still changes it.
27. **Session plans and reviews are issues** (Jim, 2026-11-12). The root posts each session's
    plan as an issue at its start and the review on it at its end, so a fresh root or a reviewer
    can resume or audit a session from the repository and its issues alone.
28. **Off-hours tasks may run in parallel under root supervision** (Jim, 2026-11-13).
    Independent structural tasks run within PROCESS's lane limits; the root supervises their
    work, reviews the evidence and lands each reviewed change
    ([PROCESS](../cleanroom/PROCESS.md#working-hours)); a landing may batch several
    independently reviewed changes, each kept as its own commits, on one CI run (Jim,
    2026-11-25).
29. **A model's State is serialisable, and the engine holds it through serde** (Jim,
    2026-11-13). A clean crate can't derive the engine's codec, so each model's State derives
    serde's `Serialize` and `Deserialize` ([DESIGN](../cleanroom/DESIGN.md), "State a caller can
    hold"), and the engine holds it as `canonical::model::Model`, which encodes, decodes and
    diffs it by the codec's own rules. serde shows a type's shape only by reading a value, so
    the engine pins each State's shape and a test holds the pin to the shape traced from the
    type: a State that changes shape fails it until the pin moves. This is how decision 21 is
    done.
30. *Retired.*
31. **Track bounded work beneath the milestone** (Jim, 2026-11-16). The milestone issue
    holds the goal and its measured progress. Work proceeds through smaller issues, each with
    one verifiable outcome, an owner, dependencies, current state and completion evidence.
    Session plans select those issues and session reviews record their actual results. A
    discovered prerequisite becomes visible work rather than an indefinite extension of the
    same issue. Closing prerequisite issues does not count as frames played alone. The
    procedure is in [PROCESS](../cleanroom/PROCESS.md#work-items).
32. **One authoritative replay process, with explicit scopes** (Jim, 2026-11-16). Acceptance
    is an index of short replay segments witnessing named behaviours, alongside integrated and
    whole-movie cases. A behaviour catalogue entry names a rule, not a test: one case may
    witness several behaviours, and a behaviour may need many cases. Scoped and integrated
    cases use the same runner and admission, advance, comparison and result semantics. Each
    case selects production responsibilities and invokes the same model and adapter entry used
    by the engine, carrying their owned State from its admitted start across the segment. No
    oracle next-state is copied into that owned State, and no verifier-only implementation
    decides gameplay. Inputs and borrowed stimuli must not be computed from the next expected
    outcome. Vectors and unit tests remain development checks; they do not form an independent
    acceptance definition.
    Each case declares its start, inputs, static content, borrowed neighbour stimuli, selected
    responsibilities, compared fields and outputs, their expectations, and scope limits. If a
    changing producer is required, the case includes the actual dependency or an explicitly
    admitted external input and reports the limited claim. Scope selection must not silently
    omit relevant consumers or claim full engine proof. A movement case can compare motion,
    pose, contact and intents without pixel compositing: rendering is downstream, and changed
    movement changes image inputs; such a case proves nothing about unchanged pixels.
    Integrated and end-to-end cases still check cross-domain contracts, and the whole-movie
    milestone (decision 14) remains. It requires the same production paths selected by the
    case, not full-frame rendering in every case. Changes to acceptance follow
    [PROCESS](../cleanroom/PROCESS.md#replay-transition).

    Declared replay segments map through their named rules to relevant production model and
    adapter files, with lookup from case to code and code to cases (Jim, 2026-11-17). The
    index supplies segment contracts and borrowed stimuli; the validated baseline supplies
    reviewed requirements. File links are navigation metadata, not dynamic coverage or
    exhaustive impact analysis. Queries preserve Unsupported boundaries and coordinate
    meanings, and never change the complete indexed requirements for landing.
33. **Remove previous gameplay owners; fail visibly where behaviour is missing** (Jim,
    2026-11-16). The engine runs the clean models without previous executable gameplay as
    fallback. Every previous behaviour owner's gameplay decision code, state and fallback
    routes are removed, with source preserved in git history. Legitimate content, codecs, exact
    encoding, scheduling and rendering services may remain after review of their
    responsibility; their location or age alone does not make them gameplay to delete.
    Required missing behaviour returns explicit `Unsupported`, never a silent no-op or
    old-gameplay fallback. Clean-room roles, root-only landing and lowest-rung prioritisation
    are unchanged, and this direction authorises no unrelated rounds.
34. **Prioritize reusable multi-frame coverage** (Jim, 2026-11-19). Production carries
    complete State across consecutive frames, proven by representative behaviour and
    continuation checks. Additional scoped cases do not establish integrated dependency closure
    or played-alone frames; borrowed inputs remain explicit and are not counted as modeled
    producers. Decision 35 sets what the rungs compare.
35. **The rungs compare at frame cuts** (Jim, 2026-11-23). Rungs M0–M2 compare the selected
    owners' State and the output bytes at each NMI cut (decision 1), with inputs from the
    movie only. Intra-frame bus-cycle order, absolute master-tick clocks and ordered CPU port
    callbacks belong to T1 and are not required until the headline milestone. A port reply
    that changes gameplay is compared on the frame it is read. A lag cut inside a rung is an
    explicit borrowed input until T1.
36. **Behaviour work leads** (Jim, 2026-11-23). No new decision here and no new process
    tooling until the frames the engine plays alone from a rung's seed grow over successive
    sessions; work that an existing decision or work item already requires is not new. Work on
    a round's cycle time is the exception: a backlog lane always runs beside the rounds
    (Jim, 2026-11-24; [PROCESS](../cleanroom/PROCESS.md#what-can-run-at-once)), and so is
    the standing backlog lane's work on workflow cleanup and landing time (Jim, 2026-11-25). Each
    session review reports the behaviour share of its commit lines
    ([PROCESS](../cleanroom/PROCESS.md#working-hours)). The transition and prior-claim
    ledgers are pruned once the witness-shaped frame cases retire.
37. **Audio is verified at its boundary with gameplay** (Jim, 2026-11-23). The rungs and the
    milestone compare gameplay State and the boundary between gameplay and audio: the sound
    requests each frame makes, and the audio replies the game reads back (a full queue, music
    ready), both at frame cuts. The audio CPU's own timing, such as when its turns run within a
    frame, stays a declared borrowed input until T1. The sequencer (U2: which note on which
    voice at which tick) is a separate comparison stream, decided later, that gates no rung;
    the audio waveform is never compared (decision 23). A scene's music requests are its
    domain's output at that boundary, not audio's (U1's) (Jim, 2026-11-25); a spec that still
    gives them to audio follows at its next touch.

What this rules out: code whose structure is the original's routines. That means a function per
bank routine, branches in the 65816's order, flags re-derived from `wrapping_sub`, and module docs
that narrate call order. The original stays the authority for *what* happens, never for how our
code is organised.

## 2. Layers

The layers separate decisions, their receipts, exact encoding and ordering. They are the clean
domain crates plus concrete `game::production` recipes:

| Responsibility | Holds | May name | Never names |
| --- | --- | --- | --- |
| `data` / `import` | typed content and reference values; ROM layout at import edge | content formats | live gameplay State |
| clean models | named domain rules and complete State | domain types, rules, intents | addresses, OAM bytes, video register layouts |
| production | complete model owners, checked content, concrete execution recipes | clean States, typed inputs and outputs | oracle records, expected outcomes, verifier case IDs |
| exact encoding | bytes from actual output intents | intents and encoding formats | gameplay decisions |
| runtime | downstream composition and presentation | immutable display/audio outputs | writable production State |

`model` decides, `exact` encodes, and production orders selected actual calls. There is no
gameplay dispatcher outside the recipes: a missing ordering, clock or consumer rule is an
explicit unsupported dependency, not a service reconstructed outside the clean room.

## 3. Behaviours are named, catalogued and switchable

Each domain's clean model holds its catalogue (`crates/clean/<crate>/src/catalogue.rs`),
one entry per spec entry, over the shared kinds in `types`. Each entry has:

```rust
pub struct Behaviour {
    pub id: BehaviourId,          // enum variant, e.g. BehaviourId::WordwiseTopSpeed
    pub summary: &'static str,    // what the player sees
    pub kind: Kind,               // Mechanic | Quirk | Timing (lag, loading) | Output | Encoding
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
  cells she touched, where the camera follows her from). **Encoding** covers the encoder's
  quirks.

**Switches are Fixed unless there is a named use.** Every Toggle is a second code path that
needs its own tests. Other entries stay Fixed until a consumer asks for them: a Map Rando
option, respin, the QoL list. A `Parameter`, a value in `Rules`, is cheap and may stay.

`SpeedCap` is the thesis in code, a switchable quirk: `SpeedCap::Wordwise` is vanilla, the
oracle-checked value (the original compares the pixel and subpixel words separately, so 2.1000
counts as past a cap of 2.C000); `SpeedCap::Exact` is the corrected rule. A unit test shows the
two differ on exactly that case, and the Reference profile stays exact.

`Rules` (the admitted production profile, [ARCHITECTURE §10](ARCHITECTURE.md#10-options-and-profiles))
carries one value for each switchable behaviour. The Reference profile holds the vanilla values,
and those are what the oracle checks. A different value is an adjusted game. The engine runs it
the same way; it just isn't compared.

A behaviour's code reads its parameter from `Rules` and its vanilla data from the `data` tables,
never from a constant buried in a routine. **The catalogue must have teeth:** a test checks that
every `BehaviourId` is exercised by at least one vector or recorded frame. Adding a behaviour
means adding a catalogue entry, a rule value and its model code (a table row, a variant, an
encoder rule).

## 4. Gates and lints

`cleanroom check` (crates/tools/src/bin/cleanroom) applies the gates and lints to the clean
crates; the crate rules, `cleanroom test` in the box and the import run it. Test code is
`#[cfg(test)]` items, `tests` directories and `tests.rs` files.

- **A value that holds behaviour is logic** (rule L3). A constant's or static's
  value is data when it is built only from literals (a minus on a literal included), other
  constants' names, paths to enum variants or unit structs, references to data, struct and
  tuple-struct literals (with a `..BASE`), arrays (`[a, b]`, `[a; N]`) and tuples, calls of a
  function or associated function whose every argument is itself data by this rule, and ranges
  whose two ends are literals or paths (`0x280..0x500`, `A..=B`). So a typed table is data:
  `Picture::new(0x009)`, `Tile::new(Picture::new(0x00F), COLOURS_3, Flip::None)`,
  `Count::new(99)`, `Held(Key::Jump)`, `Some(Fixed(3))`; the constructor's body is logic where
  it is defined. Anything else makes the whole value logic: control flow (`if`,
  `match`, loops, `?`), a block, a closure, a macro, a method call on a value
  (`Divisor::new(100).expect(..)`, `.unwrap()`, `.map(..)`), a field or an index (`A.0`,
  `T[2]`), a turbofish, a cast, a range with an operator or a call at either end (`0..A + 1`,
  `..16`), and every other operator but a minus on a literal (`A | B << 4`, `-A`). Chart rows
  are data while they state the spec's chart; L3 and review guard the line. `cleanroom check`
  applies the same rule.
- **An exhaustive lookup is data.** A table the compiler checks for a missing or repeated member
  is data as a constant's rows are: the arms of a `match` of two or more arms with no guard,
  whose every pattern is `_` or keys joined by `|` (a path to a variant or a constant, a
  literal, or a range of two literals; a tuple variant's fields may hold such patterns in turn,
  `Outer::A(Inner::B) =>`, so long as nothing is bound) and whose every value is data by L3 and
  names no local (`K::A => 3`, `0 | 1 => Picture::new(0x009)`), and the rows of `enum_map!` on
  the same test (`K::A => Speed(2)`). The `match`, its scrutinee and its braces, and the macro's
  name and brackets, stay logic. A guard, a block arm, a binding, a local, a field, a method
  call or an operator in any arm keeps a decision there, and the whole `match` stays logic.
  `cleanroom check`'s L2 exempts the same values.
- **Gates** are hard ceilings on the clean crates.
  - **G5, the clean room's clippy configuration** (`crates/clean/.clippy.toml`, which
    `cleanroom export` writes as the export's `clippy.toml` and the trunk's gate and Quality run on its
    clean crates): at most one `bool` parameter (`max-fn-params-bools = 1`), no block nested 5
    deep (`excessive_nesting`, denied in `[workspace.lints]` and off wherever no threshold is
    set), cognitive complexity at most 15. `too-many-arguments-threshold` stays at 7: a lower
    one would make a struct of the arguments a compliance tool.
  - **G6, `argument-object`** (`cleanroom check`, over a crate's library sources together): a struct taken by exactly one signature and built only in
    the argument lists of calls to that function, with no `impl` (no methods, no trait), and not
    a field's or a variant's type, a return type, a constant's or an alias's type or named in a
    trait. That is a bag of one call's arguments, which a parameter-count rule would reward;
    the values are passed as they are, or the concept gets a role.
  - **G7, `layering`** (the same scan): a crate's modules name each other in one direction
    only. A file's module is its path; a reference is a path through `crate::`, `super::`,
    `self::` or a child module's name, `use` groups included, and any cycle among them is
    refused, a child naming its parent while the parent reads the child included. It stops
    passing a whole owner (`&Samus`) down to a helper module to shorten its signatures.
- **Three lints**, as `cleanroom check` tags them on the clean crates:
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
    static's value while L3 makes it data (`Picture::new(0x009)` is; a `.expect(..)` on it
    is not), in the value of an exhaustive lookup's arm or an `enum_map!` row (`K::A => 3`;
    a key's own number is still refused, so `7 => ..` names its 7), in declarations (enum
    discriminants, array lengths in types), in an array length `[x; N]`, as a tuple field
    (`.2`), in attributes and in test code. Every number traces to a named constant, a `Rules`
    field or a behaviour.
  - **L4, `variant-import`.** No `use` of an enum's variants outside test code (`use
    Kind::{A, B}`, `use Kind::*`, at any depth of a use tree): a variant is named through its
    enum, so a reader sees which enum a bare name belongs to.

## 5. How the work is planned

**By mechanism, not by divergence.** A piece of work builds or extends one general mechanism
that covers a family of behaviours: a rule table, the instruction-list interpreter
(`Program<F>`, which enemies have and PLMs need), the collision sweep, an encoder rule. Indexed
production replay is the acceptance check; first differing observations identify *which
behaviour* is wrong. They don't decide *what code to write*. The order of the work is
[domains.md §4](domains.md#4-order).

**A domain is a unit that can be verified on its own.** Its boundary is one the player sees:
where the original splits one player-visible behaviour across systems (motion and pose
selection), the domain spans both. A stand-in that answers from the oracle copies what it
touches, so it is unverified; a domain that needs one merges with what it stands in for.

**The round**, its steps, roles and who sees what, is [PROCESS](../cleanroom/PROCESS.md). This
section holds the rules a round relies on.

**What counts as a leak (Jim, 2026-11-07).** Structure leaks; perceivable quantities don't.
- **Allowed:** quantities a player can perceive or measure (positions, speeds, timers, counts) and
  the exact arithmetic on them.
- **A leak:** the original's routine boundaries, entry points and call order; its internal flags and
  modes; its command and result words; enum orders that match its RAM codes; how its tables are
  indexed; its routine and variable names; and its addresses, in any form.

**An adversary gates the clean room (Jim, 2026-11-07).** An adversary reads both the original
and everything an implementer will see, and tries to prove a leak; it succeeds by finding one.
It gates when decision 20 says. A leak it finds stops the work until it is removed; a reduced
leak is not a fixed one.

**The clean room is enforced by construction, not by trust (Jim, 2026-11-07).** Every worktree
shares one object store and every ref, so a sparse checkout is not enough. So:
- **The implementer works in an export,** not a worktree: a fresh repository of one commit
  that `cleanroom export` builds with `git archive` from a fixed list (the clean crates,
  [DESIGN](../cleanroom/DESIGN.md), the domain's spec and vectors, the implementer skill). It
  has no history, no other refs and no remote, and lives outside the home. Results come back
  as patches, applied by `cleanroom import`, which reads the export through a mirror of its
  objects alone, at most `CLEANROOM_MIRROR_MAX_MB` (1024). The box mounts the export's
  `.git/config`, `.git/info` and `.git/hooks` read-only, and `run` and `import` refuse an export
  whose `.git` holds anything a session's git doesn't leave.
- **The session runs in a box.** `cleanroom run` starts the agent in a container with no
  network whose file system is the export, the toolchain and the tool, so nothing else is
  there to read. The model's API is the one way out, through a relay that holds the
  credential and refuses a request that asks the API itself to reach the web. A Rust test
  proves the box reads its export and nothing beside it.
- **Scans run on everything that enters the export or the clean crates**, briefs included
  (`cleanroom scan`; `cleanroom export` and `run` scan the brief, and `cleanroom test` and the
  import scan what the implementer changed, `cleanroom test` through the relay, since the
  lexicon stays outside the box):
  - addresses in every form: `$XX:XXXX`, split bank/offset pairs, `RomAddress`/`Source::bank`, and
    a hashed set of the original's known WRAM and ROM addresses;
  - names, against a lexicon drawn from the bank logs, with a reviewed allowlist of player vocabulary;
  - shape: enums whose variant order equals a RAM code's values, fields decoded as one word copied
    as-is;
  - routine mirroring (`scan.sh --mirror`): the share of entries with a single routine as source
    (gated at 30 % or less), and entry points that map to one routine.
- **Briefs are saved and scanned before the spawn.** The root and the observer are relays too.
- **The clean crates.** The model, the catalogue and the domain types an implementer needs live
  in their own crates. They depend on each other, the standard library and the registry crates
  of `tools/cleanroom/allowed_deps.txt` (decision 12); the root declares those in a clean
  crate's `Cargo.toml`. They contain **no address, bank, RAM, ROM-code or original routine name
  anywhere, comments included**; the scans refuse one. `data` stays on the observer side and
  turns ROM bytes into the clean types at the edge. `deny.toml` holds the dependency rule and
  `tools/check_crate_rules.sh` refuses SNES address literals in `game`, `runtime` and `app`,
  both in CI.
- **No addresses where an implementer can see them.** `Source:` lines live only in the observer's
  decode notes. The spec carries behaviour ids, rules, parameters and vectors. The catalogue
  carries ids, summaries, kinds and switches.
- **Only implementer commits change the clean crates.** The import refuses any commit to them
  that lacks the trailer `Role: implementer`. An integrator who needs a model change asks for it.

**What the implementer may read:** the clean room's design, its domain's spec and vectors, the
replay's first differences, `data`'s public types, and the model code. **Not:** the bank logs or the asm,
`verify/src/{decode,source}.rs` (they hold WRAM addresses), the old module it replaces, and
git history.

**The observer works behaviour first (Jim, 2026-11-07).** An observer who reads routines writes
a spec shaped like them.
- The spec describes **what the player sees** and what the oracle's records show over time, in
  player terms: speeds and positions as they change, how each input changes them, how the world
  stops them.
- The original's code is consulted only to pin down **exact arithmetic and edge cases** that the
  records can't settle. Its routine boundaries, call order, internal flags and RAM layout never
  shape the spec.
- **The spec review is a gate.** An entry named after its one routine, an entry point that
  matches a routine, or a type that mirrors a RAM word stops the spec until it is regrouped.

**No fitting to the oracle.**
- Every literal and every branch in the clean model traces to a `BehaviourId`, and values come from
  `Rules` or `data`.
- The catalogue holds the spec's entries one for one, and a test in the clean crate holds it to
  the vectors: the observer adds an entry to the spec, the implementer to the catalogue.
- No conditions on frame numbers, positions or room IDs that no spec entry names.

**The spec template.**
- **Header:** the domain's scope, what it reads from other domains and what it leaves to them,
  and its notation.
- **Rules are arms.** Each rule is one bullet of the form `<conditions> → <outcome> [vectors]`,
  citing vectors or recorded frames that exist. Exact formulas are in backticks. `checked` is
  its own bullet.
- **Each quantity:** its units, a domain type (an enum, never ROM codes), and **Read by**: the
  behaviour IDs that read it on a later frame, or "output" if it reaches bytes, or "none". "None"
  means it isn't State, so it is not decoded or compared (decision 3). A quantity that is State
  is one a caller can set to any value the spec allows and read back, so the engine can carry,
  save and restore it; the spec says so where the model's memory is listed. ROM codes belong in
  the decode notes.
- **Each entry:** id, kind, summary, rule, params, switch, and **checked** (`oracle (V3, V4)` or
  `original only`). Its source in the original goes in the decode notes, never in the spec.
- **Not allowed:** headings named after routines; RAM addresses outside the Source line; step lists
  that follow a routine. Order is stated only in "Order that matters", and only where it is
  visible at a cut.

**A brief names behaviours and the mechanism** by spec entry IDs (for example: "`BallAtRest`
coasts like standing; add its row"). It carries no bank-log excerpt and no address. A new
free function named after a routine is a review stop.

## 6. How progress is measured

The headline remains whole-movie acceptance from power-on with movie inputs only, every
compared cut and output byte matching (decision 14). Scoped, integrated and whole-movie
witnesses are reported separately. Borrowed facts are never counted as modeled producers.

The replay index defines witness coverage and the committed baseline defines reviewed result,
count and observation obligations. Every declared cut is accounted for. An expected unsupported
boundary remains unsupported rather than a successful case. Missing data, empty comparisons
and unreached outputs do not establish exactness. Meaningful planted faults establish whether a
witness can distinguish incorrect behaviour. The transition ledger names replacement claims and
reviewed losses. [STATUS](../STATUS.md) gives current commands and report sources.
