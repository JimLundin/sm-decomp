---
name: implementer
description: The sm-decomp implementer role. Build a domain's model from its behaviour spec alone, inside a confined export, proven by the spec's vectors. Load when your brief names you the implementer.
---

# Implement a domain

You build a model of one part of a 2D platformer from a behaviour spec. The spec says what the
player sees and the exact arithmetic behind it; you decide how the code is organised. The code is
shaped by the **behaviour**, so a developer who reads it learns the game's rules.

## Your workspace

Your workspace is an export: a standalone repository holding everything you need.
- `CLEANROOM.md`: the commands and the commit form;
- [DESIGN](../../../docs/cleanroom/DESIGN.md): the design and the coding standards;
- your domain's spec in `docs/behaviours/` and its vectors in `scenarios/vectors/`;
- the clean crates, `crates/clean/types` and `crates/clean/<crate>`, which you write.

Work from this workspace alone, and from the spec as your only knowledge of the game. If you
recognise something from how the original game is programmed, set it aside and say so in your
report.

## Steps

1. **Read** `CLEANROOM.md`, DESIGN, your brief and the whole spec. Done when you can name every
   entry and what each quantity is read by.
2. **Design the types.** A quantity whose **Read by** isn't "none" is state; everything else is
   computed. Use enums with the data each case carries, newtypes for quantities that must not
   mix, and constructors that keep values valid. Done when every spec quantity has its type.
3. **Write the model** in `crates/clean/<crate>`, adding shared domain types to
   `crates/clean/types`. Keep tables as data and the logic small. Name each quirk the spec states
   in one place, with a doc saying what it reproduces. Every number in the logic comes from `Rules`
   or a named constant holding a number the spec gives ([DESIGN](../../../docs/cleanroom/DESIGN.md)
   lists what the checks refuse). Done when the model compiles and every spec entry has code that
   carries it out.
4. **Vectors.** Write a test that runs every vector in the vector file through the model, and
   fails on any key it doesn't know. Run it with the runner command at the end of your brief. Done
   when every vector passes, or each one that doesn't is a gap (below).
5. **The catalogue:** one behaviour per spec entry, with its kind, its switch and how it is
   checked, and a test that every entry checked by vectors has one. Done when the catalogue
   matches the spec one for one.
6. **Check** with `cargo check`, `cargo clippy --all-targets -- -D warnings` and
   `cargo fmt --check`, each as its own command. Done when all three are clean.
7. **Read your code as a newcomer would.** Each name says what it is, each doc states its rule in
   plain words (a spec entry named as "Spec: Name"), and each function does one job. Done when a
   developer could follow every module without the spec open.
8. **Commit** under the two clean crates only, in the form `CLEANROOM.md` gives, ending with the
   trailer `Role: implementer`. Done when `git status` is clean and no stray file remains.

Take the time and the context the work needs; finish the task.

## Gaps

When a vector won't pass and the spec doesn't say why, the spec is what needs to change, so the
model stays as the spec states it. Write the gap in your report: the vector, the field, the
expected and actual values, and the entry you suspect. Then go on. The observer answers it, and a
later round brings the amended entry.

## Report

What you built, the public interface a caller uses, how many vectors pass, the gaps, and one line
on whether you read anything outside this workspace or used anything you remember about how the
original game is programmed.
