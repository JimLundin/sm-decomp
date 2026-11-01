---
name: sm-decouple
description: Move sm_game state off SNES RAM onto typed Session state, using typed copies (session_cohort) or the scheduler's lease. Use it when a brief asks to cut sm_game RAM sites or move a family off Game.
---

# Decouple sm_game from RAM

Progress is `sm_game_ram_sites` falling in `python tools/ram_ratchet.py`, which
counts every RAM-by-name access. Renaming a RAM word does not count. A word is
decoupled only when sm_game reads and writes it as a typed Session field.

## Pick the tool

| The code... | Use |
| --- | --- |
| runs on the scheduler's lease (actors, bosses) and still has a `Host for Game` path | move the family onto the lease |
| reads or writes a group of Session-projected words through a view (`boss!`, `menu!`, a `*_view`) | a typed copy (`session_cohort`) |
| touches words no Session field holds yet | add the field first (with its RAM encoding, checkpoint lane and layout test), then a copy |

## Typed copy

Read `crates/sm_game/src/session_cohort.rs` and one finished example, such as the
pause menu's `MenuState`.

1. Group the words one function (or one frame) uses into a `Cohort`.
2. Load the copy once at the top. Work on its typed fields. Store once at the end.
   The store writes only the changed words, as one lease while a retained Session
   is current.
3. Delete the view once its last caller is gone.

**Hold rule.** While a copy is held, nothing reaches its words any other way.
Before converting, grep the function and everything it calls for raw addresses
in the copy's range: `ram()`, `word_transfers!`, address tables in ROM, and lease
reads. The verify guard catches projection access only. Past misses:
- Samus's frame read the cinematic table as enemy projectiles.
- The pause sprites addressed their timers through a bank $82 table.

## Lease move

Read a finished example: `git log --grep "scheduler's lease"` (Ridley, the corpses, Atomic).

1. Run the family's arms from the lease Runtime (its `Lease` trait), not from `Game`.
2. Delete each `Host for Game` impl once its last caller moves.
3. Mark the commit `(behaviour: lease merge)` when the lease reorders writes; otherwise `[exact]`.

## Measure

- Report `sm_game_ram_sites` and `sm_game_coupled_files` before and after each commit.
- For lease moves, also tally Game-hosted calls in a local instrumented build: count
  calls in `SlotAccess for Game` against the lease impl, by caller. The moved family
  must show 0. Never commit the instrumentation; your brief may name a ready script.

## Prove it

Use `sm-check`. Run lockstep and check-lock on every row that executes the code,
and for chain rooms when actors move. If cost could change, time intro and credits
back to back against the base.

Done when the ratchet fell, every listed row passes, and the view or Host impl is deleted.
