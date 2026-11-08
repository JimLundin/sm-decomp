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
  be. One rule lives in one place. The spec's header gives a size target; work until you reach it.
  A table's values are counted apart from the logic, as data. A row states something the spec
  gives as a fact (a speed, a cell, a row of a chart); a row that encodes a decision the code would
  otherwise make, such as which branch to take, is logic and counts as such.

## How your code is measured

- **The score** is one weighted count of your model's logic tokens. Free: type declarations
  (`struct`, `enum`, `type` items and attributes), the values of constants and statics (data),
  constant names, `use` and `mod` lines, trailing commas, the `const` of `const fn`, comments and
  tests. A function's signature counts half; `true` and `false` count double; each level of
  nesting adds a quarter to every token inside it (a flat `match` doesn't nest). A pair of
  parameter types that recurs in three or more signatures, or a pair of arguments at three or
  more call sites, costs more: group values that travel together into a concept with a name.
  Define as many shapes as the behaviour needs.
- **The budget** comes from the spec: its header gives it by area. Over it, keep reshaping and say
  where the rest goes and why; it isn't a reason to stop early.
- **The lints.** Clippy here enforces at most one bool parameter, nesting at most five deep (test
  code included) and cognitive complexity at most 15. The rest are checked when your work is
  imported, so keep to them as you go:
  - No bare number other than 0 and 1 in the model's logic. Name it, or put it in `Rules` or a
    named constant. A constant that holds a number the spec gives is a fact of the spec, not one
    you invented.
  - No `allow` attribute at all; every `expect` gives a `reason`. Outside tests, no `expect` names
    a lint the workspace denies by name, a denied group or `warnings`; a lint denied only through
    a group (such as `struct_excessive_bools`) may be expected with its reason.
  - No struct that only wraps one call's arguments (taken by one function, built only at its
    calls, with no methods).
  - Modules import in one direction only: a module and its parent, or two siblings, may not each
    import from the other.
  - A constant's value that holds control flow, a closure or a function call is logic, not data.

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

- Write only in `crates/clean_model/<domain>` and `crates/clean_types`. They depend on nothing but
  each other and the standard library.
- End every commit message with the trailer `Role: implementer`, and use the spec's words and entry
  ids in it.
- Check with `cargo check`, `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check`. Run the
  tests with the one runner command your brief gives.
- What you read is checked after the lane. Read only this repository.
- **Use nothing you remember about how the original game is programmed:** its code, its memory
  layout, disassemblies or annotated listings of it. Build only from the spec. If you notice you
  recognise something from such a source, say so in your report and don't use it.
