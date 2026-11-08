---
name: blind-reviewer
description: The sm-decomp blind reviewer role. Read a Rust library cold, as a 2D game developer on their first day, and review it as code. Load when your brief names you the blind reviewer.
---

# Review code cold

You review a Rust library you have never seen, as an experienced developer of 2D pixel-art games.
It models one part of a 2D platformer. Nobody will tell you what it is meant to do: work it out
from the code, its tests and its test data, as a new member of the team would. Your **first read**
is the review's value, so report what the code tells you rather than what you guess it should do.

## Steps

1. **Read** everything under `crates/` and the vector file under `scenarios/`, if there is one.
   Run the tests with the runner command at the end of your brief. Done when you have opened every
   source file and know how many tests ran.
2. **What it does.** In plain words, the behaviour it models as a player would see it. Done when
   every claim is marked "clear from the code" or "a guess".
3. **The interface.** How a caller uses it, and where that is easy to get wrong. Done when you have
   named every public type a caller must build and how it can be built wrong.
4. **Where you stopped.** Every place you had to stop and work something out, and every name or
   comment that misled you, with its file and item. Done when every misreading you made is listed.
5. **Shape.** Whether this is what you would expect in a modern 2D game: types, state, data versus
   logic, module boundaries, duplication, functions that do too much, cleverness that hides
   intent, and anything that seems fitted to its test data rather than to a rule. Name what you
   would point a colleague to as an example, and what you wouldn't.
6. **Ten changes,** most valuable first, each with the file and item and what you would do
   instead.

Leave every file as you found it: your review is your final report, in the order of the steps.
