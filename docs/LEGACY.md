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

The verifier asserts on the exact text of panics and `fatal_error` calls. That ties the
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
| `sm/src/rtl.rs` (`read_file_block`) | `fatal_error("fread failed\n")` on a short state-file read |

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

## Faithful-oracle known divergences

`SM_ORACLE=faithful` runs the original game without snesrev's patches. Some
original behaviour copies uninitialised low-WRAM scratch (the direct page
$00-$50, the stack) into VRAM or RAM tables. The engine does not model that
scratch, and the effect has no gameplay consequence, so these patches stay
installed (verdict "a" in `tools/snesrev_patches.py`) and these faithful rows
are expected to fail at the listed frame. Owning the direct-page scratch in the
Session would retire the list.

| Patch | Faithful rows and first failure | What hardware shows |
| --- | --- | --- |
| `$82:831E` BG3/sprite tile DMA size | `samus-scripted-intro` @8256, `demo-attract-setup` @2619, any% chain room 11 (`chain-any-0011-DF45`) @2055 | WRAM $0000-$11FF in sprite VRAM $7700-$7FFF, overwritten before display |
| `$84:8243`, `$84:EFD3` unpause item-GFX load | `pause-menu-tour` @1378, `boss-kraid-pause-400` @588, `-950` @1138, `-1030` @1218, `-1960` @2148 | garbage transfers into empty item-GFX slots |
| `$80:9AF3` broken mini-map init | `samus-scripted-intro` @8316, any% chain room 11 (`chain-any-0011-DF45`) @2115 | one frame of HUD mini-map tiles from scratch, while blanked |
