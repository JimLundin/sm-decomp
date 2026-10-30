# Legacy behavior register

Behavior we keep only because a check depends on it, not because the game
needs it. Each entry is marked in the code with a `LEGACY(<kind>)` comment, so
the register and the code cannot drift:

```sh
grep -rn 'LEGACY(' --include=*.rs sm crates
```

Do not add new dependencies of these kinds. When a check is changed so it no
longer needs an entry, remove the held behavior, its tags and its row here in
the same change.

## `LEGACY(message-text)`: checks that compare failure text

The verifier asserts on the exact text of panics and `Die` calls. That ties the
engine to wording it does not care about: production code has to spell out
`index out of bounds: the len is N but the index is i` by hand, and a file read
still reports `fread failed` after the C stream calls were replaced by
`std::fs`. The checks should assert the kind of failure (which table or read
rejected, and where execution stopped), not the words.

| Held in production | What is kept |
| --- | --- |
| `crates/sm_data/src/table.rs` (`check_index`) | slice-index panic text for jump-table selectors |
| `crates/sm_game/src/player_projectiles/{weapons,specials,bomb_terrain}.rs` | 11 hand-written slice-index panics |
| `crates/sm_game/src/actors/mother_brain_fight.rs` | 1 hand-written slice-index panic |
| `sm/src/rtl.rs` (`read_file_block`) | `Die("fread failed\n")` on a short state-file read |

| Checks that compare the text | Tags |
| --- | --- |
| `sm/src/verification/native_content_checks.rs` | 7 |
| `sm/src/verification/player_movement_edges.rs` | 4 |
| `sm/src/verification/{actor_population_animation_cases,collision_mechanism_checks,player_minimap_edges}.rs` | 2 each |
| `sm/src/verify.rs` | 2 |
| `sm/src/verification/{actor_movement_cases,player_minimap_boss,room_background_cases,room_contact_start_cases,room_reference_cases,room_world_cases,room_world_dll,samus_game_binding_cases}.rs` | 1 each |

To retire an entry: change the check to assert the failure kind, then replace
the held text with whatever reads best (a plain slice index, an `assert!` with
its own message, or a typed error) and delete the tags.
