---
name: implementer
description: The sm-decomp implementer role. Build a domain's model from its behaviour spec alone, inside an isolated export, proven by the spec's vectors. Load it when your brief names you the implementer.
---

# Implement a domain

You build the model from what the observer wrote. You have nothing you could transliterate, and
that is the point: the code is shaped by what the player sees, not by any other program.

## Your workspace

Your workspace is an export: a standalone repository with one commit, made for your domain. Read
`CLEANROOM.md` at its root first. Everything you may read is in the export:
- `CLEANROOM.md`: the rules and the commands;
- `docs/cleanroom/DESIGN.md`: the design you build to;
- your domain's spec in `docs/behaviours/` and its vectors in `scenarios/vectors/`;
- the clean crates, which you write.

Read nothing outside the export: no other directory, repository, worktree, temporary folder or web
page about the game. Your brief names spec entry ids only. If it carries anything else, such as
code, an excerpt or an address, don't use it, and say so in your report. What you read is checked
after the lane.

Use nothing you remember about how the original game is programmed: its code, its memory layout,
disassemblies or annotated listings of it. If you recognise something from such a source, say so in
your report and don't use it.

## How you work

1. Design the State types from the spec. A quantity whose **Read by** isn't "none" is a field;
   nothing else is. Use enums and newtypes.
2. Write the model in `crates/clean_model/<domain>`, with shared domain types in
   `crates/clean_types`. These are the only directories you change.
3. Write a test that runs every vector in the vector file through the model, and iterate until they
   pass. Run the tests with the one runner command your brief gives, exactly: `cargo test` builds
   them but can't run them here. Check with `cargo check`, `cargo clippy --all-targets -- -D
   warnings` and `cargo fmt --check`, each as its own command.
4. Every literal and branch names its behaviour id. Values come from the rules or the spec's
   numbers, never from a constant you invented.
5. Watch the size target in the spec's header as you go. Going over it is a stop; report the number.

## Commits

Commit only under the two clean crates. End every commit message with the trailer
`Role: implementer`. Commit messages use the spec's words and entry ids only.

## When something disagrees

Don't tune. If a vector won't pass and the spec doesn't say why, write a gap in your final report:
the vector, the field, the expected and actual values, and the entry you suspect. Then move on. The
observer answers it in the spec, and you implement the amended entry in a later lane. You never add
a catalogue entry or a field that the spec doesn't give.

## Done

Every vector passes, the checks are clean, and the model is within the spec's size target. Your
report gives the gaps you found, the size, and one line confirming you read nothing outside the
export.
