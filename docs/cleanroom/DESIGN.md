# The design you build to

You build one domain of a Super Metroid engine: a model of what the game does, written from a
behaviour spec. This note is the part of the project's design that applies to you. Everything you
need is in this repository; read nothing outside it.

## What you are building

- **A model of the game's behaviour,** in its own terms. The spec says what the player sees and
  the exact arithmetic behind it. You decide how the code is organised: the types, the functions,
  the tables. Shape it by the behaviour, not by any guess about how another program might do it.
- **Exact.** The spec's numbers and rules are exact, quirks included, and the vectors check them.
  The finished model is later checked against the real game, frame by frame.
- **Small and data-driven.** One kind of movement or situation is one row of a table, where it can
  be. One rule lives in one place. A row states something the spec gives as a fact (a speed, a
  cell, a row of a chart); a decision the code would otherwise make, such as which branch to take,
  stays in the code, and a constant or a table holds values, not control flow, closures or calls.
- **A table keyed by an enum** names each member once, in one of three forms: an exhaustive
  `match` from each variant (or a constant, or a literal) to its value, with no guard and no
  block arm; `enum_map!` rows; or a constant's rows, each naming its member, gathered with
  `types::by_member`. The compiler checks the first two for a missing or repeated
  member; the third needs a test that checks the set. Its values are values: literals,
  constants, variants and constructors given only values, never a local, a field or a
  computation. Choose the form that reads best where the table is used.
- **Shaped by types.** Define as many types as the behaviour needs. Values that travel together
  through several functions are a concept with a name. A flat `match`, a helper and an early
  return read better than decisions nested inside each other. A state is an enum, not a flag.
- **State a caller can hold.** Everything the spec lists as what the model remembers from one
  frame to the next is state another part of the game carries between frames, saves and restores,
  and sets when play starts part-way. So a caller can build each part of the state at any value
  the spec allows, and read each part back. The State derives serde's `Serialize` and
  `Deserialize` (a normal dependency), so the game can save it as it is: every field is
  written, in order, so no `skip`, `flatten`, `untagged` or `default` on it, no custom serde
  code, and a table by enum is an `EnumMap` with enum-map's `serde` feature on.
- **Concise.** Write the smallest clear code that models the spec. Each rule is stated once.
  Where two things run the same steps, write the steps once and let each supply only what
  differs. Every type, function and module carries a rule or a concept of the spec; a value is
  read where it lives, rather than copied forward or handed back by an accessor that adds
  nothing.

## What the model checks, and what it leaves to others

The model stays **total**: it doesn't refuse content or rules outside the spec, and every value
its types admit gives an answer. Checking is placed by what is checked:

- **A value the spec gives a range is a checked newtype.** Its constructor refuses a value
  outside the range (a count of at most 99, a frame within a clip), and the rest of the model
  then relies on it.
- **Relations between fields and the shape of content are the importer's.** That a clip's marks
  fall inside its frames, or that a table has a row for each screen, is checked where the game's
  content is loaded into this format, outside this repository. The model doesn't check them
  again, and its docs don't say they are checked.
- **A trap inside the domain is closed by types.** A call the spec allows but that is easy to
  get wrong gets a type that rules it out: two arguments of one type that a caller could swap
  get a type each, or a struct whose fields name them; an `Option` or a sentinel value standing
  for something the domain doesn't have (`contains(None)`, a number set aside to mean "none")
  gives way to a type that holds only the domain's values.

## Load-bearing versus incidental

Some details of the game are behaviour, and some are only how its hardware held the behaviour.
The spec states the first kind; the second is yours to replace.

- **Load-bearing, kept exactly and stated:** a timer that wraps around, a borrow or carry
  between a value's whole and sub-unit parts (pixels and subpixels), and the order in which a
  frame's updates read each other's results. Each is a named behaviour where the spec names it,
  and the code says where it happens.
- **Incidental, replaced by the plain modern form:** storage widths (a quantity takes the type
  it needs; a width matters only where the spec says a value wraps or stops at it), code
  pointers and jump tables (an enum and a `match`), and hardware layouts (packed bits,
  interleaved tables, one table indexed by another's position).

## Names and docs

- A doc cites a behaviour by its entry id (`LeadingHer`), the id the catalogue uses, and states
  the rule in plain words. It never cites a section number of the spec, which moves when the
  spec does.
- A type has a `Default` only where the spec gives its default value (a new file, an empty
  set). A stand-in for content a caller hasn't supplied gets a named constructor that says what
  it is, never a `Default` that reads as real content.
- One name means one thing in a crate: no type named like a variant of another enum or like
  another type in another module, and no one word for two quantities.

## What the checks refuse

Clippy here refuses more than one bool parameter, a block nested five deep (test code included),
cognitive complexity above 15 and more than seven arguments. The rest are checked when your work
is imported, so keep to them as you go:

- No bare number other than 0 and 1 in the model's logic. Name it, or put it in `Rules` or a
  named constant. A constant that holds a number the spec gives is a fact of the spec, not one
  you invented.
- A constant's value may hold numbers while it states data: literals (a negative one included),
  other constants, enum variants, references, struct and tuple-struct literals, arrays, tuples,
  a constructor such as `T::new(..)` or `f(..)` given only data as its arguments, and a range
  whose two ends are literals or constants (`0x280..0x500`). A table
  of typed values such as `[Picture::new(0x009), Picture::new(0x000)]` or
  `Tile::new(Picture::new(0x00F), COLOURS_3, Flip::None)` is data. A value that also does
  something is logic, and its numbers are bare: control flow (`if`, `match`, loops, `?`), a
  block, a closure, a macro, a method call on a value (`Divisor::new(100).expect(..)`,
  `.unwrap()`), a field or an index, a cast, a range with arithmetic or a call at an end
  (`0..A + 1`), or any other operator but the minus of a negative literal (`A | B`). Where a constructor can fail, name its number in a constant of
  its own first (`Divisor::new(ENERGY_DIVISOR).expect(..)`).
- The value of each arm of a table `match` or each `enum_map!` row may hold numbers the same
  way (`Kind::Wave => 3`), when the `match` has two or more arms, no guard and no block arm,
  every pattern is `_`, a variant, a constant or a literal (`A | B` joins them), and no value
  reads a local or a field or computes. A number used as a key is still named (`TOP =>`, not
  `7 =>`).
- No `allow` attribute at all; every `expect` gives a `reason`. Outside tests, no `expect` names
  a lint the workspace denies by name, a denied group or `warnings`; a lint denied only through
  a group (such as `struct_excessive_bools`) may be expected with its reason.
- No `use` that imports an enum's variants outside tests (`use Kind::{A, B}`, `use Kind::*`):
  write `Kind::A` where it is used, so a reader always sees which enum a name belongs to.
  Crates written before this check may still hold such imports; when your work is imported
  they are refused in every crate your workspace holds, so remove any you find.
- No struct that only wraps one call's arguments (taken by one function, built only at its
  calls, with no methods).
- Modules import in one direction only: no module reaches itself through the modules it imports,
  by a loop of any length. Two modules naming each other is the shortest such loop; a module
  that names one which, through others, names it back closes a longer one.
- Use the crates your manifests name where they model the domain better than hand-written code:
  `enum-map` for a table with one entry per variant of an enum, `serde` to read text into typed
  records, `toml` for the vector files. Nothing else from outside std: no other crate, no feature
  of these beyond the ones the manifests turn on, no file, network, process or environment
  access, no `unsafe`, no build script. If another widely used crate is the standard tool for
  something your model needs and std lacks, ask for it in your report and say what it would
  model.
- No item, tests included, is named `File`, `TcpStream`, `UdpSocket` or `Command`: the check
  can't tell a type of yours from the standard library's, so it refuses the names themselves.

## What a modern developer would expect

Write the code a developer of a 2D pixel-art game today would expect to find, and would know
their way around without being told. Use the field's common shapes and names wherever the spec's
behaviour fits them:

- **A character controller** for the player: her motion as velocity and position in fixed-point
  subpixels, stepped at a fixed 60 Hz timestep, with gravity, acceleration and caps as tuning
  parameters in `Rules`.
- **A state machine** for what she is doing: her states and the moves between them as rows, each
  move with its effects. The pose chart is these rows.
- **Input as actions** (jump, shoot, aim up, run), not raw buttons; the binding to buttons is the
  caller's.
- **Tilemap collision:** move and collide one axis at a time against a grid of tiles, each with a
  collision shape (solid, a slope with its surface heights, air of a kind); actors against actors
  as axis-aligned boxes.
- **Animation as clips:** named sequences of frames, with the durations a spec gives, and what
  happens at a clip's end; where a spec leaves timing out, so does the model. Drawing is someone else's: the model says which clip and pose, never which sprite.
- **Events out, not side effects:** what she touched or met is returned for the caller to act on.

Where the spec's exact behaviour departs from what these shapes would do on their own (a quirk),
keep the shape and state the quirk as its own named behaviour, rather than bending the shape to
hide it.

## Behaviours are named and switchable

Every behaviour in the spec has an id, and the model has a catalogue with one entry per spec entry:

```rust
pub struct Behaviour {
    pub id: BehaviourId,          // an enum variant, one per spec entry
    pub summary: &'static str,    // what the player sees
    pub kind: Kind,               // Mechanic | Quirk | Timing | Encoding | Output
    pub switch: Switch,           // Fixed | Toggle | Parameter
}
```

- A **mechanic** is a deliberate game rule. A **quirk** is unintended behaviour the game has and
  the model reproduces. **Timing** covers slowdown and loading time. **Encoding** covers output
  quirks. An **output** is something the step reports for the caller to show or act on (which
  sprite, which cells she reached), not a rule of her motion.
- The model reads its values from a `Rules` value, whose reference profile holds the spec's vanilla
  values. A different `Rules` is an adjusted game.
- Most entries are `Fixed`. An entry is a `Toggle` only where the spec says so; the spec names the
  vanilla setting and the alternative.
- Every literal and every branch in the model traces to a behaviour id. A test checks that every
  `BehaviourId` is exercised by at least one vector.

## The State

The model's State holds only what a later frame needs. The spec says, for each quantity it gives,
what reads it. Hidden memory the spec describes as behaviour ("after she lets go of run while
halted, her next step loses the dash") is yours to represent, as the smallest state that makes the
behaviour exact. Use enums with data and newtypes, never raw numbers for a mode or a kind.

## No fitting

- Every value comes from the spec's numbers or from `Rules`. Don't add a condition on a frame
  number, a position or a place that no spec entry names.
- If a vector won't pass and the spec doesn't say why, don't tune the model to make it pass. Report
  it as a gap with the vector, the field, the expected and actual values, and the entry you suspect.
  The spec is amended, and you implement the amendment in a later lane.

## The rules of this repository

- Write only in `crates/clean/<crate>` and `crates/clean/types`. They depend on each other,
  the standard library and the crates their manifests name.
- End every commit message with the trailer `Role: implementer`, and use the spec's words and entry
  ids in it.
- Check with `cargo check`, `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check`. Run the
  tests with the one runner command your brief gives.
- What you read is checked after the lane. Read only this repository.
- **Use nothing you remember about how the original game is programmed:** its code, its memory
  layout, disassemblies or annotated listings of it. Build only from the spec. If you notice you
  recognise something from such a source, say so in your report and don't use it.
